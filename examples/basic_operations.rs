use bitcask_rs::{db, options::Options};
use bytes::Bytes;

fn main() {
    let opts = Options::default();
    let engine = db::Engine::open(opts).expect("failed to open bitcask engine");

    let res1 = engine.put(Bytes::from("name"), Bytes::from("bitcask-rs"));
    assert!(res1.is_ok());

    let res2 = engine.get(Bytes::from("name"));
    assert!(res2.is_ok());
    let val = res2.ok().unwrap();
    println!("val = {:?}", String::from_utf8(val.to_vec()));

    let res3 = engine.delete(Bytes::from("value"));
    assert!(res3.is_ok());
}
