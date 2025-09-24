use coat_check::hasher::hash_key;

#[test]
fn key_hashing_is_deterministic() {
    let hashed = hash_key("coat check");
    assert_eq!(hashed, "20b63d38d5ea19e7de07c2f80f255546");
}
