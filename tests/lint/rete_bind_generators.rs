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

/// Bindings `NAME` of a `symbol-node` call whose string starts with `?`.
/// The call is parsed; the head matches when its canonical identity is
/// `:wat::core::symbol-node` (`:wat::core::symbol-node` or `wat.core/symbol-node`).
fn qvar_symbol_nodes(src: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    if let Ok(forms) = wat::parse_all_with_file(src, "<symbol-node>") {
        for form in &forms {
            walk_qvar(form, src, &mut out);
        }
        return out;
    }
    // A specimen can carry a quasiquote tail the whole-file reader refuses.
    // Each balanced list that names symbol-node is parsed on its own; the
    // binding name is the ident in the original text before that list.
    let bytes = src.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] != b'(' {
            i += 1;
            continue;
        }
        let Some(close) = matching_paren(src, i) else {
            i += 1;
            continue;
        };
        let form = &src[i..=close];
        if form.contains("symbol-node") {
            if let Ok(forms) = wat::parse_all_with_file(form, "<symbol-node>") {
                if let Some(wat::WatAST::List(items, _)) = forms.first() {
                    if qvar_mint(items) {
                        if let Some(name) = binding_name_before(&src[..i]) {
                            out.insert(name);
                        }
                    }
                }
            }
        }
        i = close + 1;
    }
    out
}

fn qvar_mint(items: &[wat::WatAST]) -> bool {
    let head_ok = items
        .first()
        .and_then(crate::decl_identity::canon)
        .as_deref()
        == Some(":wat::core::symbol-node");
    let q = items.iter().find_map(|n| match n {
        wat::WatAST::StringLit(s, _) => Some(s.as_str()),
        _ => None,
    });
    head_ok && q.is_some_and(|q| q.starts_with('?'))
}

fn walk_qvar(node: &wat::WatAST, src: &str, out: &mut HashSet<String>) {
    let wat::WatAST::List(items, span) = node else {
        for child in node.children().iter() {
            walk_qvar(child, src, out);
        }
        return;
    };
    if qvar_mint(items) {
        if let Some(line) = src.lines().nth((span.line as usize).saturating_sub(1)) {
            let before: String = line.chars().take((span.col as usize).saturating_sub(1)).collect();
            if let Some(name) = binding_name_before(&before) {
                out.insert(name);
            }
        }
    }
    for child in items {
        walk_qvar(child, src, out);
    }
}

fn matching_paren(src: &str, open: usize) -> Option<usize> {
    let chars: Vec<char> = src.chars().collect();
    let mut byte_at = Vec::with_capacity(chars.len());
    let mut b = 0usize;
    for c in &chars {
        byte_at.push(b);
        b += c.len_utf8();
    }
    let start = byte_at.iter().position(|&x| x == open)?;
    let mut depth = 0i32;
    let mut in_str = false;
    let mut esc = false;
    for (idx, c) in chars.iter().enumerate().skip(start) {
        if in_str {
            if esc {
                esc = false;
            } else if *c == '\\' {
                esc = true;
            } else if *c == '"' {
                in_str = false;
            }
            continue;
        }
        match c {
            '"' => in_str = true,
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(byte_at[idx]);
                }
            }
            _ => {}
        }
    }
    None
}

fn binding_name_before(before: &str) -> Option<String> {
    let t = before.trim_end();
    let ident: String = t
        .chars()
        .rev()
        .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '*' | '!' | '?'))
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    if ident.is_empty() { None } else { Some(ident) }
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
    fn a_converted_symbol_node_is_the_same_mint() {
        let src = "     fact-sym  (wat.core/symbol-node \"?fact\")\n";
        assert_eq!(
            qvar_symbol_nodes(src),
            HashSet::from(["fact-sym".to_string()]),
            "wat.core/symbol-node is the same mint as :wat::core::symbol-node"
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
