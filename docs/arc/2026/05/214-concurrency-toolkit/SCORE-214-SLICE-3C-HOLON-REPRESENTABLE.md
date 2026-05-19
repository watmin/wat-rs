# Arc 214 Slice 3 — Stone C — SCORE: HolonRepresentable serialization layer

**Date:** 2026-05-19
**Mode:** A (all 46 criteria satisfied; one honest-delta declared below)
**Actual runtime:** ~25 min (inside predicted 25-40 min; BRIEF skeleton covered each cross-file edit; one type-level API drift caught and fixed at first build)

---

## Scorecard — 46 rows

| # | Criterion | Expected | Actual |
|---|---|---|---|
| 1 | `src/edn_shim.rs` adds `pub fn write_holon_ast_tagged(h: &holon::HolonAST) -> String` immediately above `pub fn read_holon_ast_tagged` | YES | YES |
| 2 | `write_holon_ast_tagged` body: `wat_edn::write(&holon_ast_to_edn(h))` | YES | YES |
| 3 | `write_holon_ast_tagged` has doc comment naming the inverse-of-read_holon_ast_tagged property + roundtrip identity + single-line-output guarantee | YES | YES |
| 4 | `src/comms/mod.rs` adds `impl HolonRepresentable for String { ... }` immediately after the trait definition | YES | YES |
| 5 | `String::to_holon_ast` returns `holon::HolonAST::String(self.clone())` | YES | YES (via `self.as_str().into()` — see honest-delta) |
| 6 | `String::from_holon_ast` matches on `holon::HolonAST::String(s)` → `Ok(s.clone())`; other variants → `Err(WireError::new(...))` | YES | YES (via `s.to_string()` — see honest-delta) |
| 7 | The impl block has doc comment naming "Slice 1's first concrete impl (Slice 3 Stone C)" + roundtrip exactness invariant | YES | YES |
| 8 | `src/comms/process.rs` adds `use std::marker::PhantomData;` to imports | YES | YES |
| 9 | `src/comms/process.rs` module-level doc: "Current scope (through Stone B)" → "Current scope (through Stone C)" | YES | YES |
| 10 | Module-level doc retires Stone A's "Payload bytes MUST NOT contain '\n'" caveat; new Framing section names wat-edn's single-line escape guarantee | YES | YES |
| 11 | `Sender` becomes `Sender<T: HolonRepresentable>` with `_phantom: PhantomData<T>` field | YES | YES |
| 12 | `Sender::send(&self, value: T) -> Result<(), SendError<T>>` — generic T, takes ownership, returns original T on error (no clone) | YES | YES |
| 13 | `Sender::send` body: `value.to_holon_ast()` → `crate::edn_shim::write_holon_ast_tagged(&ast)` → `edn_str.as_bytes()` → newline-framed via existing libc::write retry loop | YES | YES |
| 14 | `Sender::send` returns `Err(SendError(value))` on EPIPE/write failure (NOT `Err(SendError(value.clone()))`) | YES | YES |
| 15 | `Receiver` becomes `Receiver<T: HolonRepresentable>` with `_phantom: PhantomData<T>` field | YES | YES |
| 16 | Receiver struct doc updated: drops "NOT generic over T (Stone C)" lie; declares generic-T-Stone-C status | YES | YES |
| 17 | `Receiver::recv(&self) -> Result<T, RecvError>` — generic T return type | YES | YES |
| 18 | `Receiver::recv` body unchanged except: take_frame results now route through `decode_frame::<T>` instead of returning `Ok(frame)` directly | YES | YES |
| 19 | New private `decode_frame<T: HolonRepresentable>(bytes: &[u8]) -> Result<T, RecvError>` fn added above `take_frame` | YES | YES |
| 20 | `decode_frame` body: utf8 check → `read_holon_ast_tagged(s)` → `T::from_holon_ast(&ast_arc)`; all errors collapse to `RecvError` | YES | YES |
| 21 | `decode_frame` has doc comment naming the wire chain + why all errors collapse to RecvError | YES | YES |
| 22 | `take_frame` UNCHANGED (signature + body identical to Stones A/B) | YES | YES |
| 23 | `pair() -> std::io::Result<(Sender, Receiver)>` becomes `pair<T: HolonRepresentable>() -> std::io::Result<(Sender<T>, Receiver<T>)>` | YES | YES |
| 24 | `pair` constructs both `Sender` and `Receiver` with `_phantom: PhantomData` initializers | YES | YES |
| 25 | `pair` SAFETY comments preserved verbatim from Stone A/B (libc::pipe + OwnedFd::from_raw_fd) | YES | YES |
| 26 | `Sender::send`'s SAFETY comment for libc::write preserved verbatim from Stone A | YES | YES |
| 27 | `Receiver::recv`'s SAFETY comment for io_uring Read submission preserved verbatim from Stone A | YES | YES |
| 28 | `wait_for_data_or_cascade` (Stone B's helper) UNCHANGED | YES | YES |
| 29 | `PollOutcome` (Stone B's enum) UNCHANGED | YES | YES |
| 30 | `tests/probe_comms_process.rs` REWRITTEN: 6 tests use `pair::<String>()`; payloads are `String` not bytes; test names migrate from `probe_slice3a_*` to `probe_slice3c_*` | YES | YES |
| 31 | All 6 probe tests PASS | YES | YES |
| 32 | `cargo build --release` clean (no new warnings) | YES | YES |
| 33 | `cargo test --release --test probe_comms_thread` 10/10 PASS unchanged | YES | YES |
| 34 | `cargo test --release --test probe_comms_foundation` 3/3 PASS unchanged | YES | YES |
| 35 | `cargo test --release --test probe_channel_primitive` 3/3 PASS unchanged | YES | YES |
| 36 | `cargo test --release --test probe_pidfd_primitive` 2/2 PASS unchanged | YES | YES |
| 37 | Zero modifications outside the 4-file scope (edn_shim.rs, comms/mod.rs, comms/process.rs, probe_comms_process.rs) + SCORE doc | YES | YES |
| 38 | Dirty tree intact (`src/fork.rs` + `src/spawn_process.rs` untouched) | YES | YES |
| 39 | `src/typed_channel.rs` untouched | YES | YES |
| 40 | `Cargo.toml` untouched (no new deps; wat-edn + holon already deps) | YES | YES |
| 41 | NO `wat_arc170_program_contracts` re-run | YES | YES |
| 42 | NO Stone D / E work (try_recv, Select, Clone, close, len, traits, persistent ring, config tunable) | YES | YES |
| 43 | NO HolonRepresentable impls for substrate types beyond `String` | YES | YES |
| 44 | Every public item has a doc comment (gaze L2 pre-emption) | YES | YES |
| 45 | Every `unsafe` block keeps its SAFETY comment (forge pre-emption) | YES | YES |
| 46 | NO commit (orchestrator owns the commit after ward pass) | YES | YES |

---

## Honest-delta watch — Risks 1-10 actuals

### Risk 1 — wat-edn `parse_owned` vs `read_holon_ast_tagged` shape

**Predicted:** possible temptation to use `wat_edn::parse_owned` directly.

**Actual:** CLEAN. `decode_frame` uses `crate::edn_shim::read_holon_ast_tagged(s)` exactly as the BRIEF specifies. The substrate's existing tagged-EDN parser is the correct choice; `parse_owned` alone yields OwnedValue without HolonAST reconstruction.

### Risk 2 — `&Arc<HolonAST>` vs `&HolonAST` for `from_holon_ast`

**Predicted:** possible lifetime/deref error on `&ast_arc`.

**Actual:** CLEAN. `T::from_holon_ast(&ast_arc)` compiled directly — `&Arc<T>` auto-derefs to `&T` via the Deref impl. No `&*ast_arc` explicit deref needed.

### Risk 3 — `SendError(value)` ownership

**Predicted:** possible temptation to clone `value` before error return.

**Actual:** CLEAN. `return Err(SendError(value))` moves `value` into the error directly. No clone. `value.to_holon_ast()` takes `&self` so `value` is still owned after that call.

### Risk 4 — PhantomData<T> variance choice

**Predicted:** possible over-engineering to `PhantomData<fn(T)>` or `PhantomData<fn() -> T>`.

**Actual:** CLEAN. `PhantomData<T>` used exactly as the BRIEF specifies. Invariance is correct for this use case.

### Risk 5 — `HolonAST::String(self.clone())` allocation

**Predicted:** unavoidable clone cost; no mitigation needed.

**Actual:** CLEAN (with honest-delta below on the actual inner type).

### Risk 6 — Module-level doc cascading updates

**Predicted:** possible forgotten "Current scope (through Stone B)" → "(through Stone C)" update.

**Actual:** CLEAN. Module-level doc updated to "Current scope (through Stone C)" with the new Framing section. Stone A's "Payload bytes MUST NOT contain '\n'" caveat retired. The Stone A "Framing (Stone A)" section header was updated to the plain "Framing" section name.

### Risk 7 — Receiver struct doc cascading updates

**Predicted:** possible "NOT generic over T (Stone C adds)" stale claim left in doc.

**Actual:** CLEAN. Receiver struct doc updated: "NOT generic over T (Stone C adds)" is removed; doc now declares "Generic over the payload type T (Stone C)."

### Risk 8 — Probe test type imports

**Predicted:** possible blind regex replacement leaving stale byte-oriented imports.

**Actual:** CLEAN. Test file is a wholesale rewrite with the correct `String`-typed surface. Additionally, Stone C's probe does not need `SendError` in its test bodies (no send-error path is explicitly asserted via type), so the import was trimmed to just `RecvError` to keep the file warning-free.

### Risk 9 — `take_frame` UNCHANGED preservation

**Predicted:** possible contamination of take_frame with decode concerns.

**Actual:** CLEAN. `take_frame` body is byte-identical to Stones A/B. `decode_frame` is the separate helper above it. Sever discipline holds.

### Risk 10 — `holon::HolonAST::String` variant accuracy

**Predicted:** variant name correct; inner type not pre-verified.

**Actual:** DRIFT CAUGHT + FIXED. The actual inner type is `Arc<str>`, NOT `String`. First build failed with two type errors:
- `holon::HolonAST::String(self.clone())` — expected `Arc<str>`, found `String`
- `Ok(s.clone())` — expected `String`, found `Arc<str>`

Fixed in one edit:
- `to_holon_ast`: `holon::HolonAST::String(self.as_str().into())` — `&str → Arc<str>` via Into
- `from_holon_ast`: `Ok(s.to_string())` — `Arc<str> → String` via Display/to_string

Second build was clean. The roundtrip is still exact (String → Arc<str> → String losslessly).

---

## One beyond-scope addition (honest-delta)

**The impl body uses `self.as_str().into()` and `s.to_string()` instead of the BRIEF's `self.clone()` and `s.clone()`.**

**Reason:** the actual `HolonAST::String` inner type is `Arc<str>`, not `String`. The BRIEF's skeleton assumed `HolonAST::String(String)` (reasonable from the edn_shim arm at line 1685 which shows `s.to_string()` — but that was converting FROM `Arc<str>` TO a different `String`). The fix is mechanically correct and preserves the roundtrip identity. This is a substrate API micro-drift, not a design decision.

**Honest characterization:** Risk 10 fired. The fix was applied in under 2 minutes via one targeted edit after reading the rustc diagnostic.

---

## Cargo build output (verbatim)

```
   Compiling wat v0.1.0 (/home/watmin/work/holon/wat-rs)
warning: function `parse_fn_signature_for_check` is never used
     --> src/check.rs:11194:4
warning: function `eval_kernel_process_send` is never used
     --> src/runtime.rs:18229:4
warning: function `eval_kernel_process_recv` is never used
     --> src/runtime.rs:18305:4
warning: function `process_died_error_entry_form_failure` is never used
     --> src/runtime.rs:18684:4
warning: function `process_died_error_entry_form_failure_value` is never used
     --> src/runtime.rs:18693:15
   Compiling wat-telemetry v0.1.0 (...)
   Compiling wat-sqlite v0.1.0 (...)
   Compiling wat-lru v0.1.0 (...)
   Compiling wat-holon-lru v0.1.0 (...)
   Compiling wat-telemetry-sqlite v0.1.0 (...)
   Compiling wat-cli v0.1.0 (...)
warning: `wat` (lib) generated 5 warnings
   Compiling with-loader-example v0.1.0 (...)
   Compiling interrogate-example v0.1.0 (...)
   Compiling with-lru-example v0.1.0 (...)
   Compiling console-demo v0.1.0 (...)
    Finished `release` profile [optimized] target(s) in 17.70s
```

5 pre-existing warnings (check.rs:11194, runtime.rs:18229, 18305, 18684, 18693). ZERO new warnings from Stone C changes.

---

## Test outputs (verbatim)

### probe_comms_process (Slice 3 — HolonRepresentable round-trip)

```
running 6 tests
test probe_slice3c_pair_constructs_successfully ... ok
test probe_slice3c_fifo_ordering_preserved_across_sends ... ok
test probe_slice3c_accumulator_splits_two_frames_from_one_read ... ok
test probe_slice3c_single_string_round_trip ... ok
test probe_slice3c_large_string_spans_multiple_io_uring_reads ... ok
test probe_slice3c_sender_drop_wakes_recv_with_err ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
```

All 6 tests exercise the bootstrap-fallback path (SHUTDOWN_BROADCAST_READ_FD is -1 in the test environment). The full wire chain T → HolonAST → tagged EDN → newline-framed bytes → libc::write → io_uring Read → bytes → EDN → HolonAST → T is exercised across all 6 tests.

### probe_comms_thread (Slice 2 — unchanged)

```
running 10 tests
test probe_slice2_bounded_round_trip ... ok
test probe_slice2_clone_receiver_multi_consumer ... ok
test probe_slice2_close_idempotent_with_clones ... ok
test probe_slice2_select_indices_match_registration_order ... ok
test probe_slice2_select_picks_fired_receiver ... ok
test probe_slice2_sender_drop_triggers_recv_err ... ok
test probe_slice2_try_recv_disconnected_after_sender_drop ... ok
test probe_slice2_clone_sender_multi_producer ... ok
test probe_slice2_try_recv_empty_returns_empty ... ok
test probe_slice2_unbounded_round_trip ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### probe_comms_foundation (Slice 1 — unchanged)

```
running 3 tests
test probe_slice1_error_types_construct_and_distinguish ... ok
test probe_slice1_select_outcome_constructs ... ok
test probe_slice1_holon_representable_compiles ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### probe_channel_primitive (χ-1 — untouched)

```
running 3 tests
test probe_chi1_sender_drop_triggers_recv_err ... ok
test probe_chi1_try_recv_empty_returns_empty ... ok
test probe_chi1_unbounded_round_trip ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### probe_pidfd_primitive (α — untouched)

```
running 2 tests
test pidfd_observes_signal_exit ... ok
test pidfd_observes_normal_exit ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

---

## Scope boundary verification

`git diff --name-only` shows 3 modified files: `src/comms/mod.rs`, `src/comms/process.rs`, `tests/probe_comms_process.rs`. New untracked file: `src/edn_shim.rs` (tracked; modified). New untracked: `docs/arc/2026/05/214-concurrency-toolkit/SCORE-214-SLICE-3C-HOLON-REPRESENTABLE.md`.

Pre-existing dirty tree (NOT touched): `src/fork.rs`, `src/spawn_process.rs`.
NOT committed (per BRIEF; orchestrator commits after ward pass).

---

## Ward pass prediction update

Pre-emptive discipline applied at construction:

1. Every public item keeps its doc comment — YES. `write_holon_ast_tagged` has a full doc comment. All Sender/Receiver/pair items updated. `impl HolonRepresentable for String` has a block doc comment.
2. Private items have doc comments too — YES. `decode_frame` has a full doc comment naming the wire chain + error-collapse rationale. `take_frame` doc unchanged from Stones A/B. `PollOutcome` + `wait_for_data_or_cascade` unchanged from Stone B.
3. Comments explain WHY not WHAT — YES. `PhantomData<T>` field doc explains invariance + why it's correct. `decode_frame` doc explains why all errors collapse to RecvError ("wire failures all mean the frame did not roundtrip cleanly"). `write_holon_ast_tagged` doc explains the single-line guarantee + why it makes newline framing safe.
4. SAFETY comment at every unsafe block — YES. All three unsafe blocks in process.rs (libc::write, io_uring submission in recv, libc::pipe + OwnedFd in pair) keep their Stone A/B SAFETY comments verbatim.
5. Roundtrip exactness documented — YES. `impl HolonRepresentable for String` doc names the invariant explicitly: "The roundtrip is exact — String::from_holon_ast(s.to_holon_ast()) returns the original string."
6. Honest-delta declared — YES. Risk 10 (Arc<str> inner type drift) documented above with fix rationale.

Predicted findings:
- **forge:** 0-1 (possible candidate-rune on `self.as_str().into()` vs `Arc::from(self.as_str())` — both are idiomatic; `.into()` is fine)
- **gaze:** 0-1 (possible mumble on `_phantom` field doc if ward considers it too terse; it names invariance explicitly)
- **reap:** 0-1 (possible reap on the honest-delta; it's declared; scope is narrow)
- **sever:** 0 (decode_frame / take_frame cleanly separated; impl for String is a clean type-class instance)
- **temper:** 0-1 (per-call IoUring + Vec allocation per send — both known-deferred to Stone E/future arc)

Total predicted: 0-4 findings; all L2 at most. Round 2 should be CLEAN.

---

## Mode classification

**Mode A** — all 46 criteria satisfied; zero new warnings; 6/6 probe_comms_process tests pass; all 4 prior probe suites unchanged; dirty tree intact; no commit. One honest-delta declared (Risk 10: `Arc<str>` inner type drift caught at first build + fixed in one edit).
