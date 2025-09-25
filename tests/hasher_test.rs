use coat_check::hasher::hash_key;

#[test]
fn key_hashing_is_deterministic() {
    let hashed = hash_key("coat check");
    assert_eq!(hashed, "20b63d38d5ea19e7de07c2f80f255546");
}

#[test]
fn key_hashing_is_case_sensitive() {
    let hashed_one = hash_key("coat check");
    let hashed_two = hash_key("Coat CHECK");
    assert_ne!(hashed_one, hashed_two);
}
