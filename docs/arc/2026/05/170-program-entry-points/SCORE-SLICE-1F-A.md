# Arc 170 slice 1f-α — SCORE

**Result:** Mode A clean.
**Runtime:** ~50 min opus (under predicted 60-90 band; well under 180 hard cap).
**Files:** 2 new + 3 modified.

## Calibration

- **Predicted runtime band:** 60-90 min opus (180 min hard cap)
- **Actual:** ~50 min — under band
- **Why faster than predicted:** pattern dropped in cleanly from
  the pre-grep citations (`eval_edn_write` template, existing
  `thread_local!` precedent, `value_to_edn_with` polymorphism).
  No substrate gaps surfaced. The mostly-mechanical eval-arm +
  thread-local + type-check registration came together quickly.

## Scorecard

| Row | What | Result |
|-----|------|--------|
| A — `eval_kernel_println` exists | ✓ `src/thread_io.rs:130`, dispatched at `src/runtime.rs:3572` |
| B — `eval_kernel_eprintln` exists | ✓ `src/thread_io.rs:159`, dispatched at `src/runtime.rs:3573` |
| C — `eval_kernel_readln` exists | ✓ `src/thread_io.rs:188`, dispatched at `src/runtime.rs:3574` |
| D — `ThreadIO` struct + `thread_local!` cell | ✓ `src/thread_io.rs:44, 67` |
| E — `install_thread_io` / `uninstall_thread_io` setters | ✓ `src/thread_io.rs:75, 87` |
| F — `RuntimeError::ServiceNotRunning` variant | ✓ `src/runtime.rs:1204`, Display arm at `:1359` |
| G — Type-check arms registered | ✓ `src/check.rs:12819-12834` |
| H — Unpopulated path returns clean error | ✓ rows A/B/C of fixture pass |
| I — Populated println sends + returns nil | ✓ row D pass |
| J — Populated eprintln sends + returns nil | ✓ row E pass |
| K — Populated readln blocks + returns form | ✓ row F pass |
| L — Polymorphic value types serialize | ✓ row G (i64, String, bool, tuple) |
| M — Type-check accepts any-T for println / eprintln | ✓ rows H + I |
| N — Type-check infers HolonAST for readln | ✓ row J |
| O — Test fixture cargo-test green | ✓ 10/10 in `cargo test --release --test wat_arc170_slice_1f_alpha_helpers` |
| P — Workspace within ±5 band | ✓ exactly 1327/855 (predicted 1327/855) |
| Q — `cargo check --release` green | ✓ |
| R — Zero new dependencies | ✓ Cargo.toml unchanged |
| S — Zero new Mutex/RwLock/CondVar | ✓ |
| T — Honest deltas surfaced | ✓ 9 surfaced (see below) |
| U — Slice 1f-i + 1f-ii artifacts untouched | ✓ (see honest delta #7) |

**21/21 rows pass.** Mode A clean.

## Honest deltas surfaced (9)

### 1. crossbeam crate path

**Classification:** cosmetic correction to BRIEF.

BRIEF cited `use crossbeam::channel::{Receiver, Sender}`. Actual
workspace dep is `crossbeam-channel`; correct import path is
`crossbeam_channel::{Sender, Receiver}` (no `crossbeam::channel`
umbrella). `src/thread_io.rs` + test fixture use the correct
path. No scope expansion.

### 2. `Value::Nil` vs `Value::Unit`

**Classification:** substrate already named differently.

Substrate's nil/unit value is `Value::Unit`, not `Value::Nil`.
TypeExpr representation is `TypeExpr::Tuple(vec![])`, accessed
via `unit_ty()` throughout `check.rs`; `:wat::core::nil` is the
user-facing FQDN spelling. opus used `Value::Unit` at runtime
and `unit_ty()` at check time — matches existing patterns.

### 3. `Arc<HolonAST>` ownership on stdin reply

**Classification:** design choice.

Used `Arc<HolonAST>` per BRIEF's slice 1f-i precedent.
`Value::holon__HolonAST(Arc<HolonAST>)` is the canonical
Value-wrap, so the reply Receiver carries `Arc<HolonAST>`
directly and `eval_kernel_readln` constructs
`Value::holon__HolonAST(ast)` with zero clone.

### 4. Module placement

**Classification:** design choice.

Placed `ThreadIO` + the three eval arms in a new
`src/thread_io.rs` module (~190 lines) rather than inlining in
`src/runtime.rs`. Reasoning: own concern (per-thread routing +
mini-TCP discipline); separates cleanly from runtime.rs's
16k-line core; keeps the ZERO-MUTEX tier-3 surface visible at
the file level. Wired via `pub mod thread_io;` in
`src/lib.rs:94`. Eval arms are `pub` and dispatched from
`runtime.rs` via `crate::thread_io::eval_kernel_*` — same
pattern as `crate::edn_shim::eval_edn_*`.

### 5. Eval-arm registration site

**Classification:** convention follow.

Registered as match arms in the existing kernel-primitives
section of `runtime.rs:3568-3574`, immediately after
`:wat::kernel::stopped?` and before `:wat::kernel::send`. Same
pattern as `:wat::edn::write` (also a `crate::edn_shim::*`
delegating arm).

### 6. Type-check registration site

**Classification:** convention follow.

Registered alongside the `:wat::edn::write` family in
`register_builtins`, using the same `unit_ty() / t_var() /
holon_ty()` helpers already defined locally in that function.

### 7. `tests/services_stdin.rs` orphan

**Classification:** pre-existing rot from slice 1f-i + 1f-ii
deletion; resolved twice (once by opus, once by orchestrator
pass-17 commit).

opus issued `git rm` on the file because workspace cargo test
was failing to BUILD (not just failing tests). Mid-session, the
orchestrator committed pass 17 (`2f03d32`) which independently
deleted the same file as part of its own slice cleanup. Both
deletions are equivalent; current HEAD has the file gone; no
diff in opus's staging area for it. Surfaced for the SCORE
audit; both layers acted correctly.

### 8. Polymorphic Tuple constructor spelling in test G

**Classification:** spec drift surfaced during test writing.

BRIEF mentioned `(:wat::core::Tuple/new 1 2)`; actual canonical
spelling per arc 109 slice 1g is `(:wat::core::Tuple x y z)`
(verb-equals-type playbook). Used canonical spelling in row G.
Bool literal in EDN-source is bare `true` / `false`, not
`:wat::core::true`.

### 9. Pass 17 docs in working tree mid-session

**Classification:** orchestrator parallel work; non-blocking.

Commit `2f03d32` landed mid-session adding pass 17 to
REALIZATIONS-SLICE-1.md + BUILD-PLAN.md edits + the
services_stdin.rs deletion. None of these touched opus's
slice's files. Working tree shows only opus's slice 1f-α
changes after commit.

## Calibration row

- **Actual runtime:** ~50 min (Mode A clean — under predicted
  60-90 band; well below 180 hard cap)
- **Workspace post-1f-α:** 1327 passed / 855 failed (verified
  locally via `cargo test --release --workspace --no-fail-fast`)
- **Fail-count delta from post-1f-deletion baseline:** 0 (855
  → 855; within ±5 band; predicted 0 — perfect)
- **Pass-count delta from post-1f-deletion baseline:** +10
  (= the 10 new test fixture rows; expected)
- **Honest deltas surfaced:** 9 (all properly classified —
  cosmetic correction, substrate-already-named, design choice,
  convention follow, pre-existing rot, spec drift, parallel
  work)
- **Pre-grep paid off:** every BRIEF citation matched substrate
  reality; the pattern was implementable as described

## Implementation choices (locked)

- **Module placement:** new `src/thread_io.rs` (separation over
  inlining)
- **Error variant:** added `RuntimeError::ServiceNotRunning { op:
  String, span: Span }`; reused existing
  `RuntimeError::ChannelDisconnected` for send/recv-on-disconnect
  (already shaped that way per arc 111 doctrine)
- **Eval-arm registration:** match-arm in `runtime.rs::eval`'s
  kernel-primitive section, delegating to `crate::thread_io::*`
  (mirrors `crate::edn_shim::*` pattern)
- **Type-check registration:** `env.register(...)` calls in
  `register_builtins` alongside `:wat::edn::write` (same
  `t_var() / unit_ty() / holon_ty()` helpers; same `TypeScheme`
  shape — `∀T. T -> nil` for println/eprintln, `() ->
  :wat::holon::HolonAST` for readln)
- **Value/Type for nil return:** `Value::Unit` at runtime,
  `TypeExpr::Tuple(vec![])` (`unit_ty()`) at check time
- **Stdin reply ownership:** `Arc<HolonAST>` (zero-clone path
  into `Value::holon__HolonAST`)

## Files modified / created

- `src/thread_io.rs` (new, 211 lines) — `ThreadIO` struct +
  thread-local cell + setters + three eval arms
- `src/lib.rs` (+1 line) — `pub mod thread_io;`
- `src/runtime.rs` (+32 lines) — `RuntimeError::ServiceNotRunning`
  enum variant + Display arm + three dispatch arms
- `src/check.rs` (+36 lines) — three TypeScheme registrations
- `tests/wat_arc170_slice_1f_alpha_helpers.rs` (new, 333 lines,
  10 test rows)

## Lessons captured

1. **Pre-grep + clear pattern citations continue to pay off.**
   The BRIEF cited every primitive opus needed (`eval_edn_write`,
   `value_to_edn_with`, `require_one_arg`, existing
   `thread_local!`) by exact location. opus had a clear
   template; ~50 min vs predicted 60-90. The pattern
   1f-i shipped (singleton; ~30 min wrong shape) is now
   re-formed cleanly in the wat-program-shape direction.

2. **Module-level placement decision wins.** New
   `src/thread_io.rs` keeps the tier-3 discipline visible at the
   file level. The runtime.rs delegation pattern stays clean
   (mirrors edn_shim.rs precedent).

3. **Honest deltas all classified correctly.** No scope
   expansion. Cosmetic BRIEF corrections (delta 1, 8) + design
   choices (delta 3, 4) + substrate-already-named (delta 2) +
   convention follows (delta 5, 6) + pre-existing rot (delta 7)
   + parallel work (delta 9) — all properly named.

4. **Substrate-as-teacher continues to deliver.** This slice
   succeeded because the architecture (passes 15+16+17) was
   locked first; the substrate primitives + patterns it needed
   already existed; the work was composition.

## What's next

1. **Commit slice 1f-α atomically** (this turn) — bundle the
   5 files + this SCORE doc
2. **Author BRIEF + EXPECTATIONS for slice 1f-β** — wat-side
   service implementations
   (`wat/kernel/services/{stdin,stdout,stderr}.wat` in canonical
   service-template + Signal::add/remove handler shape per pass
   16)
3. **Spawn slice 1f-β** (opus + wat-author; predicted 90-150
   min per BUILD-PLAN)

## Cross-references

- BRIEF: [`BRIEF-SLICE-1F-A.md`](./BRIEF-SLICE-1F-A.md)
- EXPECTATIONS: [`EXPECTATIONS-SLICE-1F-A.md`](./EXPECTATIONS-SLICE-1F-A.md)
- BUILD-PLAN ref: §3 slice 1f-α
- DESIGN + REALIZATIONS pass 15-17 (architecture this slice
  instantiates)
- Predecessor: pass 17 corrective (commit `2f03d32`)
- Pattern ancestor: `crate::edn_shim::*` (eval-arm template);
  arc 089 slice 5 (mini-TCP ack discipline); arc 124
  (hermetic-via-fork)
