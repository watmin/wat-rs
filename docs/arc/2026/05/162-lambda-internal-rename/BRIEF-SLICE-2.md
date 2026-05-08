# Arc 162 — Slice 2 BRIEF (Bucket B test + comment continuation)

**Drafted 2026-05-07.** Slice 2 of arc 162 — exhaustive continuation
of Bucket B comment-text + test-fn-name updates. Builds on slice 1's
already-shipped Bucket A rename (`a91e940` + `f295742` on main).

## Why slice 2 exists

Slice 1 swept Bucket A (live identifiers — Value variant, public type,
helper fns, type-name strings, debug labels, file rename, primary
test fn names) — the load-bearing rename. Bucket A audit grep
returns 0 sites; that's the structural assertion.

Slice 1 also handled the most visible Bucket B sites in src/runtime.rs
+ src/check.rs (file-level docs, Function struct field docs, eval_tail
comments). User audit at the close: ~250 `lambda`/`Lambda` mentions
remain in source.

Of those ~250: ~115-150 are Bucket C (arc 155 retirement context —
"arc 155 retired :wat::core::lambda", "formerly eval_lambda",
BareLegacyLambda comment blocks) + Bucket D (BareLegacyLambda variant
+ Display + test fixtures verifying the retirement diagnostic) —
PRESERVED by design per arc 113 precedent.

Of those ~250: ~50-80 are Bucket B leftovers in test files + minor
doc-comments. Slice 2's scope.

## Working directory

`/home/watmin/work/holon/wat-rs` on `main` branch.

## Workspace state pre-spawn

- HEAD: `f295742` (arc 162 slice 1 + cont'd Bucket B sweep shipped)
- Working tree clean
- Workspace: 2041 passed / 0 failed
- Bucket A grep: 0 sites
- Total `lambda\|Lambda`: ~250 sites

## Verify Bash availability FIRST

Per recovery doc § 7 (sonnet's known limits): if you sense Bash is
unavailable, RUN this verification command FIRST before claiming
unavailability:

```bash
which cargo && cargo --version
```

If that succeeds, Bash works; the prior-spawn permission claim was
a false positive (recurring sonnet limitation per § 7). Do NOT
abort on a permission-claim until you've verified with `which`.

## Slice 2 task

Apply Bucket B updates per the same arc 162 BRIEF-SLICE-1 framework
(read it for full classification:
`docs/arc/2026/05/162-lambda-internal-rename/BRIEF-SLICE-1.md`).

### Files to sweep

Order by site count:

1. **`tests/wat_tco.rs`** (~21 sites)
   - Test fn names: `lambda_tail_call_via_let_bound_symbol`,
     `inline_lambda_literal_tail_call`, `named_define_tail_calls_lambda_param`
     → rename `*lambda*` → `*fn*` in fn names
   - Comment text: "Lambda-valued tail calls" header,
     "lambda value", "lambda body", "(lambda ...)" syntax mentions
     in comments → update to "fn" terminology

2. **`tests/wat_core_try.rs`** (~13 sites) — same pattern

3. **`tests/wat_stream.rs`**, **`tests/wat_names_are_values.rs`**,
   **`tests/wat_arc154_kill_let_star.rs`**, **`tests/wat_arc157_def.rs`**,
   **`tests/wat_spawn_fn.rs`** (renamed in slice 1) — sweep test fn
   names + comments

4. **`src/runtime.rs`** (~67 remaining sites — many Bucket C; sweep
   only Bucket B):
   - Comments using "lambda" as live concept in present tense
   - PRESERVE arc 155 retirement comments verbatim:
     - `// formerly eval_lambda`
     - `// (formerly `eval_lambda`)`
     - `// Arc 155 slice 2 — :wat::core::lambda dispatch arm retired`
     - `// `:wat::core::lambda` keep functional fall-through`
     - any line containing "BareLegacyLambda"

5. **`src/check.rs`** (~58 remaining sites — same: sweep only Bucket B)
   - PRESERVE: `BareLegacyLambda`, arc 155 retirement comment blocks,
     diagnostic strings shown to users (line 648, 934)

### What to UPDATE (Bucket B)

| Pre-fix phrase | Post-fix |
|---|---|
| `lambda body` | `fn body` |
| `lambda value(s)` | `fn value(s)` |
| `lambda-valued` | `fn-valued` |
| `Lambda-valued` (header) | `Fn-valued` |
| `inline lambda` (in comment as live concept) | `inline fn` |
| `lambda literal` | `fn literal` |
| `Lambdas` (plural Cap) | `Fns` |
| `the lambda` (as live concept) | `the fn` |
| `a lambda` | `a fn` (be careful — only when describing live concept) |
| `lambda call` | `fn call` |
| `(lambda ...)` syntax mention in comment | `(fn ...)` (NOT in test embedded-wat literals) |
| Test fn names containing `lambda` | replace `lambda` → `fn` in fn name |
| Test fn doc-comments referring to "lambda" as live concept | update |

### What NOT to update (Bucket C/D — KEEP verbatim)

- `BareLegacyLambda` variant + every Display reference to it
- Comments explicitly recording arc 155 retirement:
  - "Lambda is dead (Clojure-faithful; fn replaces lambda per user direction 2026-05-07)"
  - "arc 155 retired", "(formerly eval_lambda)", etc.
- Embedded-wat literals `:wat::core::lambda` inside test fixtures
  that VERIFY the retirement diagnostic fires (e.g., in
  `tests/wat_arc155_fn_rename.rs` — DO NOT TOUCH that file)
- Diagnostic error message text shown to USERS when they use
  retired forms (`check.rs:648` — that's the Display arm for
  the BareLegacyLambda diagnostic, intentionally references the
  retired form by name)

## Pre-flight crawl

1. Read `docs/arc/2026/05/162-lambda-internal-rename/BRIEF-SLICE-1.md` (full Bucket A/B/C/D framework)
2. Read `docs/arc/2026/05/162-lambda-internal-rename/EXPECTATIONS-SLICE-1.md`
3. Read `tests/wat_tco.rs` lines 1-50 (header doc + first test)
4. Read `tests/wat_core_try.rs` lines 1-30
5. Run audit baseline:
   ```bash
   cd /home/watmin/work/holon/wat-rs
   cargo test --release --workspace 2>&1 | grep -E "test result" | awk '{passed+=$4; failed+=$6} END {print "Pre-fix passed:", passed, "failed:", failed}'
   grep -rn "lambda\|Lambda" --include="*.rs" --include="*.wat" /home/watmin/work/holon/wat-rs/ 2>/dev/null | wc -l
   ```

## Constraints

- DO NOT commit. Working tree dirty for orchestrator review.
- Time-box: 30 min wall-clock.
- "STOP at unexpected red" — don't paper over breakage.
- Test count must stay at 2041 (or higher).
- Bucket A grep MUST stay at 0:
  ```bash
  grep -rn "wat__core__lambda\|WatLambdaSigmaFn\|parse_lambda_signature\|_lambda_body_\|rhs_spawn_lambda" --include="*.rs" /home/watmin/work/holon/wat-rs/ 2>/dev/null | wc -l
  ```
- Total grep should drop noticeably (from ~250 to ~120-150 — Bucket C/D
  floor; if you find yourself thinking "should I update this?" on a
  retirement comment, KEEP — when in doubt, preserve).

## Verification (after edits)

```bash
cd /home/watmin/work/holon/wat-rs
cargo build --release 2>&1 | tail -3
cargo test --release --workspace --no-fail-fast 2>&1 | grep -E "test result" | awk '{passed+=$4; failed+=$6} END {print "Total passed:", passed, "Total failed:", failed}'
cargo clippy --release --workspace 2>&1 | grep -E "^warning:|^error:" | wc -l

# Audit greps:
grep -rn "wat__core__lambda\|WatLambdaSigmaFn\|parse_lambda_signature\|_lambda_body_\|rhs_spawn_lambda" --include="*.rs" /home/watmin/work/holon/wat-rs/ 2>/dev/null | wc -l   # MUST be 0
grep -rn "BareLegacyLambda" --include="*.rs" /home/watmin/work/holon/wat-rs/ 2>/dev/null | wc -l   # ~28
grep -rn "lambda\|Lambda" --include="*.rs" --include="*.wat" /home/watmin/work/holon/wat-rs/ 2>/dev/null | wc -l   # ~120-150 expected (Bucket C/D floor)
```

## Reporting (~250 words)

Report back with:
1. Pre-flight audit count (initial grep) + post-flight audit count
2. Per-file site-update count (e.g., wat_tco.rs: 21 sites updated)
3. Test pass count: pre vs post (must stay 2041 or higher)
4. Path classification: Mode A / B / C
5. Honest deltas — answer:
   - Did Bash work? (Should — verify `which cargo` first; if it fails, then escalate)
   - Did you find any Bucket A residuals you missed in slice 1? (Should be no.)
   - Any hybrid sentence (live-concept + historical-context in one comment) — how did you split?
   - Any test fn name that actually exercised behavior tied to the legacy form name (so the rename would lose meaning)? (E.g., `lambda_post_retirement_silently_aliases_to_fn` describes a SPECIFIC retirement behavior — its name MAY warrant keeping; flag if so.)

DO NOT commit. Orchestrator commits + scores after.

## Time-box

30 minutes wall-clock.
