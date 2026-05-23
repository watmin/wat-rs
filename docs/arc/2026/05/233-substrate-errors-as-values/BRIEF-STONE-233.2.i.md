# BRIEF — Arc 233 Stone 233.2.i — flip eval signature to TrackedValue

## What we're doing

Flip the eval boundary to return `Result<TrackedValue, RuntimeError>` instead of `Result<Value, RuntimeError>`:

- `pub fn eval` at `src/runtime.rs:4512`
- `pub fn eval_in_frozen` at `src/freeze.rs:1260`
- `pub fn eval_digest_in_frozen` at `src/freeze.rs:1283`
- `pub fn eval_signed_in_frozen` at `src/freeze.rs:1314`

**Critical adaptation:** at the eval boundary, UNWRAP any `Value::Tracked` variant into TrackedValue's inner+provenance fields. This preserves producer-attached provenance from Stones 233.2.b/c without requiring producer migration (that's Stone 233.2.j). The internal substrate keeps wrapping with Value::Tracked; the boundary translates.

**Boundary wrap logic:**

```rust
pub fn eval(ast, env, sym) -> Result<TrackedValue, RuntimeError> {
    let value = eval_inner(ast, env, sym)?;  // existing logic; returns Value
    Ok(match value {
        Value::Tracked { inner, provenance } => TrackedValue::new(*inner, provenance),
        other => TrackedValue::from(other),  // Provenance::Unknown
    })
}
```

Sonnet picks the cleanest split: rename current `eval` body to `eval_inner` (or similar; internal) and add a thin public wrapper. OR inline the wrap at the end of the current body.

After this stone: every internal `eval(...)?` call site needs `.value_owned()` to extract bare Value. Cargo enumerates the cascade.

## Design substrate (READ FIRST; MANDATORY)

1. **`docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.g.md`** — sub-DESIGN; Shape A verdict; execution decomposition. Stone 233.2.i + 233.2.j + 233.2.k together eliminate the trap-door class. This stone establishes the BOUNDARY.

2. **`tests/probe_eval_signature_returns_tracked_value.rs`** (commit `df7dcb8`) — FM 2-bis disconfirming probe. Pre-stone: 3 type-mismatches showing `eval_in_frozen` returns `Value`. **The probe IS the success criterion** — sonnet flips 0/3 → 3/3.

3. **`docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.h.md`** — TrackedValue mint precedent (commit `38acd60`). The type sonnet now returns from eval.

4. **`docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.d.md`** — uniform list_span sweep precedent. Same substrate-as-teacher cascade shape per FM 15.

5. **`docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 15** — substrate-as-teacher pattern. **Short BRIEF; sonnet iterates from compile errors.** Cargo enumerates the worklist.

## Implementation surface

1. **Boundary wrap** — `pub fn eval` returns `Result<TrackedValue, RuntimeError>`. Extract Value::Tracked variant into TrackedValue at the return point.
2. **Cascade through internal callers** — every `eval(...)?` call site inside other eval_<name> fns adds `.value_owned()` to extract bare Value. Cargo errors enumerate.
3. **Public surfaces** — `eval_in_frozen` + `eval_digest_in_frozen` + `eval_signed_in_frozen` flip their signatures to return TrackedValue (mostly by changing how they call eval and propagating the return type).
4. **Helper signatures** (`require_X` / `expect_X` family in `src/time.rs`, `src/spawn.rs`, possibly others) — take `TrackedValue` parameter; extract `.value_owned()` internally before pattern-match.
5. **External tests** — call sites in `tests/*.rs` may use `eval_in_frozen`; they update with `.value_owned()` or `.value()` to get bare Value if they pattern-match.

## What does NOT change

- Internal `fn eval_<name>` fns CONTINUE returning `Result<Value, RuntimeError>`. ~336 fns untouched at the signature level.
- `Value::Tracked` variant STAYS (retired in Stone 233.2.k).
- Producers continue wrapping with Value::Tracked (migrated in Stone 233.2.j).
- Pattern-matches on extracted Value continue to work with same Value::Tracked vulnerability as today. **The class isn't closed yet** — that's Stone 233.2.k.

## Out of scope (affirmative scope-bounding)

- **Producer migration** — Stone 233.2.j. This stone uses the existing Value::Tracked wrapping at producers; boundary unwraps.
- **Value::Tracked variant retirement** — Stone 233.2.k.
- **Display impl on TrackedValue** — defer.
- **Adding Eq/PartialEq/Hash on TrackedValue** — STOP-6 if borrow checker tempts. Per Stone 233.2.h: forced explicit comparison.
- **Internal eval_<name> signature changes** — they stay returning Value.
- **holon-rs** — NOT touched.
- **HARD CUT** — no `eval_tracked` parallel API; the existing `eval` IS the new boundary.

## Verification flow

```
cargo test --release --test probe_eval_signature_returns_tracked_value 2>&1 | tail -5    # 3/3 PASS post-stone
cargo build --release -p wat 2>&1 | tail -5                                              # 0 errors
cargo test --release --lib -p wat --no-fail-fast 2>&1 | tail -3                          # ≥ 827 passed; 0 failed
cargo test --release --test probe_substrate_symmetry_list_span_threading 2>&1 | tail -3  # 1/1 PASS
cargo test --release --test probe_diagnostic_value_snapshot_in_errors 2>&1 | tail -3     # 8/8 PASS
cargo test --release --test probe_value_tracked_transparency 2>&1 | tail -3              # 8/8 PASS
cargo test --release --test probe_diagnostic_dynamic_keyword_invocation 2>&1 | tail -3   # 8/8 PASS
cargo test --release --test probe_tracked_value_mint_contract 2>&1 | tail -3             # 6/6 PASS
cargo clippy --release --lib -p wat -- -D warnings 2>&1 | grep -c "warning"              # ≤ 54
git -C /home/watmin/work/holon/holon-rs/ status --short                                  # empty
```

## STOP triggers (REJECTION criteria)

- **STOP-1:** unexpected compile errors NOT tracing to the cascade (e.g., logic break)
- **STOP-2:** baseline lib tests regress below 827
- **STOP-3:** **180 min elapsed** (this is the BIG cascade; bigger time-box than prior stones)
- **STOP-4:** holon-rs touched
- **STOP-5:** new clippy warning above 54
- **STOP-6:** scope creep — touching Value::Tracked variant body, producer logic, or internal eval_<name> signatures
- **STOP-7:** probe still has failures post-stone
- **STOP-8:** existing arc 233 probes regress
- **STOP-9 (NEW for this stone):** if the cascade exceeds the time-box, surface partial state — orchestrator decides whether to sub-slice or extend

Per FM 2-bis: STOP triggers are REJECTION criteria; never permission-to-defer.

## Trap-door audit

- **NO new parallel API.** Do NOT mint `eval_tracked`. The existing `eval` IS the new boundary.
- **NO retirement of Value::Tracked variant.** Stone 233.2.k owns that.
- **NO producer migration.** Stone 233.2.j owns that.
- **Internal eval_<name> fns stay returning Value.** Only the BOUNDARY fns flip.
- **The cascade is substrate-as-teacher (FM 15).** Don't enumerate 500+ sites upfront; let cargo enumerate by failing.
- **Helper signatures change.** `require_i64(op, v: Value)` → `require_i64(op, tv: TrackedValue)`; internally `let v = tv.value_owned();` then pattern-match. Each helper updates per its current shape.
- **External test files (`tests/*.rs`)** that call `eval_in_frozen` and pattern-match on the result need `.value_owned()` extraction. The cascade reaches them.
- **`wat::parse_one!` + `eval_in_frozen` pattern in tests** is the common idiom; tests using this pattern need adaptation.

## Scope reminders

- Mode `model: "sonnet"` (orchestrator sets explicitly per FM 12)
- HARD CUT — no aliases
- Per `feedback_inscription_immutable`: SCORE is a new file
- Per `feedback_no_broken_commits`: do NOT commit. Orchestrator commits after independent verification
- This is the BIG cascade per Stone 233.2.g sub-DESIGN. Expect substantial wall-clock; substrate-as-teacher iteration is the working pattern

## Cross-references

- `docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.g.md` — sub-DESIGN; Shape A pivot
- `docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.h.md` — TrackedValue mint precedent
- `docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.d.md` — substrate-as-teacher cascade precedent
- `tests/probe_eval_signature_returns_tracked_value.rs` — FM 2-bis probe (commit `df7dcb8`)
- `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 15 — substrate-as-teacher
- `feedback_sonnet_writes_substrate` — protocol
