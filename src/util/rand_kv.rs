use bytes::Bytes;

#[allow(dead_code)]
pub fn get_test_key(i: usize) -> Bytes {
    Bytes::from(std::format!("bitcask-rs-key-{:09}", i))
}

#[allow(dead_code)]
pub fn get_test_value(i: usize) -> Bytes {
    Bytes::from(std::format!("bitcask-rs-value-{:09}", i))
}

#[test]
fn test_get_test_key_value() {
    for i in 0..=10 {
        assert!(get_test_key(i).len() > 0)
    }

    for i in 0..=10 {
        assert!(get_test_value(i).len() > 0)
    }
}
