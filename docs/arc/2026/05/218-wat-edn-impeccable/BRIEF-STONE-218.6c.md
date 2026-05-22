# BRIEF — Arc 218 Stone 218.6c — Toward impeccable: fixes + demotions + rune rebalance

**Stone scope (sonnet portion):** 10 items addressing all 9 L1 findings from VIGILIA-REPORT-2026-05-22-CHECKPOINT.md, plus 2 rune removals justified by Part A's silent-Null fix. JSON bridge surface KEPT per user direction 2026-05-22 — arc 116 cargo integration + IPC-BRIDGE.md vision are real consumers; the discipline is right-shape the public surface and make every remaining rune strongly justified.
**Type:** Sonnet Mode A.
**Time budget:** 20-30 min target; 45 min STOP.
**Depends on:** Stone 218.6b (`c79eb5f` — 10/10 PASS) + VIGILIA-REPORT-2026-05-22-CHECKPOINT.md (`c556af9`).
**Calibration:** 218.6 ~8 (band 30-45), 218.6b ~6 (band 15-25), both below band. 218.6c is larger surface (4 source files + USER-GUIDE + lib.rs visibility); pure mechanical except Part A.2 needs honest replacement and USER-GUIDE Part A.3 regen needs running the writer.
**Unblocks:** Stone 218.6d (L2 sweep) → vigilia recast → arc 218 closure conversation.

## User direction 2026-05-22 (load-bearing for this stone)

> *"impeccable is zeros - we still have non-zeros or no?"*
> *"do we need to rune anything? runes require significant justification"*
> *"we keep the JSON support - we just need to make sure its fucking impressive - i think we're at impressive but have either little or no utilization - i don't know if that negates the engineering work that went into making it great"*

The discipline: every remaining rune holds significant justification. Every L1 either fixes at root, demotes to internal, or earns a properly-justified rune. JSON bridge stays — arc 116 cargo integration (`src/edn_shim.rs:105,166` calls `to_json_string`) + `interop-tests/json_consumer.rs` + `interop-tests/json_producer.rs` are real consumers.

## Pre-flight verified (orchestrator-grep'd 2026-05-22)

### Part A — Substrate fixes (4 items)

**1. `is_scalar` missing BigInt/BigDec — `crates/wat-edn/src/writer.rs:45-58`**

```rust
fn is_scalar(v: &Value) -> bool {
    matches!(
        v,
        Value::Nil
            | Value::Bool(_)
            | Value::Integer(_)
            | Value::Float(_)
            | Value::String(_)
            | Value::Char(_)
            | Value::Symbol(_)
            | Value::Keyword(_)
            | Value::Inst(_)
            | Value::Uuid(_)
    )
}
```

`Value::BigInt(_)` and `Value::BigDec(_)` are scalar by every honest reading — they have no sub-elements, they print inline as `42N` / `3.14M`. The function's omission causes `write_pretty_indented` to break them to multi-line inside collections, which is wrong. struere L1 in 2026-05-22 cast (was L2 in 2026-05-21 cast; upgraded under strict polishing).

**2. Silent-Null fallback in finite-float branch — `crates/wat-edn/src/json.rs:113-124`**

```rust
Value::Float(f) => {
    if f.is_nan() {
        sentinel("#float", JV::String("nan".into()))
    } else if f.is_infinite() {
        let tag = if f.is_sign_negative() { "neg-inf" } else { "inf" };
        sentinel("#float", JV::String(tag.into()))
    } else {
        Number::from_f64(*f)
            .map(JV::Number)
            .unwrap_or(JV::Null)        // ← silent-Null fallback
    }
}
```

`Number::from_f64` returns `None` only for NaN/Inf. The branches above already handle NaN + Inf. The `.unwrap_or(JV::Null)` is dead defensive code that silently converts unrepresentable f64 → JSON null. This dishonesty is what makes the existing `struere(invariant-coupling)` rune on `to_json_string` (json.rs:170-174) a LIE: the rune claims "closed construction guarantees no failure" but this fallback violates that guarantee. Replace `.unwrap_or(JV::Null)` with `.expect("finite f64 must convert to serde_json::Number per from_f64 contract")` — the panic IS structurally unreachable.

**3-5. USER-GUIDE drift × 3 — `crates/wat-edn/docs/USER-GUIDE.md`**

- **3a. `:805-822` ErrorKind listing** omits 2 live variants: `UnexpectedToken(&'static str)` (error.rs:17) and `Utf8(String)` (error.rs:33). Add both.
- **3b. `:824-837` JsonError listing** omits 2 live variants: `InvalidSet(String)` (json.rs:89; added Stone 218.6) and `InvalidMapKey { key: String, reason: String }` (json.rs:91-93). Add both.
- **3c. `:457-463` pretty-print example** is structurally impossible given the code: code uses `INDENT = "  "` (2 spaces); map entries are indented at `level+1 = 1` (2 spaces); closing `}` is always on its own line at level 0 after `\n` + push_indent (writer.rs:119-124). USER-GUIDE shows 1-space-indented entries + `}` on same line as last entry. **Regenerate the example** by running `cargo run --release -p wat-edn --example bench` OR writing a small Rust snippet that parses the example input and calls `write_pretty`, then paste the actual output verbatim.

### Part B — Public-API rebalance (4 items)

**6. Demote `edn_to_json` to `pub(crate)` — `crates/wat-edn/src/json.rs:100`**

Currently `pub fn edn_to_json(v: &Value<'_>) -> JV`. Remove from `lib.rs:84-87` re-export list. Change `json.rs:100` to `pub(crate) fn edn_to_json(...)`. Used internally by `to_json_string` + `to_json_string_pretty`. No external consumers verified via grep across `src/`, `crates/wat-edn-clj/` (doesn't exist), `interop-tests/`. The honest API surface is the high-level `to_json_string`; the transform is mechanic.

**7. Demote `json_to_edn` to `pub(crate)` — `crates/wat-edn/src/json.rs:204`**

Same shape as #6. Currently `pub fn json_to_edn(v: &JV) -> JsonResult<OwnedValue>`. Used internally by `from_json_string`. No external consumers. Demote to `pub(crate)`.

**8. Add `purgare(public-api)` rune on `to_json_string_pretty` — `crates/wat-edn/src/json.rs:185`**

Append a `purgare(public-api)` rune block ABOVE the existing struere+temperare runes:

```rust
// rune:purgare(public-api) — symmetric pretty variant of to_json_string
// (consumed by src/edn_shim.rs for WAT_TEST_OUTPUT cargo integration per
// arc 116). Impressive JSON bridges ship both compact and pretty forms;
// removing this would leave an asymmetric surface. The pretty variant
// is the natural API for human-readable JSON output (debug logs, error
// envelopes, REPL inspection).
pub fn to_json_string_pretty(v: &Value<'_>) -> String {
    ...
}
```

**9. Add `purgare(public-api)` rune on `write_to` — `crates/wat-edn/src/writer.rs:187`**

Append a `purgare(public-api)` rune block above the function:

```rust
// rune:purgare(public-api) — buffer-reuse ergonomic for performance-
// conscious consumers; symmetric with the actively-consumed `write` fn.
// Documented in crates/wat-edn/docs/IPC-BRIDGE.md:95 as part of the
// future Clojure-IPC bridge surface. The append-to-existing-buffer
// shape is the canonical Rust pattern for output composition; removing
// it would force consumers to allocate per write or write a wrapper.
pub fn write_to(v: &Value<'_>, out: &mut String) {
    ...
}
```

### Part C — Rune removal (1 item; addresses 2 sites)

**10. Delete 2 `struere(invariant-coupling)` runes — `crates/wat-edn/src/json.rs:167-172` + `:177-182`**

After Part A.2 lands (silent-Null fallback fixed), the `.expect()` on `serde_json::to_string` is structurally unreachable for honest reasons: `edn_to_json` truly produces only well-formed `serde_json::Value` graphs (no NaN-in-Number) because the finite-float branch now panics on impossible input rather than silently producing `JV::Null`. The struere rune block becomes redundant — the `.expect()` call itself names the assertion; the rune adds no signal the code lacks.

Delete the rune block on both `to_json_string` (json.rs:167-172) and `to_json_string_pretty` (json.rs:177-182). KEEP the existing `temperare(serde-api-shape)` rune blocks on both — those name the serde API double-materialization trade-off, which is real and unchanged.

## Working dir + constraints

- `/home/watmin/work/holon/wat-rs/`
- Branch: `arc-170-gap-j-v5-deadlock-state`
- Linux only; Zero Mutex; no `--no-verify`
- Substrate trust binary; every L1 must land cleanly

## Your scope (sonnet)

Execute Parts A, B, C IN ORDER (A.2 must land before C.10).

### Part A.1 — is_scalar fix

At `crates/wat-edn/src/writer.rs:45-58`, add `Value::BigInt(_) | Value::BigDec(_)` arms to the `matches!`.

### Part A.2 — Silent-Null fallback fix

At `crates/wat-edn/src/json.rs:119-123`, replace `.unwrap_or(JV::Null)` with `.expect("finite f64 must convert to serde_json::Number per from_f64 contract")`.

### Part A.3a — USER-GUIDE ErrorKind listing

At `crates/wat-edn/docs/USER-GUIDE.md:805-822`, add the 2 missing variants in their natural position within the enum block.

### Part A.3b — USER-GUIDE JsonError listing

At `crates/wat-edn/docs/USER-GUIDE.md:824-837`, add the 2 missing variants. `InvalidMapKey` is a struct-form variant `{ key: String, reason: String }` — preserve that shape in the doc.

### Part A.3c — USER-GUIDE pretty-print example regeneration

The current example at `:457-463` is wrong. Regenerate by writing a tiny Rust snippet that parses the surrounding fixture (or build the equivalent `Value` directly) and calls `write_pretty`. Paste actual output. The shape will be: 2-space indented entries; closing `}` on its own line at outer level; nested vectors break at 2-space increments.

A pragmatic path: in `crates/wat-edn/tests/pretty.rs` (if a relevant test exists already) or in a quick ad-hoc binary, produce the output and copy it into the USER-GUIDE. Honest delta accepted: alternative example shape if the existing fixture is hard to match — pick a fixture that's clearly pretty-print-illustrative.

### Part B.6 — edn_to_json demotion

1. At `crates/wat-edn/src/json.rs:100`, change `pub fn` → `pub(crate) fn`.
2. At `crates/wat-edn/src/lib.rs:84-87`, remove `edn_to_json` from the `pub use json::{...}` list.

### Part B.7 — json_to_edn demotion

Same shape as B.6 at `json.rs:204` and `lib.rs:84-87`.

### Part B.8 — to_json_string_pretty rune

At `crates/wat-edn/src/json.rs:185` (above `pub fn to_json_string_pretty`), add the `purgare(public-api)` rune block per Part B.8 above. Place it BEFORE the existing struere + temperare rune blocks (which sit between the doc comment and the function body).

### Part B.9 — write_to rune

At `crates/wat-edn/src/writer.rs:187` (or wherever `pub fn write_to` is — verify the line), add the `purgare(public-api)` rune block per Part B.9 above.

### Part C.10 — Delete 2 struere runes

After Part A.2 lands and tests pass: at `crates/wat-edn/src/json.rs:167-172` (struere block above `to_json_string`) delete the entire `// rune:struere(invariant-coupling) — ...` comment block (5 lines). Same at `json.rs:177-182` (struere block above `to_json_string_pretty`). KEEP the temperare blocks intact.

### Verification (must run; no exceptions)

1. `cargo build --release -p wat-edn` — 0 warnings / 0 errors
2. `cargo test --release -p wat-edn` — expected 344 PASS (no test count change; all fixes are correctness-preserving). Report actual.
3. `cargo test --release --lib -p wat` — 824/0 PASS (`edn_shim.rs` uses `to_json_string` which stays public; no consumer regression)
4. `cargo clippy --release --all-targets -p wat-edn -- -D warnings` — 0 warnings
5. From `crates/wat-edn/interop-tests/`:
   - `cargo build --release` — 0 warnings (`json_consumer.rs` uses `from_json_string` which stays public; `json_producer.rs` uses `to_json_string` which stays public)
   - `cargo clippy --release --all-targets -- -D warnings` — 0 warnings
6. **Interop-tests 4 handshakes (mandatory per `feedback_wat_edn_touch_runs_interop_tests`):**
   ```sh
   cd crates/wat-edn/interop-tests
   cargo run --release --bin wat-edn-interop-tests | clojure -M clj/consume.clj
   clojure -M clj/produce.clj | cargo run --release --bin reader
   cargo run --release --bin shape_matrix | clojure -M clj/consume_shapes.clj
   clojure -M clj/produce_shapes.clj | cargo run --release --bin shape_matrix_reader
   ```

NOTE: if the piped `cargo run | clojure -M` form gets denied by sub-agent permissions, ship everything else + report; orchestrator runs the 4 handshakes during independent scoring per the Stone 218.6b precedent.

## STOP triggers

- **STOP-1 (existing test breaks on is_scalar fix):** if any test currently asserts that BigInt/BigDec break to multi-line inside collections, that test was asserting wrong-now behavior — report + adjust.
- **STOP-2 (`.expect()` panics in a test):** would mean an existing test path hits the silent-Null fallback today; investigate which test (the panic itself is the diagnostic).
- **STOP-3 (edn_to_json or json_to_edn used by an unexpected external consumer):** vigilia + orchestrator grep say zero; if a hit surfaces, report + STOP for guidance.
- **STOP-4 (pretty-print example regeneration breaks doc-test):** if there's a doc-test asserting the prior wrong example, the doc-test was asserting wrong-now behavior — update.
- **STOP-5 (interop handshake fail):** likely sub-agent permission denial (see verification note above) — report + ship; orchestrator runs them.
- **STOP-6 (45 min elapsed):** wall-clock STOP.

## Out-of-scope

- L2 sweep (struere L2 × 3, intueri L2 × 6, etc.) — Stone 218.6d
- INSCRIPTION — arc 218 closure deferred per user direction "218 has work we haven't expressed yet"
- New public surface beyond the rune-justified `to_json_string_pretty` + `write_to` 
- Touching tagged-literal naming / wat-edn syntax — encoding doctrine locked
- Performance optimization beyond surfaced items
