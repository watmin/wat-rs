# Arc 214 Slice 3 Stone C — WARD PASS

Per the kernel-impeccability protocol (INTERSTITIAL § 2026-05-19): per-stone trust gate = BRIEF scorecard verification + ward pass before commit. This doc captures the ward round-trip for Stone C (HolonRepresentable serialization layer).

5 wards (gaze + forge + reap + sever + temper) — established protocol from Slice 2 + Stones A+B.

## Round 1 — initial ward pass (after sonnet SCORE Mode A)

Targets across 4 files:
- `src/edn_shim.rs` — new `pub fn write_holon_ast_tagged` (~13 LOC)
- `src/comms/mod.rs` — new `impl HolonRepresentable for String` (~29 LOC)
- `src/comms/process.rs` — generic-T refactor (Sender<T>, Receiver<T>, pair<T>(), decode_frame::<T>, PhantomData<T> markers, module doc + Receiver doc cascading updates)
- `tests/probe_comms_process.rs` — rewritten 6 String-typed tests (`probe_slice3c_*`)

### gaze — CLEAN

> "Module-level doc 'Current scope (through Stone C)' is accurate; no stale 'NOT generic over T' claim. Receiver struct doc uses 'Generic over the payload type T (Stone C)'. write_holon_ast_tagged doc names inverse-of-read + roundtrip identity + single-line guarantee. impl HolonRepresentable for String doc names 'Slice 1's first concrete impl (Slice 3 Stone C)' + roundtrip-exactness invariant + embedded-`\n` edge case. PhantomData<T> field doc explains type-marker rationale. decode_frame doc names the wire chain + error-collapse rationale. All six test names are full imperative sentences."

The risk-10 adaptation (Arc<str> not String) was documented honestly in the impl, NOT copied verbatim from the BRIEF's wrong assumption text. No L1 or L2 findings.

### forge — CLEAN

> "PollOutcome enum: clean ADT, two variants, private, correctly scoped. write_holon_ast_tagged: pure delegation; values in, values out. impl HolonRepresentable for String: `self.as_str().into()` for Arc<str> construction; `s.to_string()` for owned String return; honest variant-mismatch WireError. PhantomData<T> invariant in T (correct default for our use case). Generic-T constraints flow cleanly through Sender<T> + Receiver<T> + pair<T>() + decode_frame<T>. All five SAFETY comments preserved verbatim (libc::pipe, libc::write, OwnedFd::from_raw_fd ×2, io_uring Read submission, plus Stone B's wait_for_data_or_cascade SAFETY block untouched). Risk-10 adaptation is clean (no silent type coercion hiding a defect)."

Both Hickey and Beckman lenses pass on the generic-T refactor. Newtype invariants preserved (Sender<T> / Receiver<T> constructible only via pair<T>()).

### reap — CLEAN

> "write_holon_ast_tagged called from Sender::send. impl HolonRepresentable for String exercised by all 6 probe tests via pair::<String>(). decode_frame::<T> called at TWO return sites in Receiver::recv (fast-path + slow-path). PhantomData<T> initialized at both Sender/Receiver construction sites in pair<T>(). Generic T flows cleanly through Sender<T> → send(T) → SendError<T>; Receiver<T> → recv() → T. take_frame UNCHANGED + still called at two sites. Stone B's wait_for_data_or_cascade + PollOutcome UNCHANGED + still called. Stone A's libc::write retry loop preserved. No legacy bytes API (Sender::send takes T directly; no `&[u8]` call site). No TODO/FIXME/unimplemented! markers. Risk-10 Arc<str> adaptation is REAL CODE, not a half-finished placeholder."

Sonnet's "honest-delta: Risk 10 fired (Arc<str> adaptation)" declaration verified — the adaptation is genuine, the roundtrip is exact, and nothing slipped in beyond the BRIEF.

### sever — CLEAN

> "Five distinct functions; each one concern: write_holon_ast_tagged (text serialization surface); impl HolonRepresentable for String (type-class instance); decode_frame::<T> (wire chain decode); Sender::send (typed send orchestration); Receiver::recv (typed recv orchestration). No braided concerns. write_holon_ast_tagged does NOT touch wire/fd state. impl HolonRepresentable for String does NOT touch wire/fd state. decode_frame does NOT touch the accumulator. Sender::send sequence (encode → frame → write) feeds output to next step, not braided. Receiver::recv structurally unchanged from Stone B; decode_frame::<T> replaces the direct Ok(frame) returns at both take_frame sites. take_frame UNCHANGED (split-first-newline concern, independent of T). wait_for_data_or_cascade UNCHANGED (multi-arm-poll concern, independent of T). PhantomData<T> markers are type-system tools; do NOT introduce a concern. The four-file edit is logically ONE concern (typed serialization wire) decomposed across modules naturally."

The cross-module separation maps cleanly: edn_shim (substrate text surface) + comms::mod (trait + first impl) + comms::process (process-tier consumer) + probe (verification).

### temper — CLEAN

> "Per-call IoUring::new(4) + IoUring::new(2) acknowledged Stone E deferrals. Sender::send 'framed: Vec<u8>' allocation: NOT unintentional waste — POSIX atomicity requires single contiguous write ≤ PIPE_BUF; writev with two iovecs would preserve atomicity but was deferred from Stone A. The framed Vec is LOAD-BEARING for atomicity. String::to_holon_ast `self.as_str().into()` + String::from_holon_ast `s.to_string()` are forced by trait shape (Slice 1 contract). decode_frame uses `from_utf8(bytes)` which is zero-copy. take_frame UNCHANGED. No new redundant computation, no new loop-invariant work, no new per-loop allocations beyond Stone A/B's pattern."

The double-allocation in Sender::send (edn_str + framed) initially looked like a temper candidate but is correctly justified: POSIX atomicity requires contiguous bytes. writev resolution deferred to Stone E.

## Verdict

**STONE C IMPECCABLE — all 5 wards clean on Round 1.**

This is the FIRST stone in arc 214 to land clean on the first ward pass (Slice 2: 9 round-1 findings; Stone A: 2 round-1 findings; Stone B: 3 round-1 findings; Stone C: ZERO round-1 findings). The pre-emption discipline encoded in the BRIEF (10 risk pre-emptions including the cascading doc updates and the SAFETY-comment preservation) worked at construction.

- gaze: code speaks; module + Receiver doc honestly reflect Stone C state; risk-10 adaptation documented honestly (not copy of BRIEF's wrong assumption)
- forge: types enforce contracts (newtype + private fields + sole pair<T>() constructor); SAFETY comments preserved verbatim; PhantomData<T> invariance documented
- reap: zero dead thoughts; every new item alive; risk-10 adaptation is real code
- sever: zero braided concerns; four file edits = one slice-stone concern (typed serialization wire) decomposed across module boundaries
- temper: known deferrals acknowledged; double-allocation justified by POSIX atomicity; no unintentional waste

The kernel-impeccability protocol's per-stone trust gate fires GREEN: BRIEF scorecard Mode A (46/46 satisfied per sonnet's SCORE; one honest-delta on risk 10 — Arc<str> vs String — caught at first cargo build and adapted in <2 min) + ward pass CLEAN on Round 1 (5/5 wards green; no fix pass needed).

After Stone C, the process tier carries fully-typed `HolonRepresentable` payloads. The wire chain `T → HolonAST → tagged-EDN string → newline-framed bytes → io_uring` is end-to-end. Slice 1's `HolonRepresentable` trait has its first concrete impl (`String`), unlocking the pattern for future substrate types.

Stone C ready to commit. Stone D (`try_recv` + `Select<'a, T>` + Clone + close + len + CommSender/CommReceiver trait impls) is the next stepping stone in Slice 3.

## Round-1-clean precedent

Stone C is the first ward-pass-round-1-clean stone in arc 214's kernel-impeccability cycle. This is evidence that:
- Risk pre-emption in BRIEFs compounds across stones (each stone's BRIEF carries lessons from prior wards)
- The 5-ward set is calibrated correctly (no over-fitting; no under-coverage at the typed-serialization level)
- The doc-cascading discipline (Stone B's gaze L1 lesson about stale "(through Stone X)" labels) carried forward correctly to Stone C's "(through Stone C)" update

Future stones may regress to round-1 findings (the substrate keeps changing; new code surfaces new concerns). The signal is the trend: BRIEFs are getting better at pre-empting; ward findings are tightening.

## Cross-references

- BRIEF-214-SLICE-3C-HOLON-REPRESENTABLE.md — work order
- EXPECTATIONS-214-SLICE-3C-HOLON-REPRESENTABLE.md — 46-row scorecard + 10 risk pre-emption
- SCORE-214-SLICE-3C-HOLON-REPRESENTABLE.md — sonnet's Mode A report
- WARD-PASS-3A-IO-URING-BYTES.md — Stone A round-trip (precedent: 2 round-1 findings)
- WARD-PASS-3B-CASCADE-AWARE-MULTI-ARM.md — Stone B round-trip (precedent: 3 round-1 findings; doc-cascading lesson)
- INTERSTITIAL § 2026-05-19 "Kernel impeccability via ward pass" — protocol doctrine
- `src/comms/mod.rs:58-63` — `HolonRepresentable` trait (Slice 1; now has first concrete impl)
- `src/edn_shim.rs:1678` — `holon_ast_to_edn` (PRIVATE; Stone C's new `write_holon_ast_tagged` is its public companion)
- `src/edn_shim.rs:1997` — `read_holon_ast_tagged` (existing PUBLIC; Stone C uses for recv decode)
- `project_holon_universal_ast` — HolonAST as universal substrate form (strange loop closed at the wire layer)
