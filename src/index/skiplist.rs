use std::sync::Arc;

use bytes::Bytes;
use crossbeam_skiplist::SkipMap;

use crate::{data::log_record::LogRecordPos, options::IteratorOptions};

use super::{IndexIterator, Indexer};

/// 实现跳表索引
pub struct SkipList {
    skl: Arc<SkipMap<Vec<u8>, LogRecordPos>>,
}

impl SkipList {
    pub fn new() -> Self {
        Self {
            skl: Arc::new(SkipMap::new()),
        }
    }
}

impl Indexer for SkipList {
    fn put(&self, key: Vec<u8>, pos: LogRecordPos) -> Option<LogRecordPos> {
        let mut result = None;
        if let Some(entry) = self.skl.get(&key) {
            result = Some(*entry.value());
        }
        self.skl.insert(key, pos);
        result
    }

    fn get(&self, key: Vec<u8>) -> Option<LogRecordPos> {
        if let Some(entry) = self.skl.get(&key) {
            return Some(*entry.value());
        }
        None
    }

    fn delete(&self, key: Vec<u8>) -> Option<LogRecordPos> {
        if let Some(entry) = self.skl.remove(&key) {
            return Some(*entry.value());
        }
        None
    }

    fn list_keys(&self) -> crate::errors::Result<Vec<bytes::Bytes>> {
        let mut keys = Vec::with_capacity(self.skl.len());
        for e in self.skl.iter() {
            keys.push(Bytes::copy_from_slice(e.key()));
        }
        Ok(keys)
    }

    fn iterator(&self, options: crate::options::IteratorOptions) -> Box<dyn super::IndexIterator> {
        let mut items = Vec::with_capacity(self.skl.len());
        // 将 SkipList 中的数据存储到数组中
        for entry in self.skl.iter() {
            items.push((entry.key().clone(), *entry.value()));
        }
        if options.reverse {
            items.reverse();
        }
        Box::new(SkipListIterator {
            items,
            curr_index: 0,
            options,
        })
    }
}

/// 跳表索引迭代器
pub struct SkipListIterator {
    items: Vec<(Vec<u8>, LogRecordPos)>, // 存储 key+索引
    curr_index: usize,                   // 当前遍历的位置下标
    options: IteratorOptions,            // 配置项
}

impl IndexIterator for SkipListIterator {
    fn rewind(&mut self) {
        self.curr_index = 0;
    }

    fn seek(&mut self, key: Vec<u8>) {
        self.curr_index = match self.items.binary_search_by(|(x, _)| {
            if self.options.reverse {
                x.cmp(&key).reverse()
            } else {
                x.cmp(&key)
            }
        }) {
            Ok(equal_val) => equal_val,
            Err(insert_val) => insert_val,
        };
    }

    fn next(&mut self) -> Option<(&Vec<u8>, &LogRecordPos)> {
        if self.curr_index >= self.items.len() {
            return None;
        }

        while let Some(item) = self.items.get(self.curr_index) {
            self.curr_index += 1;
            let prefix = &self.options.prefix;
            if prefix.is_empty() || item.0.starts_with(&prefix) {
                return Some((&item.0, &item.1));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skl_put() {
        let skl = SkipList::new();
        let res1 = skl.put(
            "aacd".as_bytes().to_vec(),
            LogRecordPos {
                file_id: 1123,
                offset: 1232,
                size: 11,
            },
        );
        assert!(res1.is_none());
        let res2 = skl.put(
            "acdd".as_bytes().to_vec(),
            LogRecordPos {
                file_id: 1123,
                offset: 1232,
                size: 11,
            },
        );
        assert!(res2.is_none());
        let res3 = skl.put(
            "bbae".as_bytes().to_vec(),
            LogRecordPos {
                file_id: 1123,
                offset: 1232,
                size: 11,
            },
        );
        assert!(res3.is_none());
        let res4 = skl.put(
            "ddee".as_bytes().to_vec(),
            LogRecordPos {
                file_id: 1123,
                offset: 1232,
                size: 11,
            },
        );
        assert!(res4.is_none());

        let res5 = skl.put(
            "ddee".as_bytes().to_vec(),
            LogRecordPos {
                file_id: 93,
                offset: 22,
                size: 11,
            },
        );
        assert!(res5.is_some());
        let v = res5.unwrap();
        assert_eq!(v.file_id, 1123);
        assert_eq!(v.offset, 1232);
    }

    #[test]
    fn test_skl_get() {
        let skl = SkipList::new();

        let v1 = skl.get(b"not exists".to_vec());
        assert!(v1.is_none());

        let res1 = skl.put(
            "aacd".as_bytes().to_vec(),
            LogRecordPos {
                file_id: 1123,
                offset: 1232,
                size: 11,
            },
        );
        assert!(res1.is_none());
        let v2 = skl.get(b"aacd".to_vec());
        assert!(v2.is_some());

        let res2 = skl.put(
            "aacd".as_bytes().to_vec(),
            LogRecordPos {
                file_id: 11,
                offset: 990,
                size: 11,
            },
        );
        assert!(res2.is_some());
        let v3 = skl.get(b"aacd".to_vec());
        assert!(v3.is_some());
    }

    #[test]
    fn test_skl_delete() {
        let skl = SkipList::new();

        let r1 = skl.delete(b"not exists".to_vec());
        assert!(r1.is_none());

        let res1 = skl.put(
            "aacd".as_bytes().to_vec(),
            LogRecordPos {
                file_id: 1123,
                offset: 1232,
                size: 11,
            },
        );
        assert!(res1.is_none());

        let r2 = skl.delete(b"aacd".to_vec());
        assert!(r2.is_some());
        let v = r2.unwrap();
        assert_eq!(v.file_id, 1123);
        assert_eq!(v.offset, 1232);

        let v2 = skl.get(b"aacd".to_vec());
        assert!(v2.is_none());
    }

    #[test]
    fn test_skl_list_keys() {
        let skl = SkipList::new();

        let keys1 = skl.list_keys();
        assert_eq!(keys1.ok().unwrap().len(), 0);

        let res1 = skl.put(
            "aacd".as_bytes().to_vec(),
            LogRecordPos {
                file_id: 1123,
                offset: 1232,
                size: 11,
            },
        );
        assert!(res1.is_none());
        let res2 = skl.put(
            "acdd".as_bytes().to_vec(),
            LogRecordPos {
                file_id: 1123,
                offset: 1232,
                size: 11,
            },
        );
        assert!(res2.is_none());
        let res3 = skl.put(
            "bbae".as_bytes().to_vec(),
            LogRecordPos {
                file_id: 1123,
                offset: 1232,
                size: 11,
            },
        );
        assert!(res3.is_none());
        let res4 = skl.put(
            "ddee".as_bytes().to_vec(),
            LogRecordPos {
                file_id: 1123,
                offset: 1232,
                size: 11,
            },
        );
        assert!(res4.is_none());
        let keys2 = skl.list_keys();
        // println!("{:?}", keys2.ok().unwrap().len());
        assert_eq!(keys2.ok().unwrap().len(), 4);
    }

    #[test]
    fn test_skl_iterator() {
        let skl = SkipList::new();

        let res1 = skl.put(
            "aacd".as_bytes().to_vec(),
            LogRecordPos {
                file_id: 1123,
                offset: 1232,
                size: 11,
            },
        );
        assert!(res1.is_none());
        let res2 = skl.put(
            "acdd".as_bytes().to_vec(),
            LogRecordPos {
                file_id: 1123,
                offset: 1232,
                size: 11,
            },
        );
        assert!(res2.is_none());
        let res3 = skl.put(
            "bbae".as_bytes().to_vec(),
            LogRecordPos {
                file_id: 1123,
                offset: 1232,
                size: 11,
            },
        );
        assert!(res3.is_none());
        let res4 = skl.put(
            "ddee".as_bytes().to_vec(),
            LogRecordPos {
                file_id: 1123,
                offset: 1232,
                size: 11,
            },
        );
        assert!(res4.is_none());

        let mut opts = IteratorOptions::default();
        opts.reverse = true;
        let mut iter1 = skl.iterator(opts);

        while let Some((key, _)) = iter1.next() {
            assert!(!key.is_empty());
        }
    }
}
