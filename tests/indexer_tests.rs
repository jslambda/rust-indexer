use rust_indexer::{IndexEntry, build_index};
use std::path::PathBuf;
use std::process::Command;

fn find_entry<'a>(entries: &'a [IndexEntry], kind: &str, name: &str) -> Option<&'a IndexEntry> {
    entries.iter().find(|e| e.kind == kind && e.name == name)
}

#[test]
fn indexes_sample_crate() {
    let root = PathBuf::from("tests/data/sample_crate");
    let entries = build_index(&root).expect("index build");

    assert!(find_entry(&entries, "module", "utils").is_some());
    let greeter = find_entry(&entries, "struct", "Greeter").expect("Greeter struct");
    assert_eq!(greeter.doc_summary.as_deref(), Some("Greeter struct."));

    assert!(find_entry(&entries, "trait", "Greet").is_some());
    assert!(find_entry(&entries, "enum", "ErrorKind").is_some());
    assert!(find_entry(&entries, "fn", "make_greeter").is_some());
    assert!(
        entries
            .iter()
            .any(|e| e.kind == "impl" && e.signature.contains("Greeter"))
    );
}

#[test]
fn prints_usage_with_help_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_rust-indexer"))
        .arg("--help")
        .output()
        .expect("run binary");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("utf8");
    assert!(
        stdout.to_lowercase().contains("usage"),
        "expected usage output, got: {}",
        stdout
    );
}
