//! Golden file capture and comparison utilities.
//!
//! Golden files record the exact output of driver methods against the seeded test
//! database. They serve as the parity contract: the plugin must produce output that
//! matches these files.
//!
//! # Regenerating golden files
//!
//! ```bash
//! cd src-tauri
//! REGENERATE_GOLDEN=1 cargo test --test postgres_integration golden -- --include-ignored --test-threads=1
//! ```

use serde::Serialize;
use std::path::{Path, PathBuf};

/// Directory where golden files are stored (relative to the test binary's CWD).
fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("postgres_integration")
        .join("golden")
}

/// Write a golden file (only when REGENERATE_GOLDEN=1 is set).
pub fn write_golden<T: Serialize>(filename: &str, data: &T) {
    if std::env::var("REGENERATE_GOLDEN").unwrap_or_default() != "1" {
        return;
    }
    let path = golden_dir().join(filename);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create golden dir");
    }
    let json = serde_json::to_string_pretty(data).expect("serialize golden data");
    std::fs::write(&path, format!("{}\n", json))
        .unwrap_or_else(|e| panic!("write golden file {:?}: {}", path, e));
    eprintln!("  [golden] wrote {}", path.display());
}

/// Assert that the given data matches the golden file exactly.
/// Panics if the golden file doesn't exist — a missing fixture in a normal
/// (non-regeneration) run means a golden file was deleted, renamed, or never
/// committed, and must fail loud rather than being silently skipped.
pub fn assert_golden<T: Serialize>(filename: &str, data: &T) {
    let path = golden_dir().join(filename);
    let actual = serde_json::to_string_pretty(data).expect("serialize for comparison");

    if !path.exists() {
        panic!(
            "Golden file missing: {}\n\
             If this is a new capture, run with REGENERATE_GOLDEN=1 to create it; \
             if it was unexpectedly deleted or renamed, restore it.\n\
             Actual output from this run:\n{}",
            path.display(),
            actual
        );
    }

    let expected = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read golden file {:?}: {}", path, e));

    assert_eq!(
        actual.trim(),
        expected.trim(),
        "Golden file mismatch: {}\n\
         To update, run: REGENERATE_GOLDEN=1 cargo test --test postgres_integration golden -- --include-ignored --test-threads=1",
        filename
    );
}
