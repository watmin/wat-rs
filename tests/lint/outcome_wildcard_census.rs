//! Excursus 001 item 3 — *an outcome arm cannot be a wildcard*, **PHASE 1: A CENSUS THAT PRINTS
//! AND PASSES.** Nothing here makes a wildcard an error; the gate is a later ruling (DESIGN
//! §"PHASE 1 IS A CENSUS. THE GATE IS A LATER RULING").
//!
//! ## Why this is not a text lint
//!
//! `tests/lint/` is the repo's established shape for a corpus rule and would have been the cheap
//! move. It is also **blind to exactly the population this stone exists for**: 12 of
//! `wat/service.wat`'s 14 wildcarded matches sit inside quasiquoted `defservice` bodies whose arm
//! heads are unquoted symbols (`~status-stopped-kw`), so no regex can attribute them to an enum.
//! The instrument here is a side-channel off the CHECKER's own exhaustiveness decision
//! (`src/check.rs`'s `infer_match` — the point where a `_` currently BLESSES a missing arm), which
//! runs on macro-EXPANDED forms and knows the scrutinee's real enum path. DESIGN trap-door 3 makes
//! the choice falsifiable, and `a_macro_generated_site_is_reached` below is that test.
//!
//! ## ⚠ WHAT THE INSTRUMENT CAN SEE — read this before quoting any number from it
//!
//! - **The stdlib is type-checked on EVERY startup.** So one root reveals every site in `wat/**`,
//!   including all 18 stdlib `defservice` call sites. That is why the walk below does not need to
//!   visit `wat/`'s 55 files one at a time.
//! - **Macro expansion SUBSTITUTES spans** (see `tests/lint/span_substitution_justified.rs`:
//!   *"Substitution destroys the location at the point of substitution; nothing downstream
//!   recovers it"*). A generated wildcard therefore reports the location of the `defservice`
//!   CALL, not the `wat/service.wat` line the `_` is written on. The checker still knows the enum,
//!   the swallowed variants and that a `_` was used — it does **not** know the authoring line.
//!   `is_literal_wildcard()` below turns that into the generated/literal discriminator: a LITERAL
//!   site's span points at a `_` in the file's own text; a substituted one does not.
//! - Every generated wildcard from one expansion collapses onto that one call-site span, so
//!   `firings` (how many matches reported at a span) is the count of generated wildcards per
//!   expansion, and the distinct-site count is the count of call sites.
//! - A variant counts as named only for a FULLY-GENERAL arm (`check.rs`'s
//!   `Coverage::EnumVariant { full: true }`). A narrowed arm (`(Message 200)`) is not credited, so
//!   the `missing` column can over-report for such a match. It cannot change the site count.
//!
//! ## Scope of the walk — and why the FLOOR's slice is the narrow one
//!
//! `WAT_OUTCOME_CENSUS` selects the scope. Unset (**the floor**) = `stdlib`: ONE baseline root,
//! which is complete for every site the stdlib's own load reaches, and costs one startup.
//! `live` adds every tracked `.wat` outside `tests/`, `wat-tests/`, `docs/`, `benches/` and
//! `wat-scripts/scratch-pad/` (261 roots). `full` walks all 1870 tracked `.wat`.
//!
//! ⛔ **The wide scopes are deliberately NOT in the floor, and that is a cost, not a tidy-up.**
//! Measured: a 73-root walk takes **30.8 s** and nextest SIGTERMs it at the 30 s kill; 261 roots
//! is ~110 s alone and `full` several minutes. The per-root cost is the stdlib re-check
//! (`.config/nextest.toml` records the same fact as 147 ms × 98 for
//! `retirement_table_is_fully_reachable`), and the box already carries a 550 s scripts gate at
//! `priority = 100` whose own kill is 600 s — adding a second CPU-heavy corpus walk to that wave
//! risks reddening THAT, which is not a trade this stone is allowed to make. So the floor weighs
//! the instrument (it fires, it reaches a generated site, it declines the out-of-scope population)
//! and the wide numbers in the SCORE come from explicit runs:
//!
//! ```text
//! WAT_OUTCOME_CENSUS=live cargo test --release --test lint -- census --nocapture
//! WAT_OUTCOME_CENSUS=full cargo test --release --test lint -- census --nocapture
//! ```
//!
//! (`cargo test`, not `cargo nextest`: libtest imposes no per-test deadline.)

use std::collections::{BTreeMap, BTreeSet};
use std::process::Command;
use std::sync::Arc;
use wat::check::outcome_wildcard_census as census;
use wat::freeze::startup_from_source;
use wat::load::loader::FsLoader;

/// Paths whose wildcards are probes / fixtures / record rather than shipped behaviour.
const PROBE_PREFIXES: &[&str] = &[
    "wat-scripts/scratch-pad/",
    "tests/",
    "wat-tests/",
    "docs/",
    "benches/",
];

fn is_probe(path: &str) -> bool {
    PROBE_PREFIXES.iter().any(|p| path.starts_with(p))
}

fn tracked_wat(root_dir: &str, glob: &str) -> Vec<String> {
    let out = Command::new("git")
        .args(["-C", root_dir, "ls-files", glob])
        .output()
        .expect("git ls-files");
    assert!(out.status.success(), "git ls-files must succeed");
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::to_string)
        .collect()
}

/// A distinct source site: which enum, and where the checker says the `_` is.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct SiteKey {
    file: String,
    line: i64,
    col: i64,
    enum_path: String,
}

struct SiteFacts {
    firings: usize,
    roots: BTreeSet<String>,
    missing: Vec<String>,
    named: usize,
    in_scope: bool,
    why: &'static str,
}

/// ⭐ The generated/literal discriminator, and the one measurement this whole report rests on.
///
/// A LITERAL wildcard arm's span points at the `_` token as written. A MACRO-GENERATED one has
/// had its span substituted to the macro call site, so the text there is something else (the head
/// of a `defservice` form). Reading the file at `line:col` and asking "is this a `_`?" therefore
/// separates the two populations without guessing. `None` = the file/line could not be read, which
/// is reported rather than folded into either bucket.
fn is_literal_wildcard(root_dir: &str, file: &str, line: i64, col: i64) -> Option<bool> {
    let src = std::fs::read_to_string(std::path::Path::new(root_dir).join(file)).ok()?;
    let text = src.lines().nth(usize::try_from(line - 1).ok()?)?;
    let idx = usize::try_from(col - 1).ok()?;
    let rest: String = text.chars().skip(idx).collect();
    Some(rest.starts_with('_'))
}

/// The source line a site reports, for quoting in the write-up.
fn source_line(root_dir: &str, file: &str, line: i64) -> String {
    std::fs::read_to_string(std::path::Path::new(root_dir).join(file))
        .ok()
        .and_then(|s| {
            s.lines()
                .nth(usize::try_from(line - 1).ok()?)
                .map(|l| l.trim().to_string())
        })
        .unwrap_or_else(|| "<unreadable>".into())
}

#[test]
fn outcome_wildcard_census_over_the_corpus() {
    let root_dir = env!("CARGO_MANIFEST_DIR");
    let scope = std::env::var("WAT_OUTCOME_CENSUS").unwrap_or_else(|_| "stdlib".into());
    let all = tracked_wat(root_dir, "*.wat");
    assert!(
        !all.is_empty(),
        "no tracked *.wat — the census would be measuring nothing"
    );

    // Derived root set. The BASELINE root is the first tracked file, and it is not arbitrary:
    // checking ANY file type-checks the loaded stdlib with it, so one startup is complete for
    // `wat/**`. Stdlib files are never used as roots themselves — a baked `wat/*.wat` loaded as a
    // standalone program does not check (measured: 15 of 18 reported errors), which would
    // contribute partial data and inflate the did-not-check count for no gain.
    let baseline = all.first().expect("at least one tracked .wat").clone();
    let mut roots: Vec<String> = vec![baseline];
    match scope.as_str() {
        "stdlib" => {}
        "live" => roots.extend(
            all.iter()
                .filter(|r| !is_probe(r) && !r.starts_with("wat/"))
                .cloned(),
        ),
        "full" => roots.extend(all.iter().filter(|r| !r.starts_with("wat/")).cloned()),
        other => panic!("WAT_OUTCOME_CENSUS must be stdlib|live|full, got {other:?}"),
    }
    roots.dedup();

    census::enable();
    let mut sites: BTreeMap<SiteKey, SiteFacts> = BTreeMap::new();
    let mut unreadable: Vec<String> = Vec::new();
    let mut did_not_check: Vec<String> = Vec::new();

    for rel in &roots {
        let src = match std::fs::read_to_string(std::path::Path::new(root_dir).join(rel)) {
            Ok(s) => s,
            Err(e) => {
                unreadable.push(format!("{rel}: {e}"));
                continue;
            }
        };
        census::set_root(Some(rel.clone()));
        // `tests/` and `wat-tests/` hold DELIBERATELY-bad fixtures, and at least one
        // (`tests/cli/wat_cli__freeze_time_panic.wat`) raises at FREEZE time — a panic, not an
        // `Err`. Catching it is what lets the full walk reach the probe corpus at all.
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            startup_from_source(&src, Some(rel.as_str()), Arc::new(FsLoader)).is_err()
        }));
        match outcome {
            Ok(false) => {}
            // Partial contribution is kept; the count is reported so coverage is not overstated.
            Ok(true) => did_not_check.push(rel.clone()),
            Err(_) => did_not_check.push(format!("{rel} (PANICKED at check/freeze time)")),
        }
        for s in census::drain() {
            let key = SiteKey {
                file: s.file.clone(),
                line: s.line,
                col: s.col,
                enum_path: s.enum_path.clone(),
            };
            let e = sites.entry(key).or_insert_with(|| SiteFacts {
                firings: 0,
                roots: BTreeSet::new(),
                missing: s.missing.clone(),
                named: s.named.len(),
                in_scope: s.verdict.is_in(),
                why: s.verdict.why(),
            });
            e.firings += 1;
            if let Some(r) = &s.root {
                e.roots.insert(r.clone());
            }
        }
    }
    census::set_root(None);
    census::disable();

    let literal = |k: &SiteKey| is_literal_wildcard(root_dir, &k.file, k.line, k.col);
    let in_scope: Vec<(&SiteKey, &SiteFacts)> = sites.iter().filter(|(_, f)| f.in_scope).collect();
    let out_scope: Vec<(&SiteKey, &SiteFacts)> = sites.iter().filter(|(_, f)| !f.in_scope).collect();

    // ── the report ────────────────────────────────────────────────────────────────────────
    println!("\n=== EXCURSUS 001 item 3 — OUTCOME-WILDCARD CENSUS (report-only) ===");
    println!(
        "scope: {} · roots walked {} of {} tracked *.wat · {} did not type-check (partial) · {} unreadable",
        scope,
        roots.len(),
        all.len(),
        did_not_check.len(),
        unreadable.len()
    );
    let firings_in: usize = in_scope.iter().map(|(_, f)| f.firings).sum();
    println!(
        "distinct wildcarded enum-match sites: {} — {} IN scope ({} firings), {} OUT of scope",
        sites.len(),
        in_scope.len(),
        firings_in,
        out_scope.len()
    );

    println!("\n--- ROW 2: IN-SCOPE per enum (sites / firings / generated sites / live sites) ---");
    let mut per_enum: BTreeMap<&str, (usize, usize, usize, usize)> = BTreeMap::new();
    for (k, f) in &in_scope {
        let e = per_enum.entry(k.enum_path.as_str()).or_insert((0, 0, 0, 0));
        e.0 += 1;
        e.1 += f.firings;
        if literal(k) == Some(false) {
            e.2 += 1;
        }
        if !is_probe(&k.file) {
            e.3 += 1;
        }
    }
    println!(
        "{:<42} {:>6} {:>8} {:>10} {:>6}",
        "enum", "sites", "firings", "generated", "live"
    );
    for (e, (n, fi, g, l)) in &per_enum {
        println!("{e:<42} {n:>6} {fi:>8} {g:>10} {l:>6}");
    }

    println!("\n--- ROWS 3 & 4: IN-SCOPE splits (bucketed by the span file — where the fix goes) ---");
    let live = in_scope.iter().filter(|(k, _)| !is_probe(&k.file)).count();
    let probe = in_scope.iter().filter(|(k, _)| is_probe(&k.file)).count();
    let gen = in_scope
        .iter()
        .filter(|(k, _)| literal(k) == Some(false))
        .count();
    let lit = in_scope
        .iter()
        .filter(|(k, _)| literal(k) == Some(true))
        .count();
    let unknown = in_scope.iter().filter(|(k, _)| literal(k).is_none()).count();
    println!(
        "LIVE {live} · PROBE/TEST {probe}   ||   MACRO-GENERATED {gen} · LITERAL {lit} · UNDETERMINED {unknown}"
    );

    println!("\n--- every IN-SCOPE site (⭐GEN = span substituted by macro expansion) ---");
    for (k, f) in &in_scope {
        println!(
            "{}{} {}:{}:{}  {}  firings={} named={} missing=[{}]",
            match literal(k) {
                Some(false) => "⭐GEN ",
                Some(true) => "     ",
                None => "  ?  ",
            },
            if is_probe(&k.file) { "probe" } else { "LIVE " },
            k.file,
            k.line,
            k.col,
            k.enum_path,
            f.firings,
            f.named,
            f.missing.join(",")
        );
        if literal(k) == Some(false) {
            println!("        span points at: {}", source_line(root_dir, &k.file, k.line));
            println!("        triggered by {} root(s)", f.roots.len());
        }
    }

    println!("\n--- ROWS 6 & 7 CONTROL: wildcarded enum matches the predicate DECLINED ---");
    let mut per_out: BTreeMap<&str, (usize, usize, &'static str)> = BTreeMap::new();
    for (k, f) in &out_scope {
        let e = per_out.entry(k.enum_path.as_str()).or_insert((0, 0, f.why));
        e.0 += 1;
        e.1 += f.firings;
    }
    println!("{:<48} {:>6} {:>8}  why", "enum", "sites", "firings");
    for (e, (n, fi, why)) in &per_out {
        println!("{e:<48} {n:>6} {fi:>8}  {why}");
    }

    if !did_not_check.is_empty() {
        println!(
            "\n--- roots that did not type-check (their sites may be incomplete): {} ---",
            did_not_check.len()
        );
        for p in &did_not_check {
            println!("    {p}");
        }
    }
    println!("=== END CENSUS ===\n");

    // ── non-vacuity. A census that cannot fail is a claim, not a measurement. ───────────────
    //
    // ⛔ SAME EXPIRY as `a_macro_generated_site_is_reached` below: these three assert that a
    // DEFECT is still present, so the phase-2 repair will redden them on correct work. When that
    // lands, repoint them at this stone's own scratch-pad fixture or retire them with the gate
    // that supersedes them — never by re-admitting a corpus wildcard.
    assert!(
        !in_scope.is_empty(),
        "the census found ZERO in-scope wildcarded outcome matches — either the corpus is clean \
         (then say so on purpose) or the hook is not firing"
    );
    assert!(
        !out_scope.is_empty(),
        "the predicate declined NOTHING — a rule that classifies everything IN is not a scope \
         rule, and rows 6/7 then have no control"
    );
    assert!(
        gen > 0,
        "no MACRO-GENERATED in-scope site in the whole live walk — that is the population this \
         instrument exists for (DESIGN trap-door 3)"
    );
}

/// ⭐ DESIGN trap-door 3 / EXPECTATIONS row 1 — the justification for choosing the checker.
/// A wildcard inside one of `wat/service.wat`'s quasiquoted, macro-generated bodies MUST be
/// reachable. A text lint cannot attribute those matches to an enum at all. If this goes red the
/// instrument choice failed and the stone must not ship.
///
/// It asserts on the SIGNATURE of the generated site, not on a line number: span substitution
/// means the reported location is the `defservice` call, so `wat/service.wat:2978` can never be
/// printed. What identifies it instead is (enum = `RecvOutcome`) × (a span that is NOT a `_` in
/// the source) × (the five swallowed variants) — see the module header.
#[test]
fn a_macro_generated_site_is_reached() {
    let root_dir = env!("CARGO_MANIFEST_DIR");
    // Derived, not hand-listed: a stdlib service is enough, and the stdlib is checked on every
    // startup, so ONE root suffices. Use the first tracked file as that root.
    let all = tracked_wat(root_dir, "*.wat");
    let root = all.first().expect("at least one tracked .wat").clone();
    let src = std::fs::read_to_string(std::path::Path::new(root_dir).join(&root)).expect("read");

    census::enable();
    census::set_root(Some(root.clone()));
    let _ = startup_from_source(&src, Some(root.as_str()), Arc::new(FsLoader));
    let collected = census::drain();
    census::set_root(None);
    census::disable();

    let raw: Vec<_> = collected
        .iter()
        .filter(|s| s.verdict.is_in())
        .filter(|s| is_literal_wildcard(root_dir, &s.file, s.line, s.col) == Some(false))
        .map(|s| (s.file.clone(), s.line, s.enum_path.clone(), s.missing.clone()))
        .collect();
    // firings per (span, enum): every generated wildcard from one expansion collapses onto the
    // macro call's span, so this count IS the number of generated wildcard arms per expansion.
    let mut firings: BTreeMap<(String, i64, String), usize> = BTreeMap::new();
    for (f, l, e, _) in &raw {
        *firings.entry((f.clone(), *l, e.clone())).or_insert(0) += 1;
    }
    let mut generated = raw.clone();
    generated.sort();
    generated.dedup();

    println!("\n⭐ MACRO-GENERATED in-scope wildcard sites the CHECKER reached (root: {root}):");
    for (f, l, e, m) in &generated {
        println!(
            "    {}:{}  {}  firings={}  swallowing [{}]\n        span points at: {}",
            f,
            l,
            e,
            firings[&(f.clone(), *l, e.clone())],
            m.join(","),
            source_line(root_dir, f, *l)
        );
    }

    assert!(
        !generated.is_empty(),
        "TRAP-DOOR 3: the checker route flagged NO macro-generated site. The instrument choice \
         failed — a text lint's blind spot is not removed. Report this, do not ship the lint."
    );
    // The population item 1 filed: the two `(RecvOutcome::Message ~resp-sym)` + `_` matches in
    // `defservice`'s generated paging path, swallowing five of RecvOutcome's six variants.
    assert!(
        generated.iter().any(|(_, _, e, m)| e == ":wat::kernel::RecvOutcome" && m.len() == 5),
        "the generated RecvOutcome paging-path wildcard (5 of 6 variants swallowed) was not \
         reached; got: {generated:?}"
    );
    // TWO per expansion — the signature of item 1's filing (`service.wat:2978` and `:3089`,
    // "five behaviour decisions x two sites"). Line numbers cannot be asserted (span
    // substitution); the pair-per-expansion count can.
    //
    // ⛔ EXPIRY, named because this assertion is aimed at a DEFECT and a defect can be repaired.
    // When the filed stone lands (naming `Closed`/`Lost`/`Stopped`/`TimedOut`/`Malformed` in
    // `defservice`'s generated paging path), `wat/service.wat` will hold ZERO in-scope generated
    // wildcards and BOTH asserts above will go red on CORRECT work. That red is not a regression.
    // The fix then is to repoint this test at a fixture this stone OWNS: add a deliberately
    // wildcarded outcome match INSIDE a `defservice` body in
    // `wat-scripts/scratch-pad/probe-outcome-wildcard-census.wat` (the expander substitutes such a
    // span identically — measured: `wat-scripts/fanout/circuit.wat`'s four user-written
    // `CallOutcome` wildcards all report at its `defservice` head, line 370). Or retire it in
    // favour of the phase-2 gate that supersedes it. Do NOT re-add a corpus wildcard to keep it
    // green.
    assert!(
        firings
            .iter()
            .any(|((_, _, e), n)| e == ":wat::kernel::RecvOutcome" && *n == 2),
        "no defservice expansion produced exactly TWO RecvOutcome wildcards — item 1 filed the \
         paging path as two sites; got firings {firings:?}"
    );
}

/// ⭐ THE DRIVEN CONTROLS — EXPECTATIONS row 7, and the ⚠ `Status` question the DESIGN asks to be
/// answered either way.
///
/// The predicate's `Status` arm has nothing live to fire on: item 1 (`every-status-arm-is-named`)
/// drove the corpus to zero `Status` wildcards. Reporting "the rule can reach `Status`" off the
/// unit test alone would be a claim about the predicate, not about the checker. So this drives the
/// whole path — real `defenum`s, real matches, the real `infer_match` hook — through one fixture,
/// `wat-scripts/scratch-pad/probe-outcome-wildcard-census.wat`, which also carries the negative
/// controls (a domain enum; an enum whose last segment IS `Status` but whose shape is not; an
/// `Option`).
#[test]
fn the_status_shape_is_reached_and_the_domain_controls_are_declined() {
    let root_dir = env!("CARGO_MANIFEST_DIR");
    let root = "wat-scripts/scratch-pad/probe-outcome-wildcard-census.wat";
    let src = std::fs::read_to_string(std::path::Path::new(root_dir).join(root)).expect("read");

    census::enable();
    census::set_root(Some(root.to_string()));
    let checked = startup_from_source(&src, Some(root), Arc::new(FsLoader));
    let collected = census::drain();
    census::set_root(None);
    census::disable();
    assert!(
        checked.is_ok(),
        "the control fixture must type-check, or it is measuring the wrong thing: {:?}",
        checked.err()
    );

    println!("\n⭐ DRIVEN controls, from {root}:");
    for s in &collected {
        println!(
            "    {:<40} {} — {}  missing=[{}]  at {}:{}",
            s.enum_path,
            if s.verdict.is_in() { "IN " } else { "OUT" },
            s.verdict.why(),
            s.missing.join(","),
            s.file,
            s.line
        );
    }

    let find = |p: &str| collected.iter().find(|s| s.enum_path == p);

    // ⚠ The answer to the DESIGN's `Status` question: YES, by shape.
    let st = find(":probe::census::Status").expect(
        "the Status-shaped enum's wildcard was not reached at all — the hook missed it, which is a \
         bigger finding than the classification",
    );
    assert!(
        st.verdict.is_in() && st.verdict.why() == "defservice Status shape",
        "a six-variant Status-shaped enum must be IN scope by shape; got {:?}",
        st.verdict
    );
    assert_eq!(
        st.missing.len(),
        5,
        "the fixture names one of six variants, so five must be reported swallowed; got {:?}",
        st.missing
    );

    // ⛔ trap-door 1 — a domain enum is ordinary code.
    let color = find(":probe::census::Color").expect("the domain-enum control was not reached");
    assert!(
        !color.verdict.is_in(),
        "a wildcard over a domain enum must NOT be flagged; got {:?}",
        color.verdict
    );

    // ⛔ The sharper control: the rule keys on the SHAPE, never on the spelling `Status`.
    let ish = find(":probe::census::Statusish::Status")
        .expect("the wrong-shape `Status` control was not reached");
    assert!(
        !ish.verdict.is_in(),
        "an enum whose last segment is `Status` but whose variant set is not the defservice six \
         must be declined — otherwise the rule fires on a name; got {:?}",
        ish.verdict
    );

    // ⛔ trap-door 2 — `Option`/`Result` are separate `MatchShape`s and cannot reach this hook.
    //
    // Asserted as the EXACT set of enum paths the fixture's own file produces, not as a
    // `contains("Option")` absence. `no_loose_string_assert` caught the loose form on my first
    // floor (a red that was mine), and the exact form is strictly better anyway: it proves the
    // `Option` match contributed nothing AND that no fourth, unexpected enum appeared. The
    // fixture has four wildcarded matches; exactly three may reach the census.
    let mut from_fixture: Vec<String> = collected
        .iter()
        .filter(|s| s.file == root)
        .map(|s| s.enum_path.clone())
        .collect();
    from_fixture.sort();
    from_fixture.dedup();
    assert_eq!(
        from_fixture,
        vec![
            ":probe::census::Color".to_string(),
            ":probe::census::Status".to_string(),
            ":probe::census::Statusish::Status".to_string(),
        ],
        "the fixture's four wildcarded matches must reach the census as exactly these three enums \
         — an `Option` entry means trap-door 2 is breached, a missing one means the hook lost a site"
    );
}
