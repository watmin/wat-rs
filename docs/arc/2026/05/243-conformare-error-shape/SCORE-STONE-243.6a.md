# SCORE — Stone 243.6a — CheckError Pattern A retrofit

## Phase A — substrate refactor verified

**Mode:** A

### Per-step audit

| Step | Status | Notes |
|---|---|---|
| S1 — Carve `src/check/error.rs` | COMPLETE | New home created: `pub struct CheckError { pub span: Span, pub kind: CheckErrorKind }` + `pub enum CheckErrorKind` (33 variants); `Display` impls split (CheckErrorKind span-free + CheckError delegates); `diagnostic()` + `CheckErrors::diagnostics()` moved here; `vigilatum` stamp NOT added (earned post-strike by live cast) |
| S2 — Wire module in check.rs | COMPLETE | `pub mod error;` + `pub use error::{CheckError, CheckErrorKind, CheckErrors};` alongside existing `pub mod env;`; `span_prefix` and `std::fmt` imports removed (unused after carve); `collect_hints` promoted to `pub(crate)` for error.rs access |
| S3 — Remove flat enum + impls | COMPLETE | Flat `enum CheckError` (33 variants, each carrying `span`) removed from check.rs; `impl fmt::Display for CheckError`, `struct CheckErrors`, `impl CheckError::diagnostic()`, `impl CheckErrors::diagnostics()` all removed (all live in error.rs now) |
| S4 — Cascade (452 in-file + 7 cross-file) | COMPLETE | All 452 check.rs construction/match sites reshaped; 7 cross-file sites (argspec/error.rs, function/infer.rs × 2) updated; 5 multi-span variants with domain-named secondaries; test files (wat_structs, wat_typealias, wat_spawn_fn, wat_core_try, wat_typed_if_match, probe_arc236_stone0) updated |
| S5 — diagnostic() collapse | COMPLETE | N-arm span extraction in old `CheckError::diagnostic()` replaced by single `self.span` access at outer struct level; `impl CheckErrors::diagnostics()` moved to error.rs unchanged |
| S6 — lib.rs export | COMPLETE | `CheckErrorKind` added to crate re-export (`pub use check::{..., CheckErrorKind, ...}`) |

### Cascade audit table

| File | Sites updated | Category |
|---|---|---|
| `src/check/error.rs` | NEW (954 lines) | Home for CheckError Pattern A |
| `src/check.rs` | 452 construction/match sites reshaped + module wire | Emitters + module |
| `src/argspec/error.rs` | 1 From impl | Emitter (From conversion) |
| `src/function/infer.rs` | 2 sites (MalformedForm + ReturnTypeMismatch) + import | Emitter |
| `src/lib.rs` | 1 re-export (add CheckErrorKind) | Export |
| `tests/wat_structs.rs` | 3 match patterns + import | Consumer (test) |
| `tests/wat_typealias.rs` | 1 match pattern + import | Consumer (test) |
| `tests/wat_spawn_fn.rs` | 1 match pattern | Consumer (test) |
| `tests/wat_core_try.rs` | 6 match patterns + import | Consumer (test) |
| `tests/wat_typed_if_match.rs` | 2 match patterns + import | Consumer (test) |
| `tests/probe_arc236_stone0_check_result.rs` | 2 construction sites + import | Consumer (test) |

**Total emitter sites updated: ~459** (452 in check.rs + 7 cross-file)
**Total consumer sites updated: ~15** (test match patterns + imports)

### Cascade waterfall

| Iteration | Errors |
|---|---|
| Baseline (before stone) | 0 (compiles but probe fails to compile) |
| After carve + module wire (type removed) | 347 E0223 (ambiguous assoc type) |
| After transformer v1 (bad remove_field) | 160 (152 E0063 missing remedies + 6 E0223 + 1 E0220) |
| After sed comma fix (first pass) | 90 (83 E0063 + 6 E0223 + 1 E0220) |
| After transformer v2 (fix_missing_commas) | 13 (6 E0063 stone-241.10-comment pattern + 6 E0223 + 1 E0220) |
| After manual comment+multispan fixes | 4 E0223 (check.rs struct-shorthand + push_* methods) |
| After final manual fixes | 0 errors |

### 5 multi-span dispositions

| Variant | Outer `span` = most-actionable | Secondary → domain-named kind field |
|---|---|---|
| `SandboxScopeLeak` | `call_span` — the invocation inside sandbox (user edits to fix; the outer-scope define is context) | `outer_define_span: Span` (best-effort outer define location) |
| `ProcessJoinBeforeOutputDrain` | `join_span` — the `Process/join-result` call (the BLOCKED site; fix = restructure the let here) | `output_accessor_span: Span` (conflicting output accessor call) |
| `ProcessJoinHoldsStdinSender` | `join_span` — the `Process/join-result` call (the BLOCKED site) | `stdin_sender_span: Span` (where the process identifier was bound) |
| `DefRedefForbidden` | `current_loc` — the new (colliding) `def` site (user edits this def to fix) | `original_def_span: Span` (prior binding location for context) |
| `DefRedefTypeChange` | `current_loc` — the new `def` site (type-stability violation here) | `original_def_span: Span` (prior binding for comparison) |

**STOP triggers checked:**
- T1 (ambiguous most-actionable span): NOT triggered. All 5 variants have clear most-actionable locations confirmed by reading their `Display` messages.
- T2 (circular import): NOT triggered. One-way edge: error.rs imports from `super::collect_hints` (check.rs); no cycle.
- T3 (construction site with NO span): NOT triggered. All 459 sites had spans available.
- T4 (remedies field dropped): NOT triggered. Preserved on all `MalformedForm` and `ReturnTypeMismatch` kind variants.
- T5 (span left on variant): NOT triggered. Pattern A enforced everywhere; zero per-variant `span:` fields in CheckErrorKind.

### Honest deltas

- `src/check.rs`: net −2,509 lines (flat enum, Display impl, diagnostic() block all removed; 452 construction sites slightly longer individually but total is dramatically smaller)
- `src/check/error.rs`: +954 lines (new home — struct + kind enum + Display × 2 + diagnostic + diagnostics)
- `src/lib.rs`: +1 line (CheckErrorKind export)
- Cross-file cascade: +~15 lines across 5 test files and 2 source files
- `tools/transform-checkerror/`: NEW (175 lines — one-shot mechanical transformation Rust binary; not a permanent substrate artifact)

### Trap-doors encountered + absorbed

| # | Trap-door | Resolution |
|---|---|---|
| TA | 452 in-file sites — mechanical cascade | Built a dedicated Rust transformer binary (`tools/transform-checkerror/`) to automate the construction-site transformation; ran once, produced 9 residual CheckError:: occurrences (struct-shorthand + push_* helper methods); fixed manually |
| TB | remove_field() removed preceding comma | Transformer had a bug: backing over comma to `field_start` removed the trailing comma of the PRECEDING field. Fixed by not backing over commas + running a `fix_missing_commas_before_remedies` pass |
| TC | Stone-241.10 comment pattern | 6 blocks had `// Stone 241.10: ...` comment between `reason:` field value and `remedies:`; the comma fix skipped over comments; fixed manually (6 sites) |
| TD | Struct-shorthand syntax (`join_span,`) | `extract_field("join_span")` failed on shorthand `join_span,` (no colon); transformer produced empty span; fixed manually (4 sites: ProcessJoinBeforeOutputDrain, ProcessJoinHoldsStdinSender, DefRedefForbidden, DefRedefTypeChange) |
| TE | push_* helper methods in test harness struct | 3 helper methods (push_type_mismatch, push_arity_mismatch, push_malformed) used old construction syntax; were missed by transformer; fixed manually |

### Final metrics

| Metric | Value |
|---|---|
| Lib tests | 895 / 0 / 1 |
| FM 2-bis probe (`probe_arc243_stone6_checkerror_pattern_a`) | 3 / 0 |
| `cargo build --release --tests` | clean (0 errors) |
| CheckErrorKind variants | 33 (zero per-variant `span:` fields) |
| outer `CheckError::span` universal | confirmed — every consumer reads `err.span` directly |
| `diagnostic()` collapse | N-arm span extraction → `self.span` (one path) |

### Structural verification results

| Check | Result |
|---|---|
| `pub struct CheckError` present | `src/check/error.rs:16` |
| `pub enum CheckErrorKind` present | `src/check/error.rs:27` |
| `pub enum CheckError` GONE from check.rs | 0 matches |
| CheckErrorKind variants carry NO `span:` field | 0 span fields on kind variants |
| `pub mod error;` in check.rs | line 51 |
| `pub use error::{CheckError, CheckErrorKind, CheckErrors};` | line 52 |
| `collect_hints` promoted to `pub(crate)` | confirmed |
| `From<ArgSpecError> for CheckError` uses Pattern A | `src/argspec/error.rs:85-90` |
| `diagnostic()` reads `self.span` | single-path access |
| `CheckErrors::diagnostics()` in error.rs | confirmed |
| `CheckErrorKind` exported from `wat::` crate root | `src/lib.rs:117` |
| CONFORMARE.md rune on check.rs:90 closed | rune:conformare(deferred-stone-243.6) referenced in DESIGN |

---

## Phase B — vigilia REMARKABLE (orchestrator-run, workflow-fanned)

**Lens selection (7, by four-questions).** The defensive set cast on this PURE TYPE-DEFINITION file: `conformare · intueri · solvere · purgare · struere · exigere · circumspicere`. **sequi** (state-threading through call chains) and **temperare** (perf hotspots) were dropped — zero purchase on a struct + enum + `Display`/`From` file with no control flow or hot loops; casting them would be guaranteed-CONVERGED theater, not bar-raising. The REMARKABLE bar is L1+L2=0 on the lenses that *apply*, not lens-count.

**Cast mechanism.** Workflow-fanned: 7 independent subagent lenses, own scope, no cross-talk. Spells consumed via the datamancy MCP by the orchestrator and staged for the workers (subagents cannot reach the MCP — the dev tree is never the consumption source). **Every finding verified against the code before any fix** — the cast is data, not a verdict (FM 9).

**Convergence — 4 fix rounds:**

| Round | Cast verdict | Fix applied |
|---|---|---|
| R1 | L1:1, L2:4 | dead `DefNotTopLevel` purge (retired Arc 170 Gap I-B); `Display` message-duplication decomplect via `fmt_with_span` helper; doc-oversell tighten |
| R2 re-cast | L1:1, L2:0 | the doc reword's "Display elides" claim outran the code — gate the 17 mid-prose PRIMARY-span branches via a `shown = span.filter(!is_unknown)` |
| R3 re-cast | L1:2, L2:1 | (deeper) `diagnostic()` emitted location unguarded (~20 arms) + SECONDARY-span Display interpolations (×4) unguarded + stale `CommCallOutOfPosition` doc (2 contexts documented, 4 in code) |
| R4 re-cast | **L1:0, L2:0** | **structural class-elimination**: one `loc_field` helper (38 call-sites, 0 raw span emissions left) makes an unguarded span emission impossible; the 4 secondary Display spans gated with fallback prose; doc → all 4 contexts |

**The lesson (inscribed).** The finding-count *rose* R2(1) → R3(3) because the fixes were **site-by-site** (convention) — the fresh ear kept finding the next unguarded span path (primary Display → secondary Display → `diagnostic()`). It fell to **0** only when R4 killed the **class** with one elide-aware mechanism per surface. Failure-engineering applied to the elision discipline itself: stop gating instances; make the wrong shape unrepresentable.

**Probe grew 3 → 6 contracts** — Pattern-A shape (×3) + unknown-span Display elision (primary + secondary) + `diagnostic()` elision. The elision invariant is now **structurally checked, not convention-held**.

**Stamp earned.** `//! vigilatum: 2026-06-01T19:18:06Z — vigilia 7-spell L1+L2=0` on `src/check/error.rs`. Gate satisfied two ways: vigilia L1+L2=0 (all 7 lenses, cast → fix → re-cast to a clean read) AND clippy-clean (0 findings in the home). Final metrics held: lib 895/0/1, probe 6/0, build clean.

**Ephemeral discipline.** The Phase-A `tools/transform-checkerror/` Cargo binary (+ a scratch `scripts/transform_checkerror.py`) were build → use → **DELETE** — removed at convergence; the substrate carries no transform tooling. (Supersedes the "NEW (175 lines)" framing in Phase A's honest-deltas — the tool was always ephemeral; this is the forward-correction, not an edit of that record.)

**243.6a CLOSED at the REMARKABLE bar.** Routed forward to **243.4**: circumspicere flagged the same "elides unknown spans" doc wording in BOTH error homes — TypeError (`src/types/error.rs`, warded 243.5) carries it too, and its `diagnostic()` elision-uniformity should be verified when 243.4 standardizes the doctrine doc.
