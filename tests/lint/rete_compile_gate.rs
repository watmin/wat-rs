//! Gate — every `.wat` under `wat-scripts/` that DECLARES A RETE RULE must COMPILE it.
//!
//! `every_wat_scripts_file_loads_on_the_current_runtime` (`tests/lint/wat_scripts_fixes_load.rs`)
//! walks the same corpus and calls `startup_from_source`, which runs `validate` (parse +
//! type-check). It never runs rete **compile** — and the four-axis fence gate lives there, in
//! wat, not in Rust: `pure ∧ det ∧ total ∧ rete-primitive` (`wat/rete/compile.wat:463`,
//! `first-failing-axis` names which axis broke). A rule violating any axis loads GREEN forever
//! and dies the instant anything actually compiles it — measured at `d7aa7c9ae`
//! (`tests/lint/rete-compile-census.sh`, the working instrument this gate must agree with):
//! 136 files declared rete rules; 125 compiled; 11 could not. Full finding:
//! `docs/arc/2026/06/278-rules-engine/the-fence-says-what-the-clause-cannot/FINDING-loading-is-not-compiling.md`.
//! Strike that built this gate and drove the corpus to zero failures:
//! `docs/arc/2026/06/278-rules-engine/strike-no-rule-that-cannot-compile/`.
//!
//! ⛔ **THE ONE CONTRACT DECISION (DESIGN.md) — ZERO EXEMPTION CATEGORIES.** There is no
//! `red-by-design` rune for "this rule is meant not to compile." After the strike's nine
//! deletions, every refusal those files demonstrated is already asserted, with `assert_eq!` on
//! the exact message, by `tests/rete/probe_fence_names_the_head.rs` (mutation-proven) and
//! `wat_scripts_grid_axes_live.rs`. If a genuine negative ever needs to live in this corpus
//! again, minting the first exemption category is a deliberate act needing its own argument —
//! do not add one here to make a red file pass.
//!
//! ## The algorithm — no binary, no `main`, no temp file
//!
//! For each rule-declaring file: read the source, extract the rete namespace(s) it declares
//! (mirroring `rete-compile-census.sh`'s own non-greedy extraction — see the header comment on
//! [`declared_namespaces`] for the trap a greedy version falls into), append a synthesized
//! zero-arg `:census::run` that returns a `PersistentVector` of one `CompileOutcome` per declared
//! namespace (`compile-all` over `collect-rules`), `startup_from_source` the combined text, look
//! `:census::run` up, `apply_function` it. `startup_from_source` never evals `:user::main`, so a
//! codemod's side effects (the ones `rete-compile-census.sh` neutralises with a `:user::main`→
//! `:user::orig-main-off` `sed` before shelling out to the binary) cannot fire here at all — the
//! `sed` step simply does not exist in this driver. Same driver `wat_scripts_fixes_load.rs`
//! already uses, one step further.
//!
//! A `where`/`:then` fence's `Option/expect` on a failing axis PANICS with an
//! [`wat::assertion::AssertionPayload`] (the same mechanism `probe_fence_names_the_head.rs`
//! catches) — it does NOT come back as a plain `Err` from `apply_function`. This gate wraps the
//! call in `catch_unwind`, exactly as that probe does, and reads the message out of the payload.
//! A `CompileOutcome::MayNotTerminate` (a *different* kind of refusal — termination analysis, not
//! an axis violation) does NOT panic; it comes back as an ordinary value, so every element of the
//! returned vector is checked structurally, not just "did it panic".
//!
//! ## Why this is SHARDED
//!
//! `every_wat_scripts_file_loads_on_the_current_runtime` needed a 300s/600s override
//! (`.config/nextest.toml`) to walk ~445 files UNSHARDED with parse+type-check alone. This gate's
//! per-file cost is that SAME startup plus a real `compile-all` (rete network construction) on
//! top, over a narrower ~130-file population (only rule-declaring files). Rather than ask for
//! another timeout override, this follows `every_wat_bad_fixture_actually_fails.rs`'s own
//! precedent over a near-identical corpus size and needs no `nextest.toml` entry at all.
//!
// rune:lint(no-inlined-wat) — this file's wat string literals are not test bodies you could move
// beside it as a fixture; they are the gate's OWN implementation. `driver_defn`'s two `format!`
// snippets ARE `DESIGN.md`'s "no binary, no main, no temp file" algorithm: the `:census::run`
// entry point it synthesizes is only known at runtime, after `declared_namespaces` reads the
// namespace list out of an ARBITRARY corpus file being walked — there is no fixed wat text a
// `.wat` fixture could hold. The `extraction` module's four literal wat-shaped strings are unit
// inputs for `declared_namespaces` itself (a parser/reader test, the RUBRIC's own named
// exception), including `extraction_is_not_greedy_across_two_rules`, which pins the exact trap
// `rete-compile-census.sh`'s own header records a greedy strip falling into.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use wat::assertion::AssertionPayload;
use wat::freeze::{startup_from_source, FrozenWorld};
use wat::load::loader::FsLoader;
use wat::runtime::{apply_function, Value};

fn collect_wat(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        // rune:lint(one-variant-separator, not-a-name) — `std::fs::DirEntry::path()`, a
        // filesystem path, not a wat enum-variant name.
        let p = e.path();
        if p.is_dir() {
            collect_wat(&p, out);
        } else if p.extension().is_some_and(|x| x == "wat") {
            out.push(p);
        }
    }
}

/// Every declared rete namespace in `src`.
///
/// The population is the set of `defrule` / `defquery` FORMS, read by identity:
/// `:wat::rete::defrule` and `wat.rete/defrule` are one head. A text needle
/// `rete::defrule :` saw only the keyword spelling, so a converted file
/// dropped out of the population and the loss was a smaller count.
///
/// ⛔ **NAMESPACE EXTRACTION MUST NOT BE GREEDY.** The census script's own header
/// records that a greedy strip on `"rete::defrule :fix::g3"` collapses to `"g3"`.
/// The rule name is the form's second item. [`rule_namespace`] drops only the
/// LAST `::` segment of its identity (`:fix::sub::q-Hit` and `fix.sub/q-Hit`
/// both yield `fix::sub`).
///
/// A token that sits inside a string literal is not a form. Those files yield
/// the empty set — the caller's `NO-RULES` skip, not a false failure. A source
/// that does not parse yields the empty set too: there is no form to read.
fn declared_namespaces(src: &str) -> std::collections::BTreeSet<String> {
    let mut out = std::collections::BTreeSet::new();
    let Ok(forms) = wat_reader::parse_all_with_file(src, "<declared-namespaces>") else {
        return out;
    };
    for form in &forms {
        walk_rule_forms(form, &mut out);
    }
    out
}

fn walk_rule_forms(node: &wat_reader::WatAST, out: &mut std::collections::BTreeSet<String>) {
    if let wat_reader::WatAST::List(items, _) = node {
        if let Some(head) = items.first().and_then(form_head_identity) {
            if head == ":wat::rete::defrule" || head == ":wat::rete::defquery" {
                if let Some(raw) = items.get(1).and_then(form_written) {
                    if let Some(ns) = rule_namespace(&raw) {
                        out.insert(ns);
                    }
                }
            }
        }
    }
    for child in node.children().iter() {
        walk_rule_forms(child, out);
    }
}

fn form_head_identity(node: &wat_reader::WatAST) -> Option<String> {
    let raw = form_written(node)?;
    Some(wat::edn::render::canonical_identity(&raw))
}

fn form_written(node: &wat_reader::WatAST) -> Option<String> {
    match node {
        wat_reader::WatAST::Keyword(k, _) => Some(k.clone()),
        wat_reader::WatAST::Symbol(id, _) => Some(id.as_str().to_string()),
        _ => None,
    }
}

/// `fix` from both `:fix::g1-keyword` and `fix/g1-keyword`. The leading colon
/// of the identity is dropped so the keyword-era tests keep the colon-free
/// namespace the shell instrument reported.
fn rule_namespace(raw: &str) -> Option<String> {
    let id = wat::edn::render::canonical_identity(raw);
    let body = id.trim_start_matches(':');
    let ns = wat_reader::identifier::path(body);
    if !ns.is_empty() {
        Some(ns.to_string())
    } else if !body.is_empty() {
        Some(body.to_string())
    } else {
        None
    }
}

/// The synthesized entry point: one `CompileOutcome` per declared namespace, collected into a
/// vector so EVERY namespace is checked — not just the last one a sequential `do` would keep.
fn driver_defn(namespaces: &std::collections::BTreeSet<String>) -> String {
    let mut calls = String::new();
    for ns in namespaces {
        calls.push_str(&format!(
            "    (:wat::rete::compile-all (:wat::rete::collect-rules :{ns}) (wat.type/PersistentVector :- [:wat::rete::Query]))\n"
        ));
    }
    format!(
        "\n(:wat::core::defn :census::run [] -> (wat.type/PersistentVector :- [:wat::rete::CompileOutcome])\n  (wat.type/PersistentVector :- [:wat::rete::CompileOutcome]\n{calls}))\n"
    )
}

/// Drive `:census::run` and classify. `Ok(())` iff every declared namespace's `compile-all`
/// returned `CompileOutcome::Compiled`. Any other outcome — a panicking fence (`Option/expect`
/// on a failed axis, caught via `catch_unwind` exactly as `probe_fence_names_the_head.rs`'s
/// `compile_message` does), a plain `Err` from `apply_function`, or a non-`Compiled` value
/// (`MayNotTerminate`) — is `Err(<diagnostic>)`.
fn drive_census(world: &FrozenWorld) -> Result<(), String> {
    let sym = world.symbols();
    let Some(func) = sym.get(":census::run") else {
        return Err("driver bug: :census::run was not defined by the appended defn".to_string());
    };
    let func = func.clone();
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        apply_function(func, vec![], sym, wat::rust_caller_span!())
    }));
    let result = match outcome {
        Ok(r) => r,
        Err(panic_payload) => {
            if let Some(p) = panic_payload.downcast_ref::<AssertionPayload>() {
                return Err(p.message.clone());
            } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                return Err(s.clone());
            } else if let Some(s) = panic_payload.downcast_ref::<&str>() {
                return Err((*s).to_string());
            }
            return Err("panic-opaque".to_string());
        }
    };
    let value = result.map_err(|e| format!("eval error: {e:?}"))?;
    let Value::wat__core__PersistentVector(items) = &value else {
        return Err(format!(":census::run returned a non-vector value: {value:?}"));
    };
    let mut bad = Vec::new();
    for v in items.iter() {
        match v {
            Value::Enum(e) if e.variant_name == "Compiled" => {}
            // rune:lint(one-variant-separator, display) — renders the variant name into a
            // human-facing test-failure message; not composing an identity to be re-parsed.
            Value::Enum(e) => bad.push(format!("CompileOutcome::{} (not Compiled)", e.variant_name)),
            other => bad.push(format!("unexpected :census::run element: {other:?}")),
        }
    }
    if bad.is_empty() {
        Ok(())
    } else {
        Err(bad.join("; "))
    }
}

/// The floor on rule-declaring files this walk must clear. Measured post-strike (this strike's
/// own census re-run): 128 (136 originally declared rules, minus the 9 this strike deletes, plus
/// 1 new scratch probe the strike's open-question repair added
/// (`wat-scripts/scratch-pad/probe-where-fallback-op-does-not-raise.wat`)). Deliberately well
/// under that — this exists to catch a walk gone blind (a moved root, a broken extraction), not
/// to pin the corpus size; it must not red on an ordinary future addition or deletion.
const RULE_FILE_FLOOR: usize = 100;

/// How many shards. See the module header — the same corpus-size argument
/// `every_wat_bad_fixture_actually_fails.rs` makes for its own 16.
const N_SHARDS: usize = 16;

fn corpus() -> Vec<PathBuf> {
    let mut entries = Vec::new();
    collect_wat(Path::new("wat-scripts"), &mut entries);
    entries.sort();
    entries
}

/// One file's verdict: `None` if it declares no rules of its own (skipped, not a failure) or
/// compiles clean; `Some((path, message))` for every other outcome.
fn check_one(path: &Path) -> Option<Result<(), (String, String)>> {
    let rel = path.to_str().expect("utf8 path");
    let src = std::fs::read_to_string(rel).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    let nss = declared_namespaces(&src);
    if nss.is_empty() {
        return None;
    }
    let driver_src = format!("{src}{}", driver_defn(&nss));
    let world = match startup_from_source(&driver_src, Some(rel), Arc::new(FsLoader)) {
        Ok(w) => w,
        Err(e) => return Some(Err((rel.to_string(), format!("startup: {e:?}")))),
    };
    match drive_census(&world) {
        Ok(()) => Some(Ok(())),
        Err(msg) => Some(Err((rel.to_string(), msg))),
    }
}

fn check_shard(shard: usize) -> (usize, Vec<String>) {
    let paths = corpus();
    assert!(
        paths.len() > 200,
        "the wat-scripts walk found only {} .wat file(s) — it is not reaching the tree it \
         claims to guard, so its green means nothing",
        paths.len()
    );

    let mut mine: Vec<&PathBuf> = paths.iter().skip(shard).step_by(N_SHARDS).collect();
    mine.sort();

    let mut rule_files = 0usize;
    let mut failures = Vec::new();
    for path in mine.drain(..) {
        match check_one(path) {
            None => {}
            Some(Ok(())) => rule_files += 1,
            Some(Err((rel, msg))) => {
                rule_files += 1;
                failures.push(format!("  {rel}\n      {msg}"));
            }
        }
    }
    (rule_files, failures)
}

macro_rules! shards {
    ($($name:ident = $idx:expr;)*) => {
        $(
            #[test]
            fn $name() {
                let (_rule_files, failures) = check_shard($idx);
                assert!(
                    failures.is_empty(),
                    "\n\n🔥 {} wat-scripts/ file(s) in shard {}/{N_SHARDS} DECLARE a rete rule \
                     that does not COMPILE. `every_wat_scripts_file_loads_on_the_current_runtime` \
                     only proves these LOAD (parse + type-check); the four-axis fence \
                     (pure ∧ det ∧ total ∧ rete-primitive, `wat/rete/compile.wat:463`) lives at \
                     rete COMPILE, one step further, and this gate is the one that reaches it.\n\
                     \n\
                     THE FIX: either the file is a genuine mistake (rewrite the fence to a total,\
                     pure, deterministic, rete-primitive expression — inline a registered \
                     `:wat::rete::` op rather than a bare user-fn call, mirroring\
                     `wat-scripts/fixes/to-faithful-clojure-net.wat`'s own repair), or delete it.\
                     \n\
                     ⛔ THIS GATE HAS NO EXEMPTION CATEGORY, BY DESIGN\
                     (`docs/arc/2026/06/278-rules-engine/strike-no-rule-that-cannot-compile/DESIGN.md`).\
                     There is no rune that waves a non-compiling rule through. If a genuine\
                     negative needs to live in this corpus, minting the first exemption category\
                     is a deliberate act with its own argument — it does not belong in a fix for\
                     one red file.\n\n{}\n",
                    failures.len(),
                    $idx,
                    failures.join("\n")
                );
            }
        )*
    };
}

shards! {
    every_wat_scripts_rete_rule_compiles_shard_00 = 0;
    every_wat_scripts_rete_rule_compiles_shard_01 = 1;
    every_wat_scripts_rete_rule_compiles_shard_02 = 2;
    every_wat_scripts_rete_rule_compiles_shard_03 = 3;
    every_wat_scripts_rete_rule_compiles_shard_04 = 4;
    every_wat_scripts_rete_rule_compiles_shard_05 = 5;
    every_wat_scripts_rete_rule_compiles_shard_06 = 6;
    every_wat_scripts_rete_rule_compiles_shard_07 = 7;
    every_wat_scripts_rete_rule_compiles_shard_08 = 8;
    every_wat_scripts_rete_rule_compiles_shard_09 = 9;
    every_wat_scripts_rete_rule_compiles_shard_10 = 10;
    every_wat_scripts_rete_rule_compiles_shard_11 = 11;
    every_wat_scripts_rete_rule_compiles_shard_12 = 12;
    every_wat_scripts_rete_rule_compiles_shard_13 = 13;
    every_wat_scripts_rete_rule_compiles_shard_14 = 14;
    every_wat_scripts_rete_rule_compiles_shard_15 = 15;
}

/// Over the whole corpus, independent of sharding: the walk must actually find the
/// rule-declaring population it claims to guard. Runs the full (cheap) extraction pass again —
/// no `startup_from_source`, no compile — so it stays fast even though `check_shard` above
/// re-reads every file per shard already.
#[test]
fn the_rule_declaring_population_is_not_vacuous() {
    let paths = corpus();
    let mut rule_files = 0usize;
    let mut no_exemption_rune_anywhere = true;
    for path in &paths {
        let rel = path.to_str().expect("utf8 path");
        let src = std::fs::read_to_string(rel).unwrap_or_else(|e| panic!("read {rel}: {e}"));
        if !declared_namespaces(&src).is_empty() {
            rule_files += 1;
        }
        if src.contains("rune:lint(red-by-design)") || src.contains("DECLARED_CATEGORIES") {
            no_exemption_rune_anywhere = false;
        }
    }
    // NON-VACUITY: a walk that comes back empty asserts nothing over nothing and reports PASS.
    assert!(
        rule_files >= RULE_FILE_FLOOR,
        "found only {rule_files} rule-declaring file(s) under wat-scripts/ — under the floor of \
         {RULE_FILE_FLOOR}, so the walk is not reaching the population it claims to guard and a \
         green verdict from the shards above would mean nothing",
    );
    assert!(
        no_exemption_rune_anywhere,
        "a wat-scripts/ file declares a rete-compile exemption rune, but this gate ships with \
         ZERO exemption categories by contract decision (DESIGN.md) — either the rune is a false \
         positive in this check, or someone minted an exemption this gate does not know about"
    );
}

#[cfg(test)]
mod extraction {
    use super::declared_namespaces;

    #[test]
    fn a_single_defrule_yields_its_namespace() {
        let src = "(:wat::rete::defrule :fix::g1-keyword :when [] :then [])";
        let ns: Vec<_> = declared_namespaces(src).into_iter().collect();
        assert_eq!(ns, vec!["fix".to_string()]);
    }

    #[test]
    fn a_deeper_namespace_drops_only_the_last_segment() {
        let src = "(:wat::rete::defquery :fix::sub::q-Hit :params [])";
        let ns: Vec<_> = declared_namespaces(src).into_iter().collect();
        assert_eq!(ns, vec!["fix::sub".to_string()]);
    }

    #[test]
    fn extraction_is_not_greedy_across_two_rules() {
        // The census script's own documented trap: a greedy `.*` strip on two matches on one
        // line collapses to the SECOND rule's bare name. This must recover BOTH namespaces.
        let src = "(:wat::rete::defrule :fix::g3 ...) (:wat::rete::defrule :other::g4 ...)";
        let ns: Vec<_> = declared_namespaces(src).into_iter().collect();
        assert_eq!(ns, vec!["fix".to_string(), "other".to_string()]);
    }

    #[test]
    fn a_token_with_no_following_identifier_yields_nothing() {
        let src = "the word rete::defrule appears here with no colon-name after it";
        assert!(declared_namespaces(src).is_empty());
    }

    #[test]
    fn a_bare_token_inside_a_string_literal_with_no_ns_match_yields_nothing() {
        // Mirrors the 8 corpus codemods FINDING-loading-is-not-compiling.md names: the token
        // sits in a string a codemod rewrites, never followed by a `:namespace::name`.
        let src = r#"(:wat::core::defn :fix::rename [] -> wat.type/String "rete::defrule renamed")"#;
        assert!(declared_namespaces(src).is_empty());
    }

    #[test]
    fn defquery_is_recognized_same_as_defrule() {
        let src = "(:wat::rete::defquery :q::find-it :params [])";
        let ns: Vec<_> = declared_namespaces(src).into_iter().collect();
        assert_eq!(ns, vec!["q".to_string()]);
    }

    /// One source, one defrule in each spelling. Both namespaces are found.
    #[test]
    fn keyword_and_symbol_defrules_in_one_source_are_both_found() {
        let src = "(:wat::rete::defrule :fix::g1 :when [] :then []) \
                   (wat.rete/defrule other/g2 :when [] :then [])";
        let ns: Vec<_> = declared_namespaces(src).into_iter().collect();
        assert_eq!(ns, vec!["fix".to_string(), "other".to_string()]);
    }
}
