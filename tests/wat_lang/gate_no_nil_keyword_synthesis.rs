//! Arc 244 Stone 244.3 — removal-of-existence gate.
//!
//! Asserts that no file under `src/` (excluding `src/parser.rs` and
//! `src/lexer.rs`) contains a literal `Keyword(":wat::core::nil"` construction.
//!
//! The legitimate occurrences are:
//!   - `src/parser.rs` / `src/lexer.rs` — these are EXEMPT because they
//!     produce `WatAST::Keyword` nodes from USER SOURCE (`:wat::core::nil`
//!     appearing as a keyword token in user-written wat code is legal in
//!     type-annotation position).
//!
//! In all SYNTHESIS paths (runtime, check, closure_extract, …) a
//! `WatAST::Keyword(":wat::core::nil")` construction is the VALUE-POSITION
//! heresy that Arc 242 Doctrine 1 forbids and Arc 244 annihilates. Every such
//! synthesis must use `WatAST::nil()` or `WatAST::NilLit(span)` instead.
//!
//! If this test fails, re-introduced heresy was committed somewhere in `src/`.
//! Fix the offending site; do not widen the exempt list.

use std::fs;
use std::path::Path;

const HERESY: &str = "Keyword(\":wat::core::nil\"";

fn scan_dir(dir: &Path, skip_files: &[&str]) -> (usize, Vec<String>) {
    let mut total = 0;
    let mut found_in = Vec::new();
    let entries = fs::read_dir(dir).expect("read_dir src/");
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let (sub_count, mut sub_found) = scan_dir(&path, skip_files);
            total += sub_count;
            found_in.append(&mut sub_found);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if skip_files.contains(&name) {
                continue;
            }
            let content = fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("could not read {}", path.display()));
            let count = content.matches(HERESY).count();
            if count > 0 {
                total += count;
                found_in.push(format!("{} ({} occurrence{})", path.display(), count, if count == 1 { "" } else { "s" }));
            }
        }
    }
    (total, found_in)
}

#[test]
fn no_nil_keyword_synthesis_in_src() {
    // Walk src/ relative to the crate root (CARGO_MANIFEST_DIR).
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let src_dir = Path::new(manifest_dir).join("src");

    // Exempt: parser.rs and lexer.rs produce Keyword nodes from user source — legitimate.
    let exempt = &["parser.rs", "lexer.rs"];

    let (total, offenders) = scan_dir(&src_dir, exempt);

    assert!(
        total == 0,
        "Arc 244 gate: found {} heretical `{}` construction(s) in src/ \
         (excluding parser.rs / lexer.rs). Each is a VALUE-POSITION synthesis \
         that must use `WatAST::nil()` or `WatAST::NilLit(span)` instead.\n\
         Offending files:\n{}",
        total,
        HERESY,
        offenders.join("\n")
    );
}
