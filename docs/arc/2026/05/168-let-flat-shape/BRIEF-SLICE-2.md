# Arc 168 slice 2 — sweep all let callsites (substrate-as-teacher)

## Goal

Migrate every legacy nested-pair-list let-binding callsite to the new
flat-shape Vector form. After this slice, `cargo test --release
--workspace --no-fail-fast` is green at the new shape; legacy outer-List
sites in user code fire `BareLegacyLetBindings` walker → translated to
the canonical form; stdlib sites in `wat/` swept via grep + mechanical
translation.

## Branch + commit policy

- **Active branch**: `arc-168-let-flat-shape` (slices 1, 2, 3, 4 share
  this branch per atomic-merge discipline)
- WIP commits welcome; commit liberally so interrupts don't lose
  progress. `git add <files> && git commit -m "..."` between each
  region of the sweep is encouraged.
- DO NOT push to main; orchestrator merges atomic to main as one
  squash commit after slice 5 closure paperwork ships
- Verify green via the inline pipeline below — DO NOT use scripts/

## Read first (in order)

1. `docs/arc/2026/05/168-let-flat-shape/DESIGN.md` — full arc scope
2. `docs/arc/2026/05/168-let-flat-shape/BRIEF-SLICE-1.md` — what slice 1
   shipped (substrate consumer + walker)
3. `docs/arc/2026/05/167-fn-flat-signature/SCORE-SLICE-3.md` — the
   precedent slice this one mirrors (the substrate-as-teacher sweep
   plus a worked-FM-5 detour you should NOT repeat)
4. `docs/COMPACTION-AMNESIA-RECOVERY.md` § 6 (FM 5, FM 15, FM 16) —
   discipline floor

## The migration recipe

### Outer shape change

Legacy outer-List bindings become flat-shape Vector:

```scheme
;; BEFORE
(:wat::core::let
  ((name1 expr1)
   (name2 expr2))
  body)

;; AFTER
(:wat::core::let
  [name1 expr1
   name2 expr2]
  body)
```

### Three binder shapes inside

**Bare symbol (canonical, post-arc-159)** — name is a Symbol:

```scheme
;; BEFORE: ((name expr) (name expr))
;; AFTER:  [name expr name expr]
```

**Typed legacy (parser still accepts; post-arc-168 walker fires)** —
binder is a List of (Symbol Keyword):

```scheme
;; BEFORE: (((name :T) expr))
;; AFTER:  [name expr]
;; (Type annotation drops; type is now inferred from expr per arc 159
;; user-side retirement.)
```

**Destructure** — binder is a list of bare symbols:

```scheme
;; BEFORE: (((a b c) rhs))
;; AFTER:  [[a b c] rhs]
;; (Inner List of symbols becomes Vector of symbols; binder sits
;; inside the outer binding Vector.)
```

### Body shape

Body is unchanged for slice 2 — single body forms stay single. Multi-form
implicit-do is purely additive (single body remains valid). Don't
reshape bodies as part of this sweep.

## How to drive the sweep (substrate-as-teacher)

Per `docs/SUBSTRATE-AS-TEACHER.md` and arc 167 slice 3 precedent.

### Loop

1. **Measure** — see the workspace failure count:

   ```
   cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk '{p+=$4; f+=$6} END {print "passed:", p, "failed:", f}'
   ```

   Expected starting baseline: ~560 failures (the slice 1 SCORE
   measured this).

2. **Read** — see which tests are failing and why:

   ```
   cargo test --release --workspace --no-fail-fast 2>&1 | grep -E "FAILED|BareLegacyLetBindings" | head -40
   ```

   Each failing test points at a source file containing a legacy
   let. The walker diagnostic includes the file location.

3. **Translate** — open each file, find the legacy let, apply the
   recipe above. Save.

4. **Re-measure** — run step 1 again. Failure count should drop. If
   it didn't drop OR a new category surfaced, STOP and report.

5. **Repeat** until step 1 reports `failed: 0`.

### Stdlib sites (no walker firing)

The walker is scoped to user-source forms (slice 1 delta A —
`freeze.rs` user-source pre-pass). Stdlib forms in `wat/*.wat`
and `wat-tests/holon/*.wat` etc. continue to work via the legacy
parser fall-through — they DON'T fail the workspace test, so the
walker loop above won't surface them.

**Hand-grep pass** to find stdlib sites:

```
grep -rEn '\(:wat::core::let\b' wat/ wat-tests/ crates/*/wat/ crates/*/wat-tests/ | head -100
```

For each hit, inspect the form. If it uses the legacy outer-List
shape `((name expr) (name expr))`, apply the recipe. If it already
uses the new Vector shape `[name expr name expr]`, skip.

After stdlib migration, re-run step 1; the count should still be
0 (stdlib continues to work via canonical parser; no regression).

### `tests/wat_*.rs` embedded wat strings

These are user-source files where the wat code lives inside Rust
`r#"..."#` literals. The walker fires on these (they're user-source).
The cargo-test failure stream surfaces them.

For each failing test in `tests/wat_*.rs`:
- Open the test file
- Find the embedded wat string (search for `:wat::core::let`)
- Apply the recipe inside the raw string
- Save

DO NOT modify the test logic, assertions, or surrounding Rust code.
Edit only the contents of the embedded wat strings.

## Discipline reminders

- DO NOT push to main; only push to slice branch
- DO NOT modify substrate code in `src/` — slice 1 shipped the
  consumer; slice 2 is sweep only
- DO NOT modify the new `walk_for_legacy_let_bindings` walker —
  it's the diagnostic source
- DO NOT modify `parse_let_binding` (the canonical parser path)
- DO NOT modify `wat/core.wat`'s defn macro
- DO NOT pipe `cargo test` through anything other than the exact
  inline pattern above (`grep "^test result" | awk '{p+=$4; f+=$6} END ...'`)
- DO NOT use scripts in `scripts/` — they're unreachable from
  subagents in this environment; the inline pattern is the only
  verification path
- DO NOT add `2>&1 | head -N`, `; echo $?`, or any other shell glue
  to the verification commands. Run them EXACTLY as written.

## FM 5 GUARDRAIL — explicit

If a substrate quirk surfaces (a test that should migrate cleanly
fails after migration, a test that doesn't fire walker actually has
legacy syntax, the canonical parser rejects something the recipe
should accept):

- STOP and report
- DO NOT bridge by modifying the test
- DO NOT modify substrate code to "make it work"
- DO NOT rewrite assertions
- The right answer is always: STOP, name the gap, let orchestrator
  decide

Arc 167 slice 3 had an FM 5 detour that cost ~45 min and required a
revert. The lesson: substrate gaps surface in the sweep; that's
honest. Bridging the test hides the gap.

## FM 16 GUARDRAIL — no tool preamble

This BRIEF intentionally does NOT remind you about Bash availability,
permission patterns, or tool-allowance details. Use your tools naturally.
If a specific Bash call surprises you with denial, surface it as an
honest delta in your report. Don't loop on it.

## Scope: ~563 sites across these regions

Per slice 1 SCORE: workspace failure count ~560 (matches DESIGN's
prediction of ~563). Sites are spread across:

- `wat/*.wat` (stdlib — walker doesn't fire; grep-driven)
- `wat-tests/*.wat` (user-source — walker fires; cargo-test driven)
- `wat-tests/holon/*.wat` (user-source — walker fires)
- `tests/wat_*.rs` (embedded wat strings — walker fires)
- `crates/*/wat-tests/` (user-source — walker fires)
- `crates/*/wat/` (stdlib — grep-driven)
- `examples/*/` (user-source — walker fires if exercised by tests)

## Report shape

When complete, report:

1. Final cargo test summary via the inline pipeline (should be
   `passed: N failed: 0`)
2. Site count by region (wat/, wat-tests/, tests/, crates/*/, examples/)
3. Honest deltas — sites that didn't fit the recipe; substrate quirks
4. Walker output sample from one failing test before migration
   (paste the diagnostic verbatim) — ONE sample, not many
5. Branch state confirmation (commits added, no push to main)
6. Actual runtime in minutes vs predicted band

## Time-box

60-120 min predicted (sonnet); 240 min hard cap. If you exceed 120 min
still iterating, STOP and report current state — orchestrator decides
on continuation.

## Cleanup

- DO NOT touch `scripts/sum-results.sh` (untracked orphan from slice 1
  session — orchestrator handles it)
- DO NOT delete or modify any `scripts/` files
