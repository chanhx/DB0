use {
    super::{error, FileNode, PageTag, Replacer, Result},
    crate::{manager::StorageManager, PageNum, DEFAULT_PAGE_SIZE},
    core::ptr::NonNull,
    snafu::ResultExt,
    std::{
        cell::RefCell,
        collections::HashMap,
        marker::PhantomData,
        path::PathBuf,
        slice,
        sync::{
            atomic::{AtomicI32, AtomicU32, Ordering},
            Arc, RwLock,
        },
    },
};

pub(super) type BufferId = usize;

pub struct BufferManager {
    storage_manager: StorageManager,

    tag_table: RefCell<HashMap<PageTag, usize>>,
    descriptors: Vec<Arc<RefCell<BufferDescriptor>>>,
    /// index of first free descriptor, -1 if there is no free descriptor
    first_free: AtomicI32,

    buffers: Vec<u8>,
    buffer_size: usize,

    replacer: RefCell<Replacer>,
    // last_page_ids: HashMap<FileNode, PageId>,
}

impl BufferManager {
    pub fn new(capacity: usize, buffer_size: usize, data_dir: PathBuf) -> Self {
        let descriptors = (0..capacity)
            .into_iter()
            .map(|i| {
                let mut free_next = i as i32 + 1;
                if free_next == capacity as i32 {
                    free_next = -1;
                }

                Arc::new(RefCell::new(BufferDescriptor {
                    page_tag: None,
                    buffer_id: i,
                    next_free: free_next,
                    state: AtomicU32::new(0),
                    content_lock: RwLock::new(()),
                }))
            })
            .collect();

        let replacer = Replacer::new(capacity);

        Self {
            storage_manager: StorageManager::new(data_dir),

            tag_table: RefCell::new(HashMap::with_capacity(capacity)),
            descriptors,
            first_free: AtomicI32::new(0),
            buffers: vec![0; buffer_size * capacity],
            buffer_size,

            replacer: RefCell::new(replacer),
        }
    }

    fn reuse_page(&self, tag: &PageTag) -> Result<(BufferId, BufferRef)> {
        let first_free = self.first_free.load(Ordering::SeqCst);
        let has_free_buffer = first_free >= 0;

        let id = if has_free_buffer {
            let id = first_free as usize;
            let desc = self.descriptors[id].clone();
            // TODO: cas
            self.first_free
                .store(desc.borrow().next_free, Ordering::SeqCst);

            id
        } else {
            let id = self
                .replacer
                .borrow_mut()
                .victim()
                .ok_or(error::NoMoreBufferSnafu.build())?;

            self.flush_page(id)?;

            id
        };

        let mut desc = self.descriptors.get(id).unwrap().borrow_mut();

        let mut tag_table = self.tag_table.borrow_mut();
        if !has_free_buffer {
            tag_table.remove(desc.page_tag.as_ref().unwrap());
        }
        tag_table.insert(tag.clone(), id);

        desc.register(tag.clone());
        self.replacer.borrow_mut().pin(id);

        let buffer_ref = self.get_buffer(id);
        buffer_ref.reset();

        Ok((id, buffer_ref))
    }

    pub fn new_page(&self, file_node: &FileNode /* page_size: u8 */) -> Result<BufferRef> {
        let page_size = DEFAULT_PAGE_SIZE;

        let path = file_node.file_path();
        let page_num = self
            .storage_manager
            .page_count(&path, page_size)
            .context(error::IoSnafu)? as u32;

        self.storage_manager
            .write(
                &path,
                page_num as u64 * page_size as u64,
                &vec![0; page_size],
            )
            .context(error::IoSnafu)?;

        let tag = PageTag {
            file_node: file_node.clone(),
            page_num,
        };

        let (id, page) = self.reuse_page(&tag)?;
        self.tag_table.borrow_mut().insert(tag, id);

        Ok(page)
    }

    pub fn fetch_page(&self, tag: PageTag) -> Result<BufferRef> {
        if let Some(id) = self.get_buffer_id(&tag) {
            let page = self.get_buffer(id);
            self.replacer.borrow_mut().pin(id);

            return Ok(page);
        }

        let (id, mut page) = self.reuse_page(&tag)?;

        self.storage_manager
            .read(
                &tag.file_node.file_path(),
                tag.page_num as u64 * DEFAULT_PAGE_SIZE as u64,
                page.as_slice_mut(),
            )
            .context(error::IoSnafu)?;

        self.tag_table.borrow_mut().insert(tag, id);

        Ok(page)
    }

    fn flush_page(&self, id: BufferId) -> Result<()> {
        let desc = self.descriptors.get(id).unwrap().borrow();
        if !desc.is_dirty() {
            return Ok(());
        }

        let page_tag = desc.page_tag.as_ref().unwrap();

        let data = self.buffer_slice(id);

        self.storage_manager
            .write(
                &page_tag.file_node.file_path(),
                page_tag.page_num as u64 * DEFAULT_PAGE_SIZE as u64,
                data,
            )
            .context(error::IoSnafu)

        // TODO: unset dirty flag
    }

    pub fn flush_pages(&self) -> Result<()> {
        self.descriptors
            .iter()
            .map(|desc| {
                let buf_id = desc.borrow().buffer_id;
                self.flush_page(buf_id)
            })
            .collect()
    }

    pub(super) fn get_buffer_id(&self, page_tag: &PageTag) -> Option<BufferId> {
        self.tag_table.borrow().get(page_tag).map(ToOwned::to_owned)
    }

    pub(super) fn get_buffer(&self, id: BufferId) -> BufferRef {
        let desc = self.descriptors.get(id).unwrap();

        BufferRef::new(
            desc.clone(),
            unsafe {
                NonNull::new_unchecked(self.buffers.as_ptr().add(id * self.buffer_size).cast_mut())
            },
            self.buffer_size,
        )
    }

    fn buffer_slice(&self, id: BufferId) -> &[u8] {
        &self.buffers[id * self.buffer_size..(id + 1) * self.buffer_size]
    }
}

#[derive(Debug)]
struct BufferDescriptor {
    page_tag: Option<PageTag>,
    buffer_id: BufferId,
    next_free: i32,
    state: AtomicU32,
    content_lock: RwLock<()>,
}

impl BufferDescriptor {
    fn register(&mut self, tag: PageTag) {
        self.page_tag = Some(tag)
    }

    fn is_dirty(&self) -> bool {
        let state = self.state.load(Ordering::SeqCst);
        state == 1
    }

    fn set_dirty(&mut self, dirty: bool) {
        let state = if dirty { 1 } else { 0 };
        self.state.swap(state, Ordering::SeqCst);
    }

    fn pin(&self, write: bool) {}
}

#[derive(Debug)]
pub struct BufferRef<'a> {
    // id: BufferId,
    desc: Arc<RefCell<BufferDescriptor>>,
    ptr: NonNull<u8>,
    page_size: usize,
    _phantom: PhantomData<&'a ()>,
}

impl BufferRef<'_> {
    fn new(desc: Arc<RefCell<BufferDescriptor>>, ptr: NonNull<u8>, page_size: usize) -> Self {
        // TODO: pin

        Self {
            desc,
            ptr,
            page_size,
            _phantom: PhantomData,
        }
    }

    pub fn page_num(&self) -> PageNum {
        self.desc.borrow().page_tag.as_ref().unwrap().page_num
    }

    pub fn set_dirty(&self) {
        self.desc.borrow_mut().set_dirty(true);
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.page_size) }
    }

    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.page_size) }
    }

    fn reset(&self) {
        unsafe { self.ptr.as_ptr().write_bytes(0, self.page_size) }
    }
}

impl Drop for BufferRef<'_> {
    fn drop(&mut self) {
        // TODO: unpin
    }
}

// #[cfg(test)]
// mod tests {
//     use std::io::Read;

//     use {super::*, crate::buffer::LruReplacer, tempfile::NamedTempFile};

// #[test]
// fn it_works() {
//     let (mut file, path) = NamedTempFile::new().unwrap().into_parts();

//     let size = NonZeroUsize::new(10).unwrap();
//     let replacer = LruReplacer::new(size);
//     let mut manager = BufferManager::new(size, replacer);

//     let page_id = {
//         let page = manager.new_page().unwrap();

//         let mut page = page.borrow_mut();
//         page.id = 0;

//         let data = page.data_mut();
//         data.fill(2);

//         page.is_dirty = true;

//         page.id
//     };

//     manager.flush_page(page_id).unwrap();

//     let mut buf = vec![0u8; PAGE_SIZE];
//     file.read_exact(&mut buf).unwrap();

//     assert_eq!(&buf, &vec![2; PAGE_SIZE]);
// }
// }
