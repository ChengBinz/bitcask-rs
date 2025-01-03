use bytes::Bytes;
use std::path::PathBuf;

use crate::{
    db::Engine,
    errors::Errors,
    options::Options,
    util::rand_kv::{get_test_key, get_test_value},
};

#[test]
fn test_engine_put() {
    let mut opts = Options::default();
    opts.dir_path = PathBuf::from("/tmp/bitcask-rs-put");
    opts.data_file_size = 64 * 1024 * 1024;
    let engine = Engine::open(opts.clone()).expect("failed to open engine");

    // 1.正常 Put 一条数据
    let res1 = engine.put(get_test_key(11), get_test_value(11));
    assert!(res1.is_ok());
    let res2 = engine.get(get_test_key(11));
    assert!(res2.is_ok());
    assert!(res2.unwrap().len() > 0);

    // 2.重复 Put key 相同的数据
    let res3 = engine.put(get_test_key(22), get_test_value(22));
    assert!(res3.is_ok());
    let res4 = engine.put(get_test_key(22), Bytes::from("a new value"));
    assert!(res4.is_ok());
    let res5 = engine.get(get_test_key(22));
    assert!(res5.is_ok());
    assert_eq!(res5.unwrap(), Bytes::from("a new value"));

    // 3.key 为空
    let res6 = engine.put(Bytes::new(), get_test_value(123));
    assert_eq!(Errors::KeyIsEmpty, res6.err().unwrap());

    // 4.value 为空
    let res7 = engine.put(get_test_key(33), Bytes::new());
    assert!(res7.is_ok());
    let res8 = engine.get(get_test_key(33));
    assert_eq!(0, res8.ok().unwrap().len());

    // 5.写到数据文件进行了转换
    for i in 0..=1000000 {
        let res = engine.put(get_test_key(i), get_test_value(i));
        assert!(res.is_ok());
    }

    // 6.重启后再 Put 数据
    std::mem::drop(engine);

    let engine2 = Engine::open(opts.clone()).expect("failed to open engine");
    let res9 = engine2.put(get_test_key(55), get_test_value(55));
    assert!(res9.is_ok());

    let res10 = engine2.get(get_test_key(55));
    assert_eq!(res10.unwrap(), get_test_value(55));

    // 删除测试的文件夹
    std::fs::remove_dir_all(opts.clone().dir_path).expect("failed to remove path");
}

#[test]
fn test_engine_get() {
    let mut opts = Options::default();
    opts.dir_path = PathBuf::from("/tmp/bitcask-rs-get");
    opts.data_file_size = 64 * 1024 * 1024;
    let engine = Engine::open(opts.clone()).expect("failed to open engine");

    // 1.正常读取一条数据
    let res1 = engine.put(get_test_key(111), get_test_value(111));
    assert!(res1.is_ok());
    let res2 = engine.get(get_test_key(111));
    assert!(res2.is_ok());
    assert!(res2.unwrap().len() > 0);

    // 2.读取一个不存在的 key
    let res3 = engine.get(Bytes::from("not existed key"));
    assert_eq!(Errors::KeyNotFound, res3.err().unwrap());

    // 3.值被重复 Put 后在读取
    let res4 = engine.put(get_test_key(222), get_test_value(222));
    assert!(res4.is_ok());
    let res5 = engine.put(get_test_key(222), Bytes::from("a new value"));
    assert!(res5.is_ok());
    let res6 = engine.get(get_test_key(222));
    assert_eq!(Bytes::from("a new value"), res6.unwrap());

    // 4.值被删除后再 Get
    let res7 = engine.put(get_test_key(333), get_test_value(333));
    assert!(res7.is_ok());
    let res8 = engine.delete(get_test_key(333));
    assert!(res8.is_ok());
    let res9 = engine.get(get_test_key(333));
    assert_eq!(Errors::KeyNotFound, res9.err().unwrap());

    // 5.转换为了旧的数据文件，从旧的数据文件上获取 value
    for i in 500..=1000000 {
        let res = engine.put(get_test_key(i), get_test_value(i));
        assert!(res.is_ok());
    }
    let res10 = engine.get(get_test_key(505));
    assert_eq!(get_test_value(505), res10.unwrap());

    // 6.重启后，前面写入的数据都能拿到
    std::mem::drop(engine);

    let engine2 = Engine::open(opts.clone()).expect("failed to open engine");
    let res11 = engine2.get(get_test_key(111));
    assert_eq!(get_test_value(111), res11.unwrap());
    let res12 = engine2.get(get_test_key(222));
    assert_eq!(Bytes::from("a new value"), res12.unwrap());
    let res13 = engine2.get(get_test_key(333));
    assert_eq!(Errors::KeyNotFound, res13.err().unwrap());

    // 删除测试的文件夹
    std::fs::remove_dir_all(opts.clone().dir_path).expect("failed to remove path");
}

#[test]
fn test_engine_delete() {
    let mut opts = Options::default();
    opts.dir_path = PathBuf::from("/tmp/bitcask-rs-delete");
    opts.data_file_size = 64 * 1024 * 1024;
    let engine = Engine::open(opts.clone()).expect("failed to open engine");

    // 1.正常删除一个存在的 key
    let res1 = engine.put(get_test_key(111), get_test_value(111));
    assert!(res1.is_ok());
    let res2 = engine.delete(get_test_key(111));
    assert!(res2.is_ok());
    let res3 = engine.get(get_test_key(111));
    assert_eq!(Errors::KeyNotFound, res3.err().unwrap());

    // 2.删除一个不存在的 key
    let res4 = engine.delete(Bytes::from("not-existed-key"));
    assert!(res4.is_ok());

    // 3.删除一个空的 key
    let res5 = engine.delete(Bytes::new());
    assert_eq!(Errors::KeyIsEmpty, res5.err().unwrap());

    // 4.值被删除之后重新 Put
    let res6 = engine.put(get_test_key(222), get_test_value(222));
    assert!(res6.is_ok());
    let res7 = engine.delete(get_test_key(222));
    assert!(res7.is_ok());
    let res8 = engine.put(get_test_key(222), Bytes::from("a new value"));
    assert!(res8.is_ok());
    let res9 = engine.get(get_test_key(222));
    assert_eq!(Bytes::from("a new value"), res9.unwrap());

    // 5.重启后再 Put 数据
    std::mem::drop(engine);

    let engine2 = Engine::open(opts.clone()).expect("failed to open engine");
    let res10 = engine2.get(get_test_key(111));
    assert_eq!(Errors::KeyNotFound, res10.err().unwrap());
    let res11 = engine2.get(get_test_key(222));
    assert_eq!(Bytes::from("a new value"), res11.unwrap());

    // 删除测试的文件夹
    std::fs::remove_dir_all(opts.clone().dir_path).expect("failed to remove path");
}

#[test]
fn test_engine_close() {
    let mut opts = Options::default();
    opts.dir_path = PathBuf::from("/tmp/bitcask-rs-close");
    opts.data_file_size = 64 * 1024 * 1024;
    let engine = Engine::open(opts.clone()).expect("failed to open engine");

    let res1 = engine.put(get_test_key(222), get_test_value(222));
    assert!(res1.is_ok());

    let close_res = engine.close();
    assert!(close_res.is_ok());

    // 删除测试的文件夹
    std::fs::remove_dir_all(opts.clone().dir_path).expect("failed to remove path");
}

#[test]
fn test_engine_sync() {
    let mut opts = Options::default();
    opts.dir_path = PathBuf::from("/tmp/bitcask-rs-sync");
    opts.data_file_size = 64 * 1024 * 1024;
    let engine = Engine::open(opts.clone()).expect("failed to open engine");

    let res1 = engine.put(get_test_key(222), get_test_value(222));
    assert!(res1.is_ok());

    let close_res = engine.sync();
    assert!(close_res.is_ok());

    // 删除测试的文件夹
    std::fs::remove_dir_all(opts.clone().dir_path).expect("failed to remove path");
}

#[test]
fn test_engine_filelock() {
    let mut opts = Options::default();
    opts.dir_path = PathBuf::from("/tmp/bitcask-rs-flock");
    let engine = Engine::open(opts.clone()).expect("failed to open engine");

    let res1 = Engine::open(opts.clone());
    assert_eq!(res1.err().unwrap(), Errors::DatabaseIsUsing);

    let res2 = engine.close();
    assert!(res2.is_ok());

    let res3 = Engine::open(opts.clone());
    assert!(res3.is_ok());

    // 删除测试的文件夹
    std::fs::remove_dir_all(opts.clone().dir_path).expect("failed to remove path");
}

#[test]
fn test_engine_stat() {
    let mut opts = Options::default();
    opts.dir_path = PathBuf::from("/tmp/bitcask-rs-stat");
    let engine = Engine::open(opts.clone()).expect("failed to open engine");

    for i in 0..=10000 {
        let res = engine.put(get_test_key(i), get_test_value(i));
        assert!(res.is_ok());
    }
    for i in 0..=1000 {
        let res = engine.put(get_test_key(i), get_test_value(i));
        assert!(res.is_ok());
    }
    for i in 2000..=5000 {
        let res = engine.delete(get_test_key(i));
        assert!(res.is_ok());
    }

    let stat = engine.stat().unwrap();
    assert!(stat.reclaim_size > 0);

    // 删除测试的文件夹
    std::fs::remove_dir_all(opts.clone().dir_path).expect("failed to remove path");
}

#[test]
fn test_engine_backup() {
    let mut opts = Options::default();
    opts.dir_path = PathBuf::from("/tmp/bitcask-rs-backup");
    let engine = Engine::open(opts.clone()).expect("failed to open engine");

    for i in 0..=10000 {
        let res = engine.put(get_test_key(i), get_test_value(i));
        assert!(res.is_ok());
    }

    let backup_dir = PathBuf::from("/tmp/bitcask-rs-backup-test");
    let bak_res = engine.backup(backup_dir.clone());
    assert!(bak_res.is_ok());

    let mut opts1 = Options::default();
    opts1.dir_path = backup_dir.clone();
    let eng2 = Engine::open(opts1);
    assert!(eng2.is_ok());

    // 删除测试的文件夹
    std::fs::remove_dir_all(opts.clone().dir_path).expect("failed to remove path");
    std::fs::remove_dir_all(backup_dir).expect("failed to remove path");
}
