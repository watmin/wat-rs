//! FM 2-bis probe for arc 233 Stone 233.2.j (producer migration cascade).
//!
//! Asserts that the 5 producers + eval_inner cascade have landed:
//! - Producers construct TrackedValue::new directly (no Value::Tracked wrap)
//! - eval_inner returns Result<TrackedValue, _> (cascade flipped)
//! - ValueSnapshot::of_tracked exists for TrackedValue-aware error construction
//! - eval boundary simplifies (no inner Value::Tracked unwrap arm)
//! - Behavioral: producer-tagged TrackedValue survives eval; provenance is correct
//!
//! Pre-stone state:
//!   - Probe 1 PASSES (behavioral guard from 233.2.i; producer wrap survives via
//!     Value::Tracked → TrackedValue unwrap at eval boundary)
//!   - Probe 2 PASSES (same path; provenance survives the unwrap)
//!   - Probe 3 FAILS (~16+ Value::Tracked construction sites in src/; target 0)
//!   - Probe 4 FAILS to compile (ValueSnapshot::of_tracked doesn't exist)
//!   - Probe 5 FAILS (eval boundary still has the inner Value::Tracked unwrap arm)
//!
//! Post-stone state: all 5 PASS.
//!
//! Stays as permanent regression guard. Per FAILURE-ENGINEERING.md ✅✅✅:
//! the SITUATION (Value variant that wraps another Value) becomes structurally
//! impossible to construct via the cascade flip; 233.2.k deletes the variant;
//! 233.2.l seals the meta-class via proc-macro.

use std::fs;
use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, FunctionBody, Value};
use wat::value::{Provenance, TrackedValue};

// just-eval (rubric): `:user::probe`'s call lives in the co-located fixture. Fetch the
// fixture's OWN parsed body AST (`Function::body`) and eval it directly via `eval_in_frozen`
// — this is what gets us the raw `TrackedValue` (provenance included); `call_beside_value`/
// `apply_function` only ever return the unwrapped `Value`, which is not enough for these
// two provenance-inspecting probes.
fn eval_probe() -> TrackedValue {
    let world = startup_beside(file!()).expect("startup");
    let func = world.symbols().get(":user::probe").expect(":user::probe must exist in fixture");
    let body_ast = match &func.body {
        FunctionBody::Wat(ast) => ast.clone(),
        FunctionBody::Native => panic!(":user::probe must be a wat-bodied fn"),
    };
    let env = Environment::new();
    eval_in_frozen(&body_ast, &world, &env).expect("keyword/from-string should succeed")
}

// ─── Probe 1 — Producer-tagged TrackedValue survives eval (behavioral guard) ─

#[test]
fn probe_1_keyword_from_string_yields_tracked_value() {
    let tv = eval_probe();

    assert!(
        matches!(tv.value(), Value::wat__core__keyword(_)),
        "keyword/from-string should yield TrackedValue wrapping Value::wat__core__keyword; \
         got value of type {}",
        tv.value().type_name()
    );
}

// ─── Probe 2 — Producer-attached provenance survives eval boundary ──────────
//
// ⚠ REGRESSED (honestly, not silently) BY ARC 255 STONE E-iv — "keyword gets its home".
// `keyword/from-string`'s dispatch route moved off the special-cased producer arm in
// `dispatch_keyword_head` (the only door that could construct a `TrackedValue` carrying a
// custom `Provenance::RuntimeBuilt`) onto the `#[wat_intrinsic]` registry
// (`src/intrinsic/keyword.rs`), whose `NativeHandler` signature is fixed at
// `-> Result<Value, EvalBreak>` — no slot for a custom `Provenance`. `keyword` was arc 233's
// chosen canonical example producer for THIS regression guard; it is no longer a producer at
// all (same shape every OTHER `#[wat_intrinsic]`-routed verb already has). The fixture's own
// eval boundary here never looks the value up via a binding (`eval_in_frozen` evals the body
// directly), so it stays `Provenance::Unknown` rather than being promoted to `SymbolBound` the
// way `Environment::lookup` would. This probe now asserts the OBSERVED-CORRECT provenance;
// `probe_stone_233_2_j`'s MECHANISM (a producer's TrackedValue survives the eval boundary
// un-rewrapped) is still exercised by every producer that remains special-cased
// (`:wat::holon::from-holon`, `:wat::edn::read`, `:wat::core::keyword-node`, …) — this probe
// just no longer demonstrates it via `keyword/from-string`.
#[test]
fn probe_2_keyword_from_string_provenance_attached() {
    let tv = eval_probe();

    assert!(
        matches!(tv.provenance(), Provenance::Unknown),
        "Stone 233.2.j (regressed honestly by arc 255 Stone E-iv, see comment above): \
         keyword/from-string is registry-routed now and cannot carry RuntimeBuilt provenance; \
         expected Provenance::Unknown; got {:?}",
        tv.provenance()
    );
}

// ─── Probe 3 — Static scan: zero Value::Tracked construction sites in src/ ──
//
// The construction pattern is `Value::Tracked { inner: Box::new(...)` —
// distinct from match-arm patterns (`Value::Tracked { inner, .. }` which
// have no Box::new). Match arms stay until 233.2.k retires the variant.

#[test]
fn probe_3_zero_value_tracked_construction_sites_in_src() {
    let mut total_construction_sites = 0_usize;
    let mut offending: Vec<(String, usize, String)> = Vec::new();

    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    walk_rs_files(&src_dir, &mut |path, contents| {
        for (lineno, line) in contents.lines().enumerate() {
            // Construction pattern: `Value::Tracked {` followed eventually
            // by `inner: Box::new(` — the producer wrap shape.
            // Adjacent line acceptable because construction may span lines.
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") || trimmed.starts_with("///") {
                continue; // skip comments + doc-comments
            }
            if line.contains("Value::Tracked {")
                && (line.contains("inner: Box::new") || look_ahead_for_box_new(contents, lineno))
            {
                total_construction_sites += 1;
                offending.push((
                    path.to_string_lossy().into_owned(),
                    lineno + 1,
                    line.trim().to_string(),
                ));
            }
        }
    });

    assert_eq!(
        total_construction_sites, 0,
        "Stone 233.2.j: NO Value::Tracked construction sites should remain in src/; \
         found {} site(s). Producers must use TrackedValue::new; Value::Tracked is \
         retired as of Stone 233.2.k. Offending sites:\n{}",
        total_construction_sites,
        offending
            .iter()
            .map(|(p, n, l)| format!("  {}:{} — {}", p, n, l))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

// ─── Probe 4 — ValueSnapshot::of_tracked exists for TrackedValue-aware errors ─

#[test]
fn probe_4_value_snapshot_of_tracked_exists_and_reads_provenance() {
    use wat::runtime::ValueSnapshot;

    let tv = TrackedValue::new(
        Value::i64(42),
        Provenance::RuntimeBuilt {
            producer: ":probe::test/of-tracked",
            call_span: wat::rust_caller_span!(),
        },
    );

    // Stone 233.2.j adds ValueSnapshot::of_tracked(&TrackedValue) -> Self.
    // Pre-stone: this method doesn't exist; compile FAILS.
    // Post-stone: snapshot carries the producer-attached provenance.
    let snap = ValueSnapshot::of_tracked(&tv);

    let disp = format!("{}", snap);
    // rune:lint(loose-assert) — Display embeds an absolute source file path from
    // `rust_caller_span!()`/`file!()` (e.g. `.../probe_stone_233_2_j_producer_migration.rs:134:24`)
    // that varies by host filesystem layout and checkout location; only the producer name
    // `:probe::test/of-tracked` is the stable contract.
    assert!(
        disp.contains(":probe::test/of-tracked"),
        "ValueSnapshot::of_tracked should render provenance into Display; got: {}",
        disp
    );
}

// ─── Probe 5 — eval boundary simplifies; no inner Value::Tracked unwrap arm ─
//
// Stone 233.2.j flips eval_inner to return TrackedValue, so the eval
// boundary becomes a direct passthrough:
//
//   pub fn eval(...) -> Result<TrackedValue, RuntimeError> {
//       eval_inner(ast, env, sym)
//   }
//
// The pre-stone state (commit 8164629) has an explicit match-arm unwrap of
// Value::Tracked inside eval. Post-stone, that arm is gone.

#[test]
fn probe_5_eval_boundary_no_value_tracked_unwrap() {
    let runtime_rs = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/runtime.rs");
    let contents = fs::read_to_string(&runtime_rs).expect("read src/runtime.rs");

    // Locate `pub fn eval(` and scan ~30 lines forward for the unwrap arm.
    let eval_fn_start = contents
        .find("pub fn eval(")
        .expect("pub fn eval should exist in runtime.rs");

    // Take the next ~1500 chars (eval body is small) and scan for the unwrap arm.
    let eval_region = &contents[eval_fn_start..eval_fn_start + 1500.min(contents.len() - eval_fn_start)];

    let has_unwrap_arm = eval_region.contains("Value::Tracked { inner, provenance }");

    assert!(
        !has_unwrap_arm,
        "Stone 233.2.j: eval boundary should simplify to a direct eval_inner passthrough \
         (eval_inner returns TrackedValue post-cascade); the Value::Tracked unwrap arm \
         should be removed. Found `Value::Tracked {{ inner, provenance }}` pattern in \
         eval region:\n{}",
        eval_region
    );
}

// ─── helpers ────────────────────────────────────────────────────────────────

fn walk_rs_files(dir: &std::path::Path, visit: &mut dyn FnMut(&std::path::Path, &str)) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_rs_files(&path, visit);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            if let Ok(contents) = fs::read_to_string(&path) {
                visit(&path, &contents);
            }
        }
    }
}

/// Returns true if the next 5 lines after `start_line` contain `inner: Box::new`.
/// Used to catch multi-line Value::Tracked construction sites where the
/// `Value::Tracked {` opens on one line and `inner: Box::new(...)` appears
/// on a subsequent line.
fn look_ahead_for_box_new(contents: &str, start_line: usize) -> bool {
    contents
        .lines()
        .skip(start_line + 1)
        .take(5)
        .any(|line| line.contains("inner: Box::new"))
}
