//! 251.8d-i-b(2) — a GENERATOR that emits `<-` as a rete bind is invisible to
//! the text rewrite (`rete-bind-arrow-to-binder` walks source, not expansion).
//!
//! Derived set (not a grep for the string `"<-"`):
//! - `symbol-node "?…"` in tracked `.wat` — the only production mint of a
//!   rete-var at expansion time is `wat/query.wat` `fact-sym` (`"?fact"`).
//! - A quasiquote `` `(~NAME <- …) `` splices that node into a 3-element
//!   list with a literal `<-` in the middle: that IS a bind clause constructor.
//! - `symbol-node "<-"` in `wat/core.wat` (defn kwargs) and
//!   `Identifier::bare("<-")` in `closure_extract` / `reflect/render` reconstruct
//!   **param/field annotations**, not binds — they do not mint a `?`-prefixed
//!   symbol, so they are out of this wall.
//!
//! The discriminator is the same one the text rewrite uses: an arrow after a
//! `?`-prefixed symbol is a bind and must be `:-`.

use std::collections::HashSet;
use std::process::Command;

fn tracked_wat() -> Vec<String> {
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

/// Bindings `NAME` of `(:wat::core::symbol-node "?…")`.
fn qvar_symbol_nodes(src: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    let bytes = src.as_bytes();
    let needle = b"symbol-node";
    let mut i = 0;
    while let Some(rel) = src[i..].find("symbol-node") {
        let at = i + rel;
        i = at + needle.len();
        // look back for the binding name: `foo  (:wat::core::symbol-node "?…")`
        let before = &src[..at];
        let Some(name) = binding_name_before(before) else {
            continue;
        };
        let after = &src[i..];
        let Some(q) = first_string_lit(after) else {
            continue;
        };
        if q.starts_with('?') {
            out.insert(name);
        }
    }
    let _ = bytes;
    out
}

fn binding_name_before(before: &str) -> Option<String> {
    // skip whitespace and the opening `(:wat::core::`
    let t = before.trim_end();
    let t = t.strip_suffix("(:wat::core::")?.trim_end(); // rune:lint(no-inlined-edn) — detector fixture: the wat call-shape prefix this parser strips from a source fragment; not a golden and cannot live in a .edn file (the fragment is deliberately not parseable EDN)
    // name is the last ident token
    let ident: String = t
        .chars()
        .rev()
        .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '*' | '!' | '?'))
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    if ident.is_empty() {
        None
    } else {
        Some(ident)
    }
}

fn first_string_lit(after: &str) -> Option<&str> {
    let start = after.find('"')?;
    let rest = &after[start + 1..];
    let end = rest.find('"')?;
    Some(&rest[..end])
}

/// `` `(~NAME <- `` — a quasiquoted list whose first child is an unquote and
/// whose next token is the left-arrow. Line numbers are 1-based.
fn splice_left_arrow_uses(src: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for (i, line) in src.lines().enumerate() {
        let code = match line.find(';') {
            Some(c) if !in_string_before(line, c) => &line[..c],
            _ => line,
        };
        let mut rest = code;
        while let Some(at) = rest.find("`(~") {
            let after = &rest[at + 3..];
            let name: String = after
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '*' | '!' | '?'))
                .collect();
            if name.is_empty() {
                rest = &rest[at + 3..];
                continue;
            }
            let after_name = &after[name.len()..];
            let trimmed = after_name.trim_start();
            if trimmed.starts_with("<-") {
                out.push((i + 1, name));
            }
            rest = &rest[at + 3..];
        }
    }
    out
}

fn in_string_before(line: &str, idx: usize) -> bool {
    let mut in_str = false;
    let mut esc = false;
    for (i, c) in line.char_indices() {
        if i >= idx {
            break;
        }
        if in_str {
            if esc {
                esc = false;
            } else if c == '\\' {
                esc = true;
            } else if c == '"' {
                in_str = false;
            }
        } else if c == '"' {
            in_str = true;
        }
    }
    in_str
}

#[test]
fn rete_bind_generators_emit_binder_not_left_arrow() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let files = tracked_wat();
    // NON-VACUITY: git ls-files of tests/wat/wat-scripts/wat-tests. Driven
    // 2026-09-21: same population as `one_member_join` (> 1500). A moved
    // root or a glob that matches nothing cannot pass over an empty set.
    assert!(
        files.len() > 1500,
        "rete_bind_generators' git ls-files walk found only {} .wat path(s) — it is not \
         reaching the corpus it claims to guard, so its green means nothing",
        files.len()
    );

    let mut qvar_nodes = 0usize;
    let mut violations = Vec::new();
    for rel in &files {
        let text = std::fs::read_to_string(root.join(rel)).unwrap_or_default();
        let qvars = qvar_symbol_nodes(&text);
        qvar_nodes += qvars.len();
        for (line, name) in splice_left_arrow_uses(&text) {
            if qvars.contains(&name) {
                violations.push(format!("{rel}:{line}  `(~{name} <- …)` splices a ?-prefixed \
                     symbol-node — that is a rete bind generator and must emit `:-`"));
            }
        }
    }

    // NON-VACUITY: `wat/query.wat` still mints `fact-sym` as `symbol-node "?fact"`.
    // A walk that cannot see that mint cannot see the generator it claims to guard.
    assert!(
        qvar_nodes > 0,
        "rete_bind_generators found no `symbol-node \"?…\"` bindings — the walk missed \
         wat/query.wat's fact-sym mint, so a green here would not prove the generator is gone"
    );

    assert!(
        violations.is_empty(),
        "a rete bind GENERATOR still emits `<-` (the text rewrite cannot reach it):\n{}",
        violations.join("\n")
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_qvar_splice_left_arrow_is_the_generator() {
        let src = "     fact-sym  (:wat::core::symbol-node \"?fact\")\n              cond `(~fact-sym <- ~tkw)]\n";
        let qvars = qvar_symbol_nodes(src);
        assert_eq!(
            qvars,
            HashSet::from(["fact-sym".to_string()]),
            "mint of ?fact must bind exactly fact-sym"
        );
        let uses = splice_left_arrow_uses(src);
        assert_eq!(uses, vec![(2, "fact-sym".to_string())]);
    }

    #[test]
    fn a_param_annotation_splice_is_not_a_qvar_mint() {
        let src = "     d-sym          (:wat::core::symbol-node \"record\")\n                        `(:wat::core::fn [~d-sym <- ~record-ty-ann] -> ~state-ty-ann x)\n";
        assert!(
            qvar_symbol_nodes(src).is_empty(),
            "symbol-node \"record\" is a param binder, not a rete-var"
        );
        assert!(
            splice_left_arrow_uses(src).is_empty(),
            "`fn [~d-sym <-` is not a quasiquoted bind list `(~name <-`"
        );
    }

    #[test]
    fn converted_bind_template_is_not_a_hit() {
        let src = "     fact-sym  (:wat::core::symbol-node \"?fact\")\n              cond `(~fact-sym :- ~tkw)]\n";
        assert_eq!(
            qvar_symbol_nodes(src),
            HashSet::from(["fact-sym".to_string()])
        );
        assert!(
            splice_left_arrow_uses(src).is_empty(),
            "`:-` is the binder; the wall is only the left-arrow"
        );
    }
}
