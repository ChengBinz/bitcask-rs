pub mod btree;
use crate::data::log_record::LogRecordPos;

/// Indexer 抽象索引接口，后续如果想要接入其他的数据结构，则直接实现这个接口即可
pub trait Indexer: Sync + Send {
    /// 向索引中存储 key 对应的数据位置信息
    fn put(&self, key: Vec<u8>, pos: LogRecordPos) -> bool;

    /// 根据 key 取出对应的索引位置信息
    fn get(&self, key: Vec<u8>) -> Option<LogRecordPos>;

    /// 根据 key 删除对应的索引位置信息
    fn delete(&self, key: Vec<u8>) -> bool;
}
