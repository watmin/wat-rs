//! THE AST-KIND ↔ NODEKIND SYNC GATE — `eval_ast_kind`'s 14 arms and
//! `:wat::grep::NodeKind`'s 14 variants must name the same set.
//!
//! STONE-node-kind-becomes-an-enum (arc 277): `Node.kind` is an enum that
//! mirrors `WatAST`. Rust is guarded (`eval_ast_kind` goes E0004 if a 15th
//! variant appears). wat is not — `kind-of` would raise at runtime. This
//! lint fails when the two lists diverge, so a new `WatAST` variant cannot
//! ship without a matching `NodeKind` constructor.
//!
//! Source of truth for the Rust side: the `match ast` arms inside
//! `eval_ast_kind` (`src/edn/render.rs`), not the doc comment.

use std::collections::BTreeSet;
use std::path::PathBuf;

fn eval_ast_kind_variants(src: &str) -> BTreeSet<String> {
    let Some(fn_at) = src.find("pub fn eval_ast_kind(") else {
        panic!("eval_ast_kind not found in src/edn/render.rs");
    };
    let rest = &src[fn_at..];
    let Some(match_at) = rest.find("let kind = match ast {") else {
        panic!("eval_ast_kind match not found");
    };
    let match_body = &rest[match_at..];
    let Some(end) = match_body.find("\n    };") else {
        panic!("eval_ast_kind match end not found");
    };
    let body = &match_body[..end];
    let mut out = BTreeSet::new();
    for line in body.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("WatAST::") {
            if let Some((name, _)) = rest.split_once('(') {
                out.insert(name.to_string());
            }
        }
    }
    out
}

fn nodekind_variants(src: &str) -> BTreeSet<String> {
    let Some(at) = src.find("defenum :wat::grep::NodeKind") else {
        panic!("defenum :wat::grep::NodeKind not found in wat/grep.wat");
    };
    let rest = &src[at..];
    let Some(end) = rest.find("defrecord :wat::grep::Node") else {
        panic!("NodeKind defenum end not found");
    };
    let body = &rest[..end];
    let mut out = BTreeSet::new();
    for line in body.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix(':') {
            if let Some((name, after)) = rest.split_once(' ') {
                if after.starts_with("[]") && name.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
                    out.insert(name.to_string());
                }
            }
        }
    }
    out
}

#[test]
fn ast_kind_arms_match_nodekind_variants() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let rust_src = std::fs::read_to_string(manifest.join("src/edn/render.rs"))
        .expect("read src/edn/render.rs");
    let wat_src = std::fs::read_to_string(manifest.join("wat/grep.wat"))
        .expect("read wat/grep.wat");

    let rust = eval_ast_kind_variants(&rust_src);
    let wat = nodekind_variants(&wat_src);

    assert_eq!(
        rust.len(),
        14,
        "eval_ast_kind has {} arms, expected 14 (one per WatAST variant)",
        rust.len()
    );
    assert_eq!(
        wat.len(),
        14,
        "NodeKind has {} variants, expected 14",
        wat.len()
    );
    assert_eq!(
        rust, wat,
        "eval_ast_kind arms and NodeKind variants diverged.\n\
         only-in-rust: {:?}\nonly-in-wat: {:?}",
        rust.difference(&wat).collect::<Vec<_>>(),
        wat.difference(&rust).collect::<Vec<_>>()
    );
}
