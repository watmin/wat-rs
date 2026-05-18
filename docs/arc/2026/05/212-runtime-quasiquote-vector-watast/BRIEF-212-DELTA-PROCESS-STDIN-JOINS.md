# Arc 212 stone δ-process-stdin-joins — Migrate `collect_process_stdin_and_joins` to children()

**Your ONE concern this spawn:** migrate ONE walker from explicit `List + Vector` arms to `node.children()` generic recursion. Verify one named test passes. Nothing else.

---

## The walker

**Function:** `collect_process_stdin_and_joins`
**File:** `/home/watmin/work/holon/wat-rs/src/check.rs`
**Line:** 3689 (look for `fn collect_process_stdin_and_joins(node: &WatAST, joins: &mut Vec<(String, Span)>, stdin_procs: &mut Vec<String>) {` — exact signature varies slightly; locate by name)

**Current shape (List + Vector arms duplicated; CRITICAL fn/lambda early-return):**

```rust
fn collect_process_stdin_and_joins(node: &WatAST, joins: ..., stdin_procs: ...) {
    match node {
        WatAST::List(items, span) => {
            if let Some(WatAST::Keyword(k, _)) = items.first() {
                match k.as_str() {
                    ":wat::kernel::Process/join-result" => { ... pushes to joins ... }
                    ":wat::kernel::Process/stdin" => { ... pushes to stdin_procs ... }
                    ":wat::core::fn" | ":wat::core::lambda" => return,  // ← LOAD-BEARING
                    _ => {}
                }
            }
            for child in items {
                collect_process_stdin_and_joins(child, joins, stdin_procs);
            }
        }
        WatAST::Vector(items, _) => {
            // Binding vectors inside inner `let` forms ...
            for child in items {
                collect_process_stdin_and_joins(child, joins, stdin_procs);
            }
        }
        _ => {}
    }
}
```

**Target shape (children() collapse; preserves keyword detection + fn/lambda boundary):**

```rust
fn collect_process_stdin_and_joins(node: &WatAST, joins: ..., stdin_procs: ...) {
    // Walker-specific List-head logic: classify Process/join-result and
    // Process/stdin call sites; STOP descent at fn/lambda boundaries
    // (separate scopes — descending would conflate inner-fn calls with
    // outer scope tracking). The early-return is load-bearing.
    if let WatAST::List(items, span) = node {
        if let Some(WatAST::Keyword(k, _)) = items.first() {
            match k.as_str() {
                ":wat::kernel::Process/join-result" => {
                    // ... existing push logic; do NOT return after — recurse into args ...
                }
                ":wat::kernel::Process/stdin" => {
                    // ... existing push logic; do NOT return after — recurse into args ...
                }
                ":wat::core::fn" | ":wat::core::lambda" => return,  // ← PRESERVE EARLY-RETURN
                _ => {}
            }
        }
    }
    // Arc 212 — generic recursion via children() covers List, Vector, and
    // StructPattern uniformly. children() returns &[] for leaf nodes (no-op).
    // The fn/lambda early-return above ensures we never descend into nested
    // fn bodies (separate scopes).
    for child in node.children() {
        collect_process_stdin_and_joins(child, joins, stdin_procs);
    }
}
```

**The migration:**
1. Replace outer `match node { ... }` with `if let WatAST::List(items, span) = node`
2. Preserve the inner `match k.as_str()` exactly — INCLUDING the `:wat::core::fn | :wat::core::lambda => return` arm
3. Move the recursion OUT of the `if let` block; route through `node.children()`
4. Add the arc 212 comment block

**LOAD-BEARING:**
- The `:wat::core::fn | :wat::core::lambda => return` early-return MUST stay. Without it, the walker descends into nested fn bodies and produces false positives.
- The Process/join-result + Process/stdin arms push but DO NOT return — they fall through to the recursion so the walker descends into the call's args.

**Do NOT change:**
- The classification logic (which keywords trigger which push)
- The `joins` / `stdin_procs` Vec mutation
- The function signature

---

## The wat-test proof gate

ONE test exercises this walker's path:

| Test | Verifies |
|---|---|
| `cargo test --release --test wat_arc202_process_join_holds_stdin` | ProcessJoinHoldsStdinSender diagnostic; arc 202 walker |

**Run POST-migration. MUST pass.**

---

## Verification protocol

1. Read `src/check.rs:3689` to see current `collect_process_stdin_and_joins` body
2. Apply the migration (see "Target shape" above)
3. Run `cargo build --release 2>&1 | tail -5` — must compile clean
4. Run the named test:
   ```bash
   cargo test --release --test wat_arc202_process_join_holds_stdin 2>&1 | tail -5
   ```
5. Write SCORE file at `docs/arc/2026/05/212-runtime-quasiquote-vector-watast/SCORE-212-DELTA-PROCESS-STDIN-JOINS.md`

---

## STOP triggers — VERBATIM

Non-negotiable. If any fires, STOP IMMEDIATELY. Do not investigate. Do not theorize. Do not open any file beyond this BRIEF's scope. Return what you have.

1. **The named test FAILS post-migration.** STOP. Revert your edit. Inscribe in SCORE that the migration broke the test. Do not investigate WHY. Do not theorize. Do not "fix" the test. Return.

   **Most likely failure cause if it happens:** you dropped the `:wat::core::fn | :wat::core::lambda => return` early-return. The test will report false positives on patterns that have a `Process/stdin` or `Process/join-result` inside a nested fn body.

2. **cargo build FAILS.** STOP. Inscribe the compile error. Return.

3. **You see a failing test OUTSIDE the named one.** STOP. Workspace failure count is NOT your concern. Do not open the file.

4. **You feel the urge to migrate another walker while you're here.** STOP. ONE walker per stone.

5. **You feel the urge to "improve" the classification logic or "simplify" the early-return.** STOP. The fn/lambda early-return is load-bearing. Preserve verbatim.

6. **Anything outside this concern surfaces.** STOP. Return what you have.

---

## What the SCORE file contains

`docs/arc/2026/05/212-runtime-quasiquote-vector-watast/SCORE-212-DELTA-PROCESS-STDIN-JOINS.md`:

1. Header
2. Summary: recursion shape changed; fn/lambda early-return preserved verbatim; StructPattern coverage extended
3. Verification: one line showing test result
4. Build line: cargo build clean
5. Mode classification

---

## Constraints

- Edit ONLY `src/check.rs`
- Touch ONLY `collect_process_stdin_and_joins` within that file
- Zero other code edits anywhere
- Zero git operations (orchestrator commits)
- Zero test-file edits
- Run ONLY the one named test + cargo build
- No `cargo test --workspace`

---

## Time prediction

5-15 min Mode A.

---

## Mode classification

- **Mode A:** migration applied; named test green; cargo build clean; SCORE written
- **Mode B (acceptable):** test fails (most likely due to early-return drop); REVERTED + inscribed
- **Mode C:** STOP rule broken

The substrate teaches; you listen; you migrate; nothing else.
