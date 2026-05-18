# Arc 212 stone δ-process-scope — SHARPEN `collect_process_calls` with scope-boundary tracking

**Your ONE concern this spawn:** sharpen ONE walker so it stops at nested let-form scope boundaries, then migrate the non-boundary recursion to `node.children()`. Verify three named tests pass. Nothing else.

This is a SHARPENING stone, BUT the sharpening is structurally SIMPLER than δ-comm-positions: it adds `:wat::core::let` as a NEW scope-boundary keyword alongside the existing `:wat::core::fn` / `:wat::core::lambda` boundary already in the walker. The pattern is set; one more arm in the match.

---

## The walker

**Function:** `collect_process_calls`
**File:** `/home/watmin/work/holon/wat-rs/src/check.rs`
**Line:** ~3749 (look for `fn collect_process_calls(`)

**Current shape (List-only; fn/lambda IS scope boundary; let is NOT yet):**

```rust
fn collect_process_calls(
    node: &WatAST,
    joins: &mut Vec<(String, Span)>,
    accessors: &mut Vec<(String, String, Span)>,
) {
    // TEMPORARY List-only comment (~30 lines) explaining the sharpening target...

    let WatAST::List(items, span) = node else { return };
    if let Some(WatAST::Keyword(k, _)) = items.first() {
        match k.as_str() {
            ":wat::kernel::Process/join-result" => {
                if let Some(WatAST::Symbol(id, _)) = items.get(1) {
                    joins.push((id.name.clone(), span.clone()));
                }
            }
            acc @ (":wat::kernel::Process/stdout"
            | ":wat::kernel::Process/stderr"
            | ":wat::kernel::Process/output") => {
                if let Some(WatAST::Symbol(id, _)) = items.get(1) {
                    accessors.push((id.name.clone(), acc.to_string(), span.clone()));
                }
            }
            // Do NOT recurse into nested fn bodies — they're separate scopes.
            ":wat::core::fn" | ":wat::core::lambda" => return,
            _ => {}
        }
    }
    for child in items {
        collect_process_calls(child, joins, accessors);
    }
}
```

**Target shape (children() collapse + let-form scope boundary):**

```rust
fn collect_process_calls(
    node: &WatAST,
    joins: &mut Vec<(String, Span)>,
    accessors: &mut Vec<(String, String, Span)>,
) {
    // Walker-specific List-head logic: classify Process/join-result and
    // Process/<accessor> call sites. STOP descent at fn/lambda boundaries
    // AND at :wat::core::let boundaries — both are separate lexical scopes.
    //
    // Per arc 212 stone δ-process-scope: the walker stops at let-form
    // boundaries because find_process_join_before_drain (this walker's
    // caller) is invoked per-let-scope from infer_let. The type-checker's
    // iteration over let-forms ensures each scope is checked independently
    // — the walker must NOT descend across let boundaries or it would
    // conflate inner-let Process accessors with the outer scope's tracking.
    if let WatAST::List(items, span) = node {
        if let Some(WatAST::Keyword(k, _)) = items.first() {
            match k.as_str() {
                ":wat::kernel::Process/join-result" => {
                    if let Some(WatAST::Symbol(id, _)) = items.get(1) {
                        joins.push((id.name.clone(), span.clone()));
                    }
                }
                acc @ (":wat::kernel::Process/stdout"
                | ":wat::kernel::Process/stderr"
                | ":wat::kernel::Process/output") => {
                    if let Some(WatAST::Symbol(id, _)) = items.get(1) {
                        accessors.push((id.name.clone(), acc.to_string(), span.clone()));
                    }
                }
                // Scope boundaries — stop descent. fn/lambda existed
                // pre-arc-212; let added in stone δ-process-scope so the
                // walker's RULE matches the caller's per-let-scope framing.
                ":wat::core::fn" | ":wat::core::lambda" | ":wat::core::let" => return,
                _ => {}
            }
        }
    }
    // Arc 212 — generic recursion via children() covers List, Vector, and
    // StructPattern uniformly. Scope-boundary arms above return without
    // descending. children() returns &[] for leaf nodes (no-op).
    for child in node.children() {
        collect_process_calls(child, joins, accessors);
    }
}
```

**The migration:**
1. Keep all existing classification logic (Process/join-result + Process/<accessor> push)
2. Keep the fn/lambda early-return (load-bearing)
3. **ADD `:wat::core::let` to the scope-boundary arm** (the sharpening — joins fn/lambda)
4. Move the recursion OUT of the `if let WatAST::List` guard; route through `node.children()`
5. Update the comment block to explain the let scope boundary

**Do NOT change:**
- The Process classification logic (which keywords push to joins vs accessors)
- The fn/lambda existing scope boundary
- The `joins` / `accessors` Vec mutation signature

---

## Why this works (the architecture)

`collect_process_calls` is invoked once per let-form by `find_process_join_before_drain`, which is called from `infer_let` (src/check.rs:7544) during type-checking. The type-checker walks every let-form and triggers the deadlock check on each.

If the walker descends across nested let boundaries, it would collect Process accessors from inner-let scopes and conflate them with the outer let's tracking → false positives.

By stopping at let boundaries, the walker collects only the calls in THIS let's scope. Each inner let is independently checked when the type-checker reaches it. The arc 117 rule ("Process/join-result and Process/<accessor> in SAME lexical scope = deadlock") is honored exactly.

The substrate's stdlib (wat/test.wat, wat/kernel/hermetic.wat, wat/kernel/sandbox.wat) contains patterns where outer + inner lets independently use Process accessors. Pre-arc-212 the walker was List-only; pre-naive-migration it didn't descend into Vector. Now with children() recursion + let scope boundary, both correctness paths are preserved.

---

## The wat-test proof gate

Three tests exercise the broader Process / deadlock discipline:

| Test | Verifies |
|---|---|
| `cargo test --release --test wat_arc170_stone_a_drain_and_join` | Process drain-and-join pattern — directly related to ProcessJoinBeforeOutputDrain semantics |
| `cargo test --release --test wat_arc202_process_join_holds_stdin` | Sibling Process walker (regression check; this walker was the δ-process-stdin-joins target) |
| `cargo test --release --test probe_run_hermetic_no_deadlock` | Explicit no-deadlock probe; should NOT false-positive |

**All three MUST pass POST-migration.**

---

## Verification protocol

1. Read `src/check.rs:3749` for `collect_process_calls`
2. Apply the migration (see "Target shape")
3. Run `cargo build --release 2>&1 | tail -5` — must compile clean
4. Run all three named tests:
   ```bash
   cargo test --release --test wat_arc170_stone_a_drain_and_join 2>&1 | tail -5
   cargo test --release --test wat_arc202_process_join_holds_stdin 2>&1 | tail -5
   cargo test --release --test probe_run_hermetic_no_deadlock 2>&1 | tail -5
   ```
5. Write SCORE file at `docs/arc/2026/05/212-runtime-quasiquote-vector-watast/SCORE-212-DELTA-PROCESS-SCOPE.md`

---

## STOP triggers — VERBATIM

Non-negotiable. If any fires, STOP IMMEDIATELY.

1. **Any named test FAILS post-migration.** STOP. Revert your edit. Inscribe in SCORE: which test, which file:line in the test. Do not investigate WHY. Do NOT modify the test. Return.

   **Sub-rule (Mode B):** if the test fails with a NEW `ProcessJoinBeforeOutputDrain` error on a pattern that was previously slipping past (genuine substrate-teaching), that's honest delta. Still STOP. Still revert. Still report.

2. **cargo build FAILS.** STOP. Inscribe the compile error. Return.

3. **You see a failing test OUTSIDE the three named.** STOP. Workspace failure count is NOT your concern.

4. **You feel the urge to migrate another walker while you're here.** STOP. ONE walker per stone.

5. **You feel the urge to "improve" the classification logic, error variant, or caller (find_process_join_before_drain).** STOP. The sharpening is bounded: add let to the scope-boundary arm + migrate recursion to children(). Nothing else.

6. **Anything outside this concern surfaces.** STOP. Return what you have.

---

## What the SCORE file contains

`docs/arc/2026/05/212-runtime-quasiquote-vector-watast/SCORE-212-DELTA-PROCESS-SCOPE.md`:

1. Header
2. Summary: recursion shape changed; let added to scope-boundary arm; coverage extends to Vector + StructPattern in non-boundary nodes
3. Verification: three lines (one per named test)
4. Build line: cargo build clean
5. Honest-delta note if Mode B
6. Mode classification

---

## Constraints

- Edit ONLY `src/check.rs`
- Touch ONLY `collect_process_calls` within that file
- Zero other code edits anywhere
- Zero git operations (orchestrator commits)
- Zero test-file edits
- Run ONLY the three named tests + cargo build
- No `cargo test --workspace`

---

## Time prediction

5-10 min Mode A. Sharpening is structurally simple (1 keyword added to existing match arm + mechanical recursion collapse). Similar in shape to mechanical migrations + the inscribed comment update.

---

## Mode classification

- **Mode A:** migration applied; let added to scope-boundary arm; three tests green; cargo build clean; SCORE written
- **Mode B (acceptable):** test fails because extended coverage now catches a previously-silent ProcessJoinBeforeOutputDrain pattern; REVERTED + inscribed
- **Mode C:** STOP rule broken

The substrate teaches; you sharpen; you migrate; nothing else.
