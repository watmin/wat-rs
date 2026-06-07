# BRIEF — Stone 8.2w: the quarry dies (survivors lift home; git rm thread_io.rs)

> Phase A of the home's completion. Stones 8.1/8.1b/8.2 left `src/thread_io.rs`
> (635 lines) holding ONLY live, perfected, universe-resident machinery — zero
> condemned code. This stone lifts the survivors into `src/services/`,
> `git rm`'s the quarry, and sweeps every `thread_io::` path (imports, calls,
> and the user-facing diagnostic string in signal.rs). Behavior-identical —
> a structural move the type system drives. (Phase B — the trio-completion
> FULL VIGILIA — is the orchestrator's, after this scores.)

## Required reading (in order)

1. `src/thread_io.rs` — whole (635 lines). Every item in it moves.
2. `src/services/mod.rs` — the home it moves into.
3. `docs/ZERO-MUTEX.md` §§ Tier 3 + Mini-TCP (the contracts the moved code
   embodies — the module docs travel with their code).

## The gate (already committed, RED at HEAD)

`tests/nursery/probe_arc214_stone82w_quarry_dead.rs` — (1) the file is gone;
(2) ZERO `thread_io::` path references in src/ + tests/. GREEN by the lift,
never by probe edit.

## The home's new shape (pinned)

`src/services/` becomes a directory-home of three concerns + an index:

- **`src/services/mod.rs`** — the module doc (updated: the home now holds
  the WHOLE stdio architecture — peer loop, client half, wat-surface verbs;
  the thread_io quarry is dead, named as history) + `pub mod peer; pub mod
  client; pub mod verbs;` + `pub use` re-exports so every existing public
  name is reachable FLAT at `crate::services::X` (ServiceMsg, ServicePeer,
  spawn_service_peer, ThreadId, ThreadIO, install_thread_io,
  uninstall_thread_io, next_thread_id, RuntimeServices,
  register_thread_with_services, deregister_thread_from_services,
  AmbientStdio, install_ambient_stdio, uninstall_ambient_stdio,
  take_ambient_stdio).
- **`src/services/peer.rs`** — the universe side: `ServiceMsg<R>`,
  `ServicePeer<R>`, `spawn_service_peer` (moved verbatim from today's
  mod.rs body, doc-comments included).
- **`src/services/client.rs`** — the per-thread side: `ThreadId`,
  `ThreadIO`, the `THREAD_IO` thread-local, `install_thread_io` /
  `uninstall_thread_io` / `with_thread_io` (make it `pub(super)` — verbs.rs
  consumes it), `NEXT_THREAD_ID` + `next_thread_id`, `RuntimeServices` +
  its Debug impl, `register_thread_with_services` /
  `deregister_thread_from_services`, and the ambient block (`AmbientStdio`,
  its thread-local, `install_ambient_stdio` / `uninstall_ambient_stdio` /
  `take_ambient_stdio`).
- **`src/services/verbs.rs`** — the wat surface: `require_one_arg` (stays
  private to this file) + `eval_kernel_println` / `eval_kernel_eprintln` /
  `eval_kernel_readln`.

Module docs and section comments TRAVEL WITH their items (they carry the
doctrine — the 8.1 fight, the EOF cascade, the RS-only lifetime design).
The retirement-record comments that narrate the quarry's own history
(StdErrServiceEvent PURGED etc.) die WITH the quarry — the SCORE docs and
git log are that history's home.

## The sweep

- `git rm src/thread_io.rs` (after the lift compiles).
- `src/lib.rs`: `pub mod thread_io` dies; its re-export lines repoint to
  `services`. (`pub mod services` already exists.)
- Every `crate::thread_io::` → `crate::services::` (src/: freeze.rs,
  runtime.rs, spawn.rs, process_stdio.rs, value/signal.rs,
  value/symbol_table.rs, services/mod.rs's own `use crate::thread_io::ThreadId`).
- Every `wat::thread_io::` → `wat::services::` (~30 test files — the gate
  probe's scan is the completeness check; `cargo check --all-targets`
  is the compiler's).
- **The diagnostic string** at `src/value/signal.rs:502` says
  `wat::thread_io::install_thread_io` — a user-facing message that would
  lie post-lift. It becomes `wat::services::install_thread_io`. The
  doc-comment links at signal.rs:262/272 likewise.
- `src/value/mod.rs:19` banked-findings comment mentions "thread_io/freeze"
  — update the wording to name the new home (`services`/freeze).

## Gates

1. Gate-probe 82w → 2/2 GREEN.
2. `cargo test --release --test nursery` → 853/4/4 exactly (the 4 known
   parked-255 reds; no new).
3. `cargo test --release --lib -p wat` → 943/0/1.
4. `cargo test --release --test wat_arc170_slice_1f_alpha_helpers` → 12/0/0.
5. `cargo check --all-targets` → 0 errors.
6. `cargo clippy --release --lib -p wat` → zero findings in src/services/.

## STOP triggers (rejection criteria)

- STOP-1: any item in thread_io.rs turns out to have a tenant that makes
  the move BEHAVIOR-CHANGING (not just path-changing). Report it; ship
  nothing.
- STOP-2: a test fails for a reason you cannot trace to the path sweep.

## Constraints

- Behavior-identical: NO logic edits, NO renames of items (paths change,
  names do not), NO doc rewrites beyond what the move + truth demand.
- Commit NOTHING — the orchestrator scores, then commits.
- The four stone-probe files (81/81b/82/82w) are read-only ground truth.
- Work only in `/home/watmin/work/holon/wat-rs/`.
