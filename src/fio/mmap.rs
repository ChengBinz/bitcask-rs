use std::{fs::OpenOptions, path::PathBuf, sync::Arc};

use log::error;
use memmap2::Mmap;
use parking_lot::Mutex;

use crate::errors::{Errors, Result};

use super::IOManager;

#[derive(Debug)]
pub struct MMapIO {
    map: Arc<Mutex<Mmap>>,
}

impl MMapIO {
    pub fn new(file_name: PathBuf) -> Result<Self> {
        match OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(file_name)
        {
            Ok(file) => {
                let map = unsafe { Mmap::map(&file).expect("failed to map the file") };
                return Ok(MMapIO {
                    map: Arc::new(Mutex::new(map)),
                });
            }
            Err(e) => {
                error!("failed to open data file: {}", e);
                println!("{:?}", e);
                return Err(Errors::FailedToOpenDataFile);
            }
        }
    }
}

impl IOManager for MMapIO {
    fn write(&self, _buf: &[u8]) -> Result<usize> {
        unimplemented!()
    }

    fn read(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        let map_arr = self.map.lock();
        let end = offset + buf.len() as u64;
        if end > map_arr.len() as u64 {
            return Err(Errors::ReadDataFileEOF);
        }
        let val = &map_arr[offset as usize..end as usize];
        buf.copy_from_slice(val);

        Ok(val.len())
    }

    fn sync(&self) -> Result<()> {
        unimplemented!()
    }

    fn size(&self) -> u64 {
        let map_arr = self.map.lock();
        map_arr.len() as u64
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::fio::file_io::FileIO;

    use super::*;

    #[test]
    fn test_mmap_read() {
        let path = PathBuf::from("/tmp/mmap-test.data");

        // 文件为空
        let mmap_res1 = MMapIO::new(path.clone());
        assert!(mmap_res1.is_ok());
        let mmap_io1 = mmap_res1.ok().unwrap();
        let mut buf1 = [0u8; 10];
        let read_res1 = mmap_io1.read(&mut buf1, 0);
        assert_eq!(read_res1.err().unwrap(), Errors::ReadDataFileEOF);

        let fio_res = FileIO::new(path.clone());
        assert!(fio_res.is_ok());
        let fio = fio_res.ok().unwrap();
        fio.write(b"aa").unwrap();
        fio.write(b"bb").unwrap();
        fio.write(b"cc").unwrap();

        // 有数据的情况
        let mmap_res2 = MMapIO::new(path.clone());
        assert!(mmap_res2.is_ok());
        let mmap_io2 = mmap_res2.ok().unwrap();
        let mut buf2 = [0u8; 3];
        let read_res2 = mmap_io2.read(&mut buf2, 0);
        assert!(read_res2.is_ok());

        let remove_res = fs::remove_file(path.clone());
        assert!(remove_res.is_ok());
    }
}
