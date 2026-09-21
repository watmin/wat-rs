//! 255.4 — a member join is `/`, always. This wall keeps colon-joined
//! Type::member from growing back.
//!
//! Two faces, both derived (not a hand-list of names):
//! - the live registries (`wat_intrinsic` + `wat_dispatch` rust_deps) live in
//!   `types::stone_255_4_no_colon_joined_type_member_in_the_registry`
//! - tracked `.wat` keyword text matching `:ns::Type::method` (Pascal type,
//!   lowercase member). Nested type names (`Cache::GetRequest`) are Pascal
//!   in the last segment and do not match. Retired `Record::def` is exempt
//!   because its replacement is `defrecord`, not a slash unify.

use std::process::Command;

fn tracked_wat() -> Vec<String> {
    // git ls-files already omits gitignored paths. Do not name an
    // ignored directory here — that string is itself a committed
    // dependency on a tree that does not exist on a clone.
    let out = Command::new("git")
        .args(["ls-files", "-z", "--", "tests", "wat", "wat-scripts", "wat-tests"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("git ls-files");
    assert!(out.status.success(), "git ls-files failed");
    String::from_utf8(out.stdout)
        .unwrap()
        .split('\0')
        .filter(|p| p.ends_with(".wat") || p.ends_with(".wat.bad"))
        .map(str::to_string)
        .collect()
}

/// `:ns::Type::method` — last join `::`, type segment PascalCase, member
/// starts lowercase. Same discriminator as `types::is_colon_joined_type_member`.
fn is_colon_joined_type_member(name: &str) -> bool {
    if name.contains('/') {
        return false;
    }
    let Some((prefix, member)) = name.rsplit_once("::") else {
        return false;
    };
    let type_seg = match prefix.rsplit_once("::") {
        Some((_, leaf)) => leaf,
        None => prefix,
    };
    let Some(tc) = type_seg.chars().next() else {
        return false;
    };
    let Some(mc) = member.chars().next() else {
        return false;
    };
    tc.is_ascii_uppercase() && (mc.is_ascii_lowercase() || mc == '_')
}

fn keyword_constituent(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, ':' | '/' | '_' | '-' | '?' | '!' | '*' | '+' | '=' | '<' | '>' | '\'')
}

const RECORD_DEF_EXEMPT: &[&str] = &[
    ":wat::core::Record::def",
    ":wat::holon::Record::def",
    ":wat::Record::def",
];

#[test]
fn no_colon_joined_type_member_in_tracked_wat() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let files = tracked_wat();
    // NON-VACUITY: git ls-files of tests/wat/wat-scripts/wat-tests. Driven
    // 2026-09-21: 2459 paths. Floor sits well under that so a moved root or
    // a glob that matches nothing cannot pass over an empty set.
    assert!(
        files.len() > 1500,
        "one_member_join's git ls-files walk found only {} .wat path(s) — it is not \
         reaching the corpus it claims to guard, so its green means nothing",
        files.len()
    );
    let mut hits: Vec<String> = Vec::new();
    for rel in files {
        let text = std::fs::read_to_string(root.join(&rel)).unwrap_or_default();
        for (i, line) in text.lines().enumerate() {
            let code = match line.find(";;") {
                Some(c) => &line[..c],
                None => line,
            };
            // Call heads only — `(` immediately before the keyword. Declaration
            // names (`deftest :ns::Type::test-foo`) and string arguments to
            // `keyword-node` are not members.
            let chars: Vec<char> = code.chars().collect();
            let mut j = 0;
            while j < chars.len() {
                if chars[j] == '(' {
                    j += 1;
                    while j < chars.len() && chars[j].is_whitespace() {
                        j += 1;
                    }
                    if j < chars.len() && chars[j] == ':' {
                        let start = j;
                        j += 1;
                        while j < chars.len() && keyword_constituent(chars[j]) {
                            j += 1;
                        }
                        let name: String = chars[start..j].iter().collect();
                        if !RECORD_DEF_EXEMPT.contains(&name.as_str())
                            && is_colon_joined_type_member(&name)
                        {
                            hits.push(format!("{rel}:{}: {name}", i + 1));
                        }
                    }
                } else {
                    j += 1;
                }
            }
        }
    }
    assert!(
        hits.is_empty(),
        "colon-joined Type::member still in tracked .wat ({}):\n{}",
        hits.len(),
        hits.join("\n")
    );
}
