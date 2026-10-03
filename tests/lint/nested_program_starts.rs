//! BRIEF-3 — every spawned program starts.
//!
//! A `(:wat::core::forms …)` child is checked only when it STARTS. This gate
//! starts every nested program that sits in a program-carrying position,
//! derived from the one door (`:wat::kernel::spawn-program`'s ProcessOpts
//! `prog`) plus the forwarding rule (unchanged, concat operand, or let
//! binding). Never a hand list of verb names.
//!
//! Child path: `InMemoryLoader` + `startup_from_forms_with_inherit` with the
//! Config a parent would pass — not `--check` (UselessMain is a source-path
//! wall the child never runs).

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::time::Instant;

fn repo_path(rel: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

use wat::ast::WatAST;
use wat::check::error::CheckErrorKind;
use wat::config::collect_entry_file;
use wat::freeze::{startup_from_forms_with_inherit, StartupError};
use wat::load::loader::InMemoryLoader;
use wat::parser::parse_all_with_file;

fn git_ls(globs: &[&str]) -> Vec<String> {
    let out = Command::new("git")
        .args(["ls-files", "-c", "-o", "--exclude-standard", "--"])
        .args(globs)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("git ls-files");
    assert!(out.status.success(), "git ls-files failed");
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect()
}

fn raw_head(n: &WatAST) -> Option<&str> {
    match n {
        WatAST::List(items, _) => match items.first() {
            Some(WatAST::Keyword(k, _)) => Some(k.as_str()),
            Some(WatAST::Symbol(id, _)) if id.is_reference() => Some(id.as_str()),
            _ => None,
        },
        _ => None,
    }
}

/// Keyword `:wat::core::forms` and symbol `wat.core/forms` are one head.
fn head_ident(n: &WatAST) -> Option<String> {
    raw_head(n).map(wat::edn::render::canonical_identity)
}

fn is_head(n: &WatAST, kw: &str) -> bool {
    head_ident(n).as_deref() == Some(kw)
}

fn name_raw(n: &WatAST) -> &str {
    match n {
        WatAST::Keyword(k, _) => k.as_str(),
        WatAST::Symbol(id, _) => id.as_str(),
        _ => "",
    }
}

/// Keys one function occupies in the carrying set.
///
/// A namespace-level name has one canonical identity (`:wat::kernel::spawn-program`
/// and `wat.kernel/spawn-program`). A member join does not: `canonical_identity`
/// leaves `:wat::foo::Bar/baz` intact because it already contains `::`, and
/// rewrites `wat.foo.Bar/baz` to `:wat::foo::Bar::baz`. Until 5d both are calls
/// of that function.
fn carrying_keys(raw: &str) -> Vec<String> {
    let primary = wat::edn::render::canonical_identity(raw);
    let mut keys = vec![primary.clone()];
    // A namespace dot (`wat.kernel/spawn-program`) is not a member join.
    // The type segment is the last component before the slash, and it is capitalised.
    let before = raw.split('/').next().unwrap_or("");
    let last = before.rsplit(['.', ':']).next().unwrap_or("");
    let member = raw.contains('/') && last.chars().next().is_some_and(|c| c.is_uppercase());
    if member && raw.contains('.') {
        // rune:lint(one-variant-separator, namespace) — member-join spelling, not an enum variant
        if let Some((pre, post)) = primary.rsplit_once("::") {
            let slash = format!("{pre}/{post}");
            if slash != primary {
                keys.push(slash);
            }
        }
    // rune:lint(one-variant-separator, namespace) — member-join spelling, not an enum variant
    } else if member && primary.contains('/') && primary.contains("::") {
        // rune:lint(one-variant-separator, namespace) — member-join spelling, not an enum variant
        let body = primary.trim_start_matches(':').replace("::", ".");
        let alt = wat::edn::render::canonical_identity(&body);
        if alt != primary {
            keys.push(alt);
        }
    }
    keys
}

fn insert_carrying(carrying: &mut HashSet<(String, usize)>, raw_name: &str, idx: usize) {
    for k in carrying_keys(raw_name) {
        carrying.insert((k, idx));
    }
}

fn carrying_index(carrying: &HashSet<(String, usize)>, raw: &str) -> Option<usize> {
    let keys = carrying_keys(raw);
    keys.iter()
        .find_map(|k| carrying.iter().find(|(ck, _)| ck == k).map(|(_, i)| *i))
}

/// A binder. A reference symbol is a type or a call (`wat.spawn/ThreadOpts`),
/// not a parameter — counting it shifts the carrying index.
fn sym_name(n: &WatAST) -> Option<&str> {
    match n {
        WatAST::Symbol(id, _) if !id.is_reference() => Some(id.as_str()),
        _ => None,
    }
}

fn param_names(vec: &WatAST) -> Vec<String> {
    match vec {
        WatAST::Vector(items, _) => items
            .iter()
            .filter_map(sym_name)
            .filter(|s| *s != "<-")
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    }
}

/// Root: spawn-program ProcessOpts `prog` is argument index 1.
const ROOT: (&str, usize) = (":wat::kernel::spawn-program", 1);

fn derive_carrying(stdlib: &[(String, Vec<WatAST>)]) -> HashSet<(String, usize)> {
    let mut carrying: HashSet<(String, usize)> = HashSet::new();
    carrying.insert((ROOT.0.to_string(), ROOT.1));
    loop {
        let before = carrying.len();
        for (_path, forms) in stdlib {
            for f in forms {
                walk_defn(f, &mut carrying, None, &HashMap::new());
            }
        }
        if carrying.len() == before {
            break;
        }
    }
    carrying
}

fn walk_defn(
    n: &WatAST,
    carrying: &mut HashSet<(String, usize)>,
    cur: Option<(&str, &[String])>,
    lets: &HashMap<String, WatAST>,
) {
    let Some(h) = head_ident(n) else {
        walk_kids(n, carrying, cur, lets);
        return;
    };
    match n {
        WatAST::List(items, _) if h == ":wat::core::defn" || h == ":wat::core::defmacro" => {
            if items.len() >= 3 {
                let name = name_raw(&items[1]);
                let pv = items
                    .iter()
                    .find(|c| matches!(c, WatAST::Vector(_, _)));
                let params = pv.map(param_names).unwrap_or_default();
                for child in items.iter().skip(2) {
                    walk_defn(child, carrying, Some((name, &params)), lets);
                }
                return;
            }
        }
        WatAST::List(items, _) if h == ":wat::core::defclause" => {
            let name = items.get(1).map(name_raw).unwrap_or("");
            for clause in items.iter().skip(2) {
                if let WatAST::List(ch, _) = clause {
                    if let Some(WatAST::Vector(_, _)) = ch.first() {
                        let params = param_names(&ch[0]);
                        for child in ch.iter().skip(1) {
                            walk_defn(child, carrying, Some((name, &params)), lets);
                        }
                    }
                }
            }
            return;
        }
        WatAST::List(items, _) if h == ":wat::core::extend-type" => {
            // Stone 255.22 — an optional `:- [P …]` binder (the marker and its vector) rides the
            // form head; the operands (child, target, methods) start after it. The marker is
            // asked of the ONE recogniser (`wat_reader::is_binder_marker`); the crate-private
            // `types::extend_type_operands` is not reachable from an integration test.
            let skip = if items.get(1).is_some_and(wat_reader::is_binder_marker) { 3 } else { 1 };
            let ops: &[WatAST] = items.get(skip..).unwrap_or(&[]);
            let surface = ops.get(1).map(name_raw).filter(|s| !s.is_empty());
            for meth in ops.iter().skip(2) {
                if let WatAST::List(ch, _) = meth {
                    if ch.len() >= 2 {
                        let mname = name_raw(&ch[0]);
                        let qname = match surface {
                            Some(s) => {
                                format!("{}/{mname}", wat::edn::render::canonical_identity(s))
                            }
                            None => mname.to_string(),
                        };
                        let params = param_names(&ch[1]);
                        for child in ch.iter().skip(2) {
                            walk_defn(child, carrying, Some((qname.as_str(), &params)), lets);
                        }
                    }
                }
            }
            return;
        }
        WatAST::List(items, _) if h == ":wat::core::let" && items.len() >= 3 => {
            let mut env = lets.clone();
            if let Some(WatAST::Vector(binds, _)) = items.get(1) {
                let mut i = 0;
                while i + 1 < binds.len() {
                    if let Some(name) = sym_name(&binds[i]) {
                        env.insert(name.to_string(), binds[i + 1].clone());
                        walk_defn(&binds[i + 1], carrying, cur, &env);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
            }
            for child in items.iter().skip(2) {
                walk_defn(child, carrying, cur, &env);
            }
            return;
        }
        WatAST::List(items, _) => {
            if let Some(idx) = raw_head(n).and_then(|raw| carrying_index(carrying, raw)) {
                if let Some(arg) = items.get(idx + 1) {
                    mark_carrying_expr(arg, carrying, cur, lets);
                }
            }
        }
        _ => {}
    }
    walk_kids(n, carrying, cur, lets);
}

fn walk_kids(
    n: &WatAST,
    carrying: &mut HashSet<(String, usize)>,
    cur: Option<(&str, &[String])>,
    lets: &HashMap<String, WatAST>,
) {
    for c in n.children().iter() {
        walk_defn(c, carrying, cur, lets);
    }
}

fn mark_carrying_expr(
    e: &WatAST,
    carrying: &mut HashSet<(String, usize)>,
    cur: Option<(&str, &[String])>,
    lets: &HashMap<String, WatAST>,
) {
    if let Some(name) = sym_name(e) {
        if let Some(bound) = lets.get(name) {
            mark_carrying_expr(bound, carrying, cur, lets);
            return;
        }
        if let Some((fname, params)) = cur {
            if let Some(i) = params.iter().position(|p| p == name) {
                insert_carrying(carrying, fname, i);
            }
        }
        return;
    }
    if is_head(e, ":wat::core::concat") {
        if let WatAST::List(items, _) = e {
            for op in items.iter().skip(1) {
                mark_carrying_expr(op, carrying, cur, lets);
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Class {
    Checked,
    Assembled,
    Template,
    Data,
}

fn class_rank(c: Class) -> u8 {
    match c {
        Class::Checked => 3,
        Class::Assembled => 2,
        Class::Template => 1,
        Class::Data => 0,
    }
}

/// A let-binding RHS is walked once in its binding context (often Data) and
/// again when a carrying call names it. Keep the stronger class per site.
fn collapse_hits(hits: Vec<Hit>) -> Vec<Hit> {
    let mut best: HashMap<(String, i64), Hit> = HashMap::new();
    for h in hits {
        let key = (h.path.clone(), h.line);
        match best.get(&key) {
            Some(old) if class_rank(old.class) >= class_rank(h.class) => {}
            _ => {
                best.insert(key, h);
            }
        }
    }
    let mut out: Vec<Hit> = best.into_values().collect();
    out.sort_by(|a, b| a.path.cmp(&b.path).then(a.line.cmp(&b.line)));
    out
}

struct Hit {
    path: String,
    line: i64,
    class: Class,
    children: Vec<WatAST>,
}

fn classify_file(
    path: &str,
    forms: &[WatAST],
    carrying: &HashSet<(String, usize)>,
    hits: &mut Vec<Hit>,
) {
    for f in forms {
        walk_class(f, path, carrying, false, None, hits, &HashMap::new());
    }
}

fn walk_class(
    n: &WatAST,
    path: &str,
    carrying: &HashSet<(String, usize)>,
    in_quasi: bool,
    parent_carrying_arg: Option<bool>,
    hits: &mut Vec<Hit>,
    lets: &HashMap<String, WatAST>,
) {
    let h = head_ident(n);
    let in_quasi = in_quasi || h.as_deref() == Some(":wat::core::quasiquote");
    if h.as_deref() == Some(":wat::core::forms") {
        let class = if in_quasi {
            Class::Template
        } else if parent_carrying_arg == Some(true) {
            Class::Checked
        } else {
            Class::Data
        };
        let kids = match n {
            WatAST::List(items, _) => items.iter().skip(1).cloned().collect(),
            _ => Vec::new(),
        };
        hits.push(Hit {
            path: path.to_string(),
            line: n.span().line,
            class,
            children: kids,
        });
    }
    if let WatAST::List(items, _) = n {
        if h.as_deref() == Some(":wat::core::let") && items.len() >= 3 {
            let mut env = lets.clone();
            if let Some(WatAST::Vector(binds, _)) = items.get(1) {
                let mut i = 0;
                while i + 1 < binds.len() {
                    if let Some(name) = sym_name(&binds[i]) {
                        env.insert(name.to_string(), binds[i + 1].clone());
                        walk_class(
                            &binds[i + 1],
                            path,
                            carrying,
                            in_quasi,
                            parent_carrying_arg,
                            hits,
                            &env,
                        );
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
            }
            for c in items.iter().skip(2) {
                walk_class(c, path, carrying, in_quasi, parent_carrying_arg, hits, &env);
            }
            return;
        }
        if h.as_deref() == Some(":wat::core::concat") {
            let concat_is_carrying = parent_carrying_arg == Some(true)
                || carrying.iter().any(|(k, _)| k == ":wat::core::concat");
            for (i, c) in items.iter().enumerate() {
                if i == 0 {
                    continue;
                }
                if is_head(c, ":wat::core::forms") {
                    let kids = match c {
                        WatAST::List(it, _) => it.iter().skip(1).cloned().collect(),
                        _ => Vec::new(),
                    };
                    hits.push(Hit {
                        path: path.to_string(),
                        line: c.span().line,
                        class: if in_quasi {
                            Class::Template
                        } else if concat_is_carrying {
                            Class::Assembled
                        } else {
                            Class::Data
                        },
                        children: kids,
                    });
                } else {
                    walk_class(
                        c,
                        path,
                        carrying,
                        in_quasi,
                        Some(concat_is_carrying),
                        hits,
                        lets,
                    );
                }
            }
            return;
        }
        let carry_idx = raw_head(n).and_then(|raw| carrying_index(carrying, raw));
        for (i, c) in items.iter().enumerate() {
            let is_carry = carry_idx.map(|ci| i == ci + 1).unwrap_or(false);
            if is_carry {
                if let Some(name) = sym_name(c) {
                    if let Some(bound) = lets.get(name) {
                        walk_class(bound, path, carrying, in_quasi, Some(true), hits, lets);
                        continue;
                    }
                }
            }
            walk_class(c, path, carrying, in_quasi, Some(is_carry), hits, lets);
        }
        return;
    }
    for c in n.children().iter() {
        walk_class(c, path, carrying, in_quasi, None, hits, lets);
    }
}

fn rustify_deftest(name: &str) -> String {
    let n = name.trim().trim_start_matches(':');
    format!(
        "deftest_{}",
        // rune:lint(one-variant-separator, namespace) — deftest keyword path (`:wat-tests::process::…`) to a rust fn name, not enum.variant
        n.replace("::", "_").replace(['-', '/'], "_")
    )
}

fn live_tests() -> HashSet<String> {
    let mut s = HashSet::new();
    for p in git_ls(&[]).into_iter().filter(|p| {
        p.ends_with(".wat") || p.ends_with(".rs")
    }) {
        let src = std::fs::read_to_string(repo_path(&p)).unwrap_or_default();
        if p.ends_with(".wat") {
            for line in src.lines() {
                let t = line.trim();
                // Split the '(' off so this prefix is not an EDN-esque `(…)` literal.
                if let Some(rest) = t.strip_prefix('(') {
                    let mut parts = rest.split_whitespace();
                    let head = parts.next().unwrap_or("");
                    let name = parts.next().unwrap_or("");
                    if wat::edn::render::canonical_identity(head) == ":wat::test::deftest"
                        && !name.is_empty()
                    {
                        s.insert(rustify_deftest(name));
                    }
                }
            }
        } else {
            let mut next_is_test = false;
            for line in src.lines() {
                let t = line.trim();
                if t.starts_with("#[test]") {
                    next_is_test = true;
                    continue;
                }
                if next_is_test {
                    if let Some(rest) = t.strip_prefix("fn ") {
                        let name = rest.split('(').next().unwrap_or("").trim();
                        if !name.is_empty() {
                            s.insert(name.to_string());
                        }
                    }
                    next_is_test = false;
                }
            }
        }
    }
    s
}

fn rune_test_name(src: &str, line: i64) -> Option<String> {
    let lines: Vec<&str> = src.lines().collect();
    let idx = (line as usize).saturating_sub(1);
    for i in [idx, idx.saturating_sub(1)] {
        if let Some(l) = lines.get(i) {
            if let Some(rest) = l.split("rune:lint(nested-program").nth(1) {
                if let Some(t) = rest.split("test(").nth(1) {
                    let name = t.split(')').next().unwrap_or("").trim();
                    if !name.is_empty() {
                        return Some(name.to_string());
                    }
                }
            }
        }
    }
    None
}

fn parent_config() -> wat::config::Config {
    collect_entry_file(Vec::new())
        .expect("empty entry file commits a default Config")
        .0
}

fn start_child(forms: Vec<WatAST>) -> Result<(), StartupError> {
    let loader = Arc::new(InMemoryLoader::new());
    let cfg = parent_config();
    startup_from_forms_with_inherit(forms, None, loader, &cfg).map(|_| ())
}

fn run_gate_on_tree() -> (usize, usize, usize, usize, Vec<String>, u128) {
    let t0 = Instant::now();
    let stdlib_paths = git_ls(&["wat/*.wat", "wat/**/*.wat"]);
    let mut stdlib = Vec::new();
    for p in &stdlib_paths {
        let src = std::fs::read_to_string(repo_path(p)).unwrap_or_default();
        let forms = parse_all_with_file(&src, p).unwrap_or_else(|_| Vec::new());
        stdlib.push((p.clone(), forms));
    }
    let carrying = derive_carrying(&stdlib);

    let mut files = git_ls(&[]);
    files.retain(|p| p.ends_with(".wat") && !p.starts_with("wat-scripts/fixes/"));
    let tests = live_tests();
    let mut hits = Vec::new();
    let mut srcs: HashMap<String, String> = HashMap::new();
    for p in &files {
        let src = std::fs::read_to_string(repo_path(p)).unwrap_or_default();
        srcs.insert(p.clone(), src.clone());
        if let Ok(forms) = parse_all_with_file(&src, p) {
            classify_file(p, &forms, &carrying, &mut hits);
        }
    }

    let hits = collapse_hits(hits);

    let mut n_checked = 0usize;
    let mut n_assembled = 0usize;
    let mut n_template = 0usize;
    let mut n_data = 0usize;
    let mut failures = Vec::new();
    for h in &hits {
        match h.class {
            Class::Assembled => n_assembled += 1,
            Class::Template => n_template += 1,
            Class::Data => n_data += 1,
            Class::Checked => {
                n_checked += 1;
                let src = srcs.get(&h.path).map(|s| s.as_str()).unwrap_or("");
                match start_child(h.children.clone()) {
                    Ok(()) => {}
                    Err(e) => {
                        if let Some(tn) = rune_test_name(src, h.line) {
                            if tests.contains(&tn) {
                                continue;
                            }
                            failures.push(format!(
                                "{}:{} rune names missing test `{tn}` (child failed: {e})",
                                h.path, h.line
                            ));
                        } else {
                            failures.push(format!("{}:{} child startup: {e}", h.path, h.line));
                        }
                    }
                }
            }
        }
    }
    (n_checked, n_assembled, n_template, n_data, failures, t0.elapsed().as_millis())
}

#[test]
fn nested_program_literals_start_on_the_child_path() {
    let (checked, assembled, templates, data, failures, ms) = run_gate_on_tree();
    eprintln!(
        "nested-program-gate: checked={checked} assembled={assembled} templates={templates} data={data} wall_ms={ms}"
    );
    assert!(
        failures.is_empty(),
        "{} nested program(s) failed child startup:\n{}",
        failures.len(),
        failures.join("\n")
    );
    let total = checked + assembled + templates + data;
    // NON-VACUITY: the census this floor is measured against found 141 nested-program literals;
    // a total below that means this walk stopped reaching most of them, not that the corpus shrank
    // that far.
    assert!(
        total >= 141,
        "census was 141 literals; got checked={checked} assembled={assembled} templates={templates} data={data} total={total}"
    ); // rune:lint(clock-verdict) — `total` is a deterministic census count, tainted only
       // because it is destructured alongside `ms` (a wall-clock reading) from the same
       // `run_gate_on_tree()` tuple; this detector pools taint over the whole tuple pattern and
       // cannot split its fields. Not clock-derived. STONE 255.78.
}

/// The historical erase child still spells the 24 as `:wat::core::…`. This stone
/// retires those keys before the variant-arm check the gate is measuring, so the
/// temp copy is respelled onto `wat.type/…` and the arm error is what remains.
fn respell_retired_hard_primitives(src: &str) -> String {
    let mut s = src.replace(":wat::WatAST", "wat.type/AST");
    for tail in [
        "PersistentVector",
        "PersistentMap",
        "HashMap",
        "HashSet",
        "String",
        "bigint",
        "rational",
        "keyword",
        "Vector",
        "Record",
        "Struct",
        "Tuple",
        "Bytes",
        "Never",
        "Value",
        "List",
        "char",
        "bool",
        "nil",
        "i64",
        "f64",
        "u8",
        "Fn",
    ] {
        let old: String = [":wat::core::", tail].concat();
        let new: String = ["wat.type/", tail].concat();
        s = s.replace(&old, &new);
    }
    s
}

#[test]
fn nested_program_gate_goes_red_on_the_pre_2b_erase_child() {
    let out = Command::new("git")
        .args([
            "show",
            "f2e0ac26b^:wat-scripts/probes/arc-170/probe-m1-ann-erase.wat",
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("git show old erase");
    assert!(out.status.success(), "git show f2e0ac26b^ erase failed");
    let src = respell_retired_hard_primitives(&String::from_utf8_lossy(&out.stdout));
    let tmp = std::env::temp_dir().join("nested-program-erase-old.wat");
    std::fs::write(&tmp, src.as_bytes()).expect("write temp erase");
    let path = tmp.to_str().unwrap();
    let forms = parse_all_with_file(&src, path).expect("parse old erase");
    let stdlib_paths = git_ls(&["wat/*.wat", "wat/**/*.wat"]);
    let mut stdlib = Vec::new();
    for p in &stdlib_paths {
        let s = std::fs::read_to_string(repo_path(p)).unwrap_or_default();
        let f = parse_all_with_file(&s, p).unwrap_or_else(|_| Vec::new());
        stdlib.push((p.clone(), f));
    }
    let carrying = derive_carrying(&stdlib);
    let mut hits = Vec::new();
    classify_file(path, &forms, &carrying, &mut hits);
    let checked: Vec<_> = hits.iter().filter(|h| h.class == Class::Checked).collect();
    assert!(
        !checked.is_empty(),
        "old erase should have a checked child"
    );
    let mut saw = false;
    for h in checked {
        assert_eq!(h.line, 34, "old erase child sits at line 34, got {}", h.line);
        let result = start_child(h.children.clone());
        wat::assert_startup_error!(
            result,
            check CheckErrorKind::MalformedForm { head, reason, .. }
                if head == ":wat::core::match"
                    && reason
                        == "arm #1: variant arm head `:probe::CMsg::Setup` is not namespaced; write `<enum>.<Variant>`"
        );
        eprintln!(
            "RED as required: {}:{} {:?}",
            h.path,
            h.line,
            result.as_ref().err()
        );
        saw = true;
    }
    assert!(saw, "old erase child must fail startup (E1)");
    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn nested_program_gate_refuses_a_rune_whose_test_does_not_exist() {
    let out = Command::new("git")
        .args([
            "show",
            "f2e0ac26b^:wat-scripts/probes/arc-170/probe-m1-ann-erase.wat",
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("git show old erase");
    assert!(out.status.success(), "git show f2e0ac26b^ erase failed");
    let mut needle = String::from(":wat::core::forms");
    needle.insert(0, '(');
    let rune = ";; rune:lint(nested-program, expected) — test(no_such_nested_program_gate_test)\n              ";
    let src = respell_retired_hard_primitives(
        &String::from_utf8_lossy(&out.stdout).replacen(&needle, &format!("{rune}{needle}"), 1),
    );
    let tmp = std::env::temp_dir().join("nested-program-missing-rune-test.wat");
    std::fs::write(&tmp, src.as_bytes()).expect("write temp sabotage");
    let path = tmp.to_str().unwrap();
    let forms = parse_all_with_file(&src, path).expect("parse sabotaged erase");
    let stdlib_paths = git_ls(&["wat/*.wat", "wat/**/*.wat"]);
    let mut stdlib = Vec::new();
    for p in &stdlib_paths {
        let s = std::fs::read_to_string(repo_path(p)).unwrap_or_default();
        let f = parse_all_with_file(&s, p).unwrap_or_else(|_| Vec::new());
        stdlib.push((p.clone(), f));
    }
    let carrying = derive_carrying(&stdlib);
    let mut hits = Vec::new();
    classify_file(path, &forms, &carrying, &mut hits);
    let checked: Vec<_> = hits.iter().filter(|h| h.class == Class::Checked).collect();
    assert!(
        !checked.is_empty(),
        "sabotaged erase should have a checked child"
    );
    let tests = live_tests();
    let mut refused = false;
    for h in checked {
        let result = start_child(h.children.clone());
        wat::assert_startup_error!(
            result,
            check CheckErrorKind::MalformedForm { head, .. }
                if head == ":wat::core::match"
        );
        let tn = rune_test_name(&src, h.line).expect("spliced rune must be found");
        assert_eq!(tn, "no_such_nested_program_gate_test");
        assert!(
            !tests.contains(&tn),
            "sabotage name must not be a live test"
        );
        refused = true;
    }
    assert!(
        refused,
        "a rune naming a missing test must be refused (E5)"
    );
    let _ = std::fs::remove_file(&tmp);
}

/// Keyword and symbol heads of a nested-program form are one identity.
#[test]
fn head_ident_keyword_and_symbol_are_one() {
    fn opened(parts: &[&str]) -> String {
        let mut s = String::new();
        s.push('(');
        s.push_str(&parts.join(" "));
        s.push(')');
        s
    }
    let kw = parse_all_with_file(&opened(&[":wat::core::forms", "1"]), "kw.wat").expect("parse keyword");
    let sy = parse_all_with_file(&opened(&["wat.core/forms", "1"]), "sy.wat").expect("parse symbol");
    assert_eq!(head_ident(&kw[0]), head_ident(&sy[0]));
    assert_eq!(head_ident(&kw[0]).as_deref(), Some(":wat::core::forms"));
    let mut empty = String::new();
    empty.push('[');
    empty.push(']');
    let kw_name = parse_all_with_file(
        &opened(&[":wat::core::defn", ":wat::kernel::spawn-program", &empty, "1"]),
        "kw-name.wat",
    )
    .expect("parse keyword defn");
    let sy_name = parse_all_with_file(
        &opened(&["wat.core/defn", "wat.kernel/spawn-program", &empty, "1"]),
        "sy-name.wat",
    )
    .expect("parse symbol defn");
    let kw_list = match &kw_name[0] {
        WatAST::List(items, _) => items,
        _ => panic!("keyword defn is a list"),
    };
    let sy_list = match &sy_name[0] {
        WatAST::List(items, _) => items,
        _ => panic!("symbol defn is a list"),
    };
    assert_eq!(
        carrying_keys(name_raw(&kw_list[1])),
        carrying_keys(name_raw(&sy_list[1]))
    );
}
