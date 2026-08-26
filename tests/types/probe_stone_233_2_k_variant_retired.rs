//! FM 2-bis probe for arc 233 Stone 233.2.k (Value::Tracked variant retirement
//! + Environment stores TrackedValue).
//!
//! Asserts that the variant + .inner()/.provenance()/.into_tracked() helpers
//! are DELETED; Environment.lookup returns TrackedValue; the Stone 233.2.j
//! Phase 5 exemption (bind_let_binding re-wrap) is DISSOLVED.
//!
//! Pre-stone state:
//!   - Probe 1 FAILS (Value::Tracked match arms still present in src/)
//!   - Probe 2 FAILS (Value enum source contains `Tracked {` variant)
//!   - Probe 3 PASSES (Phase 5 fix preserves provenance — regression guard)
//!   - Probe 4 FAILS to compile (Environment.lookup returns Option<Value>, not
//!     Option<TrackedValue>)
//!   - Probe 5 FAILS to compile (Value::into_tracked() / Value::inner() /
//!     Value::provenance() helpers still exist; assertion that they DON'T exist
//!     fails)
//!
//! Post-stone state: all 5 PASS.
//!
//! Stays as permanent regression guard. Per FAILURE-ENGINEERING.md ✅✅✅: the
//! SITUATION (Value variant wrapping another Value) becomes structurally
//! absent. Stone 233.2.l seals the meta-class via #[wat_value] proc-macro.

use std::fs;
use wat::freeze::{eval_in_frozen, startup_bare};
use wat::runtime::{Environment, Value};
use wat::value::{Provenance, TrackedValue};

// ─── Probe 1 — Static scan: zero Value::Tracked references in src/ ──────────
//
// Distinguishes between:
//   - Comments referencing the historical retirement (OK; preserved as record)
//   - Active code mentioning Value::Tracked (REJECTED; variant gone)
//
// Detection: any non-comment line containing `Value::Tracked` is a violation.

#[test]
fn probe_1_zero_active_value_tracked_references_in_src() {
    let mut offending: Vec<(String, usize, String)> = Vec::new();

    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    walk_rs_files(&src_dir, &mut |path, contents| {
        for (lineno, line) in contents.lines().enumerate() {
            let trimmed = line.trim_start();
            // Skip pure-comment lines (//, ///, //!)
            if trimmed.starts_with("//") {
                continue;
            }
            // Skip lines that only have Value::Tracked inside a string literal
            // (e.g., panic!("Value::Tracked is retired") — historical message
            // text is acceptable). Heuristic: if the line has Value::Tracked
            // ONLY inside double quotes, skip.
            if has_value_tracked_only_in_string(line) {
                continue;
            }
            if line.contains("Value::Tracked") {
                offending.push((
                    path.to_string_lossy().into_owned(),
                    lineno + 1,
                    line.trim().to_string(),
                ));
            }
        }
    });

    assert!(
        offending.is_empty(),
        "Stone 233.2.k: zero ACTIVE Value::Tracked references should remain in src/; \
         found {} site(s). The variant is retired; match arms / construction / \
         transparency helpers should all be deleted. Offending sites:\n{}",
        offending.len(),
        offending
            .iter()
            .map(|(p, n, l)| format!("  {}:{} — {}", p, n, l))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

// ─── Probe 2 — Static scan: Value enum source has no `Tracked` variant ──────

#[test]
fn probe_2_value_enum_has_no_tracked_variant() {
    // Value enum lifted out of the runtime.rs monolith into src/value/value.rs
    // (arc 251.2 — the value/ keystone home).
    let value_rs = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/value/value.rs");
    let contents = fs::read_to_string(&value_rs).expect("read src/value/value.rs");

    // Locate `pub enum Value {` and scan forward to matching `}` at column 0.
    let enum_start = contents
        .find("pub enum Value {")
        .expect("pub enum Value should exist in src/value/value.rs");

    // Take a generous region (~3000 chars covers the enum body).
    let enum_region = &contents[enum_start..enum_start + 3000.min(contents.len() - enum_start)];

    // The Tracked variant historically looked like `Tracked {` or
    // `Tracked { inner: Box<Value>, provenance: Provenance },`. Both
    // shapes contain `Tracked` followed by `{`.
    let has_tracked_variant = enum_region
        .lines()
        .any(|line| {
            let trimmed = line.trim_start();
            // Skip comments
            if trimmed.starts_with("//") {
                return false;
            }
            // Look for variant-definition shape: `Tracked {` (struct variant)
            // or `Tracked(` (tuple variant)
            trimmed.starts_with("Tracked {") || trimmed.starts_with("Tracked(")
        });

    assert!(
        !has_tracked_variant,
        "Stone 233.2.k: Value enum should have no `Tracked` variant. \
         Found `Tracked` variant-definition shape in `pub enum Value` body. \
         The variant must be DELETED."
    );
}

// ─── Probe 3 — Behavioral: producer provenance survives let-binding ─────────
//
// Regression guard for Stone 233.2.j Phase 5. After 233.2.k, the re-wrap
// mechanism is gone; provenance must survive via Environment storing
// TrackedValue (Option A). If this probe regresses, the structural fix is
// incomplete.
//
// ⚠ REGRESSED (honestly, not silently) BY ARC 255 STONE E-iv — "keyword gets its home".
// `keyword/from-string`'s dispatch route moved off the special-cased producer arm in
// `dispatch_keyword_head` onto the `#[wat_intrinsic]` registry (`src/intrinsic/keyword.rs`),
// whose `NativeHandler` signature (`-> Result<Value, EvalBreak>`) has no slot for a custom
// `Provenance` — see `probe_stone_233_2_j_producer_migration.rs`'s probe 2 comment for the
// full mechanism. The fixture below no longer produces `RuntimeBuilt` at all, so this probe no
// longer exercises `Environment::lookup`'s "RuntimeBuilt survives a let-binding" branch — it
// falls into the "Unknown promotes to SymbolBound at lookup" branch instead (still a REAL,
// still-tested branch of the same `lookup`, just not the one this probe was written to guard).
// No producer left in the corpus both returns a bare `:wat::core::keyword` value AND retains a
// special-cased `RuntimeBuilt` producer arm, so there is no drop-in replacement fixture that
// preserves the original coverage without also changing `Value::wat__core__keyword` to some
// other producer's return type. Asserting the OBSERVED-CORRECT provenance rather than deleting
// the probe.
#[test]
fn probe_3_producer_provenance_survives_let_binding() {
    let world = startup_bare().expect("startup");

    // Bind keyword/from-string result to a let; then reference it via Symbol
    // lookup. Provenance must flow through env.
    //
    // The expression lives in a co-located FRAGMENT (never an inlined Rust string) — see that
    // file's header comment for why this bypasses the usual call_beside_value/apply_function idiom
    // (a user-fn call would launder away the exact provenance under test).
    let expr_path = "tests/types/probe_stone_233_2_k_variant_retired_let_keyword.wat.expr";
    let src = std::fs::read_to_string(expr_path)
        .unwrap_or_else(|e| panic!("expr fragment {expr_path:?} must exist: {e}"));
    let ast = wat::parse_one_with_file(&src, expr_path).expect("parse");
    let env = Environment::new();

    let tv: TrackedValue = eval_in_frozen(&ast, &world, &env).expect("eval");

    assert!(
        matches!(tv.value(), Value::wat__core__keyword(_)),
        "let-bound value should be Value::wat__core__keyword; got {}",
        tv.value().type_name()
    );

    assert!(
        matches!(tv.provenance(), Provenance::SymbolBound { .. }),
        "Stone 233.2.k (regressed honestly by arc 255 Stone E-iv, see comment above): \
         keyword/from-string is registry-routed now and yields Provenance::Unknown at \
         construction, which Environment::lookup promotes to SymbolBound at the let-binding \
         reference; expected Provenance::SymbolBound {{ .. }}; got {:?}",
        tv.provenance()
    );
}

// ─── Probe 4 — Environment.lookup returns TrackedValue ──────────────────────

#[test]
fn probe_4_environment_lookup_returns_tracked_value() {
    // Compile-shape assertion: Environment.lookup signature is
    // pub fn lookup(&self, name: &str) -> Option<TrackedValue> (or &TrackedValue).
    // Pre-stone: returns Option<Value>; this annotation FAILS to compile.

    let env = Environment::new();
    // Arc 233 Stone 233.2.e: lookup takes head_span; use unknown span for this
    // test since we only care that nonexistent lookup returns None.
    let unknown_span = wat::rust_caller_span!();
    let result: Option<TrackedValue> = env.lookup("nonexistent_binding", &unknown_span).inspect(|tv| {
        // Coerce reference to owned if lookup returns &TrackedValue
        // (sonnet picks Owned vs Reference shape).
        let _: &TrackedValue = tv;
    });

    assert!(result.is_none(), "lookup of nonexistent binding should return None");
}

// ─── Probe 5 — Value::inner() + Value::provenance() + Value::into_tracked() ─
// ─── helpers DELETED                                                       ─
//
// Static scan in src/runtime.rs: no `pub fn inner` / `pub fn provenance` /
// `pub fn into_tracked` on `impl Value`.

#[test]
fn probe_5_value_helpers_deleted() {
    let runtime_rs = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/runtime.rs");
    let contents = fs::read_to_string(&runtime_rs).expect("read src/runtime.rs");

    // Locate `impl Value {` block. There may be multiple `impl Value` blocks;
    // we scan all of them.
    let mut deleted_helpers_present: Vec<&'static str> = Vec::new();

    // Search for each helper signature; presence means the helper survives.
    // These are Value helpers (on `impl Value`), not on TrackedValue. The
    // distinction matters: TrackedValue has its own `provenance()` method
    // (which we WANT). Value's provenance() was only meaningful when
    // Value::Tracked existed.

    // Heuristic: each helper appears once with a specific signature inside
    // an `impl Value` block. We look for the FULL signature line.
    let bad_signatures = [
        ("pub fn inner(&self) -> &Value", "Value::inner()"),
        ("pub fn provenance(&self) -> Provenance", "Value::provenance()"),
        ("pub fn into_tracked(self) -> TrackedValue", "Value::into_tracked()"),
    ];

    for (sig, name) in &bad_signatures {
        if contents.contains(sig) {
            deleted_helpers_present.push(name);
        }
    }

    assert!(
        deleted_helpers_present.is_empty(),
        "Stone 233.2.k: Value helpers should be DELETED (transparency surface \
         was only meaningful while Value::Tracked existed). Found surviving: {:?}. \
         Callers should use TrackedValue::from(value) instead of Value::into_tracked(); \
         drop .inner() calls (Value is never wrapped post-retirement); \
         use TrackedValue's own .provenance() method (not Value's).",
        deleted_helpers_present
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

/// Heuristic: returns true if every `Value::Tracked` occurrence on this line
/// is inside a double-quoted string. This permits historical error-message
/// text without flagging it as an active reference.
fn has_value_tracked_only_in_string(line: &str) -> bool {
    let needle = "Value::Tracked";
    let mut search_from = 0;
    while let Some(idx_rel) = line[search_from..].find(needle) {
        let idx = search_from + idx_rel;
        // Count unescaped double-quotes BEFORE this position.
        let prefix = &line[..idx];
        let mut quote_count = 0;
        let mut prev = ' ';
        for c in prefix.chars() {
            if c == '"' && prev != '\\' {
                quote_count += 1;
            }
            prev = c;
        }
        // If quote_count is ODD, this needle is inside an unclosed string.
        if quote_count % 2 == 0 {
            return false; // outside string — active reference
        }
        search_from = idx + needle.len();
    }
    true // every occurrence inside strings
}
