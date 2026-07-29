/// Read a fixture file from `vendor/opds-test-catalog/`.
///
/// Delegates to `livtet-test-utils` to avoid duplicating the path
/// resolution logic across OPDS crates.
pub fn fixture(path: &str) -> String {
    livtet_test_utils::fixture(path)
}
