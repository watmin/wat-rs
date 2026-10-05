//! Stone 255.94 — which `::` keywords are names, and which are data.
//!
//! An example, not a test. It walks wat's own reader tree. It does not match
//! keyword text with a regex. The scratch runtime counter is not this file.
//!
//! ```text
//! keyword_not_a_name [--root DIR] [--out FILE] [--proof]
//! ```
//!
//! `--proof` parses one planted program and checks each marked spelling
//! landed on the position this file claims. The planted program is not a
//! corpus file.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use syn::spanned::Spanned;
use wat::codemod_driver::is_candidate_wat;
use wat::embedded_wat::{extract_literal_spans, replace_placeholders_preserving_len};
use wat::{parse_all_with_file, Name, WatAST};

const DECL_LEAVES: &[&str] = &[
    "defn",
    "defn-",
    "defmacro",
    "def",
    "defclause",
    "defrecord",
    "defenum",
    "defstruct",
    "structtype",
    "deftest",
    "defservice",
    "defsurface",
    "defalias",
    "extend-type",
    "derive",
    "declare-acronyms",
    "defrule",
    "defquery",
];

#[derive(Clone, Copy)]
struct Frame {
    quote: u32,
    type_pos: bool,
    pattern: bool,
    rete: bool,
    fact_map: bool,
}

struct Hit {
    population: &'static str,
    file: String,
    line: i64,
    col: i64,
    spelling: String,
    position: String,
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let mut root = PathBuf::from(".");
    let mut out: Option<PathBuf> = None;
    let mut proof = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--root" => {
                root = PathBuf::from(args.next().expect("--root needs a directory"));
            }
            "--out" => {
                out = Some(PathBuf::from(args.next().expect("--out needs a path")));
            }
            "--proof" => proof = true,
            other => {
                eprintln!("unknown argument {other}");
                return ExitCode::from(2);
            }
        }
    }

    if proof && !run_proof() {
        return ExitCode::from(1);
    }

    let mut hits: Vec<Hit> = Vec::new();
    let mut parse_fail: Vec<(String, String, String)> = Vec::new();
    let mut files: BTreeMap<&'static str, u64> = BTreeMap::new();

    for (population, dir) in [
        ("stdlib", "wat"),
        ("corpus", "wat-scripts"),
        ("corpus", "wat-tests"),
        ("corpus", "tests"),
        ("other", "docs"),
        ("other", "examples"),
        ("other", "crates"),
        ("other", "benches"),
    ] {
        let base = root.join(dir);
        if !base.is_dir() {
            continue;
        }
        let mut paths = Vec::new();
        collect(&base, "wat", &mut paths);
        files.entry(population).or_default();
        *files.get_mut(population).expect("just inserted") += paths.len() as u64;
        for path in paths {
            let text = match fs::read_to_string(&path) {
                Ok(text) => text,
                Err(err) => {
                    parse_fail.push((
                        population.to_string(),
                        path.display().to_string(),
                        err.to_string(),
                    ));
                    continue;
                }
            };
            let rel = display_rel(&root, &path);
            match parse_all_with_file(&text, &rel) {
                Ok(forms) => walk_forms(&forms, population, &rel, 0, &mut hits),
                Err(err) => parse_fail.push((population.to_string(), rel, err.to_string())),
            }
        }
    }

    let mut rust_files = 0u64;
    let mut rust_literals = 0u64;
    for dir in ["src", "crates", "tests"] {
        let base = root.join(dir);
        if !base.is_dir() {
            continue;
        }
        let mut paths = Vec::new();
        collect(&base, "rs", &mut paths);
        rust_files += paths.len() as u64;
        for path in paths {
            let text = match fs::read_to_string(&path) {
                Ok(text) => text,
                Err(err) => {
                    parse_fail.push((
                        "rust".to_string(),
                        path.display().to_string(),
                        err.to_string(),
                    ));
                    continue;
                }
            };
            let rel = display_rel(&root, &path);
            scan_rust(&rel, &text, &mut hits, &mut rust_literals);
        }
    }

    let mut lines = Vec::new();
    lines.push("kind\tpopulation\ta\tb\tc\td".to_string());
    for (population, count) in &files {
        lines.push(format!("files\t{population}\t{count}\t\t\t"));
    }
    lines.push(format!("files\trust\t{rust_files}\t\t\t"));
    lines.push(format!("rust-literals-tried\trust\t{rust_literals}\t\t\t"));

    emit_tables(&hits, &parse_fail, &mut lines);
    lines.extend(intrinsic_rows(&root));

    let body = lines.join("\n") + "\n";
    if let Some(path) = out {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).expect("out directory");
            }
        }
        fs::write(&path, &body).expect("write tsv");
        eprintln!("wrote {} rows to {}", lines.len(), path.display());
    } else {
        print!("{body}");
    }
    ExitCode::SUCCESS
}

fn emit_tables(hits: &[Hit], parse_fail: &[(String, String, String)], lines: &mut Vec<String>) {
    let mut by_pos: BTreeMap<(&str, &str), u64> = BTreeMap::new();
    let mut spellings_at: BTreeMap<(&str, &str), BTreeSet<&str>> = BTreeMap::new();
    let mut by_file: BTreeMap<(&str, &str, &str), u64> = BTreeMap::new();
    let mut spelling_pos: BTreeMap<(&str, &str), BTreeSet<&str>> = BTreeMap::new();
    let mut spelling_count: BTreeMap<(&str, &str), u64> = BTreeMap::new();
    let mut spelling_enter: BTreeMap<(&str, &str), bool> = BTreeMap::new();
    let mut spelling_from: BTreeMap<(&str, &str), bool> = BTreeMap::new();
    let mut example: BTreeMap<(&str, &str), (String, i64, i64)> = BTreeMap::new();
    let mut total: BTreeMap<&str, u64> = BTreeMap::new();

    for hit in hits {
        *total.entry(hit.population).or_default() += 1;
        *by_pos
            .entry((hit.population, hit.position.as_str()))
            .or_default() += 1;
        spellings_at
            .entry((hit.population, hit.position.as_str()))
            .or_default()
            .insert(hit.spelling.as_str());
        *by_file
            .entry((hit.population, hit.position.as_str(), hit.file.as_str()))
            .or_default() += 1;
        spelling_pos
            .entry((hit.population, hit.spelling.as_str()))
            .or_default()
            .insert(hit.position.as_str());
        *spelling_count
            .entry((hit.population, hit.spelling.as_str()))
            .or_default() += 1;
        let entered = Name::enter(&hit.spelling).is_some();
        let from = Name::from_keyword(&hit.spelling).is_some();
        spelling_enter.insert((hit.population, hit.spelling.as_str()), entered);
        spelling_from.insert((hit.population, hit.spelling.as_str()), from);
        example
            .entry((hit.population, hit.spelling.as_str()))
            .or_insert_with(|| (hit.file.clone(), hit.line, hit.col));
    }

    for (population, count) in &total {
        lines.push(format!("denominator\t{population}\t{count}\t\t\t"));
    }
    for ((population, position), count) in &by_pos {
        let distinct = spellings_at
            .get(&(population, position))
            .map(|set| set.len())
            .unwrap_or(0);
        lines.push(format!(
            "position\t{population}\t{position}\t{count}\t{distinct}\t"
        ));
    }

    let mut top: BTreeMap<(&str, &str), Vec<(&str, u64)>> = BTreeMap::new();
    for ((population, position, file), count) in &by_file {
        top.entry((population, position))
            .or_default()
            .push((file, *count));
    }
    for ((population, position), files) in &mut top {
        files.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
        for (file, count) in files.iter().take(8) {
            lines.push(format!(
                "top-file\t{population}\t{position}\t{file}\t{count}\t"
            ));
        }
    }

    for ((population, spelling), positions) in &spelling_pos {
        let entered = spelling_enter[&(*population, *spelling)];
        let from = spelling_from[&(*population, *spelling)];
        let verdict = verdict(entered, positions);
        if verdict == "NAME" {
            continue;
        }
        let (file, line, col) = &example[&(*population, *spelling)];
        let mut pos_list: Vec<&str> = positions.iter().copied().collect();
        pos_list.sort_unstable();
        let count = spelling_count[&(*population, *spelling)];
        lines.push(format!(
            "ruling\t{population}\t{verdict}\t{spelling}\t{}\t{count}\t{file}:{line}:{col}\tfrom_keyword={from}\tenter={entered}",
            pos_list.join(",")
        ));
    }

    let mut name_spellings: BTreeMap<&str, u64> = BTreeMap::new();
    let mut data_spellings: BTreeMap<&str, u64> = BTreeMap::new();
    let mut mixed_spellings: BTreeMap<&str, u64> = BTreeMap::new();
    for ((population, spelling), positions) in &spelling_pos {
        match verdict(spelling_enter[&(*population, *spelling)], positions) {
            "NAME" => *name_spellings.entry(population).or_default() += 1,
            "DATA" => *data_spellings.entry(population).or_default() += 1,
            "MIXED" => *mixed_spellings.entry(population).or_default() += 1,
            _ => {}
        }
    }
    for (population, count) in &name_spellings {
        lines.push(format!("name-spellings\t{population}\t{count}\t\t\t"));
    }
    if *name_spellings.get("stdlib").unwrap_or(&0) <= 32 {
        for ((population, spelling), positions) in &spelling_pos {
            if *population != "stdlib" {
                continue;
            }
            if verdict(spelling_enter[&(*population, *spelling)], positions) != "NAME" {
                continue;
            }
            let mut pos_list: Vec<&str> = positions.iter().copied().collect();
            pos_list.sort_unstable();
            let count = spelling_count[&(*population, *spelling)];
            lines.push(format!(
                "stdlib-name\tstdlib\t{spelling}\t{}\t{count}\t",
                pos_list.join(",")
            ));
        }
    }
    for (population, count) in &data_spellings {
        lines.push(format!("data-spellings\t{population}\t{count}\t\t\t"));
    }
    for (population, count) in &mixed_spellings {
        lines.push(format!("mixed-spellings\t{population}\t{count}\t\t\t"));
    }

    lines.push(format!(
        "parse-fail-count\tall\t{}\t\t\t",
        parse_fail.len()
    ));
    for (population, file, err) in parse_fail {
        let flat = err.replace(['\n', '\t'], " ");
        lines.push(format!("parse-fail\t{population}\t{file}\t{flat}\t\t"));
    }
}

fn verdict(entered: bool, positions: &BTreeSet<&str>) -> &'static str {
    if !entered {
        return "DATA";
    }
    let name_here = positions.iter().any(|pos| {
        matches!(
            *pos,
            "call-head"
                | "declaration-name"
                | "type"
                | "match-variant"
                | "reference"
                | "rete"
                | "fact-field"
                | "quote"
        )
    });
    let stored = positions
        .iter()
        .any(|pos| matches!(*pos, "map-key" | "map-value" | "set" | "top"));
    if name_here && stored {
        "MIXED"
    } else if name_here {
        "NAME"
    } else {
        "DATA"
    }
}

fn scan_rust(rel: &str, text: &str, hits: &mut Vec<Hit>, tried: &mut u64) {
    let chars: Vec<char> = text.chars().collect();
    let spans = std::panic::catch_unwind(|| extract_literal_spans(&chars));
    let Ok(spans) = spans else {
        return;
    };
    for literal in spans {
        if !literal.decoded.contains("::") {
            continue;
        }
        let substituted = replace_placeholders_preserving_len(&literal.decoded);
        if !is_candidate_wat(&substituted) {
            continue;
        }
        *tried += 1;
        let parsed = std::panic::catch_unwind(|| parse_all_with_file(&substituted, rel));
        let Ok(Ok(forms)) = parsed else {
            continue;
        };
        let start_line = chars[..literal.raw_quote_start.min(chars.len())]
            .iter()
            .filter(|ch| **ch == '\n')
            .count() as i64
            + 1;
        let before = hits.len();
        walk_forms(&forms, "rust", rel, 0, hits);
        for hit in &mut hits[before..] {
            hit.line = start_line + hit.line - 1;
        }
    }
}

fn walk_forms(forms: &[WatAST], population: &'static str, file: &str, line_base: i64, hits: &mut Vec<Hit>) {
    let _ = line_base;
    let frame = Frame {
        quote: 0,
        type_pos: false,
        pattern: false,
        rete: false,
        fact_map: false,
    };
    for form in forms {
        walk(form, population, file, frame, "top", hits);
    }
}

fn walk(
    node: &WatAST,
    population: &'static str,
    file: &str,
    frame: Frame,
    role: &str,
    hits: &mut Vec<Hit>,
) {
    match node {
        WatAST::List(items, _) if !items.is_empty() => walk_list(items, population, file, frame, hits),
        WatAST::Vector(items, _) => {
            for item in items {
                walk(item, population, file, frame, "vec", hits);
            }
        }
        WatAST::Map(pairs, _) => {
            for (key, value) in pairs {
                let key_role = if frame.fact_map || frame.pattern {
                    "fact-field"
                } else {
                    "map-key"
                };
                walk(key, population, file, frame, key_role, hits);
                walk(value, population, file, frame, "map-value", hits);
            }
        }
        WatAST::Set(items, _) => {
            for item in items {
                walk(item, population, file, frame, "set", hits);
            }
        }
        WatAST::Keyword(text, span) if text.contains("::") => {
            let position = position_of(frame, role);
            hits.push(Hit {
                population,
                file: file.to_string(),
                line: span.line,
                col: span.col,
                spelling: text.clone(),
                position,
            });
        }
        _ => {}
    }
}

fn position_of(frame: Frame, role: &str) -> String {
    if frame.quote > 0 {
        return "quote".to_string();
    }
    if frame.type_pos && role != "fact-field" {
        return "type".to_string();
    }
    match role {
        "list-head" if frame.pattern => "match-variant".to_string(),
        "list-head" => "call-head".to_string(),
        "decl-name" => "declaration-name".to_string(),
        "type" => "type".to_string(),
        "pattern" | "vec" if frame.pattern => "match-variant".to_string(),
        "fact-field" => "fact-field".to_string(),
        "map-key" => "map-key".to_string(),
        "map-value" => "map-value".to_string(),
        "set" => "set".to_string(),
        "top" => "top".to_string(),
        "arg" | "vec" if frame.rete => "rete".to_string(),
        "arg" | "vec" => "reference".to_string(),
        other => format!("undecided:{other}"),
    }
}

fn walk_list(items: &[WatAST], population: &'static str, file: &str, frame: Frame, hits: &mut Vec<Hit>) {
    let leaf = head_leaf(&items[0]);
    let rete = frame.rete || head_is_rete(&items[0]);
    let ctor = head_is_constructor(&items[0]);
    let colon_type = items.get(1).is_some_and(is_colon_dash);

    let head_frame = Frame {
        type_pos: frame.type_pos || colon_type,
        rete,
        ..frame
    };
    let head_role = if head_frame.type_pos { "type" } else { "list-head" };
    walk(&items[0], population, file, head_frame, head_role, hits);

    if is_quote_leaf(leaf.as_deref()) {
        let inner = Frame {
            quote: frame.quote.saturating_add(1),
            rete,
            ..frame
        };
        for item in items.iter().skip(1) {
            walk(item, population, file, inner, "arg", hits);
        }
        return;
    }
    if is_unquote_leaf(leaf.as_deref()) {
        let inner = Frame {
            quote: frame.quote.saturating_sub(1),
            rete,
            ..frame
        };
        for item in items.iter().skip(1) {
            walk(item, population, file, inner, "arg", hits);
        }
        return;
    }
    if leaf.as_deref() == Some("match") && frame.quote == 0 {
        if let Some(scrut) = items.get(1) {
            walk(
                scrut,
                population,
                file,
                Frame { rete, ..frame },
                "arg",
                hits,
            );
        }
        for arm in items.iter().skip(2) {
            walk_match_arm(arm, population, file, Frame { rete, ..frame }, hits);
        }
        return;
    }
    if leaf.as_deref().is_some_and(|leaf| DECL_LEAVES.contains(&leaf)) && frame.quote == 0 {
        if let Some(name) = items.get(1) {
            walk(
                name,
                population,
                file,
                Frame { rete, ..frame },
                "decl-name",
                hits,
            );
        }
        if items.len() > 2 {
            walk_after_name(
                &items[2..],
                population,
                file,
                Frame { rete, ..frame },
                ctor,
                hits,
            );
        }
        return;
    }
    if colon_type {
        let inner = Frame {
            type_pos: true,
            rete,
            ..frame
        };
        for item in items.iter().skip(1) {
            walk(item, population, file, inner, "type", hits);
        }
        return;
    }
    // A binder spec is a list whose second child is `->`:
    // `((g :fn(T)->R) -> ret)`. The parser does not mark the type slot;
    // the arrow is the slot. Children before the arrow are binders.
    if frame.quote == 0 && items.get(1).is_some_and(is_arrow) {
        let typed = Frame {
            type_pos: true,
            rete,
            ..frame
        };
        walk(&items[0], population, file, typed, "type", hits);
        walk(&items[1], population, file, typed, "type", hits);
        if let Some(ret) = items.get(2) {
            walk(ret, population, file, typed, "type", hits);
        }
        for item in items.iter().skip(3) {
            walk(item, population, file, Frame { rete, ..frame }, "arg", hits);
        }
        return;
    }

    let inner = Frame {
        rete,
        fact_map: ctor && !frame.type_pos,
        ..frame
    };
    for item in items.iter().skip(1) {
        walk(item, population, file, inner, "arg", hits);
    }
}

fn walk_after_name(
    items: &[WatAST],
    population: &'static str,
    file: &str,
    frame: Frame,
    ctor: bool,
    hits: &mut Vec<Hit>,
) {
    let mut index = 0;
    let mut seen_arrow = false;
    while index < items.len() {
        if is_arrow(&items[index]) {
            seen_arrow = true;
            walk(
                &items[index],
                population,
                file,
                Frame {
                    type_pos: true,
                    ..frame
                },
                "type",
                hits,
            );
            index += 1;
            if index < items.len() {
                walk(
                    &items[index],
                    population,
                    file,
                    Frame {
                        type_pos: true,
                        ..frame
                    },
                    "type",
                    hits,
                );
                index += 1;
            }
            continue;
        }
        if !seen_arrow && matches!(items[index], WatAST::Vector(_, _)) {
            walk_params(&items[index], population, file, frame, hits);
            index += 1;
            continue;
        }
        walk(
            &items[index],
            population,
            file,
            Frame {
                fact_map: ctor,
                ..frame
            },
            "arg",
            hits,
        );
        index += 1;
    }
}

fn walk_params(node: &WatAST, population: &'static str, file: &str, frame: Frame, hits: &mut Vec<Hit>) {
    let WatAST::Vector(items, _) = node else {
        return;
    };
    for item in items {
        match item {
            WatAST::Keyword(_, _) => walk(
                item,
                population,
                file,
                Frame {
                    type_pos: true,
                    ..frame
                },
                "type",
                hits,
            ),
            WatAST::List(_, _) | WatAST::Vector(_, _) => walk(
                item,
                population,
                file,
                Frame {
                    type_pos: true,
                    ..frame
                },
                "type",
                hits,
            ),
            _ => walk(item, population, file, frame, "arg", hits),
        }
    }
}

fn walk_match_arm(arm: &WatAST, population: &'static str, file: &str, frame: Frame, hits: &mut Vec<Hit>) {
    let items: Vec<&WatAST> = match arm {
        WatAST::Vector(items, _) | WatAST::List(items, _) => items.iter().collect(),
        other => {
            walk(
                other,
                population,
                file,
                Frame {
                    pattern: true,
                    ..frame
                },
                "pattern",
                hits,
            );
            return;
        }
    };
    if items.is_empty() {
        return;
    }
    let last = items.len() - 1;
    for (index, item) in items.iter().enumerate() {
        if index == last {
            walk(item, population, file, frame, "arg", hits);
        } else {
            walk(
                item,
                population,
                file,
                Frame {
                    pattern: true,
                    ..frame
                },
                "pattern",
                hits,
            );
        }
    }
}

fn head_leaf(node: &WatAST) -> Option<String> {
    match node {
        WatAST::Keyword(text, _) => Name::from_keyword(text)
            .map(|name| name.name().to_string())
            .or_else(|| Some(text.clone())),
        WatAST::Symbol(id, _) => {
            let method = id.method();
            if !method.is_empty() {
                Some(method.to_string())
            } else {
                Some(id.leaf().to_string())
            }
        }
        _ => None,
    }
}

fn head_is_rete(node: &WatAST) -> bool {
    match node {
        WatAST::Keyword(text, _) => text.contains("::rete::") || text.contains("::rete/"),
        WatAST::Symbol(id, _) => {
            id.namespace().contains("rete") || id.as_str().contains("rete/")
        }
        _ => false,
    }
}

fn head_is_constructor(node: &WatAST) -> bool {
    let Some(leaf) = head_leaf(node) else {
        return false;
    };
    leaf.chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_uppercase())
}

fn is_colon_dash(node: &WatAST) -> bool {
    match node {
        WatAST::Symbol(id, _) => id.as_str() == ":-",
        WatAST::Keyword(text, _) => text == ":-" || Name::from_keyword(text).is_some_and(|name| name.name() == ":-"),
        _ => false,
    }
}

fn is_arrow(node: &WatAST) -> bool {
    match node {
        WatAST::Symbol(id, _) => id.as_str() == "->",
        WatAST::Keyword(text, _) => {
            text == ":->" || Name::from_keyword(text).is_some_and(|name| name.name() == "->")
        }
        _ => false,
    }
}

fn is_quote_leaf(leaf: Option<&str>) -> bool {
    matches!(leaf, Some("quote" | "quasiquote"))
}

fn is_unquote_leaf(leaf: Option<&str>) -> bool {
    matches!(leaf, Some("unquote" | "unquote-splicing"))
}

fn collect(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if path.is_dir() {
            if name == "target" || name.starts_with('.') {
                continue;
            }
            collect(&path, ext, out);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some(ext) {
            out.push(path);
        }
    }
}

fn display_rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

const PROOF: &str = r#"
(:wat::core::defn :proof::declared [] -> :wat::type::i64
  (:wat::core::i64/+ 1 2))
(:wat::core::defn :proof::uses [] :wat::core::-> :wat::type::i64
  :proof::declared)
(:wat::core::defn :proof::branch [x :wat::type::i64] -> :wat::type::nil
  (:wat::core::match x
    [(:wat::core::Option::Some v) v]
    [:wat::core::Option::None nil]))
(:wat::core::defn :proof::map [] -> :wat::type::nil
  {:proof::key :proof::val})
(:wat::core::defn :proof::fact [] -> :wat::type::nil
  (:proof::Person {:proof::name "a"}))
(:wat::core::defn :proof::quoted [] -> :wat::type::nil
  (:wat::core::quote :proof::inside-quote))
(:wat::rete::insert :proof::fact-class)
(:wat::core::defn :proof::grouped [] -> :wat::type::nil
  #{:proof::in-set})
:proof::top
(:wat::core::fn(T)->R)
"#;

fn intrinsic_rows(root: &Path) -> Vec<String> {
    let mut rows = Vec::new();
    for dir in ["src/intrinsic", "src/reflect"] {
        let base = root.join(dir);
        if !base.is_dir() {
            continue;
        }
        let mut paths = Vec::new();
        collect(&base, "rs", &mut paths);
        for path in paths {
            let text = match fs::read_to_string(&path) {
                Ok(text) => text,
                Err(_) => continue,
            };
            let rel = display_rel(root, &path);
            let file = match syn::parse_file(&text) {
                Ok(file) => file,
                Err(_) => continue,
            };
            let mut visitor = IntrinsicVisit {
                rel,
                fn_name: String::new(),
                intrinsic: false,
                rows: Vec::new(),
            };
            syn::visit::Visit::visit_file(&mut visitor, &file);
            rows.extend(visitor.rows);
        }
    }
    rows.sort();
    rows
}

struct IntrinsicVisit {
    rel: String,
    fn_name: String,
    intrinsic: bool,
    rows: Vec<String>,
}

impl<'ast> syn::visit::Visit<'ast> for IntrinsicVisit {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let previous = (self.fn_name.clone(), self.intrinsic);
        self.fn_name = node.sig.ident.to_string();
        self.intrinsic = node.attrs.iter().any(|attr| {
            attr.path()
                .segments
                .last()
                .is_some_and(|seg| seg.ident == "wat_intrinsic")
        });
        syn::visit::visit_item_fn(self, node);
        self.fn_name = previous.0;
        self.intrinsic = previous.1;
    }

    fn visit_expr_macro(&mut self, node: &'ast syn::ExprMacro) {
        if let Ok(exprs) = node
            .mac
            .parse_body_with(syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated)
        {
            for expr in &exprs {
                self.visit_expr(expr);
            }
        }
        syn::visit::visit_expr_macro(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if call_builds_keyword(&node.func) {
            let line = node.func.span().start().line;
            let flag = if self.intrinsic { "wat_intrinsic" } else { "helper" };
            let built = keyword_payload(node);
            self.rows.push(format!(
                "intrinsic\t{}\t{}\t{}\t{flag}\t{built}",
                self.rel, line, self.fn_name
            ));
        }
        syn::visit::visit_expr_call(self, node);
    }
}

fn call_builds_keyword(func: &syn::Expr) -> bool {
    let syn::Expr::Path(path) = func else {
        return false;
    };
    path.path
        .segments
        .last()
        .is_some_and(|seg| seg.ident == "wat__core__keyword")
}

fn keyword_payload(call: &syn::ExprCall) -> String {
    let Some(arg) = call.args.first() else {
        return "no-arg".to_string();
    };
    match arg {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(text),
            ..
        }) => text.value(),
        other => {
            let text = quote_expr(other);
            if text.chars().count() > 80 {
                let short: String = text.chars().take(80).collect();
                format!("{short}…")
            } else {
                text
            }
        }
    }
}

fn quote_expr(expr: &syn::Expr) -> String {
    match expr {
        syn::Expr::MethodCall(call) => {
            let method = call.method.to_string();
            format!(".{method}()")
        }
        syn::Expr::Call(call) => {
            if let syn::Expr::Path(path) = &*call.func {
                if let Some(seg) = path.path.segments.last() {
                    return format!("{}(…)", seg.ident);
                }
            }
            "call".to_string()
        }
        syn::Expr::Path(path) => path
            .path
            .segments
            .last()
            .map(|seg| seg.ident.to_string())
            .unwrap_or_else(|| "path".to_string()),
        syn::Expr::Macro(mac) => mac
            .mac
            .path
            .segments
            .last()
            .map(|seg| format!("{}!", seg.ident))
            .unwrap_or_else(|| "macro!".to_string()),
        _ => "expr".to_string(),
    }
}

fn run_proof() -> bool {
    let src = PROOF;
    let forms = match parse_all_with_file(src, "keyword_not_a_name_proof.wat") {
        Ok(forms) => forms,
        Err(err) => {
            eprintln!("proof did not parse: {err}");
            return false;
        }
    };
    let mut hits = Vec::new();
    walk_forms(&forms, "proof", "keyword_not_a_name_proof.wat", 0, &mut hits);
    let want = [
        (":wat::core::defn", "call-head"),
        (":proof::declared", "declaration-name"),
        (":wat::type::i64", "type"),
        (":wat::core::i64/+", "call-head"),
        (":proof::declared", "reference"),
        (":wat::core::Option::Some", "match-variant"),
        (":proof::key", "map-key"),
        (":proof::val", "map-value"),
        (":proof::Person", "call-head"),
        (":proof::name", "fact-field"),
        (":proof::inside-quote", "quote"),
        (":wat::rete::insert", "call-head"),
        (":proof::fact-class", "rete"),
        (":proof::in-set", "set"),
        (":proof::top", "top"),
        (":wat::core::->", "type"),
    ];
    let mut ok = true;
    for (spelling, position) in want {
        let found = hits.iter().any(|hit| hit.spelling == spelling && hit.position == position);
        if !found {
            eprintln!("proof missed {spelling} at {position}");
            ok = false;
        }
    }
    let undecided: Vec<&Hit> = hits
        .iter()
        .filter(|hit| hit.position.starts_with("undecided:"))
        .collect();
    if !undecided.is_empty() {
        eprintln!("proof left {} keyword(s) undecided", undecided.len());
        ok = false;
    }
    if let Some(hit) = hits.iter().find(|hit| hit.spelling.contains('(')) {
        let entered = Name::enter(&hit.spelling).is_some();
        let from = Name::from_keyword(&hit.spelling).is_some();
        eprintln!(
            "proof rendered spelling {} position {} from_keyword={from} enter={entered}",
            hit.spelling, hit.position
        );
    }
    if ok {
        eprintln!("proof ok, {} :: keyword(s)", hits.len());
    } else {
        eprintln!("proof hits:");
        for hit in &hits {
            eprintln!("  {} {} {}:{}", hit.position, hit.spelling, hit.line, hit.col);
        }
    }
    ok
}
