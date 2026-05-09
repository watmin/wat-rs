# Arc 168 slice 4 — sweep src/ lib unit-test legacy let fixtures

## Goal

Migrate 81 legacy let-binding sites in `src/runtime.rs` (70)
and `src/check.rs` (11) lib unit-test fixtures (embedded wat
strings inside `#[test]` functions) from legacy nested-pair-list
shape AND legacy typed-single binder shape to flat-vector shape.
These were slice 3 leftovers: slice 2 delta A walker scoping made
substrate-internal `mod tests` fixtures invisible to the slice 2
sweep stream. Slice 3's parser deletion now surfaces them as 81
failing tests.

After this slice ships green, arc 168's workspace is clean (modulo
the 5 pre-existing kernel/signal failures unrelated to arc 168) and
ready for slice 5 closure.

## Branch + commit policy

- **Active branch**: `arc-168-let-flat-shape` (slices 1+2+3+4 share
  this branch per atomic-merge discipline; slice branch carries all
  WIP commits; main untouched)
- Multiple WIP commits + pushes welcome on the branch for backup
- DO NOT push to main; orchestrator merges atomic to main as one
  squash commit after slice 5 closure paperwork ships

## Scope: edit ONLY embedded wat strings inside `#[test]` blocks

**THIS IS NOT SUBSTRATE WORK.** The 81 sites are wat-source-code
fixtures embedded inside Rust `#[test]` functions in `src/runtime.rs`
and `src/check.rs`. The legitimate work touches ONLY the contents
of `r#" ... "#` raw-string literals inside `#[test] fn ...()` blocks.

**DO NOT modify:**
- Substrate Rust code (eval, infer, check, parsers, walkers)
- Production functions (anything outside `#[test]` blocks)
- `parse_let_binding` / `eval_let` / `infer_let` / their helpers
- `wat/core.wat` defn macro
- Any non-test files

If a site needs changes outside the embedded wat string, STOP and
report. The discipline is: **mechanical translation of wat code
inside test fixtures, nothing else.**

## The migration recipe (two legacy shapes retired in slice 3)

### Shape 1 — legacy nested-pair-list outer

```scheme
(:wat::core::let ((name1 expr1) (name2 expr2)) BODY)
```

Translate to:

```scheme
(:wat::core::let [name1 expr1 name2 expr2] BODY)
```

### Shape 2 — legacy typed-single binder (arc 159 retired in user code)

```scheme
(:wat::core::let (((name :T) expr)) BODY)
```

Translate to (type drops; inferred):

```scheme
(:wat::core::let [name expr] BODY)
```

### Shape 3 — legacy List-destructure binder (rare)

```scheme
(:wat::core::let (((a b c) tup)) BODY)
```

Translate to:

```scheme
(:wat::core::let [[a b c] tup] BODY)
```

### Empty bindings

```scheme
(:wat::core::let () BODY)         ;; legacy empty
```

Translate to:

```scheme
(:wat::core::let [] BODY)         ;; flat-shape empty
```

### Multi-form body — preserved

Multi-form bodies pass through unchanged. Only the binding shape
migrates.

## Sites breakdown

`src/check.rs` — 11 sites in `mod tests`
`src/runtime.rs` — 70 sites in `mod tests`

Line numbers will drift across the sweep as you edit; use the
failing-test list (verification step below) to navigate.

Sample failing test cited by opus's slice-3 report:
- `runtime::tests::arc159_destructure_three_element` at
  `src/runtime.rs:23991:50` — legacy fixture string at
  `src/runtime.rs:20593`

## Verification

Use this inline pipeline (BRIEF-3 used the same — empirically
allowed for sonnet subagents):

```bash
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk '{p+=$4; f+=$6} END {print "passed:", p, "failed:", f}'
```

To get failing test names (paste into terminal):

```bash
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test " | grep " FAILED" | head -100
```

Sweep procedure:

1. Run the inline pipeline — note the current `failed: N` count
   (should start at 86; arc-168 territory is 81 of those)
2. Run the failing-test grep — get the failing test names
3. For each failing test: open the file at the test's location,
   find the embedded wat string with the legacy shape, apply
   the mechanical recipe, save
4. Re-run the inline pipeline — count drops as you go
5. Repeat until `passed: 2080 failed: 5` (or thereabouts —
   the 5 pre-existing kernel/signal failures are out of arc 168's
   scope per SCORE-SLICE-2 delta C and remain failed)

The error message from the parser will guide each fix; sample:
> `MalformedForm { reason: "let bindings must be a flat vector \`[name expr ...]\`" }`

That's the parser's clear signal that this site needs the
mechanical translation.

## Discipline reminders

- DO NOT push to main; only push to slice branch
- DO NOT modify substrate Rust code (only embedded wat strings
  inside `#[test]` blocks)
- DO NOT modify the canonical let parser or evaluator
- DO NOT modify `wat/core.wat`'s defn macro
- DO NOT bridge by re-adding the legacy parser (that's exactly
  the retired-cruft slice 3 just deleted)
- USE the inline pipeline for progress measurement (no scripts)
- DO NOT pipe `cargo test` through more than the documented `awk`
  pattern — sonnet has empirically been fine with this exact
  pipeline; don't elaborate
- If a site doesn't fit the mechanical recipe (e.g., the test is
  testing legacy syntax intentionally; the wat string has nested
  quoting; some other quirk), STOP and report; don't bridge

## FM 5 GUARDRAIL — explicit

- If a test fails AFTER your migration of its fixture (i.e., your
  edit didn't make the test green), STOP and report
- DO NOT rewrite the test's assertions to match a different
  outcome
- DO NOT modify substrate code to "make the test work"
- DO NOT re-add legacy parser arms — they're retired permanently
  per slice 3's commit `f108a13`
- The right answer is always: STOP, report what you observed, let
  orchestrator decide

## Report shape

When complete, report:
1. Final cargo test summary via the inline pipeline (target:
   `passed: 2080 failed: 5` — the 5 pre-existing kernel/signal
   unrelated)
2. Site count by file (calibration: ~70 in `src/runtime.rs`,
   ~11 in `src/check.rs`)
3. Honest deltas — sites that didn't fit the recipe; substrate
   quirks discovered (note: per discipline, substrate quirks
   should trigger STOP-and-report, not workarounds)
4. Branch state confirmation
5. Actual runtime in minutes vs predicted band

## Time-box

Per EXPECTATIONS-SLICE-4.md (60-120 min predicted, 240 min hard cap).
If you exceed the upper bound still iterating, STOP and report
current state.
