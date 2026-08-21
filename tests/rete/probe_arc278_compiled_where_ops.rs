//! DISCONFIRMING PROBE — DESIGN-STONE-compiled-where, drawn BEFORE the brief (FM 2-bis).
//!
//! The stone's `Op` set is a CLAIM about what a `where` predicate can be compiled into. Two pieces
//! of it are non-trivial compositions the brief must not assert, because if either fails the op set
//! shrinks and the rider would discover it mid-strike:
//!
//!   1. **A record accessor without an `Environment`.** The corpus's most common non-arithmetic
//!      shape is `(:arena::Route/status ?route)`. Today that reaches `eval_inner`'s head dispatch,
//!      misses user-fn lookup / def-bound / sandbox, and lands in the LAST arm — the
//!      keyword-as-accessor fall-through (`runtime.rs:6053-6097`), which calls
//!      `keyword_accessor_record` (`:6119`) and LINEAR-SCANS the field names. The compiled executor
//!      must reach that value with no `Environment` and no head dispatch. `keyword_accessor_record`
//!      is a PRIVATE fn in `runtime.rs`, so this probe measures whether the value is reachable at
//!      all from outside — and by what door.
//!
//!   2. **Dimension derivation (the (b) scout).** The DDoS lab's tree is tractable because
//!      `FieldDim` is a DECLARED enum of 15 fixed dimensions (`holon-lab-ddos/veth-lab/filter/
//!      src/tree.rs:46`). Ours would have to DERIVE dimensions by canonicalizing arbitrary
//!      expressions. This probe asks the corpus the question directly: across the real `where`
//!      predicates, how many DISTINCT key expressions are there, and how many rules share one? That
//!      ratio is what decides whether (b) is a tree or a rounding error — and it must be known
//!      before (a) commits to an `Op` shape, because (b) reads that shape.
//!
//! RED BY CONSTRUCTION: `crate::rete::compiled_where` does not exist. This probe does NOT reference
//! it — it probes the SUBSTRATE the module would stand on, so it compiles and runs at HEAD and
//! reports what is and is not reachable. It goes GREEN today or it names the gap; either way the
//! brief is written against a measurement.

use wat::freeze::{call_beside_value, startup_beside};
use wat::runtime::Value;

/// STOP-1's question, answered by a RUN: can a compiled executor read
/// `(:p::Route/status <record>)` without building an `Environment`?
///
/// The fixture beside this file supplies both halves. `:p::status-via-accessor` reads the field the
/// way the interpreter does today — through the keyword-as-accessor fall-through, the LAST arm of
/// `eval_inner`'s head dispatch. `:p::the-record` hands back the same record so the probe can read
/// field 0 directly, which is what a compiled `Op::Field` would do. If the two agree, `Op::Field` is
/// a faithful specialization and the only question left is how the executor learns the INDEX (the
/// second assertion).
#[test]
fn accessor_value_is_reachable_without_the_head_dispatch() {
    let via_accessor = call_beside_value(file!(), ":p::status-via-accessor")
        .expect("the accessor entry must evaluate");
    assert_eq!(
        via_accessor,
        Value::i64(200),
        "wat mouth :p::status-via-accessor must read Route.status; got {via_accessor:?}"
    );

    let record =
        call_beside_value(file!(), ":p::the-record").expect("the record entry must evaluate");
    // rune:vocare(vantage-bypass-test) — Op::Field index scout reads host Aggregate.fields / TypeEnv; not a wat caller surface
    let via_field_index = match &record {
        Value::Aggregate(a) => a.fields[0].clone(),
        other => panic!("expected a record aggregate, got {other:?}"),
    };

    assert_eq!(
        via_accessor, via_field_index,
        "the interpreted accessor and a direct field-index read disagree — `Op::Field` would NOT \
         be a faithful specialization of `(:p::Route/status ?r)`, and the stone's op set must \
         shrink to route accessors through Op::Interp"
    );

    // The field INDEX must be derivable from the TypeEnv at compile time, given the class. This is
    // what `keyword_accessor_record` does at RUNTIME on every call (a linear scan over
    // `field_names`); the stone resolves it ONCE. If the registry is not reachable this way, the
    // executor must carry the field NAME and scan — still cheaper than head dispatch, but a
    // different op.
    // rune:vocare(vantage-bypass-test) — TypeEnv field-index scout is host layout, not a wat caller surface
    let world = startup_beside(file!()).expect("the fixture must freeze");
    let types = world.symbols.types().expect("the frozen world must carry a TypeEnv");
    let idx = match types.get(":p::Route") {
        Some(wat::types::TypeDef::Aggregate(a)) => {
            a.field_names().position(|n| n == "status")
        }
        _ => None,
    };
    assert_eq!(
        idx,
        Some(0),
        "could not resolve `:p::Route/status` to a field index from the TypeEnv — `Op::Field` \
         cannot be compile-time resolved and the stone must carry the field name instead"
    );
}

/// The (b) SCOUT, answered by the corpus rather than by reasoning: across every real `where`
/// predicate, how many rules share a key expression?
///
/// The lab's tree pays off when MANY rules discriminate on FEW dimensions (its `FieldDim` enum is
/// 15 fixed dimensions across up to 1M rules). It is a rounding error when the ratio is ~1:1. This
/// test does not assert a verdict — it PRINTS the ratio per rule-set file, so (b)'s design is drawn
/// against the corpus's real shape instead of node-share's authored 50:1.
#[test]
fn corpus_key_expression_sharing_ratio() {
    use std::collections::HashMap;

    // The key expression of `(op <expr> <const>)` / `(op <const> <expr>)` is `<expr>` — the thing a
    // tree level would evaluate once per token. A predicate with no constant has no key expression
    // under this reading and is counted separately (it cannot form a discrimination level as-is).
    let mut per_file: HashMap<String, (usize, HashMap<String, usize>, usize)> = HashMap::new();

    let roots = ["wat-scripts", "wat-tests", "tests", "wat"];
    let mut files: Vec<std::path::PathBuf> = Vec::new();
    for root in roots {
        collect_wat(std::path::Path::new(root), &mut files);
    }

    for path in &files {
        let src = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => continue,
        };
        for raw in src.lines() {
            let line = raw.trim();
            if line.starts_with(";;") || !line.contains(":wat::rete::where") {
                continue;
            }
            let entry = per_file
                .entry(path.display().to_string())
                .or_insert_with(|| (0, HashMap::new(), 0));
            entry.0 += 1;
            match key_expression_of(line) {
                Some(k) => *entry.1.entry(k).or_insert(0) += 1,
                None => entry.2 += 1,
            }
        }
    }

    let mut rows: Vec<(String, usize, usize, usize)> = per_file
        .into_iter()
        .map(|(f, (n, keys, nokey))| (f, n, keys.len(), nokey))
        .collect();
    rows.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    let mut table = String::from(
        "\n(b) SCOUT — key-expression sharing across the corpus's `where` predicates\n\
         the lab's tree pays off at MANY rules : FEW dimensions (FieldDim = 15 fixed, up to 1M \
         rules)\n\
         \x20 preds  distinct-keys  no-key  ratio  file\n\
         \x20 ----------------------------------------------------------------------------\n",
    );
    let (mut tot_p, mut tot_k, mut tot_n) = (0usize, 0usize, 0usize);
    for (f, n, k, nokey) in &rows {
        let ratio = if *k > 0 { (n - nokey) as f64 / *k as f64 } else { 0.0 };
        table.push_str(&format!("  {n:>5}  {k:>13}  {nokey:>6}  {ratio:>5.1}  {f}\n"));
        tot_p += n;
        tot_k += k;
        tot_n += nokey;
    }
    table.push_str(&format!(
        "  ----------------------------------------------------------------------------\n\
         \x20 {tot_p:>5}  {tot_k:>13}  {tot_n:>6}         TOTAL\n"
    ));
    println!("{table}");

    // Non-vacuity ONLY. This test measures the corpus; it does not rule on (b). A zero here means
    // the scan found nothing and every ratio above is an artifact.
    assert!(
        tot_p > 0,
        "found ZERO `where` predicates across {} .wat files — the scan is broken and the table \
         above says nothing about the corpus",
        files.len()
    );
}

/// Extract the key expression of a single-line `where` predicate: the operand of a comparison whose
/// OTHER operand is a literal. Returns `None` when the predicate has no literal operand (a user fn,
/// a bare var, a var-to-var comparison) — those cannot form a discrimination level as written.
///
/// Deliberately CRUDE and line-based: this is a scout that reports a ratio, not a compiler. It sees
/// only single-line predicates; multi-line ones (node-share, strat-neg) are counted by the caller's
/// `where` match but yield no key here, which UNDERSTATES sharing. Stated so the number is read as
/// a floor, never a census (`[[feedback_a_greps_count_is_not_an_enumeration]]`).
fn key_expression_of(line: &str) -> Option<String> {
    let at = line.find(":wat::rete::where")? + ":wat::rete::where".len();
    let body = line[at..].trim();
    let inner = body.strip_prefix('(')?;
    let (head, rest) = inner.split_once(' ')?;
    if !matches!(
        head,
        ":wat::core::=" | ":wat::core::not=" | ":wat::core::<"
            | ":wat::core::>" | ":wat::core::<=" | ":wat::core::>="
    ) {
        return None;
    }
    let operands = split_top_level(rest);
    if operands.len() != 2 {
        return None;
    }
    let lit = |s: &str| {
        s.parse::<i64>().is_ok() || s.starts_with('"') || s.starts_with(":")
    };
    match (lit(&operands[0]), lit(&operands[1])) {
        (true, false) => Some(operands[1].clone()),
        (false, true) => Some(operands[0].clone()),
        _ => None, // var-to-var, or literal-to-literal: no discriminating key expression
    }
}

/// Split a predicate body into top-level operands, respecting nesting and strings.
fn split_top_level(s: &str) -> Vec<String> {
    let (mut out, mut depth, mut cur, mut in_str) = (Vec::new(), 0i32, String::new(), false);
    for c in s.chars() {
        match c {
            '"' => {
                in_str = !in_str;
                cur.push(c);
            }
            '(' if !in_str => {
                depth += 1;
                cur.push(c);
            }
            ')' if !in_str => {
                depth -= 1;
                if depth < 0 {
                    break; // the predicate's own closing paren
                }
                cur.push(c);
            }
            ' ' if !in_str && depth == 0 => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            _ => cur.push(c),
        }
    }
    let trimmed = cur.trim_end_matches([')', ']']).to_string();
    if !trimmed.is_empty() {
        out.push(trimmed);
    }
    out
}

fn collect_wat(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    let rd = match std::fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return,
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_wat(&p, out);
        } else if p.extension().map(|x| x == "wat").unwrap_or(false) {
            out.push(p);
        }
    }
}
