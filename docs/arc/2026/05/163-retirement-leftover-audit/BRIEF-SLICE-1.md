# Arc 163 — Slice 1 BRIEF (`let*` retirement leftover sweep)

**Drafted 2026-05-07.** Slice 1 of arc 163 — apply Bucket A/B/C/D
classification to arc 154's `let*` retirement leftovers. User
surfaced this as the concrete instance: *"i found `let*` refs when
we killed it in favor of just `let`."*

## Why slice 1 starts with `let*`

Pre-flight orchestrator audit found:
- `let*` total: 243 sites
- `BareLegacyLetStar` (Bucket D, KEEP): 19 sites
- **Bucket A live identifiers** (Rust fn names processing the
  UNIFIED `let` form but still named with `let_star`): ~48 sites
  in runtime.rs + check.rs
- Bucket B comment text: ~150-180 sites

Arc 154 retired `:wat::core::let*` but missed the internal-identifier
sweep — same pattern as arc 162 surfaced for `lambda`. Slice 1 closes
this for `let*`.

## Working directory

`/home/watmin/work/holon/wat-rs` on `main` branch.

## Workspace state pre-spawn

- HEAD: `e2f1470` (arc 162 closed)
- Working tree clean
- Workspace: 2041 passed / 0 failed

## Bash availability

Bash works for sub-agents in this environment (verified 2026-05-07
with probe `which cargo && cargo --version` returning
`cargo 1.93.0`). If your initial prompt-read makes you suspect
"Bash unavailable," run this one command:

```bash
which cargo && cargo --version && echo "BASH_VERIFIED"
```

If it succeeds (it should), proceed normally. Per memory
`feedback_verify_sonnet_tool_claims.md` — sonnet false-claims
of tool unavailability are the failure pattern; verify before
escalating.

## Bucket framework (recap from arc 162)

For each `let*` site, classify into one of four buckets:

- **A — RENAME**: live identifiers using legacy name as concept.
  Examples specific to `let*`:
  - `infer_let_star` → `infer_let` (Rust fn in check.rs)
  - `eval_let_star_tail` → `eval_let_tail`
  - `step_let_star` → `step_let`
  - `check_let_star_for_scope_deadlock_inferred` → `check_let_for_scope_deadlock_inferred`
  - Local var names containing `let_star` → `let`
  - Test fn names containing `let_star` (e.g., `let_star_destructures_a_pair`)

- **B — UPDATE**: comment text using `let*` as live concept.
  Examples:
  - `// the let* body is...` → `// the let body is...` (since
    the substrate now uses unified `let`)
  - `// `let*` semantics ...` (in present tense) → `// `let`
    semantics ...`
  - `:wat::core::let*` mentions inside comments describing
    CURRENT substrate behavior → `:wat::core::let`

- **C — KEEP**: comments recording arc 154 retirement.
  Examples to PRESERVE verbatim:
  - "Arc 154 — `:wat::core::let*` retired"
  - "(formerly let_star)"
  - "BareLegacyLetStar variant + Display preserved"
  - "single-letform vocabulary"

- **D — KEEP**: orphaned scaffolding per arc 113 precedent.
  - `BareLegacyLetStar` variant + every Display ref (~19 sites)
  - `validate_legacy_let_star` fn + walker body (the variant's
    associated walker that fires `BareLegacyLetStar` — its name
    explicitly refers to the legacy form it rejects, so KEEP)
  - Embedded-wat `:wat::core::let*` literals inside test fixtures
    that VERIFY the retirement diagnostic fires (any test file
    in `tests/` with names like `wat_arc154_kill_let_star.rs` —
    its test fixtures are Bucket D)

## Pre-flight crawl

1. Read `docs/arc/2026/05/163-retirement-leftover-audit/DESIGN.md`
2. Read `docs/arc/2026/05/162-lambda-internal-rename/BRIEF-SLICE-1.md`
   (full Bucket A/B/C/D framework)
3. Read `docs/arc/2026/05/162-lambda-internal-rename/INSCRIPTION.md`
   (what shipped; refines understanding of D-preservation precedent)
4. Read arc 154's INSCRIPTION:
   `docs/arc/2026/05/154-kill-let-star/INSCRIPTION.md` (or wherever
   arc 154's record lives — search if the path differs)
5. Read `src/check.rs` lines 1522-1535 (the `infer_let_star` /
   `check_let_star_for_scope_deadlock_inferred` references) +
   the `validate_legacy_let_star` definition
6. Read `src/runtime.rs` lines 2336-2475 (the `eval_let_star_tail`
   doc + `step_let_star` mentions)
7. Read `tests/wat_arc154_kill_let_star.rs` lines 1-50 — confirm
   shape of test fixtures (Bucket D — DO NOT TOUCH)

## Audit baseline (run BEFORE editing)

```bash
cd /home/watmin/work/holon/wat-rs
cargo test --release --workspace 2>&1 | grep -E "test result" | awk '{passed+=$4; failed+=$6} END {print "Pre passed:", passed, "Pre failed:", failed}'
echo "Total let* / let_star sites:"
grep -rn "let\*\|let_star\|LetStar" --include="*.rs" --include="*.wat" . 2>/dev/null | wc -l
echo "BareLegacyLetStar (Bucket D, must stay 19):"
grep -rn "BareLegacyLetStar" --include="*.rs" . 2>/dev/null | wc -l
```

## Procedure

### Step 1 — Bucket A renames (compiler-driven)

Rename Rust live identifiers one at a time:
1. `infer_let_star` → `infer_let` (start with this; it's the
   primary check-side fn). Use `replace_all: true` per file.
   Run `cargo build --release` after — the compiler will guide
   any callsite issues.
2. `eval_let_star_tail` → `eval_let_tail`. Build.
3. `step_let_star` → `step_let`. Build.
4. `check_let_star_for_scope_deadlock_inferred` → `check_let_for_scope_deadlock_inferred`.
   Build.
5. Test fn names containing `let_star` (e.g.,
   `let_star_destructures_a_pair`, `step_let_star_substitute`,
   `step_let_star_peel_first`) → drop `_star` suffix:
   `let_destructures_a_pair`, `step_let_substitute`, etc.
   These are unit-test fn names — replace via Edit.
6. Local variables containing `let_star` → `let_*` shape.

**EXCEPTION (Bucket D — DO NOT RENAME):** `validate_legacy_let_star`
fn keeps its name — it's the walker that fires `BareLegacyLetStar`
on legacy `:wat::core::let*` keyword usage. Per arc 113 precedent,
the walker's name names the legacy form it rejects. KEEP verbatim.

### Step 2 — Bucket B comment sweep

For each `.rs` and `.wat` file with `let*` mentions, classify:

- Comments using `let*` in present tense as if it's a current concept
  (e.g., `// the let* body is the form's tail position`) → UPDATE
  to use unified `let`
- Comments saying "`:wat::core::let*` semantics" → UPDATE
- Doc-comments on retired-walker-related fns referring to `let*` →
  if the doc describes the WALKER (Bucket D context), KEEP; if it
  describes substrate behavior in present tense, UPDATE

For each `.wat` source file (not `wat-tests/` — those may be test
fixtures): if the file has a live `let*` keyword in actual user code,
that's Bucket A (CHECK if arc 154's sweep missed live consumer code).
Verify against `cargo test --workspace` baseline — if test count
holds, consumer code is fine.

### Step 3 — Verify build + tests

```bash
cd /home/watmin/work/holon/wat-rs
cargo build --release 2>&1 | tail -5
cargo test --release --workspace --no-fail-fast 2>&1 | grep -E "test result" | awk '{passed+=$4; failed+=$6} END {print "Pass:", passed, "Fail:", failed}'
```

Expect: build clean; workspace 2041 passed / 0 failed (or higher;
must NOT decrease).

### Step 4 — Audit grep verification

```bash
echo "Bucket A live identifiers (must drop to ~0 — only validate_legacy_let_star + tests/wat_arc154 remain):"
grep -rn "let_star\|LetStar" --include="*.rs" . 2>/dev/null | grep -v "BareLegacyLetStar\|validate_legacy_let_star\|wat_arc154" | wc -l
echo "BareLegacyLetStar (Bucket D, must stay 19):"
grep -rn "BareLegacyLetStar" --include="*.rs" . 2>/dev/null | wc -l
echo "Total let* / let_star residual:"
grep -rn "let\*\|let_star\|LetStar" --include="*.rs" --include="*.wat" . 2>/dev/null | wc -l
```

Expect post-fix:
- Bucket A grep: ~0 (only `validate_legacy_let_star` + `tests/wat_arc154_kill_let_star.rs` Bucket D refs)
- `BareLegacyLetStar`: 19 (unchanged — Bucket D preserved exactly)
- Total: dropped substantially from 243 (Bucket B sweep)

## Constraints

- DO NOT commit. Working tree dirty for orchestrator review.
- "STOP at unexpected red" — don't paper over breakage.
- Test count must stay 2041 (or higher).
- Bucket D (`BareLegacyLetStar`, `validate_legacy_let_star`,
  `tests/wat_arc154_kill_let_star.rs`, embedded-wat fixtures
  testing the retirement diagnostic) MUST be preserved exactly.
- Time-box: 60 min wall-clock.

## Reporting (~250 words)

Report back with:
1. Pre-flight audit count → post-fix audit count
2. Per-step summary (Step 1 Bucket A renames count, Step 2 Bucket B
   updates count)
3. Test pass count: pre vs post (must stay 2041)
4. Path classification: Mode A / B / C
5. Honest deltas:
   - Did `infer_let_star` / `eval_let_star_tail` / `step_let_star`
     have any callsites that revealed surprising coupling?
   - Test fn names — any that legitimately needed to keep the
     legacy name because the test SPECIFICALLY documents legacy-
     form behavior? (e.g., `let_star_*` test that exercises the
     retired keyword's coexistence-with-let behavior)
   - `.wat` source files — any LIVE consumer code using `:wat::core::let*`
     that arc 154's sweep missed? Surface as a distinct category
     since that's a live Bucket A in user wat code, not just internal.
   - Hybrid sentences mixing live-concept + retirement context —
     how did you split?

DO NOT commit. Orchestrator commits + scores after.

## Time-box

60 minutes wall-clock.

## Why slice 1 matters

User direction: *"let's figure out what cruft we left in our code
base before we proceed - i want a strong foundation to stand on."*
The cruft surface is large; arc 163 closes it systematically.
Slice 1 takes the user-surfaced concrete instance (`let*`) first;
slices 2+ apply the same recipe to other retirement arcs (Vec,
list, Queue, stream, etc.).
