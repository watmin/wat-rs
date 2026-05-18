# Arc 212 stone δ-def-restricted — Migrate `walk_for_def_restricted_call` to children()

**Your ONE concern this spawn:** migrate ONE walker from explicit `List + Vector` arms to `node.children()` generic recursion. Verify one named test passes. Nothing else.

---

## The walker

**Function:** `walk_for_def_restricted_call`
**File:** `/home/watmin/work/holon/wat-rs/src/check.rs`
**Line:** ~3208 (look for `fn walk_for_def_restricted_call(`)

**Current shape (List + Vector arms duplicated):**

```rust
fn walk_for_def_restricted_call(
    node: &WatAST,
    enclosing_fn: &str,
    env: &SymbolTable,
    errors: &mut Vec<CheckError>,
) {
    match node {
        WatAST::List(items, _) => {
            if let Some(WatAST::Keyword(head, head_span)) = items.first() {
                if let Some(prefixes) = env.get_defined_value_restriction(head) {
                    if !caller_matches_prefix_list(enclosing_fn, prefixes) {
                        errors.push(CheckError::DefRestrictedCallerNotAllowed {
                            callee: head.clone(),
                            enclosing_fn: enclosing_fn.into(),
                            prefixes: prefixes.clone(),
                            span: head_span.clone(),
                        });
                    }
                }
            }
            for item in items {
                walk_for_def_restricted_call(item, enclosing_fn, env, errors);
            }
        }
        WatAST::Vector(items, _) => {
            for item in items {
                walk_for_def_restricted_call(item, enclosing_fn, env, errors);
            }
        }
        _ => {}
    }
}
```

**Target shape (children() collapse; preserves List-head restriction check):**

```rust
fn walk_for_def_restricted_call(
    node: &WatAST,
    enclosing_fn: &str,
    env: &SymbolTable,
    errors: &mut Vec<CheckError>,
) {
    // Walker-specific List-head logic — fire DefRestrictedCallerNotAllowed
    // when a call head names a def-restricted binding whose whitelist
    // excludes the enclosing fn FQDN. Call heads always appear in List
    // position; this guard preserves the pre-arc-212 check.
    if let WatAST::List(items, _) = node {
        if let Some(WatAST::Keyword(head, head_span)) = items.first() {
            if let Some(prefixes) = env.get_defined_value_restriction(head) {
                if !caller_matches_prefix_list(enclosing_fn, prefixes) {
                    errors.push(CheckError::DefRestrictedCallerNotAllowed {
                        callee: head.clone(),
                        enclosing_fn: enclosing_fn.into(),
                        prefixes: prefixes.clone(),
                        span: head_span.clone(),
                    });
                }
            }
        }
    }
    // Arc 212 — generic recursion via children() covers List, Vector,
    // and StructPattern uniformly. Pre-arc-212 this walker had explicit
    // List + Vector arms but no StructPattern — call sites buried inside
    // StructPattern slipped past restriction enforcement. children()
    // returns &[] for leaf nodes (no-op).
    for child in node.children() {
        walk_for_def_restricted_call(child, enclosing_fn, env, errors);
    }
}
```

**The migration:**
1. Keep the List-head restriction check in an `if let WatAST::List` guard
2. Replace the duplicated List + Vector recursion + `_ => {}` arms with a single `for child in node.children()` recursion outside the guard
3. Add the arc 212 comment block

**Do NOT change:**
- The `env.get_defined_value_restriction(head)` lookup
- The `caller_matches_prefix_list(enclosing_fn, prefixes)` predicate
- The `DefRestrictedCallerNotAllowed` error construction
- The function signature

---

## The wat-test proof gate

ONE canonical test exercises this walker's `DefRestrictedCallerNotAllowed` diagnostic:

| Test | Verifies |
|---|---|
| `cargo test --release --test wat_arc198_def_restricted` | Arc 198's `def-restricted` substrate primitive — the diagnostic this walker fires |

**Run POST-migration. MUST pass.**

---

## Verification protocol

1. Read `src/check.rs:3208` to see current `walk_for_def_restricted_call` body
2. Apply the migration
3. Run `cargo build --release 2>&1 | tail -5` — must compile clean
4. Run the named test:
   ```bash
   cargo test --release --test wat_arc198_def_restricted 2>&1 | tail -5
   ```
5. Write SCORE file at `docs/arc/2026/05/212-runtime-quasiquote-vector-watast/SCORE-212-DELTA-DEF-RESTRICTED.md`

---

## STOP triggers — VERBATIM

Non-negotiable. If any fires, STOP IMMEDIATELY. Do not investigate. Do not theorize. Do not open any file beyond this BRIEF's scope. Return what you have.

1. **The named test FAILS post-migration.** STOP. Revert your edit. Inscribe in SCORE that the migration broke the test. Do not investigate WHY. Do not theorize. Do not "fix" the test. Return.

   **Sub-rule:** if the test fails because the migration NOW catches a restriction violation that was previously slipping past silently (inside a StructPattern), that is Mode B honest delta. Still STOP. Still revert. Still report. Do NOT modify the test.

2. **cargo build FAILS.** STOP. Inscribe the compile error. Do not fix beyond syntactic transcription error. Return.

3. **You see a failing test OUTSIDE the named one.** STOP. Workspace failure count is NOT your concern. Do not open the file. Return.

4. **You feel the urge to migrate another walker while you're here.** STOP. ONE walker per stone.

5. **You feel the urge to "improve" the restriction-matching predicate, the error variant, or the env lookup.** STOP. Out of scope.

6. **Anything outside this concern surfaces.** STOP. Return what you have.

Honest STOP + clean report = Mode A's sibling, not failure.

---

## What the SCORE file contains

`docs/arc/2026/05/212-runtime-quasiquote-vector-watast/SCORE-212-DELTA-DEF-RESTRICTED.md`:

1. Header
2. Summary: recursion shape changed; List-head restriction check preserved; StructPattern coverage extended
3. Verification: one line showing test result
4. Build line: cargo build clean
5. Honest-delta note if Mode B
6. Mode classification

---

## Constraints

- Edit ONLY `src/check.rs`
- Touch ONLY `walk_for_def_restricted_call` within that file
- Zero other code edits anywhere
- Zero git operations (orchestrator commits)
- Zero test-file edits (Mode B reports the test; orchestrator decides)
- Run ONLY the one named test + cargo build
- No `cargo test --workspace`

---

## Time prediction

5-15 min Mode A. Mechanical 10-line edit.

---

## Mode classification

- **Mode A:** migration applied; named test green; cargo build clean; SCORE written
- **Mode B (acceptable):** test fails because extended StructPattern coverage catches a previously-silent violation; REVERTED + inscribed
- **Mode C:** STOP rule broken

The substrate teaches; you listen; you migrate; nothing else.
