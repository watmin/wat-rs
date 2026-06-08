# BRIEF — Stone 6.2: the dead-path purge (the compiler names the corpses)

> Slice 6's second face: "retire the dead fork/spawn paths." The census is
> done; the corpses are ENUMERATED below; the compiler's `never used`
> warning count (currently 6 in `cargo check --release --lib`) is the
> natural meter — it reaches ZERO at convergence. Delete-only: NO logic
> changes, NO new code in src/channel/ (the home is minted-but-unwarded;
> the quarry never grows before 6.w).

## Context (orchestrator-verified, mirror it)

- `:wat::kernel::process-send`/`process-recv` are RETIRED-WITH-HINT verbs
  (arc 170 Stone C Pattern 2): the runtime dispatch arms at runtime.rs:4063+
  are DELIBERATE tombstones (migration-hint Errs) and the check.rs scheme
  registrations + retire-hint plumbing (589/1014/2787/16027/16066) feed the
  infer-time hint. **The retirement mechanism is ALIVE — do not touch it.**
  Only the ORPHANED eval bodies behind it die.
- `:wat::kernel::wait-child` has NO dispatch arm and NO check registration —
  the verb is fully gone; its eval fn is a corpse and the docs that still
  cite it as live are stale.

## The corpses (verify zero-callers with your own grep before each cut)

1. `src/runtime.rs` `eval_kernel_process_send` (~:19809) — orphaned body of
   the retired verb. Dies with any PRIVATE helper whose only tenant it is.
2. `src/runtime.rs` `eval_kernel_process_recv` (~:19882) — same.
3. `src/runtime.rs` `process_died_error_entry_form_failure` (~:20256) +
   `process_died_error_entry_form_failure_value` (~:20265) — the old
   spawn-process child branch's error builders; tenantless.
4. `src/runtime.rs` `function_byte_equivalent` (~:682) — tenantless.
5. `src/check.rs` `is_i64` (~:12327) — tenantless.
6. `src/fork.rs` `eval_kernel_wait_child` (~:280) — the fully-retired verb's
   orphaned body; dies with sole-tenant helpers.

## The doc true-up (the stale wait-child citations)

These comments cite `:wat::kernel::wait-child` as a LIVE consumer; rewrite
each to the live story (Drop SIGKILLs+reaps un-waited children; arc-112's
exit-status path is the live read; `cached_exit` idempotence serves Drop +
the exit-status accessor):
- `src/value/value.rs:171`
- `src/fork.rs:176-177`, `:226`, `:252` (and any sibling the grep finds —
  `grep -n "wait-child" src/` is your set; historical retirement-record
  phrasing ("the retired wait-child") is FINE, present-tense liveness claims
  are not).

## Gates

1. **THE METER**: `cargo check --release --lib -p wat 2>&1 | grep -c "never used"`
   → **0** (from 6).
2. `cargo test --release --lib -p wat` → 943/0/1.
3. `cargo test --release --test nursery` → 863/4/4 (the 4 known parked).
4. `cargo check --all-targets` → 0 errors.
5. `cargo clippy --release --lib -p wat` → no NEW findings (the flat-file
   baseline noise is known).
6. `grep -rn "eval_kernel_process_send\|eval_kernel_process_recv\|eval_kernel_wait_child\|process_died_error_entry_form_failure\|function_byte_equivalent" src/ tests/`
   → ZERO.

## STOP triggers (rejection criteria)

- STOP-1: a "corpse" turns out to have a live caller your grep finds
  (the census was wrong) — report it, cut nothing at that site.
- STOP-2: deleting a corpse breaks a test — that test was a hidden tenant;
  report verbatim.
- STOP-3: the retirement-hint machinery (check.rs schemes/hints, the
  tombstone dispatch arms) would need edits — it must not.

## Constraints

- Delete-only + doc true-up. Commit NOTHING — the orchestrator scores and
  commits. Work only in /home/watmin/work/holon/wat-rs/.
