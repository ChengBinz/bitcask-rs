use std::path::PathBuf;

#[derive(Clone)]
pub struct Options {
    //  数据库目录
    pub dir_path: PathBuf,

    // 数据文件大小
    pub data_file_size: u64,

    // 是否每次写都持久化
    pub sync_writes: bool,

    // 累计写到多少字节后进行持久化
    pub bytes_per_sync: usize,

    // 索引类型
    pub index_type: IndexType,

    // 是否用 mmap 打开数据库
    pub mmap_at_startup: bool,

    // 执行数据文件 merge 的阈值
    pub data_file_merge_ratio: f32,
}

#[derive(Clone, PartialEq)]
pub enum IndexType {
    /// BTree 索引
    BTree,

    /// 跳表索引
    SkipList,

    /// B+树索引，将索引存储到磁盘上
    BPlusTree,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            dir_path: std::env::temp_dir().join("bitcask-rs"),
            data_file_size: 256 * 1024 * 1024,
            sync_writes: false,
            bytes_per_sync: 0,
            index_type: IndexType::BTree,
            mmap_at_startup: true,
            data_file_merge_ratio: 0.5,
        }
    }
}

/// 索引迭代器配置项
pub struct IteratorOptions {
    pub prefix: Vec<u8>,
    pub reverse: bool,
}

impl Default for IteratorOptions {
    fn default() -> Self {
        Self {
            prefix: Default::default(),
            reverse: false,
        }
    }
}

/// 索引迭代器配置项
pub struct WriteBatchOptions {
    // 一个批次当中最大的数据量
    pub max_batch_num: usize,

    // 提交的时候是否进行 sync 持久化
    pub sync_writes: bool,
}

impl Default for WriteBatchOptions {
    fn default() -> Self {
        Self {
            max_batch_num: 10000,
            sync_writes: true,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum IOType {
    // 标准文件 IO
    StandardFIO,

    // 内存文件映射
    MemoryMap,
}
