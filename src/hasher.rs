use md5::{Digest, Md5};

pub fn hash_key(key: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(key.as_bytes());
    let result = hasher.finalize();
    String::from(format!("{:x}", result))
}
