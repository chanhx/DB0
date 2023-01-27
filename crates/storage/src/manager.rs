use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::{Read, Result, Seek, SeekFrom, Write},
    os::unix::{fs::OpenOptionsExt, prelude::MetadataExt},
    path::{Path, PathBuf},
};

const O_DIRECT: i32 = 0o0040000;

pub(crate) struct StorageManager {
    data_dir: PathBuf,
    opened_files: RefCell<HashMap<PathBuf, File>>,
}

impl StorageManager {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            data_dir,
            opened_files: RefCell::new(HashMap::new()),
        }
    }

    fn open_file(&self, path: &Path, create: bool) -> Result<RefMut<'_, File>> {
        let path = self.data_dir.join(path);
        let file = RefMut::filter_map(self.opened_files.borrow_mut(), |files| files.get_mut(&path));

        Ok(match file {
            Ok(file) => file,
            Err(files) => {
                if create && let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)?;
                }

                let file = OpenOptions::new()
                    .create(create)
                    .read(true)
                    .write(true)
                    .custom_flags(O_DIRECT)
                    .open(&path)?;

                RefMut::map(files, |files| files.entry(path).or_insert(file))
            }
        })
    }

    pub fn read(&self, file_path: &Path, offset: u64, data: &mut [u8]) -> Result<()> {
        let mut file = self.open_file(file_path, false)?;

        file.seek(SeekFrom::Start(offset))?;
        file.read_exact(data)
    }

    pub fn write(&self, file_path: &Path, offset: u64, data: &[u8]) -> Result<()> {
        let mut file = self.open_file(file_path, false)?;

        file.seek(SeekFrom::Start(offset))?;
        file.write_all(data)
    }

    pub fn page_count(&self, file_path: &Path, page_size: usize) -> Result<usize> {
        let file = self.open_file(file_path, true)?;

        Ok(file.metadata()?.size() as usize / page_size)
    }
}

#[cfg(test)]
mod tests {
    use {super::*, crate::DEFAULT_PAGE_SIZE, tempfile::NamedTempFile};

    #[test]
    fn write_and_read() -> Result<()> {
        let path = NamedTempFile::new().unwrap().into_temp_path();
        let file_name = PathBuf::from(path.file_name().unwrap());

        let manager = StorageManager::new(path.parent().unwrap().to_path_buf());

        let data_w = [123; DEFAULT_PAGE_SIZE];
        let mut data_r = [0; DEFAULT_PAGE_SIZE];

        manager.write(&file_name, 0, &data_w)?;
        manager.read(&file_name, 0, &mut data_r)?;

        assert_eq!(data_w, data_r);

        Ok(())
    }
}
