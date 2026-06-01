# DESIGN — Stone 243.7c — `RuntimeError` → Pattern A (the shape retrofit)

**Status:** DRAFT (Phase 1 crawl complete; FM 2-bis probe owed before STRIKE-READY). Child of arc 243 (conformare). The second half of the banked "RuntimeError → Pattern A" obligation: **243.7b** removed the control signals (the blocker); **243.7c (this stone)** reshapes the now-pure-diagnostic enum to Pattern A — the same class-elimination as TypeError (243.3) / CheckError (243.6a), at ~2.5× CheckError's cascade.

## Why this stone

`RuntimeError` (`src/runtime.rs:2170`, ~30 diagnostic variants post-243.7b) is the last large flat diagnostic enum where the span-discipline is hand-written convention, not structure: each variant carries its own `span` field (or, for the freeze pair, none at all). Display does N-arm span routing; no consumer reads span uniformly. Peer retrofit to TypeError (243.3) + CheckError (243.6a) — Pattern A makes the spanless shape **structurally unrepresentable** at construction. With the signals gone (243.7b), this is the mechanical repeat of a known move; it completes the conformare goal for the substrate's central error type.

## What it delivers

- `pub struct RuntimeError { pub span: Span, pub kind: RuntimeErrorKind }` + `pub enum RuntimeErrorKind` — **flat in `src/runtime.rs`** (NO home carve, NO vigilatum: runtime.rs is wards-optional per `feedback_selective_lift_and_ward`; the `src/runtime/` home is a separate future undertaking).
- Every consumer reads `err.span` — one path; Display/EDN collapse their N-arm span routing.
- The freeze pair (`UserMainMissing`, `EvalVerificationFailed`) become kind variants with NO span field; they construct with outer `Span::unknown()`, honestly elided (the zero-exceptions case, handled by the existing `span_prefix`/elision mechanism — not a separate location type).

## The algorithm (mirror `src/check/error.rs` / `src/types/error.rs`)

1. **Reshape to Pattern A** in `src/runtime.rs`:
   - `pub struct RuntimeError { pub span: Span, pub kind: RuntimeErrorKind }`.
   - `pub enum RuntimeErrorKind { … ~30 variants, span handling per §contract … }`.
   - **Single-span variants (~25):** drop the `span` field; it moves to the outer struct. (Tuple variants like `UnboundSymbol(String, Span)` → `RuntimeErrorKind::UnboundSymbol(String)` with span on the outer struct.)
   - **Multi-span variants (2):** outer `span` = the most-actionable location; secondary span(s) stay as domain-named kind fields (§contract).
   - **Freeze pair (2):** `UserMainMissing` (unit) + `EvalVerificationFailed { err }` — kind variants with NO span; construct with outer `Span::unknown()`.
   - Preserve all payload fields (`got: Box<ValueSnapshot>`, `attempted_clauses: Vec<ClauseAttempt>`, `available: Vec<String>`, etc.) on the kind variant — unchanged.
2. **Display.** Split `impl fmt::Display for RuntimeErrorKind` (span-free, per-variant message) + `impl fmt::Display for RuntimeError` (delegates to `kind`; prefixes via `span_prefix(&self.span)` when known, elides unknown). Mirror `types/error.rs` / `check/error.rs`.
3. **EDN serializer** (`src/runtime_error_edn.rs`): the per-variant match collapses to read `self.span` once + match `self.kind`; mirror the CheckError EDN reshape (243.6a). Elide unknown spans (the freeze pair).
4. **diagnostic()/structured path:** if RuntimeError has a structured diagnostic conversion (confirm during strike), collapse span routing to `self.span` + a `RuntimeErrorKind` message helper, mirroring `check.rs`'s `loc_field` elision.
5. **Cascade (substrate-as-teacher):** ~1186 `RuntimeError::` references across `src/` + `crates/` (913 in runtime.rs; the rest in io/time/freeze/string_ops/thread_io/marshal/runtime_error_edn/…). Construction `RuntimeError::Variant { …, span }` → `RuntimeError { span, kind: RuntimeErrorKind::Variant { … } }`; match sites destructure `.kind`. **The weapon is an ephemeral *Rust* Cargo tool** (build → use → DELETE; NEVER Python/shell — both sandbox-blocked) per `feedback_cascade_ephemeral_tool` + DUNGEON-CRAWL. Reshape the type, let cargo name every site, iterate to green (fail-count = the meter).

## The error contract (the one surface decision, pinned)

The **2 multi-span variants** — outer `span` = the most-actionable location (the site the user edits to fix); secondary keeps a domain-descriptive kind field:

| Variant | spans | outer `span` = most-actionable | secondary → domain-named kind field |
|---|---|---|---|
| `SandboxScopeLeak` | 2 | `call_span` (where the offending call is) | `outer_define_span` (the outer-scope define) |
| `PostconditionFailed` | 2 | `body_span` (where the returned value was produced) | `ensure_span` (the `:ensure :fn` declaration) |

(Identical shape to `CheckError::SandboxScopeLeak`'s contract — call-site is the actionable span; the secondary is the context.)

**The freeze-pair decision (pinned):** `UserMainMissing` / `EvalVerificationFailed` carry outer `Span::unknown()`. Rationale (four-questions): they are rare startup-time conditions with genuinely no source span; `Span::unknown()` + honest elision is **uniform** with the other Pattern-A homes and honest because Display/EDN elide unknown spans (no synthetic `<runtime>:0:0` leaks). A separate location type for two startup variants fails Simple. This is the CONFORMARE.md "domain whose location is genuinely not a source span" case, resolved by the elision contract rather than a new type.

## Files touched

- `src/runtime.rs` — the enum reshape + Display split + the in-file construction/match cascade (913 sites) + `span_prefix` reuse.
- `src/runtime_error_edn.rs` — EDN serializer collapse to `self.span`/`self.kind`.
- Cascade fan-out: `src/io.rs`, `src/time.rs`, `src/freeze.rs`, `src/string_ops.rs`, `src/thread_io.rs`, `src/rust_deps/marshal.rs`, `crates/wat-telemetry-sqlite/…`, + wherever cargo names a construction/match site.
- `tests/probe_arc243_stone7c_runtimeerror_pattern_a.rs` — the FM 2-bis probe.

## Out of scope (REJECTED, not deferred)

- `src/runtime/` home carve + vigilatum REMARKABLE → **future undertaking** (24k-line flat-untrusted file; wards-optional; not this chain).
- No error-semantics change (no merging, no recovery, no message rewrites) — location-discipline reshape only.
- The signal split → already shipped (243.7b). The `EvalBreak`/`EvalSignal` types are unaffected (they wrap `RuntimeError` by value; `EvalBreak::Diagnostic(RuntimeError)` still holds — confirm the wrap survives the struct reshape; no change expected).
- `clippy::result_large_err`: the struct reshape may shift sizes — if it fires, box per 243.7a precedent (no `#[allow]`).

## Probe contracts (`tests/probe_arc243_stone7c_runtimeerror_pattern_a.rs` — committed; disconfirms at HEAD)

1. `runtimeerror_outer_span_field_required` — `RuntimeError { span, kind: RuntimeErrorKind::DivisionByZero }` constructs; `err.span` universally accessible.
2. `runtimeerrorkind_variants_have_no_span_field` — kind variants span-free (single-span ones).
3. `runtimeerror_span_access_is_single_path` — universal `err.span` (no N-arm match).
4. `runtimeerror_freeze_pair_elides_unknown_span` — `RuntimeError { span: Span::unknown(), kind: UserMainMissing }`.to_string() contains no `<runtime>`.

**Expected disconfirmation at HEAD:** `RuntimeErrorKind` unresolved (E0433/E0412) + `RuntimeError` is an enum not a struct (E0574) + no field `span` (E0609). Post-stone: pass.

## Trap-doors

| # | Risk | Detection | Resolution |
|---|---|---|---|
| **T1** | ~1186-site cascade — voluminous | cargo fail-count | substrate-as-teacher; ephemeral **Rust** tool (build/use/delete); waterfall to 0 |
| **T2** | Multi-span variants — which span is primary | the §contract table | pinned: SandboxScopeLeak=call_span, PostconditionFailed=body_span |
| **T3** | Freeze pair has no span | construct with `Span::unknown()` | outer unknown + honest elision (probe contract 4) |
| **T4** | `EvalBreak::Diagnostic(RuntimeError)` wrap breaks | cargo + probe_arc243_stone7b | the wrap holds a value; struct reshape is transparent; confirm 7b probe still 4/0 |
| **T5** | `result_large_err` shifts on the struct | clippy | box the large kind payload (already `Box<ValueSnapshot>` on the hot ones); no `#[allow]` |
| **T6** | Payload fields dropped during reshape | probe + cargo | preserve every field on the kind variant; cargo names any drop |
| **T7** | Cross-crate sites (wat-telemetry-sqlite) | cargo --tests | reshape reaches them; cargo names; fix in-place |

## Calibration

Larger than 243.6a (459 → ~1186 sites). 30 variants + 2 multi-span + 2 freeze-pair + Display split + EDN collapse + the big cascade.
- **120–240 min Mode A.** STOP at 480 min. **Ephemeral Rust Cargo tool MANDATORY** for the construction-site reshape (the by-hand path is infeasible at 1186).
- **Gate:** lib parity — `cargo test --release --lib -p wat` at baseline (895/0/1 + any 7c-probe additions) + `cargo build --release --tests` clean + clippy `result_large_err` 0 + the 7b probe still 4/0 (EvalBreak wrap intact). **No vigilia REMARKABLE** (flat file).
- Behavior-preserving: a moved/failed lib test = a behavior change to undo.

## Cross-references

- Template: `DESIGN-STONE-243.6a.md` + `src/check/error.rs` (the CheckError Pattern A — this mirrors it flat) + `SCORE-STONE-243.6a.md` + `tests/probe_arc243_stone6_checkerror_pattern_a.rs`.
- `DESIGN-STONE-243.7b.md` + `SCORE-STONE-243.7b.md` (the signal split that unblocked this) + `tests/probe_arc243_stone7b_signal_split.rs`.
- `docs/CONFORMARE.md` § zero-exceptions + § Multi-span; `feedback_cascade_ephemeral_tool` (Rust tool, never Python); `feedback_selective_lift_and_ward` (why flat = no vigilatum).
- arc 243 `DESIGN.md` (243.7… rolling-audit row).
