# CONFORMARE — Cast Report

**Target:** every Rust error type in `src/`, `crates/wat-reader/`, `crates/wat-edn/` that can reach a
wat program's diagnostic output (check, type, parse/lex, load, macro, runtime, startup, EDN-read,
process/peer-death envelopes), plus the wat-declared `:wat::kernel::LociDiedError` envelope
(`wat/kernel/diagnostics.wat:122-136`) and the `Failure`/`Fault`/`Frame` records it carries, and every
`From<E1> for E2` between them.

**Repo/commit:** `/home/john/work/holon/wat-rs` @ `1c4c09c82` (branch `reason/little-wat-findings`).
Read-only cast: no files edited, no `cargo` run.

**Method:** the casting agent read the wat-declared envelope (`wat/kernel/diagnostics.wat`,
`wat/core.wat`), the `WatError` trait (`src/edn/contract.rs`), `Span`/`Location`/`Frame` shapes, the
substrate's own standing doctrine (`docs/CONFORMARE.md`) and its closing inscription
(`docs/arc/2026/05/243-conformare-error-shape/INSCRIPTION.md`), plus dispatched four parallel
sub-audits, each reading its assigned files in full: **check/type/rete/resolve**, **parse/lex/EDN**,
**runtime/macro/load**, **startup/host/comms/misc**, and cast an independent direct pass over the
**wat-declared envelope + Rust registration boundary**. All five returned findings; three of the five
(against instructions) wrote directly to this file in sequence, each overwriting its predecessor —
their findings were recovered from the surviving file content and from each sub-audit's own completion
summary, and this final version is the casting agent's own synthesis and correction of all five,
not a re-publish of any one sub-audit's draft. Findings are cross-checked against the codebase's own
prior conformare casts (`docs/arc/2026/05/243-conformare-error-shape/CONFORMARE-FIRST-CAST.md`,
2026-05-30, and its closing `INSCRIPTION.md`) and against the in-flight excursus this branch belongs
to (`docs/excursus/2026/09/003-the-little-wat-findings/`, stones A–R already landed at this commit) so
an already-cured defect is not re-reported as new.

## Cross-check against the substrate's own standing doctrine

This is not a green-field audit. `docs/CONFORMARE.md` already names Pattern A ("outer struct + kind
enum, mandatory domain-typed location field") as the substrate's zero-exceptions answer, and
`docs/arc/2026/05/243-conformare-error-shape/INSCRIPTION.md` records an eleven-stone campaign
(243.1–243.M) that drove essentially every Rust error type in this audit's scope to that shape. This
cast's job was therefore **not** "pick a pattern" — it was (a) verify the retrofit actually holds
against current source, (b) find what Pattern A's per-variant guarantee does *not*, by itself, close,
and (c) check the doctrine documents themselves for the same claim-vs-code gap they exist to catch.

Two doctrine-level findings fell out of (c) before any type-by-type finding:

- **`docs/CONFORMARE.md`'s own "Rolling audit" table (lines 138–146) is stale.** It lists `ParseError`,
  `LexError`, `LoadError`/`LoadFetchError`, `ResolveError`, `HashError`, `StdlibError`, `MacroError`,
  `LowerError`, `EdnReadError`, `ClauseGrammarError`, `ConfigError` as **"Pending"** and `RuntimeError`
  as **"In flight → Stone 243.7a"**, carrying `attested-arc` runes. Direct read of current source shows
  every one of these is Pattern A today, and `INSCRIPTION.md` §III (`243.7a`–`243.M`, dated after the
  table) records the campaign that finished them, closing with *"the class structurally eliminated."*
  Zero `rune:conformare(attested-arc)` comments exist anywhere in the tree (verified: `grep -rn
  "rune:conformare" src/ crates/ wat/` returns exactly two hits, both a **different**, doctrine-retired
  category — see Finding 13). An inherited "Pending" row is a claim about a past state, not a current
  one; this table should be replaced with a short "retrofit complete as of arc 243; audit continues as
  backsliding-protection" line before the next reader re-derives the same nine types from scratch.
- **Two named, deferred items already exist for exactly this cast's headline finding.**
  `docs/arc/2026/06/296-diagnostics-fully-edn/DESIGN-296-typed-causes.md` §"Out of scope (affirmative
  cuts)" explicitly defers **S3/S4** ("`ProcessDiedError` EDN-in-String → typed field — a breaking
  change to a registered wat type; its own four-questions decision, later") and **S5**
  ("`ThreadDiedError::Panic` assertion-envelope — L2, later"). S3/S4 is this report's Finding 6
  (`LociDiedError`'s `message` field smuggling a serialized structured error); S5 is this report's
  Finding 15 (a plain panic's Rust location never reaching the structured envelope). Both are
  legitimate `rune:conformare(attested-arc)` candidates — the arc is real, the DESIGN.md exists, the
  deferral is deliberate — **but no rune marks either site**. `runtime.rs:12447-12454` names the design
  doc in a *prose comment* three call-frames from the actual `LociDiedError` declaration
  (`wat/kernel/diagnostics.wat`) and construction sites (`src/kernel/error.rs`, `src/process/died.rs`);
  neither of those carries the rune the doctrine requires to make a deferral inspectable in place. This
  matters independently of whether S3/S4/S5 are fixed: the doctrine's own text says *"A rune with
  empty/vague reason fails the spell"* — an unruled-but-real deferral fails the **Obvious** axis the
  same way an unexplained gap does, because a future reader at the type declaration has no signal a
  decision was ever made.

Every finding below is additional to these two; none of the per-type findings restate what
`docs/CONFORMARE.md` and its arc already know.

---

## Substrate-wide manifest

| Error type | File | Shape | Conformance |
|---|---|---|---|
| `CheckError`/`CheckErrorKind`/`CheckErrors` | `src/check/error.rs:24-375` | Pattern A (+4 documented secondary spans) | **Conformant** |
| `TypeError`/`TypeErrorKind` | `src/types/error.rs:15-280` | Pattern A, private fields, single `new()` door | **Conformant** |
| `ReteCheckError`/`ReteCheckErrorKind`/`ReteCheckErrors` | `src/rete/validate/error.rs:23-641` | Pattern A | Conformant internally; **L1 at the `StartupError::Validator` extension-point boundary it plugs into** (Finding 1) |
| `ResolveError`/`UnresolvedReference` | `src/resolve/error.rs:13-65` | Single-variant enum wrapping a mandatory-span item struct | **Conformant** (structurally: only 1 variant exists today; the guarantee rests on that, not on struct-literal enforcement — worth noting for the next variant author) |
| `ParseError`/`ParseErrorKind` | `crates/wat-reader/src/parser.rs:21-207` | Pattern A | **Conformant** |
| `LexError`/`LexErrorKind` + `LocatedLexError` | `crates/wat-reader/src/lexer.rs:190-340` | Two-tier: spanless inner (byte position, internal-only) escalated to a real `Span` exactly once at the public boundary | **Conformant** (honest two-tier; no bare `LexError` crosses the module boundary — Stone O confirms) |
| `wat_edn::Error`/`ErrorKind` | `crates/wat-edn/src/error.rs:8-14` | Single-variant `Error::Parse{pos,kind}`, `pos` mandatory via `Error::at()` | **Conformant** — byte position genuinely tracked and threaded through every parser call site |
| `JsonError` | `crates/wat-edn/src/json.rs:52-98` | Flat enum, 13 variants, **zero** carry position | **Finding 2 (L2)** — discards `serde_json::Error`'s own line/col and the crate's own `wat_edn::Error::pos` |
| `EdnReadError`/`EdnReadErrorKind` | `src/edn/render.rs:1821-1829` | Claims "Pattern A" in its own doc comment | **Finding 3 (L2; one arm borderline L1)** — the span is a synthetic Rust call-site at **all 30** construction sites |
| `EdnCoerceError` | `src/edn/render.rs:2371` | Bare struct, no span field, structural `path` pointer instead | **Conformant** — honestly spanless, always re-spanned by its one caller |
| `WatEdnBridgeError` | `src/edn/bridge.rs:302-332` | Flat enum, 7 variants, genuinely spanless (no false claim) | **Finding 4 (L2)** — 7-way structured distinction flattened to `String` at both production call sites |
| `RuntimeError`/`RuntimeErrorKind` | `src/value/signal.rs:107-328` | Pattern A, private fields, single `new()`/`kind()`/`into_kind()` doors | Type conformant; **Findings 9/10/11 (L1/L1/L2) at ~5+ construction sites** discard an in-scope real span for `rust_caller_span!()` |
| `MacroError`/`MacroErrorKind` | `src/macros/error.rs:8-42` | Pattern A | **Conformant** (calibration reference) |
| `LoadFetchError`/`LoadError`/`LoadErrorKind` | `src/load/loader.rs:192-295` | Pattern A; `LoadFetchError` always re-spanned at the boundary | **Already cured (Stone P)** — confirmed by direct read: the old blanket `From<LoadFetchError> for LoadError` is gone |
| `StdlibError`/`StdlibErrorKind` | `src/load/stdlib.rs:641-722` | Pattern A, span always `rust_caller_span!()` by necessity (stdlib is `include_str!`'d — no filesystem call site ever exists) | Behaviorally **conformant**; **Finding 12 (L2)** — stale doc comment claims `:location` is nil, contradicted by arc 298.2's "always emitted" contract and the type's own passing test |
| `StartupError` (Rust, `crate::freeze::StartupError`) | `src/freeze.rs:688-729` | Transparent delegating wrapper over 10 variants | **Finding 5 (L1)** — `SigmaFn(String)`/`MainSignature(String)` are flat, unspanned, unruled |
| `validate_user_main_signature`/`validate_user_main_not_useless` | `src/freeze.rs:1637,1690` | `Result<(), String>` | **Finding 5a (L1)** — structurally cannot carry a span; one call site provably had one (`ast.span()`) and dropped it |
| `GuestError` | `src/host/guest.rs:77` | Flat enum wrapping `StartupError`/`RuntimeError` + 2 flat `String` variants | **Cascades Finding 5** (`MainSignature(String)`); `StdioSnapshot(String)` undocumented-spanless |
| `FreezeValidatorError` (trait) | `src/freeze/validator.rs:25-32` | `trait FreezeValidatorError: ToEdn + Debug + Display + Send + Sync` — `WatError` NOT in the bound | **Finding 1 (L1)** |
| `:wat::core::Error`/`:wat::core::Fault` (wat-declared) | `wat/core.wat:2182-2207` | `defsurface`, mandatory (non-`Option`) `location <- Location` on every conforming record | **Conformant** — the doctrine's own structural floor, correctly non-optional |
| `:wat::kernel::Location`/`:wat::core::Span` (wat-declared) | `wat/core.wat:2161-2164,2233-2237` | Two location shapes: `Location{file,line,col}` (no `end`) vs `Span{file,line,col,end:Option<Pos>}` | **Conformant split** — `Location` is the Error-surface's point location, `Span` the AST-range type; `end` is legitimately absent from a point location, not a discard |
| `:wat::kernel::Frame` (wat-declared) | `wat/kernel/diagnostics.wat:24-27` | `{file, line, symbol}` — no `col` | **Finding 7 (L2)** — both Rust producers hold a full `Span` (incl. `col`) and drop it |
| `:wat::kernel::Failure` (wat-declared) | `wat/kernel/diagnostics.wat:104-113` | `{error<-Error, frames<-Vector<Frame>, actual<-Option<String>, expected<-Option<String>}` | **Mostly conformant** — `error` carries the mandatory-location `Error` surface; `actual`/`expected` are the one remaining raw-`String` (not structurally-typed-value) pair in an otherwise de-stringified record (**Finding 16, L2, cosmetic**) |
| `:wat::kernel::StartupError` (wat-declared, distinct from the Rust type above) | `wat/kernel/diagnostics.wat:29-41` | `defstruct [message <- String]` only; own doc admits "extensible... if a real consumer surfaces" | **Conformant / narrow** — confirmed (`runtime.rs:12440-12454`) this type is a known anti-pattern the builder already rejected once ("why is message wrapping a structured edn form?") and the *current* freeze-failure path (`check_failed_cause`) deliberately routes around it via `:wat::core::Error`'s `causes` chain instead. Effectively vestigial today; not itself constructed as a lossy carrier. |
| `:wat::kernel::LociDiedError` (wat-declared) | `wat/kernel/diagnostics.wat:122-136` | 8-variant enum; 6 of 8 declared `[message <- String]` only, 1 (`StartupError`) carries a real `error <- Error` | **Finding 6 (L1)** — attested-arc-eligible (296 S3/S4) but unruled at the site; message field is, by design, sometimes a full serialized `error_edn()` blob |
| `AssertionFailure` (wat-declared) | `wat/kernel/diagnostics.wat:154-161` | `location <- Option<Location>` | **Finding 16b (L2, minor)** — inconsistent optionality against `:wat::core::Error.location` (mandatory) a few hundred lines away in the same dependency chain; each choice is locally defensible (a pre-`AssertionPayload` panic genuinely has none) but the split is undocumented as deliberate |
| Plain-panic path (no `AssertionPayload`) | `src/panic_hook.rs:89-96`, `src/kernel/error.rs:76-97` | Hook downcast on `AssertionPayload`; non-matching panics go to `previous(info)` only | **Finding 15 (L1)** — attested-arc-eligible (296 S5) but unruled; `PanicHookInfo::location()` (free, always available) is never captured for the structured `LociDiedError::Panic{failure: None}` case |
| `ExtractionError`/`ExtractionErrorKind` | `src/closure_extract.rs:88-96` | Pattern A, mandatory outer span | **Finding 8 (L1)** — every construction site (incl. user-facing `ImpureCapture`) fills the mandatory span with `rust_caller_span!()`; one conversion boundary also flattens structured fields to `String` |
| `HashError` | `src/hash.rs:471` | Flat enum, no span | **Conformant / out of scope** — digest/crypto domain, always wrapped by a spanned outer type (`RuntimeErrorKind::EvalVerificationFailed`, `LoadErrorKind`); this is `INSCRIPTION.md`'s own named exception ("a payload that is never tossed to wat") |
| `SendError<T>`/`TrySendError<T>`/`RecvError`/`WireError` | `src/comms/mod.rs:312-408` | Flat, no span | **Out of scope** — transport-tier plumbing, never itself wire-serialized to a wat program; `WireError` verified **zero callers outside its own file** (dead code, independent of conformare) |
| `MatchArmError` | `src/match_arm.rs:25` | Bare struct, mandatory `span` | **Conformant** |
| `LowerError`/`LowerErrorKind` | `src/lower.rs:45-53` | Pattern A (explicitly documented) | **Conformant** |
| `ConfigError`/`ConfigErrorKind` | `src/config.rs:213-222,338` | Pattern A (explicitly documented) | **Conformant** |
| `ClauseGrammarError`/`ClauseGrammarErrorKind` | `src/form_match.rs:83-91` | Pattern A (explicitly documented) | **Conformant** |
| `ArgSpecError`/`ArgSpecErrorKind` | `src/argspec/error.rs:17-28` | Pattern A (explicitly documented, arc 241/243 founding precedent) | **Conformant** — its 4 `From` impls into `RuntimeError`/`CheckError`/`TypeError`/`MacroError` all verified preserving span |
| `Error` (crate-root MVP enum) | `src/lib.rs:521-523` | Thin `Parse`/`Lower` wrapper for the algebra-only demo entry point | **Conformant / narrow scope** — inner types keep their own spans; never reaches the main `WatError` wire |
| `rune:conformare(spanless-by-domain)` usage | `src/capability/registry.rs:93`, `src/collection/eval.rs:22` | 2 live sites use a category `docs/CONFORMARE.md:136` records as **"Retired at Stone 243.4"** | **Finding 13 (L1)** |

### Already cured (verified against current source, not re-reported)

- **Stone O** (`d3bed0d72`) — `From<LocatedLexError> for ParseError` (`crates/wat-reader/src/parser.rs:203-207`) now reads `e.span` (the lexer's real file/line/col, attached once at `lex_with_comments`'s return boundary). No `rust_caller_span!()` remains in that impl.
- **Stone P** (`587f9bd78`) — `impl From<LoadFetchError> for LoadError` is **deleted**; `fetch_source`/`fetch_payload` (`src/load/loader.rs:561,580`) now take the caller's `form_span` explicitly. No such blanket `From` impl exists in current `loader.rs` (only a historical comment at line 437).
- **Stone B** (F-006, F-114, variant-singleton) — `check.rs`'s `UnknownNamedType`/`ImpureFieldInPureAggregate` refusals and variant-singleton registration now locate the user's declaration (`tests/diagnostics/probe_ex003_diagnostic_locates_the_user.rs`).
- **Arc 243 (all eleven stones, 243.1–243.M)** — `docs/CONFORMARE.md`'s own "Pending"/"In flight" rows for `ParseError`, `LexError`, `LoadError`/`LoadFetchError`, `ResolveError`, `HashError`, `StdlibError`, `MacroError`, `LowerError`, `EdnReadError`'s *shape* (its span-source-honesty is a separate, live finding — see Finding 3), `ClauseGrammarError`, `ConfigError`, `RuntimeError` are all done; see "Cross-check" above.

These are named so the manifest is legible against the substrate's own ledger — not restated as findings.

---

## Findings

### Finding 1 (L1) — the `StartupError::Validator` extension point cannot preserve a validator's own location, by trait-bound construction

- `src/freeze/validator.rs:25-32` — `trait FreezeValidatorError: ToEdn + Debug + Display + Send + Sync {}`, blanket-implemented for any type meeting that bound. **`WatError` is not in the bound.**
- `src/freeze.rs:703` — `StartupError::Validator(Box<dyn FreezeValidatorError>)`.
- `src/macros/error_edn.rs:177` — `SE::Validator(_) => OwnedValue::Nil` (location), unconditionally.
- `src/macros/error_edn.rs:157` — `SE::Validator(e) => crate::edn::contract::first_line(e.to_string())` (message). For the one current registrant, `ReteCheckErrors`, `Display` is `f.write_str(&to_wire_edn(self))` (`src/rete/validate/error.rs:577-580`) — the **entire recursively-composed EDN wire text of the whole error batch**, not a headline; `first_line()` is a no-op here since that text has no literal `\n`.
- Today this is harmless only because the sole registrant genuinely has no single primary span (a collection). But `validator.rs`'s own doc says *"Any crate depending on `wat` can register its own freeze-time validator the same way"* — a future single-error, Pattern-A validator would have its real primary span **silently and structurally discarded**, with no compile-time signal, because the trait contract cannot express "this validator error has a primary location."
- Four-questions on the extension point: **Obvious** NO (nothing at the registration site signals the location will be thrown away) · **Simple** — moot, no path exists · **Honest** NO (the bound admits types that structurally cannot answer `location()`, yet `StartupError` claims a uniform floor for every variant) · **Good UX** NO (a future validator author gets silent data loss, not a compile error).
- Pattern proposal: widen `FreezeValidatorError`'s bound to `WatError` (already implies `ToEdn` via `error_edn()`). Four-questions on the fix: **Obvious YES / Simple YES / Honest YES / Good UX YES** — a validator that genuinely has no primary span can still honestly return `Nil` from its own `location()`; the difference is that becomes the *concrete type's* choice, not a structural impossibility baked into the extension point.
- Cascade: 1 registrant today (`src/rete/validate/mod.rs:126-134`); the fix is one trait-bound widen plus adding the three missing `WatError` methods to that registrant (non-breaking — its `location()` can honestly stay `Nil`, matching current behavior).

### Finding 2 (L2) — `JsonError` discards a real, available position twice over

- `crates/wat-edn/src/json.rs:52-98` — 13 variants, **none** carry position, unlike its sibling `wat_edn::Error` in the same crate.
- `JsonError::Parse(String)` (line 55), built at `json.rs:227`: `serde_json::from_str(s).map_err(|e| JsonError::Parse(e.to_string()))` — `serde_json::Error` has real `.line()`/`.column()` accessors; both discarded into an opaque `String`.
- `JsonError::InvalidMapKey { key, reason }` (line 97) — `reason` built at `decode_map_key` (`json.rs:327-330`) as `e.to_string()` where `e: wat_edn::Error` (the crate-mate that IS position-tracked) — that `pos` thrown away into a `String`.
- Reaches wat diagnostics: **not out of scope**. `wat_edn::from_json_string` is called from `src/edn/render.rs:333` (`eval_edn_read_json`); the error reaches `:wat::edn::read-json`'s `ReadJsonOutcome::Malformed` payload via `e.to_string()` (`render.rs:338`) — already-flattened prose, compounding rather than exempting the finding.
- Four-questions: Obvious NO (no doc explains why this type, unlike its crate-mate, carries no position) · Simple YES (flat enum) · Honest NO (looks ordinary; gives no signal it is weaker than `wat_edn::Error`) · Good UX NO.
- Pattern proposal: Pattern A — add a position field (reuse `wat_edn::Error`'s shape, or `serde_json::Error::line()/column()`) threaded through the one construction site.
- Cascade: small — few construction sites, one bridge into wat diagnostics.

### Finding 3 (L2; one arm borderline L1) — `EdnReadError` claims Pattern A but its span is synthetic at all 30 construction sites

- `src/edn/render.rs:1821-1824` — `struct EdnReadError { span: Span, kind: EdnReadErrorKind }`; the doc comment (1815-1820) explicitly claims *"Pattern A (Stone 243.7d): span at the outer struct level"* — the same label used elsewhere for a genuinely useful location.
- **All 30** construction sites use `span: crate::rust_caller_span!()` (verified: no non-`rust_caller_span!()` `span:` assignment exists in the file). Two sub-classes:
  - **Tree-walk arms** (e.g. `render.rs:2334,3584`) — defensible: `wat_edn::Value`/`OwnedValue` carries no span on any variant once EDN text is parsed into a tree, so position is gone from the data itself, not discarded by choice (documented in-file, "arc 138: no span... no WatAST in scope").
  - **Top-level parse arm** (`render.rs:1961`, `read_edn_caps`) — a **real discard**: `wat_edn::parse_owned(s).map_err(|e| EdnReadError { span: crate::rust_caller_span!(), kind: EdnReadErrorKind::Other(format!("EDN parse error: {e}")) })`. `e: wat_edn::Error` DOES carry a real byte `pos`, unused, additionally flattened via `format!` into a `String`.
- **Consequence**: `EdnReadError` never reaches the wire directly — every wat-facing caller converts via `.to_string()` into `RuntimeErrorKind::MalformedForm{reason}`, wrapped in a real, correctly-spanned `RuntimeError`, so the *final* user-visible location is the real `(:wat::edn::read ...)` call site. But `EdnReadError::Display` (`render.rs:1908-1913`) renders `"{rust_caller_span}: {kind}"`, and that whole string — **including the Rust file:line prefix** — becomes the `reason` text: a Rust source line leaks verbatim into prose a wat programmer reads for their own malformed-EDN mistake. This is the borderline-L1 arm.
- Four-questions: Obvious NO (doc comment's "Pattern A" claim is false for every instance) · Simple YES (mechanically one field) · Honest NO (asserts a guarantee it never keeps) · Good UX NO (misleading prose reaches the user).
- Pattern proposal: for the top-level-parse arm, thread `wat_edn::Error`'s `pos` (no file/line/col at this crate boundary, but a real position) instead of a synthetic Rust span. For the tree-walk arms, either correct the doc comment's claim or route through `FlatMessage`/a genuine spanless marker — zero runes exist anywhere in this file for 30 unspanned sites.
- Cascade: 30 construction sites in one file; the doc-comment fix is a 1-line no-risk change; the top-level-parse-arm fix threads one `usize` one call deep.

### Finding 4 (L2) — `WatEdnBridgeError`'s 7-way distinction is flattened to `String` at both production call sites

- `src/edn/bridge.rs:302-332` — 7 variants, genuinely and honestly spanless (no false Pattern-A claim, unlike Finding 3).
- Both production call sites (`src/process/boot/mod.rs:326,781`, inside `boot_err()`) do `format!("...{e}")` into `RuntimeErrorKind::MalformedForm{reason: String}` — nothing downstream can recover which of the 7 classes fired.
- The `rust_caller_span!()` used there is **legitimate** (pre-runtime process-boot phase, no wat world yet) — not itself a location-discard finding; the structure-flattening is independent of the span question.
- Notable: the same file's success-path doc comments (`bridge.rs:404-415,553-559,645-648`) are unusually explicit about span discipline for decoded `WatAST` nodes — the authors solved this exact class of problem for the happy path; the error path of the same bridge was not brought to the same standard.
- Four-questions: Obvious YES (no false claim) · Simple YES · Honest YES (doesn't pretend to have a span) · Good UX **NO on structure** (real diagnostic value thrown away).
- Pattern proposal: give `WatEdnBridgeError` a `WatError` impl (`location() -> Nil`, honestly), and carry the structured error as a typed `causes()` entry instead of `format!`-ing it into `reason`.
- Cascade: 2 call sites, 1 type.

### Finding 5 (L1) — `StartupError::SigmaFn`/`MainSignature` are flat, unspanned Strings, cascading into `GuestError` and the process-death envelope

- `src/freeze.rs:711-724` — `StartupError::SigmaFn(String)`, `StartupError::MainSignature(String)`. `src/macros/error_edn.rs:179-180` — both map `location()` to `OwnedValue::Nil` unconditionally.
- Cascades: `src/host/guest.rs:82` (`GuestError::MainSignature(String)`, built directly from Finding 5a's `Result<(), String>`); `src/process/died.rs:114,129` (`process_died_error_main_signature*` build `LociDiedError::MainSignature` the same way, feeding Finding 6).
- Four-questions: **Honest** fails — a `StartupError` can be built with zero location information and nothing signals it. **Obvious** fails — a reader must know which of 10 variants they're looking at to know whether a location exists.
- Pattern proposal: **thread a real span** (Finding 5a shows one is directly recoverable). ⚠ A prior draft of this cast proposed adding `// rune:conformare(spanless-by-domain)` here "per the codebase's own established convention" — that is **wrong under current doctrine**: `docs/CONFORMARE.md:136` records that exact rune category as **retired at Stone 243.4**, and `INSCRIPTION.md` states the only honest spanlessness left is a payload *never tossed to wat* — `StartupError` is tossed to wat. The correct fix is threading the span (Finding 5a), not a rune; see Finding 13.
- Cascade: 2 `StartupError` variants → 1 `GuestError` variant → 2 `LociDiedError` construction call sites.

### Finding 5a (L1) — `validate_user_main_signature`/`validate_user_main_not_useless` return `Result<(), String>`, and one call site provably had a span in scope and dropped it

- `src/freeze.rs:1637` / `:1690` — both `pub fn(...) -> Result<(), String>`, structurally incapable of carrying a span (a function-contract problem, not a per-call omission).
- `validate_user_main_not_useless` (`freeze.rs:1695-1707`): `if let FunctionBody::Wat(ast) = &func.body { if matches!(&**ast, WatAST::NilLit(_)) { return Err("...".to_string()) } }` — `ast: &WatAST` is in local scope and every `WatAST` node has a `.span()` accessor (`crates/wat-reader/src/ast.rs:229`) — the exact location of the useless `nil` body is available and unused.
- `validate_user_main_signature` has no `WatAST` directly in scope (already-lowered `TypeExpr`s with no span of their own); this arm's spanlessness is closer to legitimate, though `func.body`'s own span, one level away, is still not consulted.
- Four-questions: Obvious NO (the `Result<(), String>` signature signals nothing) · Simple — moot · Honest NO (silently drops an in-scope span) · Good UX NO.
- Pattern proposal: change both signatures to `Result<(), StartupError>` or a small local Pattern-A type; `ast.span()`/`func.body`'s span already exists to fill it. Closes Finding 5's `MainSignature` arm at the root.
- Cascade: `freeze.rs:961-962`, `src/distribution/mod.rs:415,460,483`, `src/host/guest.rs:110`, and the two `process_died_error_main_signature*` builders in `src/process/died.rs`.

### Finding 6 (L1, attested-arc-eligible but unruled) — `:wat::kernel::LociDiedError`'s "message" field is, by design, sometimes an entire serialized error tree — unreachable as structure through the type's own accessors

- `wat/kernel/diagnostics.wat:122-136` — the wat-declared enum: 6 of 8 variants (`Panic.message`, `RuntimeError`, `EntryFormFailure`, `MainSignature`, `BadReturn`) declare `[message <- String]` only. Only `StartupError` carries a real `error <- Error`.
- `src/process/died.rs:107-109,129-131,150-152` (`process_died_error_{runtime,main_signature,bad_return}_value`) build `message` via `crate::edn::contract::to_wire_edn(e)` — the **full floor-composed EDN text** of the original error, not a headline. `src/kernel/error.rs:257-263` (`thread_crash_runtime_edn`) does the identical thing for `RuntimeError`.
- Consequence: on the wire this is double-encoded — the outer `LociDiedError` chain is EDN; the inner structured error is a STRING VALUE inside it (escaped text), because the wat-declared field type is `String`, so the generic decoder has no reason to recursively parse it. `eval_died_error_message` (`src/kernel/error.rs:300-378`) returns this blob verbatim as "the message"; `eval_died_error_to_failure` (`:396-508`) builds a `message_only_failure` from the SAME blob. **No accessor re-parses the embedded EDN** — a wat program asking "where did my peer's `RuntimeError` actually happen" has no path through the declared surface. Compare `loci_died_from_send_error` (`kernel/error.rs:588`), which passes a *plain* io-error string through the exact same constructor — the field cannot distinguish "plain prose" from "serialized floor form."
- **This is `docs/arc/2026/06/296-diagnostics-fully-edn/DESIGN-296-typed-causes.md`'s named, deferred S3/S4** ("`ProcessDiedError` EDN-in-String → typed field — a breaking change to a registered wat type; its own four-questions decision, later"). The deferral is legitimate under the doctrine's `attested-arc` rune category — but **no rune exists at the `LociDiedError` declaration or either construction site**; the only trace is a prose comment three call-frames away (`src/runtime.rs:12447-12454`, discussing a *different* type's near-miss with the same anti-pattern). A future reader at `wat/kernel/diagnostics.wat:122` has no signal this is a tracked decision rather than an oversight.
- Four-questions: **Obvious** NO — the schema says `message <- String`; nothing signals it is sometimes a serialized structured error. **Simple** NO — recovering the real location requires bypassing the declared accessors entirely; no variant offers a path. **Honest** NO — `thread_died_error_runtime(message: String)` accepts any string; two producers populate the nominally-identical field with semantically different content. **Good UX** NO — richness is a serialization convenience the author must remember to both produce and manually consume.
- Pattern proposal: (1) at minimum, land the rune at `wat/kernel/diagnostics.wat:122` and at both Rust construction homes, citing this arc/design path, so the deferral is inspectable; (2) the actual cure — deferred by the substrate's own choice, not this cast's to schedule — is a second field `cause <- :wat::core::Error` on `RuntimeError`/`MainSignature`/`BadReturn`/`EntryFormFailure`, mirroring `StartupError`'s already-correct shape, with `message` reduced to a genuine headline.
- Cascade: 2 Rust construction homes, 1 wat declaration, every consumer of `LociDiedError/message` and `/to-failure`, and wat-level `recv'`/`select'` sites matching a peer's death.

### Finding 7 (L2) — `:wat::kernel::Frame` drops a real `Span.col` at both of its Rust producers

- `wat/kernel/diagnostics.wat:24-27` — `[file, line, symbol]` — no `col`, unlike `:wat::kernel::Location`/`:wat::core::Span` (both carry `col`, `wat/core.wat:2161-2164,2233-2237`).
- `src/value/frame.rs:29-32` — `FrameInfo { callee_path, call_span: Span }` — every call-stack frame internally carries a full `Span` (file, line, **col**, end).
- Two independent producers both drop it: `src/runtime.rs:12007-12021` (`value_from_frame_info`, uses only `.file`/`.line`) and `src/panic_hook.rs:240-249` (`frame_to_map`, same discard, independently implemented — verified by direct read).
- Consequence: `Failure.error.location` (the primary site) is precise; every entry in `Failure.frames` is coarsened to file:line only, unlike every other location this substrate emits.
- Not documented as a deliberate domain choice anywhere (no rune, no comment) — plausible that call-stack frames don't conventionally need column precision, but the data exists at both sites and the choice is unstated.
- Four-questions: Obvious YES (3-field declaration, easy to see the gap) · Simple YES · Honest — arguable, doesn't claim to be a full location, but isn't documented as an intentional reduction either · Good UX: minor loss.
- Pattern proposal: add `col <- i64` to the wat declaration, matching `Location`; update both Rust producers to emit `call_span.col`.
- Cascade: 1 wat declaration, 2 Rust producer sites, `frame_names()` (generated from the wat form).

### Finding 8 (L1) — `ExtractionErrorKind::ImpureCapture` (a user-reachable, user-actionable diagnostic) never carries a real span; its one conversion boundary also flattens structure to `String`

- `src/closure_extract.rs:88-91` — `struct ExtractionError { span: Span, kind: ExtractionErrorKind }` — Pattern A, mandatory span field, matching the doc comment's claim.
- **Every** construction site (`closure_extract.rs:178,266,1702,1741,1771,2334,2348,2360,2388`, and the whole `encode_value_with_path` match from `:1992`) uses `span: crate::rust_caller_span!()`. `encode_value_with_path` (`:1987-1994`) takes `(v: &Value, binding_name, path, state)` — **no span parameter exists in the signature** — it operates on an already-evaluated runtime `Value` with genuinely no wat-source AST in scope (a captured `Sender`/`Receiver`, once captured, carries no source pointer). This part is legitimately hard.
- But `ImpureCapture`'s own doc comment (`closure_extract.rs:80-86`) states this is a **user-facing, user-actionable diagnostic**: *"names the offending capture, its type, the field path inside... points the user at pipes/restructure."* Pattern A's mandatory field is satisfied at every site with a value carrying zero diagnostic content — **Pattern A guarantees a span is present; it cannot guarantee the span is meaningful.** No `rune:conformare(...)` anywhere in the file documents this gap.
- At the ONE conversion boundary (`src/closure_extract.rs:589-593`, `eval_kernel_fn_forms`): `list_span` (the real span of the user's `fn-forms`/`spawn-process` call) is substituted for `e`'s own useless span — so the *final* user-visible location is real, coarsened to "the whole spawn call" rather than the exact capture site. But `e.to_string()` flattens `ImpureCapture`'s structured fields (`name`, `type_name`, `path: Vec<String>`) into unstructured prose — a tool wanting the binding/type/path programmatically has no access to it.
- Four-questions on `ExtractionError` itself: Obvious NO (Pattern A's mandatory field reads as a guarantee it doesn't deliver here) · Simple YES (one path, mechanically) · **Honest NO** (every site can and does supply a meaningless span; the structural guarantee is satisfied vacuously) · Good UX NO — this is the clean demonstration that **Pattern A alone is necessary but not sufficient**: the field-presence question and the field-meaning question are different questions, and only the first is closed by construction.
- Pattern proposal: add the substrate's rune wherever a mandatory span field is filled with a synthetic sentinel because none is recoverable (all `closure_extract.rs` sites qualify — captured runtime `Value`s carry no AST) — but see Finding 13: the specific `spanless-by-domain` category is retired, so this needs either a fresh, doctrine-legal category or (preferably) the fix is to make `Pattern A`'s own doc/lint require flagging bare `rust_caller_span!()` fills the way the excursus's own probes already flag bare `.rs` locations (Stone O/P's own mechanism). Separately, carry `ImpureCapture`'s structured fields through the `eval_kernel_fn_forms` boundary as a nested `causes()` entry instead of `e.to_string()`.
- Cascade: ~10 construction sites in `closure_extract.rs`, 1 conversion boundary, reached from `:wat::kernel::fn-forms` and transitively from `spawn-process`'s sandbox walker.

### Finding 9 (L1) — three `RuntimeError::new(rust_caller_span!(), ...)` sites in `src/io.rs`, each with a sibling function in the same file that correctly threads the real span it discarded

- `src/io.rs:1389-1407` — `snapshot_writer(op, &writer)` takes no span parameter, raises with `rust_caller_span!()` (line 1402). **Both** its callers (`eval_iowriter_to_bytes` `:1355-1367`, `eval_iowriter_to_string` `:1371-1385`) hold a real `list_span: &Span` one line above (already used for `arity(...)`) and never pass it down.
- `src/io.rs:1649-1659` — `WatTempFile::path`, `None` arm (line 1654), same shape. Caller `eval_io_temp_file_path` (`:1712-1727`) holds `list_span` unused for this purpose. Its own sibling `WatTempFile::new` (`:1637-1647`) is explicitly documented as correctly threading the caller's span.
- `src/io.rs:1681-1691` — `WatTempDir::path`, identical shape; sibling `WatTempDir::new` correctly wired with the same contrastive comment.
- Four-questions: Honest NO, Good UX NO — a snapshot/`TempFile.path` failure points into `src/io.rs` instead of the user's `.wat`, and the fix pattern exists one function away in the same file.
- Pattern proposal: add a `list_span: &Span` parameter to all three, mirroring the `::new` siblings.
- Cascade: 3 functions, 3 call sites, one file. (Sampled from a larger population — see the substrate note under "Pattern recommendation.")

### Finding 10 (L1) — `assertion.rs::eval_opt_string` discards the real span for 2 of 3 sibling arguments of the identical call form

- `src/assertion.rs:190-209` (`eval_opt_string`), called from `eval_kernel_assertion_failed` (`:127-181`) once for `actual` (`args[1]`, line 154) and once for `expected` (`args[2]`, line 155). Both its error arms (lines 196, 203) raise with `crate::rust_caller_span!()`.
- **Six lines above** (`assertion.rs:146`), the sibling check on `message` (`args[0]`) for the *exact same op* correctly uses `args[0].span().clone()`. `args[1]`/`args[2]` sit in the calling scope one line before each `eval_opt_string` call — available, threaded for argument 0, silently dropped for arguments 1 and 2 of the identical `(:wat::kernel::assertion-failed! ...)` form.
- This is the cleanest instance in the whole audit of the ward's own "span presence differs silently across variants" case, here across sibling arguments of one call, six lines apart, inside a diagnostic-*about*-a-diagnostic path.
- Pattern proposal: thread `args[1].span()`/`args[2].span()` into `eval_opt_string`'s two error arms.
- Cascade: 1 helper, 2 call sites, both in the substrate's own assertion-failure path.

### Finding 11 (L2) — `freeze.rs::resolve_env_program` has a parsed `ast` in scope and raises with `rust_caller_span!()` anyway

- `src/freeze.rs:1427-1462` — parses `src` into `ast` (line 1428), which stays in scope. Both "env-fn returned a non-record" error arms (`:1443,1454`) use `rust_caller_span!()` though `ast.span()` (a real span inside the CLI-supplied env-fn source) is directly available and unused.
- Lower severity than Findings 9/10 — a CLI/process-boot configuration path, not a hot user-program path — but the shape (an AST with `.span()` sitting unused beside `rust_caller_span!()`) is identical.

### Finding 12 (L2) — `StdlibError`'s doc comment asserts a stale, now-false claim about `:location`

- `src/load/stdlib.rs:717-719` — *"the baked stdlib has no wat-source span, so `:location` is nil."* False against the current contract: `src/edn/contract.rs:212-220` (`location_from_span`) states *"Arc 298.2: every span is a real location... so always emitted"*, and `StdlibError`'s own test (`stdlib.rs:865-917`) asserts `:location` IS always populated (with a `.rs` file, honestly — the stdlib genuinely has no wat-source call site).
- Not a location-loss defect — actual behavior is correct — but a documentation-truthfulness gap that could mislead a future maintainer into believing this type is location-less when it structurally is not. Zero-risk fix.

### Finding 13 (L1) — the two live `rune:conformare(spanless-by-domain)` sites cite a category the substrate's own doctrine says is retired

- `src/capability/registry.rs:93` and `src/collection/eval.rs:22` — the only two `rune:conformare(...)` comments anywhere in the tree (`grep -rn "rune:conformare" src/ crates/ wat/` → exactly these two). Both read `// rune:conformare(spanless-by-domain) — ...`.
- `docs/CONFORMARE.md:136`: *"**Retired at Stone 243.4:** `rune:conformare(spanless-by-domain)`. It excused a missing location by domain... Zero-exceptions retires it: a non-source domain carries its appropriate location TYPE... There is no honest 'this error has no location.'"* `docs/CONFORMARE.md:125-127` names the **only** remaining legal category as `attested-arc` (cites an open arc + DESIGN.md path). `INSCRIPTION.md` §II reinforces: *"The only honest spanlessness is a payload that is never tossed to wat."*
- Both live sites are, on their own text, plausible domain-genuine cases (`cap_decode_error` reconstructs off a trusted wire body with no source position; the `collection/eval.rs` `_inner` helpers operate on pre-evaluated `&Value` with no AST in any call path) — but under the substrate's own **current, ratified** doctrine, "domain-genuine spanlessness" is exactly the excuse that was struck. Neither site was updated when the category was retired (243.4 predates both comments' current wording, per the doctrine's own "Retired" framing), and no `attested-arc` rune (the doctrine's replacement) was substituted.
- Four-questions: Obvious NO (a reader who trusts the comment believes a currently-legal exemption applies) · Simple — moot (this is a doctrine-citation defect, not a span-access-path defect) · Honest NO (the rune claims doctrinal standing it doesn't have) · Good UX NO (a future author copying either comment — the natural thing to do, since it's the only precedent in the tree — propagates a retired pattern).
- Pattern proposal: either (a) re-derive both sites' spans under the doctrine's actual non-source-domain rule (`EdnReadError`'s `pos`-typed alternative for `cap_decode_error`; genuinely nothing recoverable for the `collection/eval.rs` value-level helpers, in which case the honest move per `INSCRIPTION.md` is to confirm these payloads are wrapped by a spanned outer type before crossing to wat — the same carve-out `HashError` already has — and cite *that*, not a retired rune), or (b) if the fix is genuinely deferred, replace both comments with `rune:conformare(attested-arc)` citing a real tracked arc. Two sites; do not use either as a template for Findings 5/8's fix.
- Cascade: 2 sites directly; indirectly, every future author who searches the tree for "how do I mark a spanless error" and finds only these two retired-category examples.

### Finding 15 (L1, attested-arc-eligible but unruled) — a plain Rust panic (no `AssertionPayload`) never reaches the structured death-report with any location, though `PanicHookInfo::location()` is free and in scope at the exact discard point

- `src/panic_hook.rs:89-96` (`install()`'s `set_hook` closure): `if let Some(payload) = info.payload().downcast_ref::<AssertionPayload>() { render_assertion_failure(payload); return; } previous(info);` — a non-`AssertionPayload` panic is handed to the *previous* hook (Rust's default, which prints file:line:col as plain stderr text) and the wat-structured path is never entered from here at all.
- `src/kernel/error.rs:76-97` (`thread_died_error_panic`): for a plain panic, `assertion: None` → `failure_field = Value::Option(None)` (`LociDiedError::Panic{message, failure: None}`) — `message` is built from the raw panic payload string at the `catch_unwind` site (a different call frame than the hook), which has no access to `std::panic::Location` at all — `PanicHookInfo::location()` is only reachable inside the hook closure at `panic_hook.rs:89`, and nothing in that closure stashes it (e.g. into a thread-local) for the `catch_unwind` site to read afterward.
- This is `docs/arc/2026/06/296-diagnostics-fully-edn/DESIGN-296-typed-causes.md`'s named, deferred **S5** ("`ThreadDiedError::Panic` assertion-envelope — L2, later") — a legitimate, tracked deferral, but **no rune marks it** at either `panic_hook.rs:89-96` or `wat/kernel/diagnostics.wat:124-125` (`:Panic` variant declaration).
- Four-questions: Obvious NO (nothing at the hook or the declaration signals this gap is known/tracked) · Simple — moot · Honest NO (the substrate elsewhere treats "Rust locations are free and wanted" as a design principle — `span.rs`'s own module doc — yet the one channel that has a real Rust location free for the taking, the panic hook itself, doesn't take it for the non-assertion case) · Good UX NO (a plain unwrap-panic inside a wat peer thread reports as a bare string with zero location to every consumer of `LociDiedError::Panic`).
- Pattern proposal: at minimum, land the rune (citing this arc/S5) at both sites named above. The substrate's own design doc marks the actual fix (S5) "later," so scheduling it is not this cast's call — but making the deferral inspectable in place is a one-line, zero-risk addition independent of when S5 lands.
- Cascade: 1 hook installation site, 1 wat-declared variant, every consumer of `LociDiedError::Panic{failure: None}`.

### Finding 16 (L2, cosmetic) — `:wat::kernel::Failure.actual`/`.expected` are the last raw-`String` pair in an otherwise de-stringified record

- `wat/kernel/diagnostics.wat:104-113` — `Failure{error<-Error, frames<-Vector<Frame>, actual<-Option<String>, expected<-Option<String>}`. `error` and `frames` are fully structured; `actual`/`expected` are bare optional strings (the rendered `Display` text of whatever value the assertion compared), not structured `Value` snapshots.
- Low severity: `actual`/`expected` are inherently "what the user wrote as text in their assertion," so a string is arguably the right shape here (unlike Finding 6, there's no separate structured representation being discarded — the rendered text *is* the payload). Recorded as an observation for completeness of the manifest, not urged as a retrofit target.

### Finding 16b (L2, minor) — `AssertionFailure.location: Option<Location>` vs `:wat::core::Error.location: Location` (mandatory) — undocumented split in optionality between the substrate's two top-level diagnostic envelopes

- `wat/kernel/diagnostics.wat:157` — `AssertionFailure`'s `location` is `(Option :- [Location])`. `wat/core.wat:2185` — `:wat::core::Error`'s `location` is mandatory, non-`Option`, `Location`.
- Each is locally defensible: `AssertionFailure` is built even for panics with no location data at all (a pre-payload plain panic reaching this path, or — per Finding 15 — even one that never fully reaches it), so `Option` reflects a real absence; `:wat::core::Error` is the doctrine's own structural floor and is correctly non-optional. But nothing documents that these are *deliberately* different contracts rather than one having drifted from the other — a reader moving between the two nearby wat files could reasonably expect the shared word "location" to mean the same guarantee both times.
- Pattern proposal: a one-line doc note at either declaration cross-referencing the other and stating why the optionality differs would close the ambiguity at zero cost.

---

## Pattern recommendation for the substrate

**Pattern A remains, and should remain, the substrate's answer.** It is already ratified
(`docs/CONFORMARE.md`, arc 243) and already applied to essentially every type in scope
(`CheckError`, `TypeError`, `ReteCheckError`, `ParseError`, `RuntimeError`, `MacroError`, `LoadError`,
`StdlibError`, `ConfigError`, `LowerError`, `ClauseGrammarError`, `ArgSpecError`, `ExtractionError`,
plus the wat-declared `:wat::core::Error`/`Fault` surface itself). This cast's own four-questions
re-derivation agrees with the 2026-05-30 first-cast verdict (Pattern A 4/4, B 2/4, C 1/4) and finds
no reason to reopen it. The `LexError`+`LocatedLexError` two-tier shape and `ResolveError`'s
single-variant-wrapping-a-spanned-item shape are legitimate Pattern-A variants for their specific
constraints, not exceptions.

**What this cast adds** is that every remaining gap is a *second-order* failure — none is "a variant
forgot `span: Span`," which Pattern A already forecloses. Six distinct modes surfaced, none fixed by
"add a field":

1. **The mandatory span is filled with a meaningless sentinel, undocumented** (Finding 8
   `ExtractionError`; the top-level-parse arm of Finding 3 `EdnReadError`). Pattern A guarantees
   *presence*, not *meaning*.
2. **A pluggable extension point's trait bound is weaker than `WatError`**, so a boxed value inside a
   Pattern-A wrapper cannot be asked for its own location even when the concrete type could answer
   (Finding 1, `FreezeValidatorError: ToEdn`).
3. **A `String`-typed field in a wat-declared schema is used, by design, to carry a full serialized
   structured error** — inverting arc 296's "structured EDN by construction" goal by hiding structure
   inside the representation meant to expose it (Finding 6, `LociDiedError`).
4. **An ordinary Rust helper is one call-frame short of a span its caller already holds**, often with a
   sibling function in the same file proving the fix trivial (Findings 9/10/11 — `io.rs` ×3,
   `assertion.rs`, `freeze.rs`).
5. **A legitimate, doctrine-sanctioned deferral (`attested-arc`) exists in the substrate's own design
   docs but carries no rune at the actual code site** (Findings 6 and 15 — arc 296's S3/S4/S5) —
   distinct from mode 3/4 because the *decision* not to fix yet is sound; only the inspectability is
   missing.
6. **A rune citing a category the doctrine itself has retired** (Finding 13) — the doctrine document
   and the code have drifted apart in the specific way `docs/CONFORMARE.md` warns against for the
   elision-claim anti-pattern, just at the rune layer instead of the `Display` layer. `docs/CONFORMARE.md`'s
   own "Rolling audit" section (see "Cross-check" above) is the same failure mode one level up — a
   status claim, not re-verified, drifting from the tree it describes.

None of these six is a Pattern-A retrofit. They need, respectively: (1) a rune-or-fix discipline pass
over the `rust_caller_span!()` population (sub-audits sampled roughly a dozen of the reported ~600 such
sites under `src/`, per `tests/diagnostics/probe_ex003_diagnostic_locates_the_user.rs`'s own count —
this cast's Findings 3/8/9/10/11/15 sites are members of that population, not the whole of it, and a
full sweep was out of this cast's time budget); (2) widening `FreezeValidatorError`'s trait bound; (3)
landing a rune now, scheduling the field-type change per the substrate's own existing decision; (4)
threading an already-in-scope `Span`/`list_span`/`ast.span()` one parameter deeper, copying the sibling
that already does it correctly; (5) landing runes at the two named-but-unmarked deferral sites; (6)
correcting or replacing the two retired-category runes, and refreshing `docs/CONFORMARE.md`'s own
status table.

## Suggested retrofit ordering

1. **Finding 12** (`StdlibError` stale comment) and **Finding 13**'s doc half (`docs/CONFORMARE.md`'s
   rolling-audit table) — zero-risk, zero-cascade doc fixes; do first so the next auditor isn't misled
   by either.
2. **Finding 10** (`assertion.rs::eval_opt_string`) — smallest code diff (thread `args[n].span()` into
   2 arms of 1 function), highest-value target (the substrate's own assertion-failure path), cleanest
   mutation-proof (flip the span back, watch exactly 2 of 3 sibling-argument tests redden).
3. **Finding 1** (`FreezeValidatorError` bound) — one trait bound + one impl; closes a structural hole
   before a second validator crate registers into it.
4. **Finding 9** (`io.rs` ×3) — mechanical parameter threading; sibling functions already show the
   correct shape.
5. **Finding 5a** (`validate_user_main_*` signatures) — closes Finding 5's `MainSignature` arm at the
   root rather than patching each downstream consumer separately.
6. **Finding 6 / Finding 15 rune half** — land `rune:conformare(attested-arc)` at the `LociDiedError`
   declaration + both Rust homes, and at the panic-hook non-assertion arm, citing arc 296 /
   `DESIGN-296-typed-causes.md` S3/S4/S5. Cheap, immediate honesty gain; does not require the
   underlying field-type change to land first.
7. **Finding 13**'s code half (the two retired-category runes) — re-derive or replace; small, isolated.
8. **Finding 7** (`Frame.col`) — additive, no breaking shape change.
9. **Finding 8** (`ExtractionError` rune/fix) and **Finding 11** (`freeze.rs::resolve_env_program`) —
   same shape as Finding 9/10, lower traffic paths.
10. **Findings 2–4** (`JsonError`, `EdnReadError`, `WatEdnBridgeError`) — lowest urgency, EDN/JSON
    interop paths rather than the primary check/type/runtime spine — but Finding 3's message-leak arm
    (a Rust source line appearing verbatim in wat-facing prose) should be pulled forward if
    `:wat::edn::read`'s error messages are user-visible in the-little-wat's own corpus.
11. **Finding 6 / Finding 15 field-type half** (the actual `cause <- Error` retrofit and the
    hook→catch_unwind location threading) — the substrate's own arc already defers these as breaking
    changes needing their own four-questions decision; not this cast's to schedule further than "the
    rune should exist regardless of when this lands" (step 6).
12. **Findings 16 / 16b** — cosmetic; land opportunistically alongside neighboring work, not worth a
    dedicated stone.

Open, sampled but not fully driven (flagged, not asserted as findings): `src/collection/eval.rs`'s
hashset/hashmap/list `#[wat_intrinsic]` family declares no span parameter at all where sibling
intrinsics elsewhere in the tree accept one and justify skipping it with a `rune:lint(unused-span)`
comment (e.g. `src/intrinsic/char.rs:51`) — not traced far enough to confirm a span is as trivially
available here as in the plain-Rust-function cases audited above; and `src/process/clone.rs::make_pipe`
(`pipe2(2)` failure) uses `rust_caller_span!()` at a rare host-resource-exhaustion path several frames
from any wat-source AST, not traced to a confirmed discard given the time budget.
