use {
    super::{PageId, PAGE_SIZE},
    std::{
        fs::{File, OpenOptions},
        io::{Read, Result, Seek, SeekFrom, Write},
        os::unix::fs::OpenOptionsExt,
        path::Path,
    },
};

pub struct DiskManager {
    db_file: File,
}

const O_DIRECT: i32 = 0o0040000; // Double check value

impl DiskManager {
    pub fn new(path: &Path) -> Result<Self> {
        let db_file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .custom_flags(O_DIRECT)
            .open(path)?;

        Ok(Self { db_file })
    }

    pub fn read_page(&mut self, id: &PageId, data: &mut [u8]) -> Result<()> {
        let offset = id * PAGE_SIZE as u64;

        self.db_file.seek(SeekFrom::Start(offset))?;
        self.db_file.read_exact(data)
    }

    pub fn write_page(&mut self, id: &PageId, data: &[u8]) -> Result<()> {
        let offset = id * PAGE_SIZE as u64;

        self.db_file.seek(SeekFrom::Start(offset))?;
        self.db_file.write_all(data)
    }
}
