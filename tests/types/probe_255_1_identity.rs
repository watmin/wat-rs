//! Stone 255.1 — identity is the (namespace, name) pair; wat.type has members.

use wat::freeze::startup_from_file;

#[test]
fn wat_type_i64_annotates() {
    startup_from_file("tests/types/probe_255_1_wat_type_membership.wat")
        .expect("wat.type/i64 must be a member and annotate");
}

#[test]
fn wat_type_non_member_is_refused_as_not_a_member() {
    let err = match startup_from_file("tests/types/probe_255_1_wat_type_non_member.wat.bad") {
        Ok(_) => panic!("expected refusal of wat.type/nope"),
        Err(e) => format!("{e:?}"),
    };
    assert!(
        err.contains("not a member of wat.type"), // rune:lint(loose-assert) — targeted presence over a large structured diagnostic; the contract is the membership refusal, not the whole freeze dump
        "non-member must say 'not a member of wat.type', not a generic unknown; got {err}"
    );
    assert!(
        !err.contains("unknown type wat.type/nope — not a declared type"), // rune:lint(loose-assert) — targeted ABSENCE of the generic unknown-type prose on the same dump
        "generic unknown-type prose must not mask the wat.type membership refusal; got {err}"
    );
}

#[test]
fn no_fourth_wat_type_strip_prefix() {
    // The three scattered `strip_prefix(":wat::type::")` sites collapsed to
    // `type_denotation` in edn/render.rs. A fourth is a regression.
    let mut hits = Vec::new();
    let walk = ["src", "crates"];
    for root in walk {
        let Ok(_entries) = std::fs::read_dir(root) else { continue };
        fn walk_rs(dir: &std::path::Path, hits: &mut Vec<String>) {
            let Ok(rd) = std::fs::read_dir(dir) else { return };
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk_rs(&p, hits);
                    continue;
                }
                if p.extension().and_then(|s| s.to_str()) != Some("rs") {
                    continue;
                }
                let Ok(src) = std::fs::read_to_string(&p) else { continue };
                for (i, line) in src.lines().enumerate() {
                    if line.contains("strip_prefix(\":wat::type::\")")
                        || line.contains("strip_prefix(\"wat::type::\")")
                    {
                        hits.push(format!("{}:{}", p.display(), i + 1));
                    }
                }
            }
        }
        walk_rs(std::path::Path::new(root), &mut hits);
    }
    assert!(
        hits.len() <= 3,
        "wat.type strip_prefix must stay in identity/denotation; found {hits:?}"
    );
}
