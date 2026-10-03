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
    let forms = wat::parse_all_with_file(src, "wat/grep.wat")
        .unwrap_or_else(|e| panic!("parse wat/grep.wat: {e:?}"));
    let items = forms
        .iter()
        .find_map(|form| {
            let wat::WatAST::List(items, _) = form else { return None };
            let head = crate::decl_identity::canon(items.first()?)?;
            if head != ":wat::core::defenum" {
                return None;
            }
            let name = crate::decl_identity::canon(items.get(1)?)?;
            (name == ":wat::grep::NodeKind").then_some(items)
        })
        .expect("defenum NodeKind not found by identity in wat/grep.wat");
    let mut out = BTreeSet::new();
    let mut i = 0usize;
    while i + 1 < items.len() {
        if let wat::WatAST::Keyword(k, _) = &items[i] {
            if let Some(name) = k.strip_prefix(':') {
                let uppercase = name.chars().next().is_some_and(|c| c.is_ascii_uppercase());
                let empty_vec = matches!(&items[i + 1], wat::WatAST::Vector(v, _) if v.is_empty());
                // rune:lint(one-variant-separator, namespace) — a NodeKind variant is a bare word; `::` here is a namespace, which a variant does not carry
                if uppercase && empty_vec && !name.contains("::") {
                    out.insert(name.to_string());
                }
            }
        }
        i += 1;
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
