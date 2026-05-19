# Arc 214 Slice 3 — Stone C — EXPECTATIONS

## Independent prediction

- **Runtime band:** 25-40 min Mode A. Larger than Stone B (more files touched) but still tractable — the BRIEF spells out the full generic-T threading, the new HolonRepresentable impl, the new public edn_shim fn, and the rewritten probe tests. Sonnet's main work: assembly + cross-file consistency.
- **LOC changed:** ~250-330 total (~5 LOC in edn_shim.rs; ~18 LOC in comms/mod.rs; ~150 LOC modified in comms/process.rs; ~110 LOC rewritten in probe_comms_process.rs).
- **New files:** 1 (SCORE doc only).
- **Surprises expected:** LOW-MEDIUM. The wire chain (T → HolonAST → EDN → bytes) is well-established; the BRIEF gives the exact API of each layer. The risk surface is mostly micro-API drift + the generic-T threading discipline.

## Honest-delta watch

### Risk 1 — wat-edn `parse_owned` vs `read_holon_ast_tagged` shape

**What:** The BRIEF uses `crate::edn_shim::read_holon_ast_tagged(s)` returning `Result<Arc<HolonAST>, EdnReadError>`. Sonnet might wonder whether to use `wat_edn::parse_owned` directly. The BRIEF's choice is correct: `read_holon_ast_tagged` is the substrate's existing tagged-EDN parser that knows about `#wat-edn.holon/Symbol` etc.; using `wat_edn::parse_owned` alone gives an OwnedValue without the HolonAST reconstruction.

**Mitigation:** The BRIEF specifies `read_holon_ast_tagged` explicitly + names why (mirror existing public PipeFd write path which uses `holon_ast_to_edn` + `wat_edn::write`; Stone C's `write_holon_ast_tagged` is the symmetric wrapper).

### Risk 2 — `&Arc<HolonAST>` vs `&HolonAST` for `from_holon_ast`

**What:** `read_holon_ast_tagged` returns `Result<Arc<HolonAST>, EdnReadError>`. `T::from_holon_ast` takes `&HolonAST`. The BRIEF's `T::from_holon_ast(&ast_arc)` relies on `&Arc<T>` auto-derefing to `&T` via the Deref impl. This is correct Rust; sonnet shouldn't second-guess it.

**Mitigation:** The BRIEF's `decode_frame` skeleton uses `&ast_arc` directly. If sonnet gets a lifetime error, the workaround is `&*ast_arc` (explicit deref). Either form works.

### Risk 3 — `SendError(value)` ownership

**What:** Stone C's `Sender::send` takes ownership of `value: T`. After `value.to_holon_ast()` returns (which takes `&self`, doesn't consume), `value` is still owned. On write error, `Err(SendError(value))` moves `value` into the error. The borrow from `to_holon_ast` is short-lived — scoped to that call. Sonnet must NOT clone `value` before the write loop (defeats the no-clone advantage over Stone A).

**Mitigation:** The BRIEF's send skeleton uses `return Err(SendError(value))` directly. Sonnet copies.

### Risk 4 — PhantomData<T> variance choice

**What:** `PhantomData<T>` makes the struct invariant in T. The BRIEF's doc-comment names this as "correct for this use case" without elaborating. Sonnet might choose `PhantomData<fn(T)>` (contravariant) or `PhantomData<fn() -> T>` (covariant) thinking they're better. They're not — for sender/receiver of concrete types without lifetime parameters, invariance is fine. Don't over-engineer.

**Mitigation:** The BRIEF says `PhantomData<T>` explicitly. Sonnet copies.

### Risk 5 — `HolonAST::String(self.clone())` allocation

**What:** `String::to_holon_ast(&self)` allocates a clone of self into the new HolonAST. This is unavoidable — `HolonAST::String(String)` owns its contents; `&self` is borrowed. Stone C accepts this allocation as the cost of `to_holon_ast` taking `&self` (Slice 1 trait shape).

**Mitigation:** The BRIEF doesn't ask for optimization here; this is the substrate's chosen shape.

### Risk 6 — Module-level doc cascading updates

**What:** Stone B's module-level doc said "Current scope (through Stone B)" + retired Stone A's "NO cascade-aware" stale text. Stone C must update the SAME section to "Current scope (through Stone C)" + retire Stone B's "NOT generic over T (Stone C)" stale text. The doc-evolution discipline must continue.

**Mitigation:** The BRIEF explicitly lists the doc-update step as Deliverable #3, sub-bullet "Module-level doc update", with the exact replacement text.

### Risk 7 — Receiver struct doc cascading updates

**What:** Stone B's Receiver struct doc was updated to "Cascade-aware (Stone B). NOT Clone (Stone D adds). NOT generic over T (Stone C adds). Per-call IoUring instance (Stone E persistifies)." Stone C must retire the "NOT generic over T (Stone C adds)" claim — generic-T IS now wired.

**Mitigation:** The BRIEF's Receiver skeleton provides updated doc text.

### Risk 8 — Probe test type imports

**What:** The Stone A/B probe imported `wat::comms::process::{pair, Sender, Receiver}` (sonnet then trimmed Sender/Receiver as unused). Stone C's probe imports only `pair` (consistent with the trim). But Stone C's recv returns `String`, not `Vec<u8>` — types are different. Test bodies need careful adaptation, not blind regex replacement.

**Mitigation:** The BRIEF provides the FULL rewritten probe content; sonnet writes the file as a wholesale replacement, not patches.

### Risk 9 — `take_frame` UNCHANGED preservation

**What:** Stone C's `Receiver::recv` body calls `take_frame` then `decode_frame::<T>`. `take_frame` itself is unchanged from Stones A/B — its concern is "split first newline-frame from a Vec<u8>". If sonnet rewrites `take_frame` to take T or to inline decoding, sever-discipline regresses (braids the "split frame" concern with the "decode T" concern).

**Mitigation:** The BRIEF says "`take_frame` UNCHANGED" + provides `decode_frame::<T>` as the separate decode helper.

### Risk 10 — `holon::HolonAST::String` variant accuracy

**What:** The BRIEF's `impl HolonRepresentable for String` uses `holon::HolonAST::String(self.clone())`. If the actual variant is named differently (e.g., `HolonAST::Str` or `HolonAST::StringValue`), it'll be a compile error. Substrate verification (`src/edn_shim.rs:1685`) confirms the variant IS `HolonAST::String(String)`.

**Mitigation:** Variant name confirmed pre-spawn via edn_shim grep. The BRIEF is correct.

## Scorecard predictions

| # | Criterion | Expected |
|---|---|---|
| 1 | `src/edn_shim.rs` adds `pub fn write_holon_ast_tagged(h: &holon::HolonAST) -> String` immediately above `pub fn read_holon_ast_tagged` | YES |
| 2 | `write_holon_ast_tagged` body: `wat_edn::write(&holon_ast_to_edn(h))` | YES |
| 3 | `write_holon_ast_tagged` has doc comment naming the inverse-of-read_holon_ast_tagged property + roundtrip identity + single-line-output guarantee | YES |
| 4 | `src/comms/mod.rs` adds `impl HolonRepresentable for String { ... }` immediately after the trait definition | YES |
| 5 | `String::to_holon_ast` returns `holon::HolonAST::String(self.clone())` | YES |
| 6 | `String::from_holon_ast` matches on `holon::HolonAST::String(s)` → `Ok(s.clone())`; other variants → `Err(WireError::new(...))` | YES |
| 7 | The impl block has doc comment naming "Slice 1's first concrete impl (Slice 3 Stone C)" + roundtrip exactness invariant | YES |
| 8 | `src/comms/process.rs` adds `use std::marker::PhantomData;` to imports | YES |
| 9 | `src/comms/process.rs` module-level doc: "Current scope (through Stone B)" → "Current scope (through Stone C)" | YES |
| 10 | Module-level doc retires Stone A's "Payload bytes MUST NOT contain '\n'" caveat; new Framing section names wat-edn's single-line escape guarantee | YES |
| 11 | `Sender` becomes `Sender<T: HolonRepresentable>` with `_phantom: PhantomData<T>` field | YES |
| 12 | `Sender::send(&self, value: T) -> Result<(), SendError<T>>` — generic T, takes ownership, returns original T on error (no clone) | YES |
| 13 | `Sender::send` body: `value.to_holon_ast()` → `crate::edn_shim::write_holon_ast_tagged(&ast)` → `edn_str.as_bytes()` → newline-framed via existing libc::write retry loop | YES |
| 14 | `Sender::send` returns `Err(SendError(value))` on EPIPE/write failure (NOT `Err(SendError(value.clone()))`) | YES |
| 15 | `Receiver` becomes `Receiver<T: HolonRepresentable>` with `_phantom: PhantomData<T>` field | YES |
| 16 | Receiver struct doc updated: drops "NOT generic over T (Stone C)" lie; declares generic-T-Stone-C status | YES |
| 17 | `Receiver::recv(&self) -> Result<T, RecvError>` — generic T return type | YES |
| 18 | `Receiver::recv` body unchanged except: take_frame results now route through `decode_frame::<T>` instead of returning `Ok(frame)` directly | YES |
| 19 | New private `decode_frame<T: HolonRepresentable>(bytes: &[u8]) -> Result<T, RecvError>` fn added above `take_frame` | YES |
| 20 | `decode_frame` body: utf8 check → `read_holon_ast_tagged(s)` → `T::from_holon_ast(&ast_arc)`; all errors collapse to `RecvError` | YES |
| 21 | `decode_frame` has doc comment naming the wire chain + why all errors collapse to RecvError | YES |
| 22 | `take_frame` UNCHANGED (signature + body identical to Stones A/B) | YES |
| 23 | `pair() -> std::io::Result<(Sender, Receiver)>` becomes `pair<T: HolonRepresentable>() -> std::io::Result<(Sender<T>, Receiver<T>)>` | YES |
| 24 | `pair` constructs both `Sender` and `Receiver` with `_phantom: PhantomData` initializers | YES |
| 25 | `pair` SAFETY comments preserved verbatim from Stone A/B (libc::pipe + OwnedFd::from_raw_fd) | YES |
| 26 | `Sender::send`'s SAFETY comment for libc::write preserved verbatim from Stone A | YES |
| 27 | `Receiver::recv`'s SAFETY comment for io_uring Read submission preserved verbatim from Stone A | YES |
| 28 | `wait_for_data_or_cascade` (Stone B's helper) UNCHANGED | YES |
| 29 | `PollOutcome` (Stone B's enum) UNCHANGED | YES |
| 30 | `tests/probe_comms_process.rs` REWRITTEN: 6 tests use `pair::<String>()`; payloads are `String` not bytes; test names migrate from `probe_slice3a_*` to `probe_slice3c_*` | YES |
| 31 | All 6 probe tests PASS | YES |
| 32 | `cargo build --release` clean (no new warnings) | YES |
| 33 | `cargo test --release --test probe_comms_thread` 10/10 PASS unchanged | YES |
| 34 | `cargo test --release --test probe_comms_foundation` 3/3 PASS unchanged | YES |
| 35 | `cargo test --release --test probe_channel_primitive` 3/3 PASS unchanged | YES |
| 36 | `cargo test --release --test probe_pidfd_primitive` 2/2 PASS unchanged | YES |
| 37 | Zero modifications outside the 4-file scope (edn_shim.rs, comms/mod.rs, comms/process.rs, probe_comms_process.rs) + SCORE doc | YES |
| 38 | Dirty tree intact (`src/fork.rs` + `src/spawn_process.rs` untouched) | YES |
| 39 | `src/typed_channel.rs` untouched | YES |
| 40 | `Cargo.toml` untouched (no new deps; wat-edn + holon already deps) | YES |
| 41 | NO `wat_arc170_program_contracts` re-run | YES |
| 42 | NO Stone D / E work (try_recv, Select, Clone, close, len, traits, persistent ring, config tunable) | YES |
| 43 | NO HolonRepresentable impls for substrate types beyond `String` | YES |
| 44 | Every public item has a doc comment (gaze L2 pre-emption) | YES |
| 45 | Every `unsafe` block keeps its SAFETY comment (forge pre-emption) | YES |
| 46 | NO commit (orchestrator owns the commit after ward pass) | YES |

## Mode classification

- **Mode A:** all 46 criteria satisfied; Stone C shipped clean
- **Mode B (acceptable; honest surface):**
  - Risk 1 fires (sonnet uses wat_edn::parse_owned instead of read_holon_ast_tagged): tests may still pass but the EDN tag handling diverges; orchestrator reviews
  - Risk 2 fires (lifetime/deref error on `&ast_arc`): sonnet uses `&*ast_arc` explicit deref; SCORE notes
  - Risk 4 fires (variance choice): sonnet picks PhantomData<fn(T)> or similar; not strictly wrong but BRIEF specified PhantomData<T>
  - One probe test fails: sonnet investigates per failure mode; reports honestly
- **Mode C (failure):**
  - Touched any file outside the 4-file scope + SCORE doc
  - Touched `src/typed_channel.rs` or the dirty tree
  - Ran `wat_arc170_program_contracts`
  - Committed the work
  - Implemented Stone D / E territory
  - Added HolonRepresentable impl for any substrate type beyond `String`
  - Cloned `value` before `SendError(value)` (loses Stone C's no-clone advantage)
  - Modified `take_frame` (regresses sever discipline)

## Calibration metadata

- **Orchestrator confidence:** MEDIUM-HIGH on first-attempt Mode A. Stone C is bigger than Stones A or B (4 files touched vs 1 file in Stone B), but the BRIEF skeleton covers each cross-file edit explicitly. The risk surface is mostly micro-API checks (variant name, lifetime/deref, parse_owned vs read_holon_ast_tagged choice).
- **Risk factors:**
  - read_holon_ast_tagged shape (Risk 1) — mitigated by explicit BRIEF reference + typed_channel.rs precedent
  - Lifetime/deref micro-issue (Risk 2) — `&Arc<T>` auto-deref; standard Rust; trivial
  - Module doc + Receiver doc cascading updates (Risks 6 + 7) — Stone B's gaze lesson; pre-empted via explicit BRIEF replacement text
- **Why this matters:** Stone C is the SERIALIZATION-LAYER stone. After Stone C, the process tier carries typed payloads; Slice 4's kernel-verb dispatcher can route arbitrary HolonRepresentable values through `:process` peers. The chain `T → HolonAST → EDN → bytes` is THE substrate's universal wire form (per `project_holon_universal_ast` — the strange-loop-closing realization of arc 057+).

## Ward pass prediction

Per the kernel-impeccability protocol: after SCORE verification, 5 wards spawn in parallel.

Predicted findings:
- **gaze:** 0-2 (possible mumble on `_phantom` if doc-comment is too terse; possible L2 on `decode_frame` error-collapse rationale being insufficiently explained)
- **forge:** 0-1 (possible candidate-rune on `PhantomData<T>` invariance choice; possible L2 on `String::from_holon_ast` error message verbosity)
- **reap:** 0 (Stone C scope tightly bounded; honest-delta self-flag expected if anything beyond BRIEF)
- **sever:** 0 (`decode_frame` cleanly separated from `take_frame`; PhantomData markers don't braid concerns; `impl HolonRepresentable for String` is a clean type-class instance)
- **temper:** 0-1 (Sender::send still allocates `framed: Vec<u8>` per call — Stone A inheritance, known-deferred; `String::to_holon_ast` clones self — known cost of trait shape)

Total predicted: 0-4 findings; most L2. Round 2 should be CLEAN.

## Tractability tiebreaker rationale

Stone C is gated on Stone B (uses the cascade-aware Receiver::recv body). No alternative ordering within Slice 3 — Stones D and E depend on Stone C's generic-T shape (D's Select fan-in must be type-aware; E's persistent ring stores per-Receiver state which Stone C establishes).

Within Stone C: ONE coherent concern (generic-T serialization wire layer). Decomposed into 4 file edits at the IMPLEMENTATION level but logically ONE concern at the stepping-stone level.

## Cross-references

- BRIEF-214-SLICE-3C-HOLON-REPRESENTABLE.md — this stone's work order
- BRIEF-214-SLICE-3A-IO-URING-BYTES.md — Stone A foundation
- BRIEF-214-SLICE-3B-CASCADE-AWARE-MULTI-ARM.md — Stone B cascade
- WARD-PASS-3A-IO-URING-BYTES.md — Stone A ward round-trip
- WARD-PASS-3B-CASCADE-AWARE-MULTI-ARM.md — Stone B ward round-trip
- `docs/arc/2026/05/214-concurrency-toolkit/DESIGN.md` — Slice 3 generic-T
- `src/comms/mod.rs:58-63` — `HolonRepresentable` trait (Slice 1)
- `src/edn_shim.rs:1678` — `holon_ast_to_edn` (private; Stone C wraps)
- `src/edn_shim.rs:1997` — `read_holon_ast_tagged` (public; Stone C uses)
- `src/typed_channel.rs:228-230` — existing PipeFd encode reference
- `project_holon_universal_ast` — HolonAST as universal substrate form (strange loop)
- INTERSTITIAL § 2026-05-19 "Kernel impeccability via ward pass" — protocol
