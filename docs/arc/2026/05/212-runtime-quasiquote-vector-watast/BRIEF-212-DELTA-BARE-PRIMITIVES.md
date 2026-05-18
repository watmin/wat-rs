# Arc 212 stone δ-bare-primitives — Migrate `walk_for_bare_primitives` to children()

**Your ONE concern this spawn:** migrate ONE walker from explicit `List + Vector` arms to `node.children()` generic recursion. Verify three named tests pass. Nothing else.

---

## The walker

**Function:** `walk_for_bare_primitives`
**File:** `/home/watmin/work/holon/wat-rs/src/check.rs`
**Line:** 2708 (look for `fn walk_for_bare_primitives(node: &WatAST, errors: &mut Vec<CheckError>) {`)

**Current shape (List + Vector arms duplicated):**

```rust
fn walk_for_bare_primitives(node: &WatAST, errors: &mut Vec<CheckError>) {
    match node {
        WatAST::Keyword(s, span) => {
            // ... keyword diagnostic checks (let*, lambda, unit, :fn(, types) ...
        }
        WatAST::List(items, _) => {
            for item in items {
                walk_for_bare_primitives(item, errors);
            }
        }
        WatAST::Vector(items, _) => {
            // Arc 167 slice 3 — recurse into Vector children.
            for item in items {
                walk_for_bare_primitives(item, errors);
            }
        }
        _ => {}
    }
}
```

**Target shape (children() collapse; preserves Keyword arm verbatim):**

```rust
fn walk_for_bare_primitives(node: &WatAST, errors: &mut Vec<CheckError>) {
    // Walker-specific Keyword-head logic — fire diagnostic for legacy
    // keywords; preserved verbatim from pre-arc-212 shape.
    if let WatAST::Keyword(s, span) = node {
        // ... existing keyword diagnostic checks COPIED VERBATIM ...
        // (the entire body of the WatAST::Keyword arm goes here)
        return;
    }
    // Arc 212 — generic recursion via children() covers List, Vector,
    // and StructPattern uniformly so legacy keywords buried inside ANY
    // bracketed shape are caught. children() returns &[] for leaf nodes
    // (no-op).
    for child in node.children() {
        walk_for_bare_primitives(child, errors);
    }
}
```

**The migration is mechanical:**
1. Replace `match node { ... }` with `if let WatAST::Keyword(s, span) = node { ... return; }` for the keyword arm
2. Copy the entire body of the existing `WatAST::Keyword(s, span) =>` arm into the new `if let` block (verbatim — every line, every condition, every `return`)
3. Add `return;` at the end of the `if let` block (Keyword has no children to recurse into)
4. Replace the entire List + Vector + `_ =>` arms with a single generic-recursion block using `node.children()`
5. Drop the old `match node` arms entirely

**Do NOT change:**
- The keyword diagnostic checks themselves (the `if s == ":wat::core::let*" { ... }` etc.)
- Any error variant names
- The type-expression check at the end of the Keyword arm (`crate::types::parse_type_expr_audit`)
- Function signature

---

## The wat-test proof gate

Three tests exercise this walker's diagnostics:

| Test | Verifies |
|---|---|
| `cargo test --release --test wat_arc154_kill_let_star` | `BareLegacyLetStar` diagnostic fires |
| `cargo test --release --test wat_arc153_nil_rename` | `BareLegacyUnitName` diagnostic fires |
| `cargo test --release --test wat_arc155_fn_rename` | `BareLegacyLambda` + `BareLegacyLowercaseFn` diagnostics fire |

**Run all three POST-migration. They MUST pass.**

---

## Verification protocol

1. Read `src/check.rs:2708` to see the current `walk_for_bare_primitives` body
2. Apply the migration (mechanical; see "Target shape" above)
3. Run `cargo build --release 2>&1 | tail -5` — must compile clean
4. Run all three named tests in sequence:
   ```bash
   cargo test --release --test wat_arc154_kill_let_star 2>&1 | tail -5
   cargo test --release --test wat_arc153_nil_rename 2>&1 | tail -5
   cargo test --release --test wat_arc155_fn_rename 2>&1 | tail -5
   ```
5. Write SCORE file at `docs/arc/2026/05/212-runtime-quasiquote-vector-watast/SCORE-212-DELTA-BARE-PRIMITIVES.md`

---

## STOP triggers — VERBATIM

Non-negotiable. If any fires, STOP IMMEDIATELY. Do not investigate. Do not theorize. Do not open any file beyond this BRIEF's scope. Return what you have.

1. **One of the three named tests FAILS post-migration.** STOP. Revert your edit. Inscribe in SCORE that the migration broke `<test name>`. Do not investigate WHY. Do not theorize about the failure. Return — the orchestrator handles the surface.

2. **cargo build FAILS.** STOP. Inscribe the compile error in SCORE. Do not fix anything beyond the most obvious syntactic transcription error (e.g., missing `;`). Return.

3. **You see a failing test OUTSIDE the three named.** STOP. Workspace failure count is NOT your concern. Do not open the file. Do not investigate. Return.

4. **You feel the urge to migrate another walker while you're here.** STOP. ONE walker per stone. Other walkers are other stones. Do not scope-creep.

5. **You feel the urge to "improve" the keyword diagnostic logic.** STOP. The Keyword arm body is preserved VERBATIM. No improvements. No simplifications. No "while I'm here" tweaks. The migration is the recursion shape; nothing else.

6. **Anything outside this concern surfaces.** STOP. Return what you have.

If you hit a STOP trigger, that IS valuable information. Honest STOP + clean report = Mode A's sibling, not failure.

---

## What the SCORE file contains

`docs/arc/2026/05/212-runtime-quasiquote-vector-watast/SCORE-212-DELTA-BARE-PRIMITIVES.md`:

1. Header: `# Arc 212 stone δ-bare-primitives — SCORE: migrate walk_for_bare_primitives to children()`
2. Summary: one paragraph naming what changed (the recursion shape) and what stayed (the Keyword arm body)
3. Verification: three lines, one per named test, showing pass/fail count from `cargo test --release --test <name> 2>&1 | tail -5`
4. Build line: cargo build clean (or compile error inscribed)
5. Mode classification

No additional commentary. No "workspace failure analysis." No "I noticed other things." Just the stone's result.

---

## Constraints

- Edit ONLY `src/check.rs`
- Touch ONLY `walk_for_bare_primitives` within that file
- Zero other code edits anywhere
- Zero git operations (orchestrator commits)
- Run ONLY the three named tests + cargo build
- No `cargo test --workspace` (out of scope)
- No exploring other walkers
- No reading test files (you don't need to — they exercise the diagnostics by emitting source with legacy keywords)

---

## Time prediction

5-15 min Mode A. The migration is mechanical (~10 line edit). Verification is three short cargo invocations.

---

## Mode classification

- **Mode A:** migration applied; three named tests green; cargo build clean; SCORE written
- **Mode B:** migration applied; one or more named tests fail; you REVERTED + inscribed which test broke; SCORE captures the failure honestly
- **Mode C:** you broke a STOP rule (touched another walker, "improved" the keyword logic, investigated unrelated failures)

The substrate teaches; you listen; you migrate; nothing else.
