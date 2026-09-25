# AUDIT — error shapes and backtraces, measured against HEAD

> Read-only measurement audit. **No build, no `cargo` invocation** — driven `target/release/wat`
> directly against `1c4c09c82` (branch `reason/little-wat-findings`). Every claim below cites a
> `file:line` read or a command run during this session; the corpus itself was **re-executed
> fresh** against HEAD rather than trusted from the prior capture (see "Corpus provenance").

## Corpus provenance — the inherited corpus is STALE; re-run, not trusted

The orchestrator's brief cited 166 non-empty `.err` files at
`/tmp/claude-1000/-home-john-work-holon/fd01e281-.../scratchpad/loc/*.err`, one per probe in
`the-little-wat/probes/` (364 probes total: 166 error, 198 clean — confirmed by byte-checking
every file for emptiness, `[ -s "$f" ]`).

That capture predates several commits on this same branch (`587f9bd78` stone P, `d3bed0d72`
stone O, `04ddfce66`/`d492a6805` stone Q, `70e6c56b0`/`1c4c09c82` stone R — all landed *after* the
corpus timestamp). **I re-ran all 364 probes fresh against `target/release/wat` at `1c4c09c82`**
(`the-little-wat/probes/<relpath>.wat`, `timeout -s KILL 25`, stderr captured) and diffed
byte-for-byte against the inherited corpus:

- **1 shape change**: `probes/ml/newtype-value.wat` — was `LociDiedError.Panic` (a genuine Rust
  `panic!` at `crates/wat-edn/src/value.rs:330:33`, `invalid keyword name "0": first character
  must be non-numeric`), now **empty (clean exit)**. This is stones Q/R (`04ddfce66`,
  `d492a6805`, `70e6c56b0`) — "printing a newtype does not panic" / "a newtype is tagged" — fixing
  exactly the corpus's *only* `Panic` instance out from under the measurement.
- **6 content diffs on files that keep their shape**, 3 of which are the Rust-location-sentinel
  fixes stones O/P describe verbatim: `probes/annoy/load-clj-missing.wat`, `probes/name-lt.wat`,
  `probes/typer/unicode-in-source.wat` each **lost** a `"src/load/loader.rs"` /
  `"crates/wat-reader/src/parser.rs"` primary `:location` and now show the user's own `.wat`
  file/line. The other 3 (`doctest/examples-census`, `doctest/unmask-failures`,
  `service-fn-put`) differ in message text from unrelated code drift (a type-rendering change, a
  renamed generated record, an assertion-count change) — not error-shape or backtrace relevant.
- **165 non-empty files are byte-identical** to the stale corpus, including all 11 of the
  "StartupError flattening" files and all 21 `RuntimeError` / 3 `MainSignature` files. **These
  bugs are confirmed live at `1c4c09c82`, not stale artifacts.**

Net: **the orchestrator's top-level shape counts (130 StartupError / 21 RuntimeError / 11
AssertionFailure / 3 MainSignature / 1 Panic) undercount by exactly the one probe that's since
been fixed** — at HEAD it is 130 / 21 / 11 / 3 / **0**, and the 199th (not 198th) probe is clean.
The "35 of 166 flatten a whole structured error into `:message`" figure — 21 RuntimeError + 11
StartupError(Validator) + 3 MainSignature — **holds exactly**, measured independently below.

## 1. Every user-facing error shape

### The LociDiedError envelope (`wat/kernel/diagnostics.wat:113-139`, current)

Seven variants, ONE enum (`:wat::enum::Pure`), each field's structuredness as declared:

| Variant | Fields | Structured? |
|---|---|---|
| `Panic` | `message <- String`, `failure <- Option<Failure>` | `message` is **String**; `failure` is structured when the panic carried an `AssertionPayload` |
| `RuntimeError` | `message <- String` | **String only** |
| `Disconnected` | (none) | n/a |
| `Stopped` | (none) | n/a |
| `StartupError` | `error <- :wat::core::Error` | **Structured** (the one variant upgraded) |
| `EntryFormFailure` | `message <- String` | **String only** (never observed built by any Rust site — see below) |
| `MainSignature` | `message <- String` | **String only** |
| `BadReturn` | `message <- String` | **String only** (never observed in the corpus) |

`src/kernel/error.rs:265-299` (`died_error_payload_message` / `eval_died_error_message`) states
this taxonomy in its own comments: *"Every OTHER carrying variant (`Panic`/`RuntimeError`/
`EntryFormFailure`/`MainSignature`/`BadReturn`) still carries a bare `Value::String`"* (line
270-271) — i.e. the Rust source **names its own gap**. `EntryFormFailure` has no builder anywhere
in `src/` (`grep -rn "EntryFormFailure"` matches only the decode side in `kernel/error.rs` and the
`types.rs` retirement comment) — a declared, dead variant.

`Failure` (`diagnostics.wat:107-111`) and `AssertionFailure` (`:141-161`) both carry `error <-
Error` (structured) plus `frames <- Vector<Frame>`, `actual`/`expected <- Option<String>`. These
are the only two record shapes with a `frames` field in the whole taxonomy.

### Where structure gets stringified — every site found

**Site 1 — `StartupError::RuntimeError`/`MainSignature`/`BadReturn` builders themselves
(`src/process/died.rs:107-108, 128-129, 149-150`).** `process_died_error_runtime_value` /
`_main_signature_value` / `_bad_return_value` all call `crate::edn::contract::to_wire_edn(e)`
(`src/edn/contract.rs:339-341`, `wat_edn::write(&e.error_edn())`) — **the full rendered EDN text
of the inner structured error** — and hand that **String** to the `LociDiedError` builder, which
has nowhere but a `message: String` field to put it (Site above). This is not a bug in
`to_wire_edn` — it's doing exactly its documented job of producing wire *text*; the bug is that
the `LociDiedError` variant it feeds has no structured slot, so a perfectly good `OwnedValue` tree
gets flattened to text one layer too early. **Measured**: 21/21 `RuntimeError` corpus files and
3/3 `MainSignature` files show exactly this — one JSON-in-JSON-style escaped blob, e.g.
`probes/ctor-wat-type-concrete.wat.err`: `:message "#wat.runtime/UnknownFunction {:message
\"unknown function: …\" :location #wat.core/Span {…} …}"`.

**Site 2 — `StartupError::message()`'s `Validator` arm (`src/macros/error_edn.rs:157`):**
```rust
SE::Validator(e) => crate::edn::contract::first_line(e.to_string()),
```
`e: &Box<dyn FreezeValidatorError>` (`src/freeze/validator.rs:21-31` — blanket-implemented for any
`ToEdn + Debug + Display + Send + Sync`, no `WatError`, so no `.message()` accessor exists on the
trait object). `e.to_string()` calls `Display`, and **every error type in this codebase**
implements `Display` as `f.write_str(&to_wire_edn(self))` (e.g. `ReteCheckErrors`,
`src/rete/validate/error.rs:577-581`) — i.e. Display means "dump the full wire EDN," not "give a
headline." `ReteCheckErrors::message()` itself is correct and short (`"N rete rule validation
errors"`, `error.rs:634-637`), and its `variant()` is correctly structured (each item via
`.error_edn()`, `error.rs:644-647`) — but `StartupError::message()`'s `Validator` arm never calls
that; it goes through `Display` instead, so the OUTER `error_edn()` composition
(`src/edn/contract.rs:109-133`, the "floor" that inserts `:message`) stamps the **entire rendered
inner error, self-included**, into the outer `:message` field, sitting *beside* the correctly
structured `:errors` vector built from `variant()`. The result is a value that carries its own
data twice: once flattened in `:message`, once correctly structured in `:errors`.

Verified directly in the corpus, e.g. `probes/rete/fire-once-oracle-empty.wat.err`:
```
[#wat.kernel/LociDiedError.StartupError {:error #wat.rete/ReteCheckErrors
  {:message "#wat.rete/ReteCheckErrors {:message \"7 rete rule validation errors\" ... 
             :errors [#wat.rete/MalformedClause {...} ...]}"      ; <- flattened duplicate
   :location nil :causes []
   :errors [#wat.rete/MalformedClause {:rule "fo::shippable" ...} ...]}}]  ; <- correct, structured
```
**Measured: exactly 11 files** exhibit this (all through the rete `defrule` wall, the only
registered `FreezeValidator` today — `src/freeze/validator.rs`, first consumer
`validate_rete_rules`): `rete/fire-verbs-compared`, `rete/accumulator-count-leak`,
`rete/derived-multiplicity(-tally)`, `rete/explain-support`, `rete/fire-once-oracle-empty`,
`rete/native-vs-oracle(-tally)`, `rete/retraction-scenarios`, `rete/insert-native-vs-oracle`,
`rete/accumulator-empty-pass` — matching the orchestrator's "11 StartupError" count exactly (I
also checked 5 *other* StartupError files with escaped quotes — `java/struct-in-pure-enum`,
`service-fn-state(-impure)`, `service-fn-put`, `name-lt` — and confirmed those are ordinary
messages quoting an identifier in prose, e.g. `"variant \"Trans\" field \"q\""`, not flattened
structure; escaped-quote *count* alone is not a reliable detector, structural size is —
4-8 occurrences of `\"` for genuine quoting vs. 62-132 for a true flatten).

**Because any future `FreezeValidator` registrant inherits the same blanket impl, this is not a
one-off — it is a trap built into the extension point itself: any validator whose `Display` follows
this codebase's own `to_wire_edn`-via-Display convention (which is otherwise the CORRECT, intended
pattern — see `StartupError`/`RuntimeError`/`CheckErrors`'s own `Debug`/`Display` impls, all
`f.write_str(&to_wire_edn(self))`) will double-flatten the moment it's wired through
`StartupError::Validator`.** The fix is not "stop using that Display convention" (arc 296 wants it)
— it's that `message()` for `Validator` needs its own short-headline source, not `.to_string()`.

**Site 3 — `runtime_error_to_eval_error_value`'s fallback arm (`src/runtime.rs:12318`):**
```rust
_ => ("runtime-error", format!("{}", err)),   // err: &RuntimeError
```
This is the wat-visible `:wat::core::EvalError` surface (what a `(try …)`/`eval-safely`-style
construct hands back to a *wat program*, not the process-death channel). `RuntimeError`'s
`Display` is `to_wire_edn(self)` (full wire EDN), so any unarmed `RuntimeErrorKind` hands the
catching wat code a message string containing the whole nested error as text. This is the site
`docs/arc/2026/06/296-diagnostics-fully-edn/NOTE-24-of-39-error-kinds-…` flagged at
`00146f9bc`+O-iv-a as 24/39 unarmed. **Re-measured fresh at HEAD** (the note's own line numbers
have drifted, so I re-derived per its own reproduction recipe):
```
awk '/pub enum RuntimeErrorKind/,/^}/' src/value/signal.rs | grep -oP '^\s{4}\K[A-Z][A-Za-z]+' | sort -u   # 40 kinds
sed -n '12247,12320p' src/runtime.rs | grep -oP 'RuntimeErrorKind::\K[A-Z][A-Za-z]+' | sort -u             # 15 armed
```
**40 kinds today (was 39), 15 armed, 25 unarmed (was 24)** — one more kind (`ReteCeiling`) has
been added since the note and inherited the same gap. This is a distinct surface from the
LociDiedError corpus above (it never appears on stderr; it's an in-process `Value`), but it's the
same architectural mistake — `Display` used where a short message was wanted — and it is NOT
closed by anything the arc-296 docs claim finished.

### Everything the AUDIT-prose-in-errors.md (2026-06-30) catalog flagged is fixed today

`docs/arc/2026/06/296-diagnostics-fully-edn/AUDIT-prose-in-errors.md` cataloged 10 findings (9 L1,
1 L2) of structure-flattened-to-prose in the `check`/`type`/`load` error families and proposed a
`#[derive(WatErrorRecord)]` constraint as the cure. I checked all 10 against HEAD by reading the
current code (not the audit's line numbers, which have moved):

| # | Finding | Status at `1c4c09c82` |
|---|---|---|
| 1 | `DefRestrictedCallerNotAllowed.prefixes` `.join(" ")` | **Fixed** — `src/check/error.rs:307-311`: plain `Vec<String>` field, blanket `Vec<T>: ToEdn` gives a real Vector |
| 2 | `NoMatchingClauseAtCallSite.called_arg_types` `.join(", ")` | **Fixed** — same file, plain `Vec<String>` field, no `.join` |
| 3 | `NoMatchingClauseAtCallSite.attempted_clauses` dropped (`_`) | **Fixed** — `error.rs:317-322`, `#[to_edn(via = crate::check::clause_attempts_to_edn)]`, which builds `[{:arity N :param-types […]} …]` (`src/check.rs:435-441`) |
| 4,5,6 | `ReturnTypeMismatch`/`MalformedForm`/`MalformedVariant` `.remedies` via `render_remedies()` prose blob | **Fixed in the EDN path** — `#[to_edn(via(key="remedies", fn=…))]` (`error.rs:99,116`) routes remedies through `remedies_to_edn` (structured `Vector<Remedy>`, `src/check.rs:399-433`). `render_remedies` still exists but is now confined to `CheckErrorKind::fmt_with_span` (`error.rs:429-494`), the **human `Display`** path — explicitly out of scope per `DESIGN.md`'s "the human-readable Display path stays" |
| 7,9 | `HashError` foreign-message via `.to_string()` | **Fixed** — `impl ToEdn for HashError` exists (`src/hash.rs:563`) |
| 8 | `LoadFetchError` "foreign opaque" via `.to_string()` | **Fixed** — `impl ToEdn for LoadFetchError` exists (`src/load/loader.rs:224`); confirmed live: driving `probes/annoy/load-clj-missing.wat` at HEAD yields `:cause #wat.kernel/NotFound {:path "no-such-file.wat"}`, structured, not a string |
| 10 | `EdnCoerceMismatch.path` dot-notation string | **Fixed** — `#[to_edn(via = crate::edn::error::edn_path_segments)]` (`src/value/signal.rs:606`) splits into a real `Vector` of segments (`src/edn/error.rs:56-60`) |
| — | `CheckErrors.:message` full multi-line Display dup of `:errors` | **Fixed** — `src/check/error_edn.rs:86-89`: `message()` is `"{n} type-check error{s}"`, a headline; `variant()` embeds each item's own `error_edn()` |

**None of this happened via the proposed `#[derive(WatErrorRecord)]`** — `grep -rn
"WatErrorRecord"` across `src/` and `crates/` returns nothing; it was never built. The fixes
instead arrived piecemeal as a field-level `#[to_edn(via = …)]` attribute mechanism on the
existing derive. **The 10-finding class the audit named is closed; the mechanism it prescribed is
not the one that closed it**, and the identical *pattern* (Display-as-headline) is what's live
today at Site 2 and Site 3 above, on a namespace (`FreezeValidator`, `EvalError`) the audit never
covered because both postdate it.

## 2. Backtraces — what exists, not a design

**Yes, the runtime keeps a wat call stack.** `CALL_STACK` (`src/value/frame.rs:29-31`,
thread-local `Vec<FrameInfo>`) is pushed/popped by `FrameGuard`, whose **only** push site in the
entire tree is `apply_function` (`src/runtime.rs:11137`): `let _frame_guard =
FrameGuard::push(callee_name_initial, cur_span.clone());`, scoped around the whole trampoline body
— every wat function call, tail-call-collapsed. `snapshot_call_stack()`
(`src/value/frame.rs:74-80`) returns it newest-first.

**At the moment a `RuntimeError` is raised, the stack is live and non-empty** — proven directly:
`RuntimeError::new(cur_span.clone(), RuntimeErrorKind::ArityMismatch {…})` at
`src/runtime.rs:11151` executes *inside* `_frame_guard`'s scope (declared at line 11137, 14 lines
above). `RuntimeError::new` (`src/value/signal.rs:127-132`) stores only `span` + `kind` — it never
calls `snapshot_call_stack()`. **The frames exist in thread-local storage at construction time and
are simply never read for this path** — and even if they were, `LociDiedError::RuntimeError` has
no `frames` field to hold them (§1). This is a two-part gap: the capture call is missing, and the
wire shape has nowhere to put the result if it were added.

**`:frames` is built exactly one way, everywhere it's built**: assemble an `AssertionPayload`
(`message, actual, expected, location, frames: snapshot_call_stack(), …`) and
`std::panic::panic_any(payload)`. Every site that populates `frames` does this identically:
`assertion.rs:160-180` (`assertion-failed!`), `kernel/abort.rs:74-90` (`raise!`),
`collection/eval.rs:1494-1503` (`nth` out-of-range), `services/verbs.rs:271-286` (`eprintln!`
terminate). The panic hook (`src/panic_hook.rs:89-98`) downcasts `info.payload()` to
`AssertionPayload`; on a hit it renders `AssertionFailure`; a `catch_unwind` elsewhere converts
the same payload into `Failure` (nested inside `LociDiedError::Panic`). **`frames` exists only on
the panic channel because `AssertionPayload` is the only vehicle that carries it, and
`Result<_, RuntimeError>` never constructs one.** This is not a considered "assertions matter more"
policy — it falls out of two independent facts (only panics build `AssertionPayload`; only
`Failure`/`AssertionFailure` declare a `frames` field) that happen to coincide.

**Non-assertion Rust panics fall through entirely**, by explicit design
(`panic_hook.rs:94-97`, `previous(info)` — "typically Rust's default"). This is not hypothetical:
the corpus's *only* pre-fix `Panic` instance, `probes/ml/newtype-value.wat` (now fixed, §
provenance), showed it happening —
```
thread 'main' (3111672) panicked at crates/wat-edn/src/value.rs:330:33:
invalid keyword name "0": first character must be non-numeric
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
[#wat.kernel/LociDiedError.Panic {:message "…" :failure #wat.core/Option.None {}}]
```
— Rust's own unstructured panic line (with a genuine, correct Rust `file:line:col`) printed
**first and separately**, then a `LociDiedError::Panic` with `failure: None` (no `AssertionPayload`
existed to recover — the downcast in the catching layer failed and fell back to the raw message).
**`RUST_BACKTRACE=1` is Rust's own mechanism** (`std::backtrace::Backtrace`, invoked by Rust's
default hook) and is completely disjoint from wat's `:frames` — it was never wired in (`grep -rn
"std::backtrace\|Backtrace::" src/` returns **zero** hits anywhere in this crate). If a user set
that env var, they'd get a second, unstructured, un-merged Rust backtrace glued above the wat EDN
line, not a unified trace.

**Where a Rust frame legitimately appears today**: exactly one live case, the `:user::main`
call itself. `src/freeze.rs:1541-1542`: `apply_function(main_func, args, runtime.symbols(),
crate::rust_caller_span!())` — the Rust host has no wat AST call form for this synthetic
top-level invocation, so it stamps its own `file!()/line!()/column!()` as the call span
(`rust_caller_span!()`, `src/span.rs:15-23` — literally `Span::new(file!(), line!(), column!())`,
the SAME `Span` shape a wat location uses; a Rust location and a wat location are
type-indistinguishable, only the string content differs). Driving `probes/assert-fail.wat` at HEAD
confirms the resulting two-level, honest frame stack:
```
:frames [#wat.kernel/Frame {:file "probes/assert-fail.wat" :line 7 :symbol ":wat::test::assert-eq"}
         #wat.kernel/Frame {:file "src/freeze.rs" :line 1542 :symbol ":user::main"}]
```
Innermost = the user's own call; outermost = the true (non-wat) caller, honestly named. **This is
the ONE place the "Clojure has Java in its traces" intent is realized today** — but it stops at
that single boundary frame. Nothing captures a Rust stack *beneath* it (inside `apply_function`,
`eval_inner`, or a native primitive's own call chain) — there is no second mechanism to extend it.

**What a full stack (wat + Rust, user location primary) would need, given what exists**: (a) a
`frames`-shaped field on `LociDiedError::RuntimeError` (currently absent — a `wat/kernel/
diagnostics.wat` change); (b) a call to `snapshot_call_stack()` at `RuntimeError::new`
construction (the data is already live there, per the proof above — no new capture mechanism, just
a missing read); (c) for genuine Rust-internal panics, either routing them through
`AssertionPayload` (impossible for a real `panic!`/`unwrap()` inside arbitrary Rust code without
touching every call site) or a global panic hook that calls `std::backtrace::Backtrace::capture()`
and folds it into the SAME EDN shape the wat frames use (unbuilt — zero references) rather than
falling through to Rust's own hook.

## 3. Arc 296's closure claims vs. today

**`INSCRIPTION.md` does not exist.** `DESIGN.md`'s status line (`docs/arc/2026/06/
296-diagnostics-fully-edn/DESIGN.md:3-5`) reads *"Status: CLOSED (2026-06-30) — slices 296.2–296.5
landed, gate 4157/0/91… See INSCRIPTION.md for the full close record."* `find docs -iname
'*INSCRIPTION*'` finds three sibling arcs' inscriptions (170, 038, 298) but **none for 296** — the
arc's own citation for its closure record is a dead link; I cannot check the claimed gate number
(4157/0/91) against anything, and this audit does not run `cargo` to re-derive it.

Row-by-row against DESIGN.md's decomposition (`DESIGN.md:44-59`):

| Slice | Claim | Holds today? |
|---|---|---|
| 296.2 | Mint `ToEdn` trait; `RuntimeError`/`MacroError`/`StartupError`/`Span`/… implement it | **Yes** — confirmed throughout `src/edn/contract.rs`, `src/macros/error_edn.rs` |
| 296.3 | "Bring the stringly holdouts under the trait… non-Macro `StartupError`, `MainSignature`, the `ProcessDiedError` family: change the payload fields from `String` → a tagged-EDN value" | **Partially false today.** Only `StartupError` got a structured field (`error <- :wat::core::Error`, via a *later* arc — H2c, `wat/kernel/diagnostics.wat:113-139`). `RuntimeError`, `Panic`, `EntryFormFailure`, `MainSignature`, `BadReturn` — 5 of 6 payload-carrying `LociDiedError` variants — **remain `message <- :wat::core::String` today**, and `src/kernel/error.rs:270-271`'s own comment says so in present tense. |
| 296.4 | Retire the interim `Diagnostic` type | Not independently verified this session (out of the corpus's reach — no probe exercises `--check-output`) |
| 296.5 | "The wall + close… a probe that a new error variant without an impl fails to compile" | **Yes, this part is real** — `to_wire_edn`'s `compile_fail` doctest (`src/edn/contract.rs:311-327`) demonstrates the generic-over-`WatError` fence |

A stronger, more falsifiable claim sits in a doc `wat/kernel/diagnostics.wat` itself points at:
`docs/arc/2026/06/278-rules-engine/DESIGN-loci-died-error.md:1-7` — *"Status: BUILT + SHIPPED
(`d60b1887`, 2026-07-24)… The whole death/crash surface is now structured EDN end-to-end: error →
`:wat::core::Error`, frames → `Vector<Frame>`, location → `Location`, chain →
`Vector<LociDiedError>`. **Zero string-wrapping remains.**"* This is directly and measurably false
today: the 21 `RuntimeError` + 3 `MainSignature` corpus files are string-wrapped by construction
(§1 Site 1), confirmed live at HEAD, not a regression since — the wire type has never had anywhere
else to put the data.

**The AUDIT-prose-in-errors.md worklist (§1 above) is fully resolved** for the 10 findings it
named, though not via its prescribed derive, and a structurally identical failure mode
(Display-as-headline) now exists on two surfaces (`StartupError::Validator`,
`runtime_error_to_eval_error_value`'s fallback) neither audit covered.

## 4. Rust-location usage — counts and sites

Measured against the **fresh** 364-probe re-run (`audit-fresh2/*.err`, this session):

- **Primary `:location`/`:span` (a Rust path replacing the user's own)**: **0 of 364** at HEAD.
  Was 3 in the stale corpus (`probes/annoy/load-clj-missing.wat` → `src/load/loader.rs:438`;
  `probes/name-lt.wat` and `probes/typer/unicode-in-source.wat` → both
  `crates/wat-reader/src/parser.rs:201`, the shared lex-error path) — all three fixed by stones
  O/P (`d3bed0d72`, `587f9bd78`), confirmed by re-driving both probes directly:
  `probes/annoy/load-clj-missing.wat` now reports `:location #wat.core/Span {:file
  "…/probes/annoy/load-clj-missing.wat" :line 3 …}`; `probes/name-lt.wat` now reports `:file
  "…/probes/name-lt.wat" :line 5 :col 19`.
- **Inside a `:frames` entry (the wanted shape)**: **1 of 364** — `probes/assert-fail.wat`, the
  `:user::main` boundary frame (`src/freeze.rs:1542`, §2). This is the only case in the corpus of
  a Rust location appearing *correctly*, as a named outer frame beside the user's own, never
  overwriting it.
- **Inside message TEXT (free prose, via Rust's own default panic hook, not wat EDN at all)**:
  0 of 364 today — the corpus's one instance (`probes/ml/newtype-value.wat`,
  `crates/wat-edn/src/value.rs:330:33`) was fixed by stones Q/R since capture (§ provenance); no
  replacement instance was found in the fresh 364-probe sweep.
- Total files with any `.rs"` substring anywhere in their stderr, fresh: **1 of 364**
  (`probes/assert-fail.wat`, the correct §2 case) — down from 4 of 166-error-files in the stale
  corpus.

This shows the excursus's stones B/O/P genuinely closed the "Rust sentinel masquerading as
`:location`" class for the cases this corpus exercises — I did not find a fourth, unfixed instance
in the fresh sweep — while the separate, still-open question (§§1-2) is that a *deliberate*
Rust frame (assert-fail's case) is the only backtrace entry point of its kind, and the
`RuntimeError`/`Panic`/`MainSignature`/`BadReturn` families have neither a place to carry more
`:frames` entries nor a capture call feeding one.

## Corrections to the orchestrator's brief

1. **Top-level shape counts are off by one probe, not by nothing**: at HEAD it is 130
   StartupError / 21 RuntimeError / 11 AssertionFailure / 3 MainSignature / **0** Panic (was 1),
   199 clean (was 198) — `probes/ml/newtype-value.wat` was fixed by stones Q/R after the corpus
   was captured. The "35 of 166 flatten" figure and the "11 StartupError" sub-count both hold
   exactly on independent re-measurement.
2. **The corpus itself is stale relative to HEAD** on at least 4 of 166 files (1 shape change + 3
   Rust-location fixes) — this matters most for item 4 (Rust-location usage), where the *right*
   answer at HEAD (0 sentinel misuses, 1 correct frame) is qualitatively different from what the
   stale corpus alone would suggest (3 sentinel misuses). I did not trust the inherited capture;
   I re-ran all 364 probes fresh and diffed.
3. **The "StartupError's variant IS structured, so where does the text come from" question has an
   exact, single-site answer** (`src/macros/error_edn.rs:157`, `SE::Validator(e) =>
   first_line(e.to_string())`) that the brief's framing ("H-2c's structured cause") doesn't
   surface — the flattening is not in `startup_error_chain_edn`'s cause construction (which is
   correctly structured, `src/process/verbs.rs:86` `cause_edn = e.error_edn()`); it's one layer
   up, in `StartupError`'s OWN `message()` accessor, feeding the outer floor's `:message` key.
4. **`src/process/verbs.rs:44-53`'s doc comment is stale relative to its own function body**: it
   describes a Macro-only structured path with "all OTHER StartupError variants" falling back to
   `process_died_error_startup_value(format!("{}", e))` — but `emit_startup_error_structured_exit`
   (lines 54-57) calls `startup_error_chain_edn` unconditionally, with no such branch; that
   fallback path does not exist in the function below the comment. Confirmed structurally correct
   for a non-Macro StartupError by driving `probes/annoy/load-clj-missing.wat` (a `Fetch`
   variant, not Macro) and getting a fully structured, non-string `:cause`.
5. Two additional stringification sites were not in the orchestrator's brief scope but bear on
   "every site that stringifies structure": `runtime_error_to_eval_error_value`'s fallback arm
   (`src/runtime.rs:12318`, the wat-visible `EvalError` surface, 25 of 40 `RuntimeErrorKind`
   variants unarmed — re-measured fresh, up from the 24/39 a 2026-08-28 note recorded), and the
   `AUDIT-prose-in-errors.md` 10-finding catalog, which is **fully resolved today** (worth knowing
   before re-deriving it) but not via its prescribed `#[derive(WatErrorRecord)]`, which was never
   built.
