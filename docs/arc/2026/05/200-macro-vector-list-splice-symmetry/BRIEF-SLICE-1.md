# Arc 200 Slice 1 BRIEF — relax macro splice for `WatAST::Vector`

**Phase:** Single slice closing arc 200 (macro-layer List/Vector splice symmetry).

**Originating signal:** Arc 170 Stone D2 STOPPED on substrate-gap probe (`tests/probe_stone_d2_splice_vector.rs`, 2 passing probes documenting Gap 1 + Gap 2). See `SCORE-STONE-D2.md` for full diagnosis and `docs/arc/2026/05/200-macro-vector-list-splice-symmetry/DESIGN.md` for arc scope.

## Goal

Make `~@` (unquote-splicing) treat `WatAST::Vector` and `WatAST::List` interchangeably in template-splice contexts. Two minimal relaxations to `src/macros.rs`:

1. **Gap 1** — `splice_argument` accepts `WatAST::Vector` (currently rejects with `SpliceNotList`)
2. **Gap 2** — `walk_template`'s Vector branch dispatches unquote-splicing on List children (mirror of List branch's existing dispatch)

Both are existing-primitive relaxations — NOT new substrate. `feedback_no_new_types` holds.

## Required path (NO alternatives)

### Gap 1 — `splice_argument` accepts `WatAST::Vector`

Current code at `src/macros.rs:1080-1086` (or thereabouts; sonnet verifies line numbers):

```rust
match bound {
    WatAST::List(items, _) => Ok(items.clone()),
    other => Err(MacroError::SpliceNotList { ... }),
}
```

After fix:

```rust
match bound {
    WatAST::List(items, _) => Ok(items.clone()),
    WatAST::Vector(items, _) => Ok(items.clone()),   // ← new arm
    other => Err(MacroError::SpliceNotList { ... }),
}
```

The `MacroError::SpliceNotList` name stays (no rename — out of scope; can be addressed later if desired). The error message stays unchanged (still applies to the `other` case).

### Gap 2 — `walk_template` Vector branch dispatches unquote-splicing

Current code at `src/macros.rs:926-941` (sonnet verifies):

```rust
WatAST::Vector(items, _) => {
    let mut out = Vec::with_capacity(items.len());
    for child in items {
        out.push(walk_template(child, ...)?);
    }
    Ok(WatAST::Vector(out, call_site_span.clone()))
}
```

After fix (mirror the List branch's splice-dispatch at `:860-898`):

```rust
WatAST::Vector(items, _) => {
    let mut out = Vec::with_capacity(items.len());
    for child in items {
        if let WatAST::List(child_items, _) = child {
            if let Some(splice_arg) = match_unquote(child_items, ":wat::core::unquote-splicing") {
                if depth == 1 {
                    let spliced = splice_argument(splice_arg, bindings, macro_name, env, sym)?;
                    out.extend(spliced);
                    continue;
                } else {
                    // Preserve + peel (depth > 1)
                    let inner = walk_template(splice_arg, bindings, macro_scope, macro_name, call_site_span, depth - 1, env, sym)?;
                    out.push(WatAST::List(
                        vec![
                            WatAST::Keyword(":wat::core::unquote-splicing".into(), call_site_span.clone()),
                            inner,
                        ],
                        call_site_span.clone(),
                    ));
                    continue;
                }
            }
        }
        out.push(walk_template(child, bindings, macro_scope, macro_name, call_site_span, depth, env, sym)?);
    }
    Ok(WatAST::Vector(out, call_site_span.clone()))
}
```

Mirror is exact. The only difference from the List branch is the outer `WatAST::Vector` reconstruction.

### Optional: factor the splice-dispatch into a helper

If sonnet sees the duplication between List and Vector branches as worth refactoring, factor the inner splice-handling loop into a helper:

```rust
fn walk_children_with_splice(items: &[WatAST], ...) -> Result<Vec<WatAST>, MacroError> {
    // shared loop body — handles both List and Vector branches
}
```

Then both branches call it and wrap the result in their respective constructor. **Optional, not required.** If the refactor adds clarity, do it; if it muddies the diff, skip.

## Tasks

### 1. Flip D2 probes from expected-failure to expected-success

`tests/probe_stone_d2_splice_vector.rs` currently passes by asserting the failure modes. After arc 200 lands, the tests must flip:

- The two probes that asserted error messages now assert the EXPECTED OUTPUT (vector-splice works; bracket-template + `~@` expands cleanly).
- Rename the file from `probe_stone_d2_splice_vector.rs` to `wat_macro_vector_splice_symmetry.rs` (concept-anchored; future agents searching for "vector splice" find it).
- Header comment updated: this is the regression test for arc 200's relaxations; references Gap 1 + Gap 2.

If renaming the file is risky (path dependencies), keep the existing name and update the test bodies. Sonnet picks based on test-discovery hygiene.

### 2. Add a positive test demonstrating the brackets+splice form expands as expected

Mirror of D2's mandated `[[:I :O f] ...]` call shape, but at the splice-mechanics level (not full run-threads). Example:

```scheme
(:wat::core::defmacro
  (:probe::vec-splice
    (& (items :AST<wat::core::Vector<wat::WatAST>>))
    -> :AST<wat::core::nil>)
  `[~@items])
```

Test: `(:probe::vec-splice 1 2 3)` → expansion = `[1 2 3]` (Vector of three IntLits). Verify the expansion structure via macroexpand or direct test inspection.

This positive test seals the YES direction; combined with the D2 probes (now passing as expected-success), the regression coverage is complete.

### 3. Build + test

```bash
cd /home/watmin/work/holon/wat-rs
cargo build --release --workspace --tests
cargo test --release -p wat --test wat_macro_vector_splice_symmetry  # (or current name)
cargo test --release -p wat --test wat_run_threads_d1  # ensure D1 still passes
cargo test --release --workspace --no-fail-fast
```

Workspace baseline: failures ≤ 4 (3 stable + lifeline flake variance — per commit `64cc793` baseline).

### 4. Slice closure paperwork

- Write `docs/arc/2026/05/200-macro-vector-list-splice-symmetry/SCORE-SLICE-1.md` — 5 rows YES/NO + evidence.
- Write `docs/arc/2026/05/200-macro-vector-list-splice-symmetry/INSCRIPTION.md` — short closure inscription documenting the two relaxations + the lesson that consumer pressure (D2) surfaced the asymmetry.

## STOP triggers (true emergencies — surface, do not paper over)

1. **Mirror of List branch doesn't compile cleanly** — surface the exact error; do NOT add new types or refactor unrelated machinery. The mirror should be near-mechanical; if it isn't, there's something the BRIEF didn't account for.
2. **D2 probes don't flip cleanly** — investigate; if the relaxation works at the unit level but the probe macros don't expand as expected, surface what you observed.
3. **Workspace baseline regresses** — STOP, surface the new failure.
4. **Any urge to mint a new substrate type, verb, struct, or special form** — STOP. This is pure existing-primitive relaxation.

## HARD constraints

- DO NOT commit. Orchestrator commits atomically after independent verification.
- **cwd discipline:** FIRST action: `cd /home/watmin/work/holon/wat-rs/`; verify with `pwd`. Harness may launch into `.claude/worktrees/agent-<id>/` — ignore it; operate on the real repo per `docs/COMPACTION-AMNESIA-RECOVERY.md` § 7-bis.
- DO NOT mint any new substrate verb, type, struct, or special form (`feedback_no_new_types`).
- DO NOT touch the runtime layer (`src/runtime.rs`) — Gap 3 (vectors-at-value-position) is out of scope.
- DO NOT touch arc 117/133 sibling-binding walker.
- DO NOT touch INSCRIPTIONs / past SCOREs / DEFERRAL-VIOLATIONS / SUPERSEDED BRIEFs / AUDIT / recovery doc / past STONE BRIEFs/EXPECTATIONS/SCOREs.
- DO NOT modify INTERSTITIAL-REALIZATIONS.md (orchestrator owns).
- DO NOT modify the arc 199 DESIGN.md or arc 170 D1/D2 BRIEFs/SCOREs (historical artifacts).
- DO NOT use `--no-verify` / `--no-gpg-sign` / skip hooks.

## SCORE methodology

5 rows YES/NO + evidence:

| Row | What | Evidence |
|-----|------|----------|
| A | Gap 1 fixed — `splice_argument` accepts `WatAST::Vector` | grep + unit test demonstrates the Vector arm fires |
| B | Gap 2 fixed — `walk_template` Vector branch dispatches unquote-splicing | grep + positive test demonstrates `[~@xs]` expands inside vector template |
| C | D2 probes flipped from expected-failure to expected-success | renamed file (or updated bodies) passes the regression |
| D | D1 still passes; positive vector-splice test passes | `cargo test ...` evidence |
| E | Workspace test failure count ≤ baseline (4) | full workspace cargo test failures ≤ baseline + flake variance |

## Honest deltas to capture in SCORE

- Did the mirror approach compile cleanly, or did you spot a worthwhile refactor (helper fn) in the process?
- Did the D2 probe rename surface any wat-discovery path issues?
- Did the positive test surface any unexpected macro-expansion behavior?
- Workspace baseline preserved exactly?

## Time-box

30-60 min predicted (small, focused, near-mechanical). Hard stop 90 min.

## Workspace baseline (commit `64cc793`)

- `cargo build --release --workspace --tests`: clean
- `cargo test --release --workspace --no-fail-fast`: 4 stable failures (lifeline flake + 3 pre-existing)

Post-arc-200 target:
- D1 test stays passing (1/1)
- D2 probes flip cleanly (2/2 still passing, but now as expected-success regressions)
- Positive vector-splice test passes (1/1 new)
- Workspace failures ≤ baseline

## On completion

1. Write SCORE-SLICE-1.md + INSCRIPTION.md per § Tasks 4.
2. Return final summary to orchestrator: rows passed/failed + workspace delta + path to SCORE + any surprises observed.

You are launching now. T-minus 0.
