# BRIEF — Stone 214.4.6b: `select'` — first-ready multiplex over same-tier peers

> Foundation + verbs landed (4.6a, `d404364e` + `54a58eeb`). DESIGN (settled):
> `DESIGN-STONE-4.6-POLYMORPHIC-VERBS.md` § 4.6b. Probe (committed, RED):
> `tests/nursery/probe_arc214_stone46b_select_prime.rs`.

## The contract (pinned)

`(:wat::kernel::select' peers)` where `peers : Vector<peer<I,O>>` →
**`Tuple<i64, O>`** — blocks until one peer's output has a value; returns
(index, value). Intrinsic — projective (O flows; PARTITION marker per
docs/DISPATCH.md). Mixed tiers need NO bespoke rejection — `Vector` homogeneity
already forbids them at check. Empty vector → `MalformedForm` "select over an
empty vector would block forever" (mirror the old select). Blocking is
cascade-aware via each tier's existing Select (thread: crossbeam + SHUTDOWN
arm; process: io_uring POLL_ADD/POLLHUP).

## The rooms (read in order)

1. `src/rust_deps/custodia.rs` — `ThreadOwnedCell` (`with_ref`/`with_mut`,
   thread-id guard). **Part 1 adds a guard-returning borrow** (e.g.
   `fn ref_guard(&self, op, span) -> Result<RefGuard<'_, T>, RuntimeError>`):
   same thread-id validation as `with_ref`, returns a lifetime-bound guard
   `Deref<Target = T>`. Rationale: `select'` must hold N peers' receivers
   SIMULTANEOUSLY to register them in a Select; closure-scoped `with_ref`
   cannot nest for dynamic N. All guards are shared borrows — no aliasing
   hazard the closure form doesn't already have.
2. `src/comms/thread.rs` — `Select` (crossbeam-based, module doc line ~15;
   SHUTDOWN_RX internal arm). `src/comms/process.rs:~830` —
   `Select<'a,T>::new/recv/select` → `SelectOutcome::Recv{index, ..}`
   (io_uring, persistent ring; empty-Select errors at select() by design).
   Register via `&'a Receiver<T>`; confirm exact signatures as you wire.
3. `src/check.rs` — `infer_spawn_program_prime` + the 4.6a-ii infer fns
   (~10530+): the templates. **Part 2: `infer_select_prime`** — infer args[0],
   apply_subst + reduce, expect `Parametric{head:"wat::core::Vector",
   args:[elem]}` whose elem reduces to
   `Parametric{Thread'|Process', [I,O]}` → return
   `TypeExpr::Tuple(vec![i64-path, O])`. Anything else → TypeMismatch
   (expected "Vector of Thread'<I,O> | Process'<I,O> peers"). Wire in
   `infer_list` beside its siblings + PARTITION marker.
4. `src/runtime.rs` — the 4.6a-ii eval arms (the neighbors). **Part 3:
   `eval_peer_select_prime`** — eval the arg → `Value::Vec`; empty →
   MalformedForm (above); downcast every element by sentinel (first element's
   tier decides; a mismatched element → TypeMismatch — the check forbids it,
   the runtime still refuses honestly); `ref_guard` every peer cell, `.as_ref()`
   the Options (`None` → "peer already closed"); register all output receivers
   in the tier's Select; `select()`; map `Recv{index, value}` →
   `Value::Tuple(vec![Value::i64(index), value])` — Process′ decodes the EDN
   String to a Value first (`read_edn`, the 4.6a-ii recv' arm shows the call).

## Also ship (coverage)

A process-tier select test in `tests/comms/` (new file, `#[ignore]`, run with
`--test-threads=1`, mirror `peer_verb_round_trip_process.rs`): two `:process`
echo peers, `send'` to ONE, `select'` over both returns that peer's index and
value; `close'` both (exit 0 each).

## STOP triggers (rejection criteria — ship nothing for that part; report)

- STOP-1: if a sound guard-returning borrow cannot be added to
  `ThreadOwnedCell` (the safety argument fails), STOP and report why.
- STOP-2: if either tier's Select cannot register `&Receiver` as assumed
  (API shape differs materially), STOP with the actual API.

## Verify (report exact numbers)

Nursery + lib run PLAIN (no setsid/timeout — leak class is dead); comms
integration keeps the envelope until Slices 6+8:

- `cargo test --release --test nursery probe_arc214_stone46b_select_prime` → **2 passed**
- `cargo test --release --test nursery probe_arc214` → **52 passed** (all prior + these)
- `cargo test --release --lib -p wat` → ~943/0/1 band (+ any unit tests you add)
- `setsid timeout 180 cargo test --release --test comms peer_verb_round_trip_process -- --ignored --test-threads=1` → 1 passed (no regression)
- `setsid timeout 180 cargo test --release --test comms <your new select test> -- --ignored --test-threads=1` → 1 passed
- `cargo clippy --release` → no new warnings in touched files

Do NOT commit — the orchestrator scores against an independent re-run and commits.

## Expectations (orchestrator scorecard)

| # | Claim | Check |
|---|---|---|
| 1 | select probe 2/2 (runtime picks index+value; wrong-return rejected) | re-run |
| 2 | all arc214 nursery probes 52/0; lib band green | re-run |
| 3 | guard API: thread-id check preserved; shared-borrow only | read custodia diff |
| 4 | empty-vector → MalformedForm; closed-peer → "peer already closed" | read eval diff |
| 5 | process-tier select integration green | re-run |
| 6 | PARTITION marker at the infer arm | read diff |
| 7 | no new clippy; tree dirty | clippy + git status |

Runtime band: 25–40 min (guard API + infer + eval + integration test).
