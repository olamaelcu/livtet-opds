/// Read a fixture file from `vendor/opds-test-catalog/`.
///
/// Resolves `path` relative to `<crate manifest dir>/../../vendor/opds-test-catalog/`,
/// matching the workspace layout where the catalog vendor tree lives at
/// the repo root and each OPDS crate sits one level deep.
pub fn fixture(path: &str) -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let base = camino::Utf8Path::new(manifest_dir)
        .parent()
        .expect("crate manifest dir has parent")
        .join("vendor/opds-test-catalog");
    let full = base.join(path);
    std::fs::read_to_string(full.as_std_path())
        .unwrap_or_else(|e| panic!("read fixture {}: {e}", full))
}
