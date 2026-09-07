//! Arc 278 census M — a `census_count` call under `src/rete/kernel/tests/`
//! may name only a `bench:`-prefixed key.
//!
//! A synthetic replica that writes a production census key has no form.
//! `census_counted(||` is the window helper and is not this rule's business;
//! neither is `census_count_n`. The exemption list is empty. A rune does not
//! save a production key.
//!
//! Gate A is the proof (structural, deterministic).

use std::path::{Path, PathBuf};

const SUBJECT: &str = "src/rete/kernel/tests";
const PREFIX: &str = "bench:";

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_rs(&p, out);
        } else if p.extension().and_then(|x| x.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

fn rust_code_of(line: &str) -> &str {
    match line.find("//") {
        Some(i) => &line[..i],
        None => line,
    }
}

/// Keys passed to `census_count("` in CODE positions. Not `census_count_n`, not
/// `census_counted`.
fn census_count_writes(src: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for (i, line) in src.lines().enumerate() {
        let code = rust_code_of(line);
        let mut search = code;
        let mut base = 0usize;
        while let Some(at) = search.find("census_count") {
            let abs = base + at;
            let after = &code[abs + "census_count".len()..];
            if after.starts_with("_n") || after.starts_with("ed") {
                base = abs + 1;
                search = &code[base..];
                continue;
            }
            let rest = after.trim_start();
            if !rest.starts_with('(') {
                base = abs + 1;
                search = &code[base..];
                continue;
            }
            let inner = rest[1..].trim_start();
            if let Some(s) = inner.strip_prefix('"') {
                if let Some(end) = s.find('"') {
                    out.push((i + 1, s[..end].to_string()));
                }
            }
            base = abs + 1;
            search = &code[base..];
        }
    }
    out
}

fn violations_in(rel: &str, src: &str) -> Vec<String> {
    census_count_writes(src)
        .into_iter()
        .filter(|(_, key)| !key.starts_with(PREFIX))
        .map(|(line, key)| {
            format!("  {rel}:{line}: census_count(\"{key}\") — only a `{PREFIX}` key is writable from {SUBJECT}")
        })
        .collect()
}

mod detector {
    use super::*;

    #[test]
    fn a_production_key_is_a_hit() {
        let src = "super::census_count(\"filter:test-reuse\");\n";
        let v = violations_in("src/rete/kernel/tests/node_share_cost.rs", src);
        assert_eq!(
            v,
            ["  src/rete/kernel/tests/node_share_cost.rs:1: census_count(\"filter:test-reuse\") — only a `bench:` key is writable from src/rete/kernel/tests"],
            "a production key must redden naming file and key; got {v:?}"
        );
    }

    #[test]
    fn a_bench_key_is_not_a_hit() {
        let src = "super::census_count(\"bench:filter-reuse\");\n";
        let v = violations_in("src/rete/kernel/tests/node_share_cost.rs", src);
        assert!(v.is_empty(), "a `bench:` key is the allowed form; got {v:?}");
    }

    #[test]
    fn census_count_n_is_not_a_hit() {
        let src = "census_count_n(\"filter:test-reuse\", 1);\n";
        let v = violations_in("src/rete/kernel/tests/accum_cost.rs", src);
        assert!(v.is_empty(), "census_count_n is not this rule; got {v:?}");
    }

    #[test]
    fn census_counted_is_not_a_hit() {
        let src = "super::census_counted(|| eval());\n";
        let v = violations_in("src/rete/kernel/tests/mod.rs", src);
        assert!(v.is_empty(), "census_counted is the window helper; got {v:?}");
    }

    #[test]
    fn a_comment_is_not_a_hit() {
        let src = "// super::census_count(\"filter:test-reuse\");\n";
        let v = violations_in("src/rete/kernel/tests/node_share_cost.rs", src);
        assert!(v.is_empty(), "comments are prose; got {v:?}");
    }
}

#[test]
fn kernel_tests_census_count_names_only_bench_keys() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = manifest.join(SUBJECT);
    let mut files = Vec::new();
    collect_rs(&root, &mut files);
    files.sort();

    // NON-VACUITY: the walk must actually find src/rete/kernel/tests/. A typo'd
    // path finding zero files would make this gate pass forever while checking
    // nothing. 8 is a floor of what the walk finds today (driven 2026-09-07),
    // not a count of the corpus.
    assert!(
        files.len() >= 8,
        "bench-census gate found only {} .rs files under {SUBJECT} — it is not looking at the tree it claims to guard",
        files.len()
    );

    let mut violations = Vec::new();
    let mut bench_writes = 0usize;
    for f in &files {
        let rel = f
            .strip_prefix(manifest)
            .unwrap_or(f)
            .to_string_lossy()
            .replace('\\', "/");
        let src = std::fs::read_to_string(f).unwrap_or_else(|e| panic!("read {rel}: {e}"));
        for (_, key) in census_count_writes(&src) {
            if key.starts_with(PREFIX) {
                bench_writes += 1;
            }
        }
        violations.extend(violations_in(&rel, &src));
    }

    assert!(
        violations.is_empty(),
        "a `census_count` under {SUBJECT} named a key that is not `{PREFIX}`-prefixed:\n{}",
        violations.join("\n")
    );

    // NON-VACUITY: the replica still writes. Deleting both bumps would leave this
    // gate passing over an empty set of census_count calls.
    assert!(
        bench_writes >= 2,
        "bench-census gate found only {bench_writes} `census_count(\"bench:…\")` writes under {SUBJECT} — the replica's two sites must still exist"
    );
}
