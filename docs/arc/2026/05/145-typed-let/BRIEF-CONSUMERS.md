# Arc 145 — Typed Let Consumer Migration BRIEF (sweep 1b)

**Drafted 2026-05-06.** Sweep 1b of arc 145's typed-let work.
Per DESIGN's slice plan + recovery doc § 7 atomic-commit-across-
coordinated-sweeps: sweep 1a shipped the substrate change; sweep
1b migrates every existing `(:wat::core::let ...)` /
`(:wat::core::let* ...)` call site to the typed shape. Atomic
commit when workspace = 0-failed.

## Pre-spawn workspace state

- HEAD: `e173bd5` (DESIGN consistency + BRIEF + EXPECTATIONS for sweep 1a)
- Working tree DIRTY with sweep 1a substrate edits (4 files):
  - `src/check.rs` (+225/-50)
  - `src/runtime.rs` (+143/-20)
  - `src/special_forms.rs` (+22/-6)
  - NEW `tests/wat_arc145_typed_let.rs` (+236)
- Pre-baseline: `cargo test --release --workspace` = 652 passed / 129 failed / 0.34s
- Failure shape: uniform `MalformedForm` migration-hint on
  `:wat::core::let*` (+ a few `:wat::core::let`) across stdlib
  + tests + per-crate substrates + embedded Rust strings
- 5 of the 10 new arc145 tests fail under sweep-1a isolation
  (they need stdlib to compile); they unblock when sweep 1b ships

## Goal

Workspace returns to **0 failed** by adding `-> :T` at HEAD of
every existing `let` / `let*` call site, where `:T` is the
DECLARED contract of that let — what the body produces AND what
the recipient expects.

## The substrate-as-teacher discipline

Per `docs/SUBSTRATE-AS-TEACHER.md`: the substrate's diagnostic
stream IS the migration brief. Sweep 1a shipped the necessary
substrate. You read the diagnostics, edit per site, iterate until
green.

**Three diagnostic kinds will guide you:**

1. **`MalformedForm` with migration-hint reason** (sweep 1a's
   addition): tells you a let/let* call site needs `-> :T`. The
   reason text contains the canonical form template:
   ```
   `:wat::core::let*` now requires `-> :T` at HEAD; write
   (:wat::core::let* -> :ResultType (((n :Type) expr) ...) body)
   ```
   File:line:col is in the diagnostic. Add `-> :T` at the named
   site.

2. **`TypeMismatch` on let body** (existing baseline): once you
   add `-> :T`, if your declared `:T` doesn't match the body's
   actual type, this fires. Tells you `expected :YourT but body
   produced :ActualT`. Adjust `:T`.

3. **`TypeMismatch` at recipient** (existing baseline): if your
   declared `:T` doesn't match what the let's caller / binding /
   field / return-position expects, this fires at the recipient
   site. Adjust `:T` to match what the recipient expects.

**The convergent T is the honest contract** — what BOTH body and
recipient agree on. Iterate per error until cargo test = 0 failed.

## Starting strategy

For your first pass, write `-> :wat::core::unit` at every
let/let* site. This is the most conservative declared type:

- Sites that ARE unit-binding chains (the let*-with-`((_ :unit)
  ...)` "do-form crutch" pattern) will land correctly on first
  pass.
- Sites that produce other types will fire body-vs-declared
  TypeMismatch, telling you the actual body type. Narrow `:T` to
  match.
- Sites where the recipient expects something other than what
  body produces will fire recipient-vs-let-return TypeMismatch.
  Adjust per the recipient's expectation.

User direction (verbatim, captured 2026-05-06):
> *":unit is fine with me - it'll probably be correct in a few places"*

Don't try to pre-decide T per site by reading body code. Let the
substrate teach. The starting `:unit` + iteration is the
discipline.

**Caveat — `:Any` is forbidden** in this substrate. There is no
escape hatch type. If the body produces something the type
system can't yet express, surface it as a Mode-C honest delta.
Don't invent a wildcard.

## Sweep order (per substrate-as-teacher § "stdlib first")

The substrate-bundled stdlib loads on every wat invocation; if
it has unmigrated sites, every test fires those errors before
producing anything useful. Sweep order:

1. **`wat/*.wat`** (stdlib bundled with the binary) — FIRST
2. **`crates/*/wat/**/*.wat`** (per-crate substrates)
3. **`wat-tests/**/*.wat`** (workspace test files)
4. **`crates/*/wat-tests/**/*.wat`** (per-crate test files)
5. **`examples/**/*.wat`** (example programs)
6. **Embedded wat strings in `tests/*.rs` + `src/*.rs`** (Rust
   tests with inline wat) — LAST

After step 1, run `cargo test --release --workspace` to confirm
stdlib boots clean (failures should drop substantially). After
each subsequent step, re-run to confirm progress. Convergence to
0 failed is the success signal.

## Constraints

- **DO NOT COMMIT.** The orchestrator commits sweeps 1a + 1b +
  SCORE docs atomically when workspace = 0-failed (per recovery
  doc § 7).
- **DO NOT touch substrate sources** (`src/*.rs`,
  `src/check.rs`, etc.) — sweep 1a already shipped substrate
  changes. If you discover a substrate-internal bug during 1b,
  STOP and report (Mode B).
- **DO NOT touch `holon-lab-trading/`** — separate workspace, out
  of scope.
- **STOP at first unexpected red.** Distinguish:
  - **Expected red:** MalformedForm migration-hint on a let/let*
    call site (drives your work) OR TypeMismatch on a let's
    body / recipient where the migration-hint guidance applies.
  - **Unexpected red:** anything else — substrate panic, parse
    error inside substrate code, runtime crash, TypeMismatch
    that doesn't trace to a let/let* contract issue. Surface
    these.
- **No `:Any` injection** anywhere.
- **No grinding.** If a single site requires more than ~3
  iterations to converge, STOP and surface as Mode C honest
  delta — the substrate's diagnostic isn't teaching cleanly at
  that site.

## Pre-flight crawl

1. **`docs/SUBSTRATE-AS-TEACHER.md`** — full read, especially
   §§ "The four-step recipe" + "When the discipline applies" +
   "Three migration patterns" (this is Pattern 1 — type-shape
   change).
2. **`docs/arc/2026/05/145-typed-let/DESIGN.md`** — full read,
   especially Status (REQUIRED `-> :T` resolution), Q1 (HEAD
   placement resolution), and the slice plan.
3. **`docs/arc/2026/05/145-typed-let/BRIEF-SUBSTRATE.md`** — what
   sweep 1a shipped (your edit space is everything OUTSIDE this
   brief's scope).
4. **`docs/arc/2026/05/145-typed-let/EXPECTATIONS-SUBSTRATE.md`** —
   the scorecard sweep 1a was scored against.
5. **`tests/wat_arc145_typed_let.rs`** — the canonical typed-let
   shape examples (read the 5 currently-passing tests for shape;
   the 5 failing tests will pass once stdlib is migrated).
6. **A sample of current MalformedForm diagnostics:** run
   `cargo test --release --workspace 2>&1 | head -200` and
   skim the patterns. Each line names a file:line:col + the
   canonical form template.

## Verification

After your sweep converges:

```bash
cargo test --release --workspace 2>&1 | grep -E "test result:|FAILED" | tail
```

Expect: 0 failed across all crates. The 5 currently-failing
arc145 tests (typed parallel happy path, typed sequential happy
path, nested, tail-call, sequential visibility) should now pass.

Sample-verify by running:
```bash
cargo test --release --test wat_arc145_typed_let 2>&1 | tail
```

Expect: 10 passed / 0 failed.

## Reporting (~300 words)

1. **Pre-flight crawl confirmation:** SUBSTRATE-AS-TEACHER,
   DESIGN, BRIEF-SUBSTRATE, EXPECTATIONS-SUBSTRATE,
   tests/wat_arc145_typed_let.rs, baseline diagnostic sample
   all read.

2. **Sweep summary:** files modified per directory bucket
   (`wat/`, `crates/*/wat/`, `wat-tests/`, `crates/*/wat-tests/`,
   `examples/`, embedded Rust). Total file count + total call-
   site count migrated.

3. **`:T` distribution:** how many sites landed on `:unit` (no
   narrowing needed); how many narrowed to other concrete types
   (give a histogram if interesting); how many surfaced
   parametric `:T` (generic functions).

4. **Iteration cycles:** how many cargo test runs you ran to
   converge. Time per cycle. Total wall-clock.

5. **Verification:**
   - `cargo test --release --test wat_arc145_typed_let` — 10/10 pass
   - `cargo test --release --workspace` — 0 failed

6. **Path:** Mode A clean (workspace converges to 0-failed via
   substrate-as-teacher) / Mode B substrate-bug surfaced / Mode
   C unclear-diagnostic at a specific site / Mode D site
   exceeding 3-iteration grinding cap.

7. **Honest deltas:** any sites where the diagnostic was unclear
   and required reading body-context to resolve; any sites where
   the convergent `:T` surprised you (e.g., parametric where you
   expected concrete); any patterns of repeated `:T` choices
   that suggest a substrate-improvement opportunity (e.g., maybe
   typealias would help).

DO NOT write a SCORE doc — orchestrator's work after sweep 1b
ships and the atomic commit lands.

## Time-box

120 minutes wall-clock (predicted upper-bound 60-90 min;
substrate-as-teacher iteration adds cycles; 2× cap = 240 min
but I'll cap at 120 to surface stalls earlier). If you hit 120
min and aren't done, STOP and report progress.

## Why this brief is short

Per substrate-as-teacher § "Brief the sonnet": the brief is
"run cargo test; read the hints; apply the migration; iterate
until green; report what wasn't obvious." That's the entire
delegation contract. The substrate's diagnostic stream IS the
detailed brief. Brief verbosity becomes a function of substrate-
message quality, not orchestrator wordcount.

If diagnostics aren't teaching cleanly at any site, surface that
gap — it's the substrate's job to teach, not the brief's.

## Mode A clean = ship-ready

When workspace = 0-failed and the 10 typed-let tests pass:
- Working tree contains BOTH sweep 1a substrate edits AND sweep
  1b consumer edits (uncommitted)
- Orchestrator commits atomically with a message naming both
  sweeps + arc 145 closure paperwork
- Then slice 2 (closure paperwork — INSCRIPTION + 058 row +
  USER-GUIDE) ships
- Arc 145 closes
- Arc 136 (typed do form, Option B per 2026-05-06 user direction)
  spawns next per "typed let then do"
