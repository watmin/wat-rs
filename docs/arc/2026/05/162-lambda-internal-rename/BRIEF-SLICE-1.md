# Arc 162 — Slice 1 BRIEF (the rename)

**Drafted 2026-05-07.** Slice 1 of arc 162 — exhaustive Rust-level
identifier rename `lambda` → `fn` to match arc 155's user-facing
retirement.

## Why this matters (user's exact frame)

User direction 2026-05-07: *"let's clean up the lambda refs - i
wasn't happy seeing left overs in the source... we need to make sure
we don't leave confusion when we do these clean ups."*

Arc 155 retired the user-visible `:wat::core::lambda` keyword. The
internal Rust identifier naming was deliberately scoped out — and
that scope-out left ~300 lambda references in the source. That's
the confusion the user is calling out. Arc 162 closes it.

## Working directory

`/home/watmin/work/holon/wat-rs` on `main` branch (this arc ships
directly to main; small mechanical scope; no WIP branch needed).

## Workspace state pre-spawn

- HEAD: `a96de13` (recovery doc FM 13 amendment shipped)
- Working tree clean
- Workspace: 2041 passed / 0 failed

## Classification framework — every "lambda" site falls into one bucket

For each site you encounter, classify into one of these buckets and
apply the corresponding action:

### Bucket A — RENAME (live identifiers using lambda as concept name)

These are Rust identifiers (types, functions, vars, modules, etc.)
that name a runtime/check-time concept using the legacy "lambda"
word. Per arc 155's surface retirement, they should use "fn".

| Site | Pre-fix | Post-fix |
|---|---|---|
| Value variant | `Value::wat__core__lambda(Arc<Function>)` | `Value::wat__core__fn(Arc<Function>)` |
| Type-name string | `"wat::core::lambda"` (Value::type_name return) | `"wat::core::fn"` |
| Sigma adapter | `WatLambdaSigmaFn` (public; exported in `lib.rs:116`) | `WatFnSigmaFn` |
| Parser fn (runtime) | `parse_lambda_signature` | `parse_fn_signature` |
| Parser fn (check) | `parse_lambda_signature_for_check` | `parse_fn_signature_for_check` |
| Walker helper | `spawn_thread_lambda_body_has_no_recv` | `spawn_thread_fn_body_has_no_recv` |
| Walker helper | `rhs_spawn_lambda_has_no_recv` | `rhs_spawn_fn_has_no_recv` |
| Local vars in those helpers | `lambda_call`, `lambda_head`, `lambda_*` | `fn_call`, `fn_head`, `fn_*` |
| Test fn names | `arc_134_no_recv_in_lambda_body_does_not_fire` | `arc_134_no_recv_in_fn_body_does_not_fire` |
| Test fn names | `typed_let_binding_with_lambda_value` | `typed_let_binding_with_fn_value` |
| Test file | `tests/wat_spawn_lambda.rs` | `tests/wat_spawn_fn.rs` |
| Module / module-level test fn names containing `lambda_` (mid-identifier) | `*lambda*` | `*fn*` |
| Debug-display strings | `<lambda@span>` (in `freeze.rs:162,191`, `check.rs:9474`) | `<fn@span>` |
| Callee-label strings | `":wat::kernel::spawn <lambda>"` (in `check.rs:7622`) | `":wat::kernel::spawn <fn>"` |

After applying these renames, `cargo build --release` will fail at
every Value-variant match arm (~10-15 sites in `runtime.rs`,
`freeze.rs`, `edn_shim.rs`, possibly `check.rs`); fix each one. The
compiler is your guide.

### Bucket B — UPDATE STALE COMMENT TEXT (lambda mentioned as live concept)

Comments that mention "lambda" as if it's a current substrate
concept (rather than historical context) read as confusion. Update
to use "fn" / "function" terminology.

Examples (these are illustrative; sonnet should sweep ALL similar
sites):

| Pre-fix | Post-fix |
|---|---|
| `// used outside any function or lambda body...` (`check.rs:6823,6930`) | `// used outside any function body...` |
| `// ─── Lambdas / functions ─────` (section header, `special_forms.rs:165`) | `// ─── Functions ─────` |
| `//! folds to 1 — the minimum geometric σ — so presence? / coincident? remain meaningful even if the user's lambda misfires.` (`sigma.rs:64`) | `//! the user's fn misfires.` |
| Doc comment on `arc_134_no_recv_in_lambda_body_does_not_fire` referring to "lambda body" as if it's a current shape | "fn body" |

The principle: **if a comment uses "lambda" to describe substrate
behavior in present tense (without "retired" / "arc 155" /
"historical" context), rewrite to use "fn" / "function" instead.**

### Bucket C — KEEP (historical retirement context)

Comments documenting that `lambda` was retired (and pointing at
arc 155's record) are historical context, not leftovers. KEEP
these:

- `// Lambda is dead (Clojure-faithful; fn replaces lambda per user direction 2026-05-07)`
- `// BareLegacyLambda variant + Display preserved as orphaned scaffolding`
- arc 155 retirement comment blocks throughout `check.rs` referring to "the legacy `:wat::core::lambda` keyword"
- arc 113 reintroduction-recipe comments mentioning lambda as the precedent
- arc 154 retirement comment block mentioning lambda as a sibling pattern

The principle: **if a comment's purpose is to RECORD the retirement
or POINT at arc 155 as historical context, KEEP it verbatim.**

### Bucket D — KEEP (orphaned scaffolding per arc 113)

CheckError variants and Display impls that name the LEGACY form
they reject:

- `BareLegacyLambda` variant + its Display arm
- `BareLegacyLowercaseFn` variant + its Display arm

The principle: **arc 113 orphaned-scaffolding precedent — variant
names referring to retired forms KEEP their legacy-name spelling
because they describe the form they reject.**

## Pre-flight crawl

1. Read `docs/arc/2026/05/162-lambda-internal-rename/DESIGN.md` (if present; the design that opened this arc)
2. Run the audit grep:
   ```bash
   grep -rn "lambda\|Lambda" --include="*.rs" --include="*.wat" /home/watmin/work/holon/wat-rs/ 2>/dev/null | wc -l
   ```
   Expect ~353 sites pre-fix.
3. Read `src/runtime.rs` line 159 — the `Value::wat__core__lambda` variant
4. Read `src/sigma.rs` lines 1-90 — the public `WatLambdaSigmaFn` type
5. Read `src/check.rs` lines 7461-7530 — the spawn-thread-lambda-* walker helpers (arc 134)
6. Read `tests/wat_spawn_lambda.rs` — confirm shape before file rename

## Verification baseline (run BEFORE editing)

```bash
cd /home/watmin/work/holon/wat-rs
cargo test --release --workspace 2>&1 | grep -E "test result" | awk '{passed+=$4; failed+=$6} END {print "Total passed:", passed, "Total failed:", failed}'
```

Expect: `Total passed: 2041 Total failed: 0`.

## Procedure (recommended order)

1. **Rename `Value::wat__core__lambda` first** — `src/runtime.rs:159`.
   Run `cargo build --release`. The compiler will fail at every
   match arm; work through each one (`runtime.rs`, `freeze.rs`,
   `edn_shim.rs`, `check.rs`).

2. **Rename helper functions next** — `parse_lambda_signature*` (×2),
   `spawn_thread_lambda_body_has_no_recv`, `rhs_spawn_lambda_has_no_recv`.
   Update their callsites (compiler-driven).

3. **Rename public type** — `WatLambdaSigmaFn` → `WatFnSigmaFn`.
   Update `src/lib.rs:116` re-export. Update its callsites.

4. **Update strings** — type-name string `"wat::core::lambda"` →
   `"wat::core::fn"`; debug-display `<lambda@…>` → `<fn@…>`;
   callee-label `<lambda>` → `<fn>`.

5. **Rename test functions in src/check.rs::tests, src/runtime.rs::tests**
   — any `*lambda*` test fn name → `*fn*`.

6. **Rename test file** — `git mv tests/wat_spawn_lambda.rs tests/wat_spawn_fn.rs`;
   update test fn names within if any.

7. **Local var renames inside helper functions** — `lambda_call`,
   `lambda_head`, etc. → `fn_call`, `fn_head`. Same scope as step 6.

8. **Stale comment text sweep** — apply Bucket B classification.
   Use grep to find `lambda body`, `Lambdas /`, `the user's lambda`,
   `lambda misfires`, etc.

9. **Verify retirement-context comments stay (Bucket C)** — sanity
   check by reading arc 154 / 155 retirement blocks; ensure they
   weren't accidentally swept.

## Post-fix verification

```bash
cd /home/watmin/work/holon/wat-rs
cargo build --release 2>&1 | tail -5
cargo test --release --workspace 2>&1 | grep -E "test result" | awk '{passed+=$4; failed+=$6} END {print "Total passed:", passed, "Total failed:", failed}'
cargo clippy --release --workspace 2>&1 | grep -E "^warning:|^error:" | wc -l
```

Expect:
- `cargo build --release`: clean
- `cargo test --release --workspace`: `Total passed: 2041 Total failed: 0` (or higher if test fn count shifted; should NOT decrease)
- `cargo clippy`: warning count unchanged from pre-fix baseline

### Final audit grep

```bash
# Should return ~30-50 sites total (Bucket C historical + Bucket D variants)
grep -rn "lambda\|Lambda" --include="*.rs" --include="*.wat" /home/watmin/work/holon/wat-rs/ 2>/dev/null | wc -l

# Should return ~28 (BareLegacyLambda variant + Display + comment refs)
grep -rn "BareLegacyLambda" --include="*.rs" /home/watmin/work/holon/wat-rs/ 2>/dev/null | wc -l

# Should return 0 (live identifiers all renamed)
grep -rn "wat__core__lambda\|WatLambda\|parse_lambda_signature\|_lambda_body_\|rhs_spawn_lambda" --include="*.rs" /home/watmin/work/holon/wat-rs/ 2>/dev/null | wc -l
```

The third grep returning 0 is the load-bearing assertion: every
live lambda identifier renamed.

## Constraints

- DO NOT commit. Working tree dirty for orchestrator review.
- "STOP at unexpected red" — if any pre-existing test breaks, stop
  and report; don't paper over with workarounds.
- Time-box: 60 min wall-clock (this is broader than typical
  mechanical sweeps because of the public type rename + test file
  rename + Bucket B comment sweep).
- The third final-audit grep MUST return 0. If it doesn't, you've
  missed a Bucket A site.

## Reporting (~250 words)

Report back with:
1. Pre-flight audit count (initial grep result)
2. Post-fix audit counts (all three final-audit greps)
3. Bucket-by-bucket summary:
   - Bucket A: how many sites renamed
   - Bucket B: how many comments updated
   - Bucket C: how many retirement-context refs preserved
   - Bucket D: how many BareLegacy* refs preserved
4. Test pass count: pre vs post (should be 2041 either way)
5. Clippy warning count: pre vs post (should be unchanged)
6. Path classification: Mode A (clean), Mode B (clean + scope creep
   you self-corrected), or Mode C (couldn't complete; surface gap)
7. Honest deltas: anything surprising. Specifically:
   - Did the public `WatLambdaSigmaFn` rename surface external
     callers we don't control? (Should be no — wat-rs is the only
     caller per memory `project_lab_reconstruction.md`.)
   - Did any retirement-context comment have a hybrid sentence
     mixing live-concept use + historical context? (E.g., "the
     lambda's body" inside a comment that ALSO says "arc 155
     retired lambda" — the second part stays, the first updates.)
   - Did any test embedded-wat string reference `:wat::core::lambda`
     as a literal that needs to fire `BareLegacyLambda`? (Those
     literals stay — they're test fixtures exercising the
     retirement diagnostic.)

DO NOT commit. Orchestrator commits + scores after.

## Time-box

60 minutes wall-clock.

## Why this matters as discipline

User's frame: *"we need to make sure we don't leave confusion when
we do these clean ups."* The gap arc 155 created (surface retired,
internals untouched) propagated for ~6 months because the discipline
of "user-visible cleanup also cleans internals" wasn't codified.
Arc 162 closes that specific instance. The orchestrator-side memory
saved post-arc-162 will codify the discipline so future surface-
retirement arcs immediately schedule the internal-rename followup
rather than leaving it as a queued backlog item that grows stale.
