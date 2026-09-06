//! Arc 278 — a raw gather-bucket walk outside `gather_bucket` has no form.
//!
//! `GATHER_VISITS` counts one examination per element a gather yields.
//! Three paths used to walk a bucket and count zero; a regression to that
//! shape is a raw `bucket.iter()` / `for … in bucket` in a gather module.
//! The one verb is `gather_bucket` (`census.rs`). Join-index probes in
//! `fire/mod.rs` are not Acc/Neg/Exists examinations and carry a rune.

use std::path::{Path, PathBuf};

const SUBJECTS: &[&str] = &[
    "src/rete/kernel/fire/acc.rs",
    "src/rete/kernel/fire/pass/accumulate.rs",
    "src/rete/kernel/fire/mod.rs",
];

const HELPER: &str = "gather_bucket";
const RUNE: &str = "rune:lint(gather-walk-not-examining)";
const MIN_REASON_CHARS: usize = 40;
const EM_DASH: char = '\u{2014}';

fn code_of(line: &str) -> &str {
    match line.find("//") {
        Some(i) => &line[..i],
        None => line,
    }
}

fn has_rune(line: &str) -> bool {
    line.contains(RUNE)
}

fn rune_reason(line: &str) -> Option<&str> {
    let i = line.find(RUNE)?;
    let rest = &line[i + RUNE.len()..];
    let rest = rest.trim_start();
    let rest = rest.strip_prefix(EM_DASH).or_else(|| rest.strip_prefix('-'))?;
    Some(rest.trim())
}

/// A raw walk: `bucket.iter(` or `for … in bucket`, not `gather_bucket(…)`.
fn window_is_raw_walk(window: &str) -> bool {
    let compact: String = window
        .chars()
        .map(|c| if c.is_whitespace() { ' ' } else { c })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if compact.contains(HELPER) {
        return false;
    }
    compact.contains("bucket.iter(") || compact.contains("in bucket")
}

fn preceding_rune<'a>(lines: &'a [&str], i: usize) -> Option<&'a str> {
    for j in (i.saturating_sub(3)..=i).rev() {
        if has_rune(lines[j]) {
            return Some(lines[j]);
        }
    }
    None
}

fn violations_in(rel: &str, src: &str) -> Vec<String> {
    let lines: Vec<&str> = src.lines().collect();
    let mut out = Vec::new();
    for i in 0..lines.len() {
        let next = lines.get(i + 1).copied().unwrap_or("");
        let window = format!("{} {}", code_of(lines[i]), code_of(next));
        if !window_is_raw_walk(&window) {
            continue;
        }
        let here = code_of(lines[i]);
        let nxt = code_of(next);
        let this_line_walks = here.contains("bucket.iter(") || here.contains("in bucket");
        let split_walk = here.trim().ends_with("bucket")
            && nxt.trim_start().starts_with(".iter(");
        if !this_line_walks && !split_walk {
            continue;
        }
        if let Some(rune_line) = preceding_rune(&lines, i) {
            match rune_reason(rune_line) {
                Some(r) if r.chars().count() >= MIN_REASON_CHARS => continue,
                Some(r) => out.push(format!(
                    "  {rel}:{}: rune reason is {} chars (under {MIN_REASON_CHARS}): {r}",
                    i + 1,
                    r.chars().count()
                )),
                None => out.push(format!(
                    "  {rel}:{}: {RUNE} with no em-dash reason",
                    i + 1
                )),
            }
            continue;
        }
        out.push(format!(
            "  {rel}:{}: raw gather-bucket walk (not `{HELPER}`)",
            i + 1
        ));
    }
    out
}

mod detector {
    use super::*;

    #[test]
    fn a_raw_iter_is_a_hit() {
        let src = "fn f(bucket: &[usize]) {\n    bucket.iter().map(|&i| i);\n}\n";
        let v = violations_in("acc.rs", src);
        assert_eq!(v.len(), 1, "raw iter must redden; got {v:?}");
    }

    #[test]
    fn a_raw_for_in_bucket_is_a_hit() {
        let src = "fn f(bucket: &[usize]) {\n    for &i in bucket {\n        let _ = i;\n    }\n}\n";
        let v = violations_in("acc.rs", src);
        assert_eq!(v.len(), 1, "raw for-in must redden; got {v:?}");
    }

    #[test]
    fn the_helper_is_not_a_hit() {
        let src = "fn f(bucket: &[usize]) {\n    for i in gather_bucket(bucket) {\n        let _ = i;\n    }\n}\n";
        let v = violations_in("acc.rs", src);
        assert!(v.is_empty(), "the helper is the one allowed walk; got {v:?}");
    }

    #[test]
    fn len_and_is_empty_are_not_walks() {
        let src = "fn f(bucket: &[usize]) {\n    let _ = bucket.len();\n    let _ = bucket.is_empty();\n    let _ = bucket.first();\n}\n";
        let v = violations_in("acc.rs", src);
        assert!(v.is_empty(), "O(1) reads examine nothing; got {v:?}");
    }

    #[test]
    fn a_runed_walk_is_exempt() {
        let src = "fn f(bucket: &[usize]) {\n    // rune:lint(gather-walk-not-examining) — HashJoin probe of the left-token bucket; not an Acc/Neg/Exists examination\n    for &i in bucket {\n        let _ = i;\n    }\n}\n";
        let v = violations_in("mod.rs", src);
        assert!(v.is_empty(), "a named non-examining walk is exempt; got {v:?}");
    }

    #[test]
    fn a_short_rune_reason_is_refused() {
        let src = "fn f(bucket: &[usize]) {\n    // rune:lint(gather-walk-not-examining) — too short\n    for &i in bucket { let _ = i; }\n}\n";
        let v = violations_in("mod.rs", src);
        assert_eq!(v.len(), 1, "short reason must redden; got {v:?}");
    }
}

#[test]
fn no_raw_gather_bucket_walk_outside_the_helper() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));

    // NON-VACUITY: the subject list is named, but a typo'd path or an emptied
    // array would make this gate pass forever while checking nothing.
    assert!(
        SUBJECTS.len() >= 3,
        "gather-walk lint subject list has {} entries — it is not looking at the gather modules",
        SUBJECTS.len()
    );

    let mut violations = Vec::new();
    let mut helper_uses = 0usize;
    for rel in SUBJECTS {
        let path: PathBuf = manifest.join(rel);
        assert!(
            path.is_file(),
            "gather-walk lint subject `{rel}` does not exist — the list is stale"
        );
        let src = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {rel}: {e}"));
        helper_uses += src.matches(&format!("{HELPER}(")).count();
        violations.extend(violations_in(rel, &src));
    }

    assert!(
        helper_uses >= 3,
        "gather_bucket is used {helper_uses} times across the gather modules — the extractor is not seeing the verb"
    );
    assert!(
        violations.is_empty(),
        "raw gather-bucket walk outside `{HELPER}`:\n{}",
        violations.join("\n")
    );
}
