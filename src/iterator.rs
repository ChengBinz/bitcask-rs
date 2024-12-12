use std::sync::Arc;

use bytes::Bytes;
use parking_lot::RwLock;

use crate::{db::Engine, errors::Result, index::IndexIterator, options::IteratorOptions};

/// 迭代器接口
pub struct Iterator<'a> {
    index_iter: Arc<RwLock<Box<dyn IndexIterator>>>, // 索引迭代器
    engine: &'a Engine,
}

impl Engine {
    /// 获取迭代器
    pub fn iter(&self, options: IteratorOptions) -> Iterator {
        Iterator {
            index_iter: Arc::new(RwLock::new(self.index.iterator(options))),
            engine: self,
        }
    }

    /// 返回数据库中所有的 key
    pub fn list_keys(&self) -> Result<Vec<Bytes>> {
        self.index.list_keys()
    }

    /// 对数据库当中的所有数据执行函数操作，函数返回 false 时终止
    pub fn fold<F>(&self, f: F) -> Result<()>
    where
        Self: Sized,
        F: Fn(Bytes, Bytes) -> bool,
    {
        let iter = self.iter(IteratorOptions::default());
        while let Some((key, value)) = iter.next() {
            if !f(key, value) {
                break;
            }
        }
        Ok(())
    }
}

impl Iterator<'_> {
    /// Rewind 重新回到迭代器的起点，即第一个数据
    pub fn rewind(&self) {
        let mut index_iter = self.index_iter.write();
        index_iter.rewind();
    }

    /// Seek 根据传入的 key 查找到第一个大于（或小于）等于的目标 key，从这个 key 开始遍历
    pub fn seek(&self, key: Vec<u8>) {
        let mut index_iter = self.index_iter.write();
        index_iter.seek(key);
    }

    /// Next 跳转到下一个 key，返回 None 则说明迭代完毕
    pub fn next(&self) -> Option<(Bytes, Bytes)> {
        let mut index_iter = self.index_iter.write();
        if let Some(item) = index_iter.next() {
            let value = self
                .engine
                .get_value_by_position(item.1)
                .expect("failed to get value from data file");
            return Some((Bytes::from(item.0.to_vec()), value));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{options::Options, util};

    use super::*;

    #[test]
    fn test_list_keys() {
        let mut opts = Options::default();
        opts.dir_path = PathBuf::from("/tmp/bit-cask-rs-list-keys");
        let engine = Engine::open(opts.clone()).expect("failed to open engine");

        let keys1 = engine.list_keys();
        assert_eq!(keys1.ok().unwrap().len(), 0);

        let put_res1 = engine.put(Bytes::from("aacc"), util::rand_kv::get_test_value(10));
        assert!(put_res1.is_ok());
        let put_res2 = engine.put(Bytes::from("eecc"), util::rand_kv::get_test_value(10));
        assert!(put_res2.is_ok());
        let put_res3 = engine.put(Bytes::from("bbac"), util::rand_kv::get_test_value(10));
        assert!(put_res3.is_ok());
        let put_res4 = engine.put(Bytes::from("ccde"), util::rand_kv::get_test_value(10));
        assert!(put_res4.is_ok());

        let keys2 = engine.list_keys();
        assert_eq!(keys2.ok().unwrap().len(), 4);

        // 删除测试的文件夹
        std::fs::remove_dir_all(opts.clone().dir_path).expect("failed to remove path");
    }

    #[test]
    fn test_fold() {
        let mut opts = Options::default();
        opts.dir_path = PathBuf::from("/tmp/bit-cask-rs-fold");
        let engine = Engine::open(opts.clone()).expect("failed to open engine");

        let put_res1 = engine.put(Bytes::from("aacc"), util::rand_kv::get_test_value(10));
        assert!(put_res1.is_ok());
        let put_res2 = engine.put(Bytes::from("eecc"), util::rand_kv::get_test_value(10));
        assert!(put_res2.is_ok());
        let put_res3 = engine.put(Bytes::from("bbac"), util::rand_kv::get_test_value(10));
        assert!(put_res3.is_ok());
        let put_res4 = engine.put(Bytes::from("ccde"), util::rand_kv::get_test_value(10));
        assert!(put_res4.is_ok());

        engine
            .fold(|key, value| {
                assert!(key.len() > 0);
                assert!(value.len() > 0);
                return true;
            })
            .unwrap();

        // 删除测试的文件夹
        std::fs::remove_dir_all(opts.clone().dir_path).expect("failed to remove path");
    }

    #[test]
    fn test_iterator_seek() {
        let mut opts = Options::default();
        opts.dir_path = PathBuf::from("/tmp/bit-cask-rs-iter-seek");
        let engine = Engine::open(opts.clone()).expect("failed to open engine");

        // 没有数据的情况
        let iter1 = engine.iter(IteratorOptions::default());
        iter1.seek("aa".as_bytes().to_vec());
        assert!(iter1.next().is_none());

        // 有一条数据的情况
        let put_res1 = engine.put(Bytes::from("aacc"), util::rand_kv::get_test_value(10));
        assert!(put_res1.is_ok());
        let iter2 = engine.iter(IteratorOptions::default());
        iter2.seek("a".as_bytes().to_vec());
        assert!(iter2.next().is_some());

        // 有多条数据的情况
        let put_res2 = engine.put(Bytes::from("eecc"), util::rand_kv::get_test_value(10));
        assert!(put_res2.is_ok());
        let put_res3 = engine.put(Bytes::from("bbac"), util::rand_kv::get_test_value(10));
        assert!(put_res3.is_ok());
        let put_res4 = engine.put(Bytes::from("ccde"), util::rand_kv::get_test_value(10));
        assert!(put_res4.is_ok());

        let iter3 = engine.iter(IteratorOptions::default());
        iter3.seek("a".as_bytes().to_vec());
        assert_eq!(Bytes::from("aacc"), iter3.next().unwrap().0);

        // 删除测试的文件夹
        std::fs::remove_dir_all(opts.clone().dir_path).expect("failed to remove path");
    }

    #[test]
    fn test_iterator_next() {
        let mut opts = Options::default();
        opts.dir_path = PathBuf::from("/tmp/bit-cask-rs-iter-next");
        let engine = Engine::open(opts.clone()).expect("failed to open engine");

        // 有一条数据的情况
        let put_res1 = engine.put(Bytes::from("eecc"), util::rand_kv::get_test_value(10));
        assert!(put_res1.is_ok());
        let iter1 = engine.iter(IteratorOptions::default());
        assert!(iter1.next().is_some());
        iter1.rewind();
        assert!(iter1.next().is_some());
        assert!(iter1.next().is_none());

        // 有多条数据的情况
        let put_res2 = engine.put(Bytes::from("aade"), util::rand_kv::get_test_value(10));
        assert!(put_res2.is_ok());
        let put_res3 = engine.put(Bytes::from("ddce"), util::rand_kv::get_test_value(10));
        assert!(put_res3.is_ok());
        let put_res4 = engine.put(Bytes::from("bbcc"), util::rand_kv::get_test_value(10));
        assert!(put_res4.is_ok());

        let mut iter_opts1 = IteratorOptions::default();
        iter_opts1.reverse = true;
        let iter2 = engine.iter(iter_opts1);
        while let Some(item) = iter2.next() {
            assert!(item.0.len() > 0);
        }

        // 删除测试的文件夹
        std::fs::remove_dir_all(opts.clone().dir_path).expect("failed to remove path");
    }

    #[test]
    fn test_iterator_prefix() {
        let mut opts = Options::default();
        opts.dir_path = PathBuf::from("/tmp/bit-cask-rs-iter-prefix");
        let engine = Engine::open(opts.clone()).expect("failed to open engine");

        let put_res1 = engine.put(Bytes::from("eecc"), util::rand_kv::get_test_value(10));
        assert!(put_res1.is_ok());
        let put_res2 = engine.put(Bytes::from("aade"), util::rand_kv::get_test_value(10));
        assert!(put_res2.is_ok());
        let put_res3 = engine.put(Bytes::from("ddce"), util::rand_kv::get_test_value(10));
        assert!(put_res3.is_ok());
        let put_res4 = engine.put(Bytes::from("bbcc"), util::rand_kv::get_test_value(10));
        assert!(put_res4.is_ok());
        let put_res4 = engine.put(Bytes::from("ddaa"), util::rand_kv::get_test_value(10));
        assert!(put_res4.is_ok());

        let mut iter_opt1 = IteratorOptions::default();
        iter_opt1.prefix = "dd".as_bytes().to_vec();
        let iter1 = engine.iter(iter_opt1);
        while let Some(item) = iter1.next() {
            assert!(item.0.len() > 0);
        }
        // 删除测试的文件夹
        std::fs::remove_dir_all(opts.clone().dir_path).expect("failed to remove path");
    }
}
