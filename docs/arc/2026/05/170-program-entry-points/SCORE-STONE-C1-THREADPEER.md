# Arc 170 Stone C1 SCORE — `ThreadPeer<I, O>` + `Thread/readln` + `Thread/println`

**BRIEF:** `BRIEF-STONE-C1-THREADPEER.md`
**EXPECTATIONS:** `EXPECTATIONS-STONE-C1-THREADPEER.md`

## Scorecard

| Row | What | Status | Evidence |
|-----|------|--------|----------|
| A | `:wat::kernel::ThreadPeer<I, O>` substrate type registered | **YES** | `src/types.rs:951` registers `TypeDef::Struct(StructDef { name: ":wat::kernel::ThreadPeer", type_params: vec!["I", "O"], fields: [rx: Receiver<I>, tx: Sender<O>] })` adjacent to `:wat::kernel::Thread<I,O>` (mirrors the precedent at `src/types.rs:885`). |
| B | `:wat::kernel::Thread/readln` + `:wat::kernel::Thread/println` verbs registered (eval handlers + dispatch arms + type signatures) | **YES** | Type schemes: `src/check.rs:13074` (`Thread/readln`) + `src/check.rs:13083` (`Thread/println`). Dispatch arms: `src/runtime.rs:4511` (`Thread/readln`) + `src/runtime.rs:4514` (`Thread/println`). Eval handlers: `eval_kernel_thread_readln` at `src/runtime.rs:17261` + `eval_kernel_thread_println` at `src/runtime.rs:17319`. Shared peer-struct unwrap: `eval_thread_peer_struct` at `src/runtime.rs:17367`. |
| C | Substrate-internal helper exists for test peer-pair construction (used by tests) | **YES** | `pub fn make_thread_peer_pair_for_test() -> (Value, Value)` at `src/typed_channel.rs:552`. Test sites: `tests/wat_arc170_stone_c1_threadpeer.rs:85` + `:128`. The `_for_test` suffix marks the substrate-vs-user boundary on every grep — Stone D's bracket macro will be the user-facing peer-pair constructor. |
| D | 3 new tests pass (type mint + verb dispatch + type-param swap) | **YES** | `cargo test --release -p wat --test wat_arc170_stone_c1_threadpeer` → `test result: ok. 3 passed; 0 failed; 0 ignored`. Tests: `stone_c1_thread_peer_type_mint_both_orientations_type_check`, `stone_c1_thread_peer_verb_dispatch_round_trips_i64`, `stone_c1_thread_peer_type_param_swap_both_directions_round_trip`. |
| E | Workspace test failure count ≤ baseline (3 stable + lifeline flake) | **YES** | Per-target counts after Stone C1: `wat_arc170_program_contracts` 23 pass / 1 fail (t6 unquote, pre-existing — baseline match), `wat::test` 176 pass / 1 fail (totally_bogus, pre-existing — baseline match), `wat-cli::wat_cli` 14 pass / 1 fail (startup_error, pre-existing — baseline match), `probe_lifeline_pipe_proof` flaked once then re-ran clean (1/100 baseline flake band). NEW: `wat_arc170_stone_c1_threadpeer` 3/0 (purely additive). Workspace cargo summary `error: 4 targets failed` is the SAME target list as baseline (`probe_lifeline_pipe_proof`, `test`, `wat_arc170_program_contracts`, `wat_cli`). NO new failures introduced. |

**5/5 PASS.**

## Honest deltas

### ThreadPeer location

**Chosen: `src/types.rs` adjacent to `:wat::kernel::Thread<I,O>` (line 911 → new block at 911-971).** Mirror of the Stone A pattern — Thread<I,O>'s `TypeDef::Struct(StructDef)` registration was the precedent, ThreadPeer<I,O> is the natural neighbor. The alternative considered (new module `src/thread_peer.rs` re-exported from `lib.rs`, mirror of arc 198 slice 2 Stone 1's `restriction_entry.rs`) was rejected for two reasons:

1. **Scope didn't warrant.** ThreadPeer is a 2-field type with auto-generated accessors. arc 198's RestrictionEntry needed a full ledger struct + serialization/format conversion logic that justified its own module. ThreadPeer fits in the existing struct-registration block alongside Thread.
2. **Adjacent reading wins.** A future reader investigating Thread<I,O> finds ThreadPeer<I,O> in the same Read range. Splitting them into separate files would have hidden the kinship.

### Field composition shape

**Chosen: explicit `Receiver<I>` + `Sender<O>` typed-channel field types.** Per EXPECTATIONS option (1) — clean correspondence between the type params and the field types, with the fields named `rx` / `tx` matching the directional intent. Option (2) (opaque `read_end` / `write_end`) was rejected — the field types ARE the typed-channel substrate (`rust::crossbeam_channel::Receiver` / `rust::crossbeam_channel::Sender`) the runtime is already wired for; hiding them behind opaque names would obscure how Stone D's bracket wires two pairs together.

The auto-generated `:wat::kernel::ThreadPeer/new`, `:wat::kernel::ThreadPeer/rx`, `:wat::kernel::ThreadPeer/tx` accessors land via the existing `register_struct_methods` machinery (`src/runtime.rs:1879`). Stone C1 does NOT exercise those accessors from wat-level code — the substrate verbs reach into the struct by field index — but they exist for future stones (Stone D's macro expansion will use the rx/tx accessor names; tests/diagnostics can introspect).

### Internal pipe-wiring helper

**Chosen: Rust-only helper `pub fn make_thread_peer_pair_for_test() -> (Value, Value)` in `src/typed_channel.rs`.** Per EXPECTATIONS prediction. Constructs two crossbeam-channel pairs, builds two `Value::Struct` peer instances with the cross-wired field assignments, returns the tuple. NOT exposed to wat. The `_for_test` suffix is the explicit boundary marker — Stone D's bracket macro is the user-facing path; this helper exists only because Stone C1's tests need peer pairs before that macro lands.

Helper API:

```rust
pub fn make_thread_peer_pair_for_test() -> (Value, Value) {
    let (tx_ab, rx_ab) = crossbeam_channel::unbounded::<Value>();  // A→B
    let (tx_ba, rx_ba) = crossbeam_channel::unbounded::<Value>();  // B→A
    let peer_a = Value::Struct(Arc::new(StructValue {
        type_name: ":wat::kernel::ThreadPeer".into(),
        fields: vec![receiver_from_crossbeam(rx_ba), sender_from_crossbeam(tx_ab)],
    }));
    let peer_b = Value::Struct(Arc::new(StructValue {
        type_name: ":wat::kernel::ThreadPeer".into(),
        fields: vec![receiver_from_crossbeam(rx_ab), sender_from_crossbeam(tx_ba)],
    }));
    (peer_a, peer_b)
}
```

The wiring matches BRIEF § "Internal pipe wiring": peer A reads what B writes; peer B reads what A writes. Type erasure at the runtime layer — `Value` ferries the payloads, the type checker enforces I/O alignment.

### Disconnect / panic semantics on the two verbs

**Chosen: substrate-level `RuntimeError::ChannelDisconnected` on partner-gone.** Per INTERSTITIAL-REALIZATIONS § "Link semantics" (the user's "we are lock step - forks are servers - their clients went away - that is a panic event" framing) — same-universe peer death is a panic event. Surfacing through `RuntimeError::ChannelDisconnected` lets the existing spawn-driver `catch_unwind` machinery propagate the cause through the parent's `Thread/drain-and-join`; no special wrapping is needed at the peer surface (`-> :I` and `-> :wat::core::nil` are the verb return types per BRIEF, NOT `Option<I>` / `Result<(), ...>`).

Decode errors on a tier-2 (PipeFd-backed) transport surface as `RuntimeError::MalformedForm` — matches the `:wat::kernel::recv` discipline at `src/runtime.rs:15880`.

### arc 117/133 walker interaction

**No interaction observed.** The three new tests construct peers via the Rust-side helper (`make_thread_peer_pair_for_test`) and bind them as opaque `Value::Struct` bindings into the eval Environment. Wat-level code in the tests never declares Sender / Receiver bindings as siblings, so the existing sibling-binding walker (still active until Stone G) doesn't have a Sender-bearing / Receiver-bearing siblings situation to fire on. The walker rule was designed around `(:wat::kernel::Thread/input thr)` + `(:wat::kernel::Thread/output thr)` style let-bindings; Stone C1's peer surface bypasses that pattern entirely.

If a future stone (Stone D's bracket macro, in particular) generates wat code that names the rx/tx fields of a ThreadPeer as siblings, the walker may need to learn about ThreadPeer as a Sender-bearing / Receiver-bearing binding. That's Stone G's scope (retire the walker machinery), not Stone C1's.

### Workspace test count vs baseline

| Target | Baseline | Post-Stone-C1 | Delta |
|---|---|---|---|
| `wat::wat_arc170_stone_c1_threadpeer` (NEW) | (did not exist) | **3 passed / 0 failed** | +3 passes |
| `wat::wat_arc170_program_contracts` | 23 pass / 1 fail (t6) | 23 pass / 1 fail (t6) | unchanged |
| `wat::test` (lib stdlib tests) | 176 pass / 1 fail | 176 pass / 1 fail | unchanged |
| `wat::probe_lifeline_pipe_proof` | flake band 1/100 | flaked then re-ran clean (band match) | unchanged |
| `wat-cli::wat_cli` | 14 pass / 1 fail | 14 pass / 1 fail | unchanged |

Net: **+3 new passes; 0 new failures.** Workspace summary `error: 4 targets failed` matches baseline target list verbatim.

### Substrate-discovery surprises

**Zero surprises.** The existing precedent stack carried Stone C1 cleanly:

1. **Struct registration pattern** — Thread<I,O> at `src/types.rs:885` is the exact template for ThreadPeer<I,O>. No new TypeDef variant; the existing `TypeDef::Struct(StructDef)` machinery handles parametric structs.
2. **Verb type-scheme pattern** — `Thread/drain-and-join` at `src/check.rs:13029` is the template for `Thread/readln` + `Thread/println` (substrate verb taking a struct + returning a parametric expression in I/O).
3. **Runtime eval pattern** — `eval_kernel_thread_drain_and_join` at `src/runtime.rs:17127` is the structural template for `eval_kernel_thread_readln` + `eval_kernel_thread_println` (unwrap struct → extract field by index → delegate to `typed_recv` / `typed_send`).
4. **Typed-channel composition** — `sender_from_crossbeam` + `receiver_from_crossbeam` (`src/typed_channel.rs:126` + `:137`) are the existing constructors for the field values; `make_thread_peer_pair_for_test` just stacks two pair allocations and builds two struct values.

Build time: ~58s for the test-build; ~62s for the full workspace test-build. Workspace test wall time ~3-5 min. Both within prediction.

## Calibration record

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 30–45 min | ~35 min |
| Scorecard rows | 5/5 PASS | 5/5 PASS |
| Workspace fail count | ≤ baseline | = baseline (4 pre-existing target failures unchanged; lifeline flake stayed inside its 1/100 band) |
| New test count | 3 | 3 |
| ThreadPeer location | `src/types.rs` OR new module | `src/types.rs` adjacent to Thread<I,O> (lines 913-971) |
| Helper shape | Rust-only OR wat-exposed | Rust-only (`pub fn make_thread_peer_pair_for_test`); explicit `_for_test` suffix |
| Walker interaction surprises | 0–1 | 0 |
| Substrate-discovery surprises | 0–2 | 0 |
| Mode | Additive | Additive (new struct + 2 new verbs + 1 new Rust helper + 3 new tests; zero modifications to Thread<I,O> / spawn-thread / Thread/join-result / Thread/drain-and-join) |

## STOP triggers encountered

**None.** Sender<T> / Receiver<T> typed-channel substrate composed cleanly into the ThreadPeer struct shape. ThreadPeer fields parameterize correctly through the existing `TypeDef::Struct(StructDef { type_params, fields })` mechanism. The arc 117/133 sibling-binding walker did not fire on the test fixture (peers constructed via Rust helper, never as wat-level let-bindings of Sender + Receiver siblings).

## What's ready for Stone C2

- ThreadPeer<I, O> precedent established: struct in `src/types.rs`; type schemes for verbs in `src/check.rs`; eval handlers + dispatch arms in `src/runtime.rs`; substrate-internal test helper in `src/typed_channel.rs`.
- ProcessPeer<I, O> (Stone C2) mirrors this template — client-side wrapper around `Process/stdin` (IOWriter) + `Process/stdout` (IOReader), then `Process/readln` + `Process/println` verbs. Process server stays ambient (uses bare `:wat::kernel::readln` / `:wat::kernel::println`).
- Stone D's `run-threads` bracket macro can now reference `:wat::kernel::ThreadPeer<I, O>` in its expansion + use `:wat::kernel::Thread/readln` / `:wat::kernel::Thread/println` as the per-peer verbs.

## Files touched

- `src/types.rs` — added `:wat::kernel::ThreadPeer<I, O>` struct registration (lines 913-971)
- `src/check.rs` — added type schemes for `:wat::kernel::Thread/readln` + `:wat::kernel::Thread/println` (lines 13045-13090)
- `src/runtime.rs` — added dispatch arms (lines 4509-4516) + eval handlers `eval_kernel_thread_readln` (line 17261), `eval_kernel_thread_println` (line 17319), `eval_thread_peer_struct` (line 17367)
- `src/typed_channel.rs` — added `pub fn make_thread_peer_pair_for_test` (line 552)
- `tests/wat_arc170_stone_c1_threadpeer.rs` — new test file with 3 tests (type mint + verb dispatch + type-param swap)
