use std::path::PathBuf;

use bitcask_rs::{db::Engine, options::Options};
use bytes::Bytes;
use criterion::{criterion_group, criterion_main, Criterion};
use rand::Rng;

#[allow(dead_code)]
pub fn get_test_key(i: u32) -> Bytes {
    Bytes::from(std::format!("bitcask-rs-key-{:09}", i))
}

#[allow(dead_code)]
pub fn get_test_value(i: u32) -> Bytes {
    Bytes::from(std::format!(
        "bitcask-rs-value-value-value-value-value-value-value-value-value-{:1009}",
        i
    ))
}

fn benchmark_put(c: &mut Criterion) {
    // 打开存储引擎
    let mut options = Options::default();
    options.dir_path = PathBuf::from("/tmp/bitcask-rs-bench");
    let engine = Engine::open(options).unwrap();

    let mut rnd: rand::rngs::ThreadRng = rand::thread_rng();

    c.bench_function("bitcask-put-bench", |b| {
        b.iter(|| {
            let i = rnd.gen_range(0..std::u32::MAX);
            let res = engine.put(get_test_key(i), get_test_value(i));
            assert!(res.is_ok());
        })
    });
}

fn benchmark_get(c: &mut Criterion) {
    // 打开存储引擎
    let mut options = Options::default();
    options.dir_path = PathBuf::from("/tmp/bitcask-rs-bench");
    let engine = Engine::open(options).unwrap();

    for i in 0..100000 {
        let res = engine.put(get_test_key(i), get_test_value(i));
        assert!(res.is_ok());
    }

    let mut rnd: rand::rngs::ThreadRng = rand::thread_rng();

    c.bench_function("bitcask-get-bench", |b| {
        b.iter(|| {
            let i = rnd.gen_range(0..std::u32::MAX);
            engine.get(get_test_key(i));
        })
    });
}

fn benchmark_delete(c: &mut Criterion) {
    // 打开存储引擎
    let mut options = Options::default();
    options.dir_path = PathBuf::from("/tmp/bitcask-rs-bench");
    let engine = Engine::open(options).unwrap();

    for i in 0..100000 {
        let res = engine.put(get_test_key(i), get_test_value(i));
        assert!(res.is_ok());
    }

    let mut rnd: rand::rngs::ThreadRng = rand::thread_rng();

    c.bench_function("bitcask-delete-bench", |b| {
        b.iter(|| {
            let i = rnd.gen_range(0..std::u32::MAX);
            let res = engine.delete(get_test_key(i));
            assert!(res.is_ok());
        })
    });
}

criterion_group!(benches, benchmark_put, benchmark_get, benchmark_delete);
criterion_main!(benches);
