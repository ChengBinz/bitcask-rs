pub mod file_io;
pub mod mmap;

use std::path::PathBuf;

use file_io::FileIO;
use mmap::MMapIO;

use crate::{errors::Result, options::IOType};

/// 抽象 IO 管理接口，可以接入不同的 IO 类型，目前支持标准文件 IO
pub trait IOManager: Sync + Send {
    /// 写入字节数组到文件中
    fn write(&self, buf: &[u8]) -> Result<usize>;

    /// 从文件的给定位置读取对应数据
    fn read(&self, buf: &mut [u8], offset: u64) -> Result<usize>;

    /// 持久化数据
    fn sync(&self) -> Result<()>;

    /// 获取文件的大小
    fn size(&self) -> u64;
}

/// 根据文件名称初始化 IOManager
pub fn new_io_manager(file_name: PathBuf, io_type: IOType) -> Box<dyn IOManager> {
    match io_type {
        IOType::StandardFIO => Box::new(FileIO::new(file_name).unwrap()),
        IOType::MemoryMap => Box::new(MMapIO::new(file_name).unwrap()),
    }
}
