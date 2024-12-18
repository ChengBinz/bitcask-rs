use std::{fs, io, path::PathBuf};

// 获取磁盘剩余空间容量
pub fn available_disk_size() -> u64 {
    if let Ok(size) = fs2::available_space(PathBuf::from("/")) {
        return size;
    }
    0
}

// 获取数据目录的大小
pub fn dir_disk_size(dir_path: PathBuf) -> u64 {
    if let Ok(size) = fs_extra::dir::get_size(dir_path) {
        return size;
    }
    0
}

#[test]
fn test_available_disk_size() {
    let size = available_disk_size();
    assert!(size > 0);
}
