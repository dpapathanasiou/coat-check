use chrono::Utc;

pub fn generate_test_file(n: i32) -> String {
    format!(
        "/tmp/test-{}-{n}.coat-check",
        Utc::now().timestamp().to_string()
    )
}
