# BRIEF — Stone 214.4.6a-ii: the four peer verbs (all intrinsics)

> Foundation landed (4.6a-i, `d404364e`): `spawn-program'` infers
> `Parametric{wat::kernel::Thread'|Process', [I,O]}`; peer `RustOpaque`s report
> their sentinel via `val_type_path`. Probe (committed, RED):
> `tests/nursery/probe_arc214_stone46aii_peer_verbs.rs`. DESIGN:
> `DESIGN-STONE-4.6-POLYMORPHIC-VERBS.md` (note the close'-classification
> forward-correction: ALL FOUR verbs are intrinsics — the parametric peer arg
> makes even close' type-level).

## The verbs (the contract, pinned)

| Verb | Check type | Runtime |
|---|---|---|
| `(:wat::kernel::send' peer v)` | `peer<I,O>, I -> nil` (v unifies with I) | Thread′: `peer.send(v)` Value pass-through. Process′: encode `value_to_edn` + `wat_edn::write` → `peer.send(String)` |
| `(:wat::kernel::recv' peer)` | `peer<I,O> -> O` | Thread′: `peer.recv()`. Process′: `peer.recv()` → `read_edn` → Value. RecvError → RuntimeError (peer closed / child gone) |
| `(:wat::kernel::try-recv' peer)` | `peer<I,O> -> Option<O>` | same with `try_recv` → `:Some`/`:None` |
| `(:wat::kernel::close' peer)` | Thread′: `-> nil`. Process′: `-> i64` (exit code) | consume the peer: Thread′ `close()+join` (join Err → RuntimeError); Process′ `close()+wait` (`Exited(n)` → i64 n; `Signaled` → RuntimeError) |

## The rooms (read in order)

1. `src/check.rs` — `infer_spawn_program_prime` (landed in 4.6a-i, ~10530): the
   in-house template. The four new infer fns sit beside it and are wired in
   `infer_list` (~3666) beside the spawn arm. Each: infer args[0]; apply subst +
   reduce; match `TypeExpr::Parametric { head, args }` with head ==
   `"wat::kernel::Thread'"` or `"wat::kernel::Process'"` and args.len()==2 →
   project I/O. Non-peer arg0 → `TypeMismatch` (expected "peer (Thread'<I,O> |
   Process'<I,O>)"). Mark the partition in-source per `docs/DISPATCH.md`
   § "Where it's declared" (a short comment at the arms naming these as
   intrinsics — projective for send'/recv'/try-recv'; ∀-parametric arg for close').
2. `src/kernel/spawn.rs` — the peer payloads + the EDN bridge precedent
   (`HolonRepresentable for Value`, `value_to_edn`/`read_edn` usage in the child
   loop). **Representation change (the close' consume path):** wrap the payloads
   in `Option` — `Arc<ThreadOwnedCell<Option<Thread<Value,Value>>>>` and
   `Arc<ThreadOwnedCell<Option<ProcessPeerBundle>>>`:
   - `send'`/`recv'`/`try-recv'`: `with_ref` → `.as_ref()` →
     `None` ⇒ RuntimeError "peer already closed" (honest use-after-close).
   - `close'`: `with_mut` → `.take()` → owned peer → thread `close()+join` /
     process `close()+wait`. Second close' ⇒ the same "peer already closed".
3. `src/kernel/peer.rs` — the 4.4 methods you are driving (send/recv/try_recv/
   close/join/wait). No changes expected here.
4. `src/runtime.rs` — the keyword-head EVAL dispatch (the 4.5 spawn arm ~4120 is
   the neighbor): add the four prime arms; each downcasts via
   `rust_opaque_arc` + `downcast_ref_opaque` on the sentinel
   (`THREAD_PEER_TYPE_PATH`/`PROCESS_PEER_TYPE_PATH`) — try Thread′ first, then
   Process′, else TypeMismatch.
5. Existing tests that the Option-wrap touches: `src/kernel/spawn.rs` lib test
   (`spawn_thread_peer_echo_round_trip`) + `tests/comms/spawn_program_prime_process.rs`
   (downcasts `ThreadOwnedCell<ProcessPeerBundle>` → becomes the Option-wrapped
   type). Update their downcast types; behavior identical.

## Also ship (coverage, not the disconfirming probe)

A process-tier wat-surface round-trip in `tests/comms/` (new file,
`#[ignore = "...run with --test-threads=1..."]` like its siblings): the probe-1
program with `:process` instead of `:thread`, driven via
`startup_from_source` + `eval_in_frozen` (mirror
`tests/nursery/probe_arc214_stone46aii_peer_verbs.rs` probe 1's harness).
Expect 42 + clean close' (exit code 0).

## Notes

- The OLD `:wat::kernel::send`/`recv` comm-POSITION walker rules (arc 110/212)
  apply to the old Result<Option<_>> shapes — the primes return plain values and
  get NO position rules. Do not extend the walker to them.
- recv' on a closed/dead peer is a RuntimeError, not a silent None — the
  3-state collapse (arc 253) lives in try-recv' only.

## STOP triggers (rejection criteria — ship nothing for that part; report)

- STOP-1: if `with_mut` on `ThreadOwnedCell` does not exist or cannot give
  `&mut Option<T>` for the take, STOP and report the cell's actual API.
- STOP-2: if projecting I/O from the peer Parametric in infer hits a subst/
  reduce shape you cannot match, STOP with the exact TypeExpr.

## Verify (report exact numbers)

- `cargo test --release --test nursery probe_arc214_stone46aii_peer_verbs` → **3 passed**
- `cargo test --release --test nursery probe_arc214_stone46i_typed_peer` → still **5 passed**
- `cargo test --release --test nursery probe_arc214_lexer_primed_generic_head` → still **3 passed**
- `cargo test --release --lib -p wat` → green band (~943/0/1 + your new unit tests if any)
- Integration (each under `setsid timeout 180`, `--ignored --test-threads=1`):
  - `cargo test --release --test comms spawn_program_prime_process -- --ignored --test-threads=1` → **2 passed**
  - `cargo test --release --test comms peer_process_round_trip -- --ignored` → **1 passed**
  - your new process-tier verb round-trip → **1 passed**
- `cargo clippy --release` → no new warnings in touched files.

Do NOT commit — the orchestrator scores against an independent re-run and commits.

## Expectations (orchestrator scorecard)

| # | Claim | Check |
|---|---|---|
| 1 | verb probe 3/3 (runtime round-trip + 2 type negatives) | orchestrator re-run |
| 2 | foundation 5/5, lexer 3/3 (no regression) | orchestrator re-run |
| 3 | lib band green | orchestrator re-run |
| 4 | integration: 4.5 probes 2/2, 4.4 1/1, new process round-trip 1/1 | orchestrator re-run |
| 5 | use-after-close = honest RuntimeError (Option take) | read the diff |
| 6 | partition marked in-source at the infer arms | read the diff |
| 7 | no new clippy; tree dirty | clippy + git status |

Runtime band: 30–45 min (4 infer fns + 4 eval arms + representation change + tests).
