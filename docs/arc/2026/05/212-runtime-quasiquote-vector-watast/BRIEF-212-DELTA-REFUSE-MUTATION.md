# Arc 212 stone δ-refuse-mutation — Migrate `refuse_mutation_forms` to children()

**Your ONE concern this spawn:** migrate ONE walker from List-only recursion to `node.children()` generic recursion. Verify two named tests pass. Nothing else.

---

## The walker

**Function:** `refuse_mutation_forms`
**File:** `/home/watmin/work/holon/wat-rs/src/freeze.rs`
**Line:** 1334 (look for `fn refuse_mutation_forms(ast: &WatAST) -> Result<(), RuntimeError> {`)

**Current shape (List-only — NO Vector arm at all):**

```rust
fn refuse_mutation_forms(ast: &WatAST) -> Result<(), RuntimeError> {
    if let WatAST::List(items, list_span) = ast {
        if let Some(WatAST::Keyword(head, _)) = items.first() {
            if is_mutation_form(head) {
                return Err(RuntimeError::EvalForbidsMutationForm {
                    head: head.clone(),
                    span: list_span.clone(),
                });
            }
        }
        for child in items {
            refuse_mutation_forms(child)?;
        }
    }
    Ok(())
}
```

**Target shape (children() recursion; preserves List-head mutation detection):**

```rust
fn refuse_mutation_forms(ast: &WatAST) -> Result<(), RuntimeError> {
    // Walker-specific List-head logic — fire EvalForbidsMutationForm on
    // mutation-form Keyword heads. Mutation form heads always appear
    // in List position; this guard preserves the pre-arc-212 check.
    if let WatAST::List(items, list_span) = ast {
        if let Some(WatAST::Keyword(head, _)) = items.first() {
            if is_mutation_form(head) {
                return Err(RuntimeError::EvalForbidsMutationForm {
                    head: head.clone(),
                    span: list_span.clone(),
                });
            }
        }
    }
    // Arc 212 — generic recursion via children() covers List, Vector,
    // and StructPattern uniformly. Pre-arc-212 this walker silently
    // accepted mutation forms buried inside Vector (let-binding-vector
    // RHSes) and StructPattern bracketed shapes — they slipped past
    // freeze-time refusal. children() returns &[] for leaf nodes (no-op).
    for child in ast.children() {
        refuse_mutation_forms(child)?;
    }
    Ok(())
}
```

**The migration:**
1. Keep the `if let WatAST::List` guard for the keyword-head mutation check
2. Move the `for child in items { refuse_mutation_forms(child)?; }` OUT of the `if let` block
3. Replace `items` with `ast.children()` so recursion covers Vector + StructPattern too
4. Add the arc 212 comment block

**Do NOT change:**
- The `is_mutation_form` predicate (out of scope)
- The `EvalForbidsMutationForm` error variant
- The function signature or its caller sites
- Any test files

---

## The wat-test proof gate

Two tests exercise this walker's `EvalForbidsMutationForm` diagnostic:

| Test | Verifies |
|---|---|
| `cargo test --release --test probe_declaration_form_lift` | Declaration-form lift discipline; exercises eval-ast! → refuse_mutation_forms |
| `cargo test --release --test wat_eval_result` | eval-ast! Result type / behavior; exercises the same path |

**Run both POST-migration. They MUST pass.**

---

## Verification protocol

1. Read `src/freeze.rs:1334` to see the current `refuse_mutation_forms` body
2. Apply the migration (mechanical; see "Target shape" above)
3. Run `cargo build --release 2>&1 | tail -5` — must compile clean
4. Run both named tests:
   ```bash
   cargo test --release --test probe_declaration_form_lift 2>&1 | tail -5
   cargo test --release --test wat_eval_result 2>&1 | tail -5
   ```
5. Write SCORE file at `docs/arc/2026/05/212-runtime-quasiquote-vector-watast/SCORE-212-DELTA-REFUSE-MUTATION.md`

---

## STOP triggers — VERBATIM

Non-negotiable. If any fires, STOP IMMEDIATELY. Do not investigate. Do not theorize. Do not open any file beyond this BRIEF's scope. Return what you have.

1. **One of the two named tests FAILS post-migration.** STOP. Revert your edit. Inscribe in SCORE that the migration broke `<test name>`. Do not investigate WHY. Do not theorize about the failure. Do not "fix" the test. Return — the orchestrator handles the surface.

   **Important sub-rule:** if a test fails because the migration NOW catches a mutation form that was previously slipping past silently (inside a Vector RHS), that is the SUBSTRATE TEACHING us about a latent bug. Still STOP. Still revert. Still report. Do NOT modify the test. The orchestrator will read the SCORE and decide whether the test or the walker needs adjustment.

2. **cargo build FAILS.** STOP. Inscribe the compile error in SCORE. Do not fix anything beyond the most obvious syntactic transcription error (e.g., missing `;`). Return.

3. **You see a failing test OUTSIDE the two named.** STOP. Workspace failure count is NOT your concern. Do not open the file. Do not investigate. Return.

4. **You feel the urge to migrate another walker while you're here.** STOP. ONE walker per stone. Other walkers are other stones. Do not scope-creep.

5. **You feel the urge to "improve" the is_mutation_form predicate or the EvalForbidsMutationForm error variant.** STOP. Those are out of scope. The migration is the recursion shape; nothing else.

6. **Anything outside this concern surfaces.** STOP. Return what you have.

If you hit a STOP trigger, that IS valuable information. Honest STOP + clean report = Mode A's sibling, not failure.

---

## What the SCORE file contains

`docs/arc/2026/05/212-runtime-quasiquote-vector-watast/SCORE-212-DELTA-REFUSE-MUTATION.md`:

1. Header: `# Arc 212 stone δ-refuse-mutation — SCORE: migrate refuse_mutation_forms to children()`
2. Summary: one paragraph naming what changed (the recursion shape; coverage EXTENDED to Vector + StructPattern) and what stayed (the List-head mutation check)
3. Verification: two lines, one per named test, showing pass/fail count from `cargo test --release --test <name> 2>&1 | tail -5`
4. Build line: cargo build clean (or compile error inscribed)
5. **Honest-delta note:** if either test broke because the extended coverage now catches a previously-silent mutation form, name it explicitly (e.g., "wat_eval_result::test_X now fails because line N has `(:wat::core::struct ...)` inside a let-binding Vector — previously slipped past silently"). Orchestrator decision required.
6. Mode classification

No additional commentary. No "workspace failure analysis." Just the stone's result.

---

## Constraints

- Edit ONLY `src/freeze.rs`
- Touch ONLY `refuse_mutation_forms` within that file
- Zero other code edits anywhere
- Zero git operations (orchestrator commits)
- Zero test-file edits
- Run ONLY the two named tests + cargo build
- No `cargo test --workspace` (out of scope)
- No exploring other walkers

---

## Time prediction

5-15 min Mode A. Migration is ~10 line edit. Two cargo invocations.

---

## Mode classification

- **Mode A:** migration applied; two named tests green; cargo build clean; SCORE written
- **Mode B (acceptable):** migration applied; one or two named tests fail BECAUSE the extended coverage caught a previously-silent mutation form; you REVERTED + inscribed the honest delta in SCORE
- **Mode C:** you broke a STOP rule (touched another walker, modified a test, "improved" is_mutation_form, investigated unrelated failures)

The substrate teaches; you listen; you migrate; nothing else.
