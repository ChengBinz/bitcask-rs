pub mod bptree;
pub mod btree;
pub mod skiplist;
use std::path::PathBuf;

use bytes::Bytes;

use crate::{
    data::log_record::LogRecordPos,
    errors::Result,
    options::{IndexType, IteratorOptions},
};

/// Indexer 抽象索引接口，后续如果想要接入其他的数据结构，则直接实现这个接口即可
pub trait Indexer: Sync + Send {
    /// 向索引中存储 key 对应的数据位置信息
    fn put(&self, key: Vec<u8>, pos: LogRecordPos) -> Option<LogRecordPos>;

    /// 根据 key 取出对应的索引位置信息
    fn get(&self, key: Vec<u8>) -> Option<LogRecordPos>;

    /// 根据 key 删除对应的索引位置信息
    fn delete(&self, key: Vec<u8>) -> Option<LogRecordPos>;

    /// 获取索引存储的所有 key
    fn list_keys(&self) -> Result<Vec<Bytes>>;

    /// 返回索引迭代器
    fn iterator(&self, options: IteratorOptions) -> Box<dyn IndexIterator>;
}

/// 根据类型打开内存索引
pub fn new_indexer(index_type: IndexType, dir_path: PathBuf) -> Box<dyn Indexer> {
    match index_type {
        IndexType::BTree => Box::new(btree::BTree::new()),
        IndexType::SkipList => Box::new(skiplist::SkipList::new()),
        IndexType::BPlusTree => Box::new(bptree::BPlusTree::new(dir_path)),
    }
}

/// 抽象索引迭代器
pub trait IndexIterator: Sync + Send {
    /// Rewind 重新回到迭代器的起点，即第一个数据
    fn rewind(&mut self);

    /// Seek 根据传入的 key 查找到第一个大于（或小于）等于的目标 key，从这个 key 开始遍历
    fn seek(&mut self, key: Vec<u8>);

    /// Next 跳转到下一个 key，返回 None 则说明迭代完毕
    fn next(&mut self) -> Option<(&Vec<u8>, &LogRecordPos)>;
}
