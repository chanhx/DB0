use std::{
    collections::{hash_map::Entry, HashMap},
    fs::{self, File, OpenOptions},
    io::{Read, Result, Seek, SeekFrom, Write},
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
};

const O_DIRECT: i32 = 0o0040000;

pub(crate) struct FileManager {
    dir: PathBuf,
    opened_files: HashMap<PathBuf, File>,
}

impl FileManager {
    pub fn new(dir: &Path) -> Result<Self> {
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }

        Ok(Self {
            dir: dir.to_path_buf(),
            opened_files: HashMap::new(),
        })
    }

    fn get_file(&mut self, file_path: &Path) -> Result<&mut File> {
        let entry = self.opened_files.entry(file_path.to_path_buf());

        Ok(match entry {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let file = OpenOptions::new()
                    .create(false)
                    .read(true)
                    .write(true)
                    .custom_flags(O_DIRECT)
                    .open(self.dir.join(file_path))?;

                entry.insert(file)
            }
        })
    }

    pub fn read(&mut self, file_path: &Path, offset: u64, data: &mut [u8]) -> Result<()> {
        let file = self.get_file(file_path)?;

        file.seek(SeekFrom::Start(offset))?;
        file.read_exact(data)
    }

    pub fn write(&mut self, file_path: &Path, offset: u64, data: &[u8]) -> Result<()> {
        let file = self.get_file(file_path)?;

        file.seek(SeekFrom::Start(offset))?;
        file.write_all(data)
    }
}

#[cfg(test)]
mod tests {
    use {super::*, crate::PAGE_SIZE, tempfile::NamedTempFile};

    #[test]
    fn write_and_read() -> Result<()> {
        let path = NamedTempFile::new().unwrap().into_temp_path();
        let file_name = path.file_name().unwrap();

        let mut manager = FileManager::new(&path.parent().unwrap())?;

        let data_w = [123; PAGE_SIZE];
        let mut data_r = [0; PAGE_SIZE];

        manager.write(&PathBuf::from(file_name), 0, &data_w)?;
        manager.read(&PathBuf::from(file_name), 0, &mut data_r)?;

        assert_eq!(data_w, data_r);

        Ok(())
    }
}
