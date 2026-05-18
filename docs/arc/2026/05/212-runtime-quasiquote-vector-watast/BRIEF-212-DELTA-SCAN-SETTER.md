# Arc 212 stone δ-scan-setter — Migrate `scan_for_setter` to children()

**Your ONE concern this spawn:** migrate ONE walker from explicit `List + Vector` arms to `node.children()` generic recursion. Verify two named tests pass. Nothing else.

---

## The walker

**Function:** `scan_for_setter`
**File:** `/home/watmin/work/holon/wat-rs/src/load.rs`
**Line:** 755 (look for `fn scan_for_setter(form: &WatAST, path: &str) -> Result<(), LoadError> {`)

**Current shape (List + Vector arms duplicated; StructPattern missing):**

```rust
fn scan_for_setter(form: &WatAST, path: &str) -> Result<(), LoadError> {
    match form {
        WatAST::List(items, _) => {
            if let Some(WatAST::Keyword(k, _)) = items.first() {
                if k.starts_with(":wat::config::set-") && k.ends_with('!') {
                    return Err(LoadError::SetterInLoadedFile {
                        loaded_path: path.to_string(),
                        setter_head: k.clone(),
                    });
                }
            }
            for child in items {
                scan_for_setter(child, path)?;
            }
        }
        WatAST::Vector(items, _) => {
            for child in items {
                scan_for_setter(child, path)?;
            }
        }
        _ => {}
    }
    Ok(())
}
```

**Target shape (children() collapse; preserves List-head setter check):**

```rust
fn scan_for_setter(form: &WatAST, path: &str) -> Result<(), LoadError> {
    // Walker-specific List-head logic — fire SetterInLoadedFile on a
    // :wat::config::set-*! keyword head. Setter heads always appear in
    // List position; this guard preserves the pre-arc-212 check.
    if let WatAST::List(items, _) = form {
        if let Some(WatAST::Keyword(k, _)) = items.first() {
            if k.starts_with(":wat::config::set-") && k.ends_with('!') {
                return Err(LoadError::SetterInLoadedFile {
                    loaded_path: path.to_string(),
                    setter_head: k.clone(),
                });
            }
        }
    }
    // Arc 212 — generic recursion via children() covers List, Vector,
    // and StructPattern uniformly. Pre-arc-212 this walker had List +
    // Vector arms but no StructPattern arm — setters buried inside
    // StructPattern slipped past load-time refusal. children() returns
    // &[] for leaf nodes (no-op).
    for child in form.children() {
        scan_for_setter(child, path)?;
    }
    Ok(())
}
```

**The migration:**
1. Keep the List-head setter check in an `if let WatAST::List` guard
2. Replace the duplicated List + Vector recursion + `_ => {}` arms with a single `for child in form.children()` recursion outside the guard
3. Add the arc 212 comment block

**Do NOT change:**
- The setter-keyword predicate (`:wat::config::set-` prefix + `!` suffix)
- The `LoadError::SetterInLoadedFile` error variant
- The function signature or callers

---

## The wat-test proof gate

Two tests exercise this walker's `SetterInLoadedFile` diagnostic:

| Test | Verifies |
|---|---|
| `cargo test --release --lib setter_in_loaded_file_halts` | In-source unit test at src/load.rs:1465 — exercises SetterInLoadedFile directly |
| `cargo test --release --test probe_declaration_form_lift` | Integration test exercising broader load-time form discipline |

**Run both POST-migration. They MUST pass.**

---

## Verification protocol

1. Read `src/load.rs:755` to see current `scan_for_setter` body
2. Apply the migration (mechanical; see "Target shape" above)
3. Run `cargo build --release 2>&1 | tail -5` — must compile clean
4. Run both named tests:
   ```bash
   cargo test --release --lib setter_in_loaded_file_halts 2>&1 | tail -5
   cargo test --release --test probe_declaration_form_lift 2>&1 | tail -5
   ```
5. Write SCORE file at `docs/arc/2026/05/212-runtime-quasiquote-vector-watast/SCORE-212-DELTA-SCAN-SETTER.md`

---

## STOP triggers — VERBATIM

Non-negotiable. If any fires, STOP IMMEDIATELY. Do not investigate. Do not theorize. Do not open any file beyond this BRIEF's scope. Return what you have.

1. **One of the two named tests FAILS post-migration.** STOP. Revert your edit. Inscribe in SCORE that the migration broke `<test name>`. Do not investigate WHY. Do not theorize. Do not "fix" the test. Return.

   **Sub-rule:** if a test fails because the migration NOW catches a setter that was previously slipping past silently (inside a StructPattern), that is Mode B honest delta. Still STOP. Still revert. Still report. Do NOT modify the test.

2. **cargo build FAILS.** STOP. Inscribe the compile error. Do not fix beyond syntactic transcription error. Return.

3. **You see a failing test OUTSIDE the two named.** STOP. Workspace failure count is NOT your concern. Do not open the file. Return.

4. **You feel the urge to migrate another walker while you're here.** STOP. ONE walker per stone.

5. **You feel the urge to "improve" the setter-keyword predicate or the SetterInLoadedFile error.** STOP. Out of scope.

6. **Anything outside this concern surfaces.** STOP. Return what you have.

Honest STOP + clean report = Mode A's sibling, not failure.

---

## What the SCORE file contains

`docs/arc/2026/05/212-runtime-quasiquote-vector-watast/SCORE-212-DELTA-SCAN-SETTER.md`:

1. Header: `# Arc 212 stone δ-scan-setter — SCORE: migrate scan_for_setter to children()`
2. Summary: one paragraph naming what changed (recursion shape; StructPattern coverage extended) and what stayed (List-head setter check)
3. Verification: two lines, one per named test, showing pass/fail count
4. Build line: cargo build clean
5. Honest-delta note if Mode B (which test, what was caught)
6. Mode classification

---

## Constraints

- Edit ONLY `src/load.rs`
- Touch ONLY `scan_for_setter` within that file
- Zero other code edits anywhere
- Zero git operations (orchestrator commits)
- Zero test-file edits (Mode B reports the test name; orchestrator decides)
- Run ONLY the two named tests + cargo build
- No `cargo test --workspace`
- No exploring other walkers

---

## Time prediction

5-15 min Mode A. Mechanical 10-line edit; two short cargo invocations.

---

## Mode classification

- **Mode A:** migration applied; two named tests green; cargo build clean; SCORE written
- **Mode B (acceptable):** migration applied; one or two tests fail BECAUSE extended coverage catches previously-silent setter in StructPattern; REVERTED + inscribed honest delta
- **Mode C:** STOP rule broken (touched another walker, modified a test, "improved" predicate, investigated unrelated failures)

The substrate teaches; you listen; you migrate; nothing else.
