# Arc 113 — INSCRIPTION

## Status

Shipped 2026-04-30. Cascading runtime errors as
`Vec<*DiedError>` chains — both intra-process (cross-thread via
crossbeam) and inter-process (cross-fork via EDN-on-stderr).
`cargo test --release` green throughout slices.

Pushed:
- Slice 1: `cb0d266` (wire-shape — Err arm widens to `Vec<*DiedError>`)
- Slice 2: `61ba64c` (`result::expect` carries upstream chain through panic)
- Slice 3: `cebd640` (cross-fork cascade via `#wat.kernel/Panics` stderr marker)
- Closure naming + raise!: `6f14a91` (Panics → ProcessPanics, raise! verb)
- Slice 4 / closure docs: this INSCRIPTION + USER-GUIDE + 058 row

DESIGN evolution: started as "Vec<ProgramDiedError> chained-cause
backtrace, lands AFTER arc 109 § J slice 10d." Mid-arc decision —
ship the chain shape NOW under the concrete `*DiedError` types
(`ThreadDiedError`, `ProcessDiedError`); arc 109 § J widens the
element type to the supertype `ProgramDiedError` later via the
typeclass dispatch slice 10d.

Architectural through-line that emerged during slice 3:

> threads pass DiedError values through crossbeam (zero-copy);
> processes pass them as EDN over kernel pipes; the chain shape
> at the caller surface (`Result<R, Vec<*DiedError>>`) is
> identical regardless of transport. only the wire differs.

Captured in commit messages, in the queue.wat typealias header,
in src/check.rs's `arc_113_migration_hint`, and in
docs/USER-GUIDE.md §13 (cascade chains).

## What this arc adds

Three layered substrate changes plus one user-facing verb:

| Layer | What | Where |
|---|---|---|
| Wire shape | `Result<R, ThreadDiedError>` → `Result<R, Vec<ThreadDiedError>>` (and `ProcessDiedError` parallel) | `comm_send_ret`, `comm_ok_option_t`, `join-result`, `Process/join-result`, `process-send`, `process-recv` schemes |
| Thread cascade | `result::expect` on Err carries the inherited chain through the panic; spawn driver conjs this thread's death onto the front | `AssertionPayload.upstream_chain`, `eval_kernel_join_result`, `eval_kernel_process_join_result` |
| Fork cascade | Child renders chain to `#wat.kernel/ProcessPanics {edn}` on stderr; parent's `extract-panics` walks stderr-lines + parses; drive-sandbox prefers the parsed chain over the singleton | `emit_panics_to_stderr` (fork.rs), `eval_kernel_extract_panics` (runtime.rs), drive-sandbox (sandbox.wat), drive-hermetic (hermetic.wat) |
| User-facing verb | `:wat::kernel::raise!` panics with EDN-rendered HolonAST as `Failure.message`; receivers recover via `:wat::edn::read` | `eval_kernel_raise` (runtime.rs) |

### Cascade Vec — the data IS the chain

Pre-arc-113:

```scheme
((Err :wat::kernel::ThreadDiedError)
  ;; one error — the immediate peer that died
  (handle-err err))
```

Post-arc-113:

```scheme
((Err died-chain)
  ;; died-chain :Vec<:wat::kernel::ThreadDiedError>
  ;; head = the immediate peer that died
  ;; tail = whatever killed it, recursively, across hosts
  (handle-chain died-chain))
```

`(:wat::core::first chain)` answers "what just died." Walking the
Vec answers "what killed it, transitively, across host
boundaries." Nothing in between is lost.

Slice 1 ships singleton chains (always 1 element); slice 2 wires
the conj-on-panic mechanism for the thread side; slice 3 wires
the EDN-on-stderr framing for the process side.

### `:wat::kernel::ProcessPanics` / `:wat::kernel::ThreadPanics`

Typealiases for the cascade Vec at each host kind:

```scheme
(:wat::core::typealias :wat::kernel::ProcessPanics
  :Vec<wat::kernel::ProcessDiedError>)

(:wat::core::typealias :wat::kernel::ThreadPanics
  :Vec<wat::kernel::ThreadDiedError>)
```

Bindings type-annotate against the named alias rather than the
verbose `Vec<wat::kernel::*DiedError>` form. Once arc 109 § J
slice 10d lands, the supertype `ProgramPanics` will be satisfied
by both — same chain shape from the caller's vantage; the
per-host concrete name is what surfaces today.

Marker tag on stderr renamed to match: `#wat.kernel/ProcessPanics`
(was `#wat.died/chain` in initial slice-3 placeholder, then
`#wat.kernel/Panics` in the first pass, then `ProcessPanics` once
the symmetry with ThreadPanics made the per-host distinction
worth carrying).

### `:wat::kernel::raise!` — data-as-payload panic

The user-facing handle for "panic with structured data, not a
string." Takes `:wat::holon::HolonAST`; renders via
`:wat::edn::write`; panics with the rendered string as
`Failure.message`. Receivers recover via:

```scheme
(:wat::edn::read (:wat::kernel::Failure/message f))
```

The architectural insight that simplified the design — and which
the user surfaced mid-implementation:

> i think the message field /is/ the data field you're producing?...
> the rust layer needs to serialize to string.. but /it is edn data/?..

**Failure's `message: String` IS the data field.** Rust's `String`
is the universal serialization; the conceptual content is EDN.
`raise!` renders the data; receivers parse it back. No new field
needed on `Failure` or `AssertionPayload`. No new struct decl.
The existing message slot already carries data, just rendered.

### Substrate-as-teacher migration hint

`arc_113_migration_hint` in `src/check.rs` (`collect_hints` arm)
fires on `TypeMismatch` + `ReturnTypeMismatch` when one side has
the bare `*DiedError` shape and the other has the `Vec<...>`
form. Tells the reader the one-token annotation fix:

```
arc 113 — every `Err` arm carrying a died-error now carries a
CHAIN: `:Result<T, :wat::kernel::ThreadDiedError>` →
`:Result<T, :Vec<:wat::kernel::ThreadDiedError>>` (and the same
for `ProcessDiedError`). The Vec is the cascade — head = the
immediate peer that died; tail = whatever killed that peer,
transitively. Migrate annotations: wrap the died-error in
`Vec<...>` everywhere it appears as a Result Err arg ...
```

Verified via slice-1 fixture sweep — sonnet swept 4 test files
clean from the hint alone, no other context. The substrate-as-
teacher pattern continues to prove itself: structural changes
that ripple across many sites get diagnostics rich enough that
the integ-test for "is the diagnostic teaching well?" is "can a
fresh sonnet sweep the fixtures from the hint output?"

One substrate-as-teacher GAP surfaced during the sweep: when a
wat program embedded in a Rust test runs as a forked child, the
child's check-error hint never reaches the parent test runner —
only the exit code does. Documented as arc-116-territory
follow-up; stderr-EDN forwarding into cargo's failure output is
the natural extension.

## Drop-ordering bug fixed alongside

Pre-existing latent bug surfaced during slice 3 emit work. In
both `child_branch` and `child_branch_from_source` (src/fork.rs),
the wat-side `stderr_writer: Arc<dyn WatWriter>` wraps an
`OwnedFd::from_raw_fd(2)`. When `main_args` drops at end of the
catch_unwind closure, the moved Arc's last reference dies and
`OwnedFd::Drop` closes fd 2. Subsequent `write_direct_to_stderr`
writes hit `EBADF`.

Fix: clone the Arc before moving into `main_args`, hold the
clone past `catch_unwind`. Refcount stays ≥ 1 through the panic
arm's writes; fd 2 stays open until `_exit`.

This bug was latent because no test exercised post-panic stderr
writes from the fork path — slice 3's marker emit was the first.

```rust
let stderr_keepalive = Arc::clone(&stderr_writer);
let main_args = vec![..., Value::io__IOWriter(stderr_writer), ...];
let outcome = std::panic::catch_unwind(...);
let _ = &stderr_keepalive; // borrow-check: clone held through here
match outcome { ... }  // post-panic writes work; fd 2 still open
```

## EDN Option round-trip fixed alongside

Slice 3's stderr-EDN path surfaced a wat-edn round-trip
incompleteness: the writer unwraps `Value::Option(Some(X))` →
bare X on the wire (and `None` → `Nil`) for compactness. Without
type-aware re-wrapping on read, struct fields declared
`Option<T>` came back as bare values — pattern matches against
`(Some _)` / `(:None _)` failed.

Fix in `src/edn_shim.rs`:

```rust
fn rewrap_option_field(fty: &TypeExpr, v: Value) -> Value {
    let is_option = matches!(
        fty,
        TypeExpr::Parametric { head, .. } if head == "Option"
    );
    if !is_option { return v; }
    match v {
        Value::Option(_) => v,
        Value::Unit => Value::Option(Arc::new(None)),
        other => Value::Option(Arc::new(Some(other))),
    }
}
```

`reconstruct_struct` and `reconstruct_enum_tagged` consult
declared field types and re-wrap Option layers during bridge.
Without this, `Failure.actual` / `.expected` / `.location`
came back as bare values on the cross-fork path and the
post-panic flow lost the structured assertion data.

## What this arc lights up

The use case from DESIGN.md "Cross-host test failure
diagnostics" — verified end-to-end via
`tests/wat_arc113_cross_fork_cascade.rs`:

```scheme
(:wat::test::assert-eq 1 2)   ; inside hermetic-forked child
```

surfaces on the parent's `RunResult.failure`:

```
Failure {
  message: "assert-eq failed"
  actual:   "1"
  expected: "2"
  location: <entry>:12:19
  frames:   [<entry>:12 → :wat::test::assert-eq, ...]
}
```

Pre-arc-113-slice-3, those were `:None` / `:None` / `forked
program exited 2`. The structured AssertionPayload arc 064
preserved across thread boundaries now also crosses fork
boundaries.

## Known limitations / deferred

- **Multi-element chains across host transitions.** The slice 1+2
  wire makes `Vec<DiedError>` chains expressible; slice 2 proves
  multi-element accumulation across spawn-thread layers
  (`join_result_cascade_accumulates_chain_across_two_levels`).
  Slice 3 ships SINGLE-element process chains (the head's
  AssertionPayload is preserved; tail propagation across `recv`
  on a dead-peer-pipe is arc 111 slice 2 territory — the
  death-aware recv mechanism deferred at arc 111 closure).

- **`ProgramPanics` supertype** — the unified element type that
  lets a chain hold mixed Thread/Process deaths. Arc 109 § J
  slice 10d work; INVENTORY rows updated with the rename
  symmetry (ProcessPanics / ThreadPanics today; ProgramPanics
  satisfied-by-both post-§J).

- **Test runner integration** — cargo-test's failure output
  doesn't yet render the chain (it shows only the head's
  message). Arc 116 territory: extending test_runner's
  Failure→Diagnostic walker to render the full Vec.

- **Non-AssertionPayload panic chain emission.** The child's
  emit only fires when the panic payload is an AssertionPayload
  (the only path that carries upstream_chain). Plain `panic!()`
  and runtime errors fall back to the singleton "exited N"
  shape. Future-arc work to widen, when callers demand.

## Cross-references

- `docs/arc/2026/04/109-kill-std/INVENTORY.md` § J — ProgramPanics
  supertype noted as part of the ProgramDiedError typeclass slice
  (10d).
- `docs/arc/2026/04/111-result-option-recv/INSCRIPTION.md` —
  arc 111 slice 1's placeholder `Err(ChannelDisconnected)` arc 113
  generalizes to chain-bearing Err.
- `docs/arc/2026/04/112-inter-process-result-shape/INSCRIPTION.md`
  — arc 112's `Process<I,O>` unification + ProcessDiedError; arc
  113 widens both DiedError types' Err arms to chains.
- `docs/arc/2026/04/116-phenomenal-cargo-debugging/INSCRIPTION.md`
  — arc 116's Diagnostic + WAT_TEST_OUTPUT plumbing arc 113 leans
  on (the chain renders structurally through the diagnostic
  layer).

## Verification

End-to-end proofs:

| Test | What it proves |
|---|---|
| `runtime::tests::join_result_cascade_accumulates_chain_across_two_levels` (slice 2) | Inner thread panics → outer thread `result::expect`s on Err → chain comes back as `[outer_TDE, inner_TDE]` in causality order |
| `tests/wat_arc113_emit_probe.rs::child_assertion_writes_died_chain_to_stderr` (slice 3) | Forked child's panic emits `#wat.kernel/ProcessPanics ...` line on stderr with full structured Failure |
| `tests/wat_arc113_cross_fork_cascade.rs::hermetic_assertion_failure_preserves_actual_and_expected` (slice 3) | Hermetic-forked assert-eq surfaces actual/expected on parent's RunResult.failure (was `:None`/`:None` pre-arc-113-slice-3) |
| `tests/wat_arc113_raise_round_trip.rs::raise_data_round_trips_through_failure_message` (closure) | `raise!` of HolonAST → run-sandboxed → `:wat::edn::read` on Failure.message recovers a HolonAST Value (not a string) |

The substrate-as-teacher pattern: the migration hint's text was
the only context a fresh sonnet needed to sweep slice 1's
fixture sites. Verified by direct delegation in commit `cb0d266`.
