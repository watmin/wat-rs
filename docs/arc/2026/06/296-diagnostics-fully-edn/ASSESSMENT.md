# Arc 296 — Error Landscape Assessment

Worklist document for the strongly-tagged error-record system.
Produced: 2026-06-30. Anchor: `/home/watmin/work/holon/wat-rs`.

---

## 1. Error Family Catalog

Each entry reports: Rust definition location, shape (Pattern A / bare enum / other),
variant/field inventory, current EDN form/tag, and how span / message / kind are handled.

---

### 1.1 `CheckError` / `CheckErrorKind` / `CheckErrors`

**Rust definition:** `src/check/error.rs:24–206` (struct + kind enum), `src/check/error.rs:616–628` (collection).

**Shape:** Pattern A — `CheckError { span: Span, kind: CheckErrorKind }`. Span mandatory at
construction; `Span::unknown()` is the explicit sentinel. The collection `CheckErrors(Vec<CheckError>)`
is a newtype wrapper with its own Display.

**Variants (30):** `ArityMismatch`, `TypeMismatch`, `ReturnTypeMismatch`, `UnknownCallee`,
`MalformedForm`, `CommCallOutOfPosition`, `ScopeDeadlock`, `ProcessJoinBeforeOutputDrain`,
`ProcessJoinHoldsStdinSender`, `ChannelPairDeadlock`, `BareLegacyPrimitive`,
`BareLegacyUnitType`, `BareLegacyUnitName`, `BareLegacyLetStar`, `BareLegacyLambda`,
`BareLegacyLowercaseFn`, `BareLegacyContainerHead`, `BareLegacyStreamPath`,
`BareLegacyTelemetryServicePath`, `BareLegacyLruCacheServicePath`, `BareLegacyKernelQueuePath`,
`SandboxScopeLeak`, `DefRedefForbidden`, `DefRedefTypeChange`, `BareLegacyMainSignature`,
`BareLegacyForkProgram`, `BareLegacySpawnProgram`, `BareLegacyConsolePath`,
`DefRestrictedCallerNotAllowed`, `NoMatchingClauseAtCallSite`, `GuardExprNotBoolean`,
`EnsureFnInvalid`, `HygieneScopeDivergence`.

**Current EDN form (post-296):** `src/check/error_edn.rs` — each variant becomes
`#wat.kernel/<VariantName> {:field … :field … :span {…}}`.
`CheckErrors` becomes `#wat.kernel/CheckErrors {:errors [<tagged-item> …]}`.

**Span handling:** Span elided when `Span::unknown()` (via `push_span`). However,
the span field key NAME is inconsistent across variants within this single serializer:
- Most variants: `:span`
- `ScopeDeadlock`, `ChannelPairDeadlock`, `BareLegacyPrimitive`, `BareLegacyUnitType`,
  `BareLegacyUnitName`, `BareLegacyLetStar`, `BareLegacyLambda`, `BareLegacyLowercaseFn`,
  `BareLegacyContainerHead`, `BareLegacyStreamPath`, `BareLegacyTelemetryServicePath`,
  `BareLegacyLruCacheServicePath`, `BareLegacyKernelQueuePath`, `BareLegacyMainSignature`,
  `BareLegacyForkProgram`, `BareLegacySpawnProgram`, `BareLegacyConsolePath`,
  `DefRestrictedCallerNotAllowed`: `:location`
- `ProcessJoinBeforeOutputDrain`: `:join-location` (primary) + `:output-location` (secondary)
- `ProcessJoinHoldsStdinSender`: `:join-location` (primary) + `:bind-location` (secondary)
- `SandboxScopeLeak`: `:call-span` (primary) + `:outer-define-span` (secondary)
- `DefRedefForbidden`, `DefRedefTypeChange`: `:prior-loc` + `:current-loc`

**Message:** Not a field in EDN — the Display message is derivable from the structured fields
but is not included as a `:message` key. Consumer must reconstruct from fields.

**Note:** `attempted_clauses` in `NoMatchingClauseAtCallSite` is dropped in EDN
(`attempted_clauses: _` at `src/check/error_edn.rs:334`) — silent data loss at the EDN boundary.

---

### 1.2 `TypeError` / `TypeErrorKind`

**Rust definition:** `src/types/error.rs:16–180`.

**Shape:** Pattern A — `TypeError { span: Span, kind: TypeErrorKind }`. Span mandatory.

**Variants (18):** `DuplicateType`, `ReservedPrefix`, `MalformedDecl`, `MalformedName`,
`MalformedField`, `MalformedVariant`, `MalformedTypeExpr`, `AnyBanned`, `CyclicAlias`,
`AliasArityMismatch`, `InnerColonInCompoundArg`, `CyclicUnion`, `EmptyUnion`,
`SingleMemberUnion`, `InvalidUnionMember`, `CyclicSubtype`, `ImpureFieldInPureAggregate`,
`ImpureVariantFieldInPureEnum`.

**Current EDN form:** Inline `ToEdn` impl at `src/types/error.rs:309–448` — each variant
becomes `#wat.kernel/<VariantName> {:span {…} <variant fields>}`.

**Span handling:** Elided when unknown via `push_span_field` (shared helper from `src/to_edn.rs`).
Span key consistently `:span` for all variants. Well-behaved.

---

### 1.3 `RuntimeError` / `RuntimeErrorKind`

**Rust definition:** `src/value/signal.rs:103–106` (struct) and `src/value/signal.rs:120–356`
(kind enum). Also exports `EvalBreak` (wrapper) and `EvalSignal` (internal signals; NOT errors).

**Shape:** Pattern A — `RuntimeError { span: Span, kind: RuntimeErrorKind }`. Span mandatory.
Two "freeze pair" variants (`UserMainMissing`, `EvalVerificationFailed`) always constructed
with `Span::unknown()`.

**Variants (28):** `UnboundSymbol`, `UnknownFunction`, `NotCallable`, `TypeMismatch`,
`ArityMismatch`, `BadCondition`, `MalformedForm`, `ParamShadowsBuiltin`, `DivisionByZero`,
`DuplicateDefine`, `ReservedPrefix`, `DeclarationInExpressionPosition`,
`EvalForbidsMutationForm`, `UserMainMissing`, `EvalVerificationFailed`,
`ChannelDisconnected`, `NoEncodingCtx`, `NoSourceLoader`, `NoMacroRegistry`,
`MacroExpansionFailed`, `PatternMatchFailed`, `EffectfulInStep`, `NoStepRule`,
`AssertionFailed`, `SandboxScopeLeak`, `ServiceNotRunning`, `EdnCoerceMismatch`,
`UnknownField`, `NoMatchingClause`, `PostconditionFailed`, `MacroAbort`.

**Current EDN form:** `src/runtime_error_edn.rs:41–269` — each variant becomes
`#wat.kernel/<VariantName> {<fields> :span {…}}`.

**Span handling — DIVERGES from CheckError:** Runtime error EDN ALWAYS emits `:span`
even when `Span::unknown()`. `span_val` calls `span_to_edn` unconditionally, producing
`{:file "<runtime>" :line 0 :col 0}` for the unknown sentinel instead of eliding it.
This is different from `CheckError`'s `push_span` (which elides unknowns).

`SandboxScopeLeak` uses `:call-span` + `:outer-define-span` (not the standard `:span`).
`PostconditionFailed` uses `:body-span` + `:ensure-span` (dual spans).
Freeze pair variants (`UserMainMissing`, `EvalVerificationFailed`) emit empty maps or
no span at all.

**Additional sub-values:** `NoMatchingClause` embeds `ClauseAttempt` / `ClauseFailureReason`
tagged-values (src/runtime_error_edn.rs:411–449). `TypeMismatch` and `NotCallable` embed
`ValueSnapshot` tagged-maps (`src/runtime_error_edn.rs:276–307`).

---

### 1.4 `MacroError` / `MacroErrorKind`

**Rust definition:** `src/macros/error.rs:8–88`.

**Shape:** Pattern A — `MacroError { span: Span, kind: MacroErrorKind }`. Span mandatory.

**Variants (11):** `DuplicateMacro`, `ReservedPrefix`, `MalformedDefmacro`, `ArityMismatch`,
`ArityTooFew`, `UnboundMacroParam`, `SpliceNotSequence`, `ExpansionDepthExceeded`,
`MalformedTemplate`, `RefusedInMacro`, `ProgramBodyIntroducesName`,
`ProgramBodyEvalFailed` (carries `Box<MacroError>`), `MacroEvalRuntimeFailed` (carries
`Box<RuntimeError>`).

**Current EDN form:** `src/macros/error_edn.rs:38–127` — each variant becomes
`#wat.kernel/<VariantName> {:span {…} <fields>}`. `ProgramBodyEvalFailed` and
`MacroEvalRuntimeFailed` carry typed sub-causes via recursive `to_edn` calls.

**Span handling:** Always emits `:span` via `span_val` (same unconditional-emit divergence
as RuntimeError, since `span_val` calls `span_to_edn` without an unknown-elide guard).

---

### 1.5 `ResolveError`

**Rust definition:** `src/resolve/error.rs:12–50`.

**Shape:** BARE ENUM — `enum ResolveError { UnresolvedReferences(Vec<UnresolvedReference>) }`.
No outer Pattern-A struct; spans live on the individual `UnresolvedReference` items (each has
a `span: Span` field at `src/resolve/error.rs:21`). This is the only error family that
breaks Pattern A.

**Current EDN form:** `src/resolve/error.rs:54–83` — produces
`#wat.kernel/UnresolvedReferences {:unresolved [#wat.kernel/UnresolvedReference {…} …]}`.
Each item carries `:path`, `:context`, and optionally `:span` (elided when unknown via
`push_span_field`).

**Span handling:** Correct — per-item spans elided when unknown. But the outer type has
NO span. For a round-trip record, there is no single canonical outer span.

---

### 1.6 `ParseError` / `ParseErrorKind`

**Rust definition:** `crates/wat-reader/src/parser.rs:25–30` (struct) and `:36+` (enum).
Re-exported through `src/parser.rs:6`.

**Shape:** Pattern A — `ParseError { span: Span, kind: ParseErrorKind }` in the foreign
`wat-reader` crate. `ToEdn` impl is a LOCAL trait impl on a FOREIGN type at `src/parser.rs:15–44`
(orphan rule allows local-trait + foreign-type).

**Variants (10):** `Lex(LexError)`, `UnexpectedRParen`, `UnclosedParen`,
`UnexpectedRBracket`, `UnclosedBracket`, `UnexpectedRBrace`, `UnclosedBrace`,
`MalformedBraceLiteral`, `TrailingContent`, `Empty`.

**Current EDN form:** `src/parser.rs:15–44` — `#wat.kernel/<VariantName> {:span {…} …}`.
`Lex` variant wraps the foreign `LexError` as a `:cause` string (honest: LexError is a leaf).

**Span handling:** Elided when unknown via `push_span_field`. Consistently `:span`.

**Notable:** This error lives in a FOREIGN crate (`wat-reader`). Any record-ification must
decide whether the record definition lives in the substrate crate or the foreign crate (or
the `ToEdn` side-file pattern continues, now with a `defrecord` side-file).

---

### 1.7 `ConfigError` / `ConfigErrorKind`

**Rust definition:** `src/config.rs:130–171`.

**Shape:** Pattern A — `ConfigError { span: Span, kind: ConfigErrorKind }`.

**Variants (8):** `SetterAfterNonSetter`, `DuplicateField`, `RequiredFieldMissing`,
`UnknownSetter`, `BadArity`, `BadType`, `BadValue`, `MalformedSetter`.

**Current EDN form:** Inline `ToEdn` impl at `src/config.rs:242–305` —
`#wat.kernel/<VariantName> {:span {…} <fields>}`.

**Span handling:** Elided when unknown via `push_span_field`. Consistently `:span`.

---

### 1.8 `LoadError` / `LoadErrorKind`

**Rust definition:** `src/load.rs:225–258`.

**Shape:** Pattern A — `LoadError { span: Span, kind: LoadErrorKind }`.

**Variants (7):** `MalformedLoadForm`, `SetterInLoadedFile`, `DuplicateLoad`,
`CycleDetected`, `Fetch(LoadFetchError)`, `Parse { path, err: ParseError }`,
`VerificationFailed { path, err: HashError }`.

**Current EDN form:** `src/load.rs:305–360` — `#wat.kernel/<VariantName> {:span {…} …}`.
`Parse` variant nests a full structured `ParseError::to_edn()` under `:cause`.
`Fetch` and `VerificationFailed` use string `:cause` (leaf foreign errors with only a message;
this is honest, not a deferral — explicitly noted in the impl comment).

**Span handling:** Elided when unknown via `push_span_field`. Consistently `:span`.
`Fetch` variant gets `Span::unknown()` by default (the `From<LoadFetchError>` impl
at `src/load.rs:362–366` installs a blank span).

---

### 1.9 `StdlibError` / `StdlibErrorKind`

**Rust definition:** `src/stdlib.rs:380–393`.

**Shape:** Pattern A — `StdlibError { span: Span, kind: StdlibErrorKind }`.
BUT: span is ALWAYS `Span::unknown()` for stdlib errors — baked-in sources have no
wat-source location. The Display impl at `src/stdlib.rs:406–409` skips the span prefix
entirely (just emits `self.kind`). This is the only Pattern-A family where the span field
is structurally present but semantically always absent.

**Variants (1):** `ParseFailed { path: &'static str, source: String }`.

**Current EDN form:** `src/stdlib.rs:416–439` — `#wat.kernel/ParseFailed {:path "…" :source "…"}`.
No `:span` emitted (unknown span elided by `push_span_field`).

---

### 1.10 `StartupError` — the pipeline union

**Rust definition:** `src/freeze.rs:516–538`.

**Shape:** Bare enum — NOT Pattern A. Each variant wraps one leaf error type:
`Parse(ParseError)`, `Config(ConfigError)`, `Load(LoadError)`, `Macro(MacroError)`,
`Type(TypeError)`, `Resolve(ResolveError)`, `Check(CheckErrors)`, `Runtime(Box<RuntimeError>)`,
`Stdlib(StdlibError)`, `SigmaFn(String)`.

**Current EDN form:** `src/macros/error_edn.rs:151–171` — each arm delegates to the
wrapped error's own `ToEdn` impl. `SigmaFn(String)` becomes
`#wat.kernel/SigmaFnError {:detail "…"}` — the only family where `:detail` is honest
(no span, no kind, no structure to lose).

**Span handling:** N/A — the outer union has no span; each inner error has its own.

**Notable:** `to_edn_values()` at `src/freeze.rs:602–609` explodes `Check` into one
record per `CheckError`, but all other arms produce a single record. This asymmetry
is an existing structural quirk.

---

### 1.11 `*DiedError` family: `ThreadDiedError` and `ProcessDiedError`

**Rust definition:** NOT Rust structs/enums. These are registered WAT TYPES, defined
in `src/types.rs:908–1004` via `register_builtin`. Their runtime representation is
`Value::Enum(EnumValue { type_path, variant_name, fields })`.

**`ThreadDiedError` variants (4):**
- `Panic { message: String, failure: Option<Failure> }`
- `RuntimeError { message: String }`
- `ChannelDisconnected` (unit)
- `Shutdown` (unit)

**`ProcessDiedError` variants (6):**
- `Panic { message: String, failure: Option<Failure> }`
- `RuntimeError { message: String }`
- `ChannelDisconnected` (unit)
- `StartupError { message: String }`
- `EntryFormFailure { message: String }`
- `MainSignature { message: String }`
- `BadReturn { message: String }`

**Current EDN form:** These round-trip via `edn_to_value` TODAY because they ARE in the
type registry. Their serialization uses `value_to_edn_with` (the generic value serializer),
NOT a dedicated `ToEdn` impl. The `RuntimeError`/`StartupError` message fields carry the
EDN text of the underlying error (produced via `to_wire_edn` at the boundary — see
`process_died_error_runtime_value` at `src/runtime.rs:22075`).

**THE KEY DIVERGENCE:** The message field in `ProcessDiedError::RuntimeError` and
`ProcessDiedError::StartupError` is a `String` that happens to contain serialized EDN
(the output of `to_wire_edn`). A consumer reading it must call `edn::read` on the string
to recover the structured error. The field is typed `:String` in the type registry —
the substrate does NOT encode it as a nested tagged value. This is the "message wraps EDN"
seam: `src/runtime.rs` comment at line 904: "The String fields aren't typed-error-objects
on purpose — wat-rs's RuntimeError enum carries its own Display impl; we extract the
formatted message at the substrate boundary."

---

### 1.12 `Failure` — the PRECEDENT

**Rust definition:** Registered in `src/types.rs:1065–1100` as
`TypeDef::Aggregate(AggregateDef { holder: Holder::Record, name: ":wat::kernel::Failure", … })`.
Constructed as `Value::Aggregate(AggregateValue::record("wat::kernel::Failure", …))` in runtime.

**Shape:** A WAT RECORD — registered in the type registry with `Holder::Record` (pure EDN data;
arc 293.W.2b flip from Struct).

**Fields:**
- `message: :wat::core::String`
- `location: :wat::core::Option<wat::kernel::Location>`
- `frames: :wat::core::Vector<wat::kernel::Frame>`
- `actual: :wat::core::Option<wat::core::String>`
- `expected: :wat::core::Option<wat::core::String>`

**Current EDN form:** Serialized via `value_to_edn_with` (the generic value-to-EDN path),
NOT a dedicated `ToEdn` impl. Tag: `#wat.kernel/Failure {:message "…" :location {…} …}`.

**Round-trip:** YES — `reconstruct_record` at `src/edn_shim.rs:2415–2463` can lift a
`#wat.kernel/Failure {…}` EDN back into a `Value::Aggregate(Record)` because `Failure`
IS in the type registry. This is the ONLY error-adjacent type that fully round-trips.

**Companion types:** `Location` (`src/types.rs:1010–1019`) and `Frame`
(`src/types.rs:1027–1054`) are also registered as `Holder::Record` — they round-trip too.

---

### 1.13 `AssertionPayload`

**Rust definition:** `src/assertion.rs:54–86`.

**Shape:** Rust struct (NOT a wat type, NOT registered). Internal transport type —
panic'd by `eval_kernel_assertion_failed`, downcast by `catch_unwind` in the sandbox,
converted to a `Failure` value by `failure_value_from_assertion_payload` in `runtime.rs`.

**Current EDN form:** Serialized as `#wat.kernel/AssertionFailure {…}` by
`payload_to_edn` at `src/panic_hook.rs:145–203` — emitted on the panic hook path to stderr.
This is a DIFFERENT envelope than `#wat.kernel/Failure`; the AssertionFailure envelope
is the raw panic report, not the reconstructable record. It does NOT round-trip.

**Fields in AssertionFailure envelope:** `:thread`, `:message`, `:location`, `:actual`,
`:expected`, `:frames`, `:upstream-chain`.

---

## 2. The Inconsistency Map

Five concrete divergences that make the current landscape hard to troubleshoot:

### 2.A Span field key has seven different names

Across the error families and their EDN serializers, the key under which the primary
source span lands differs:
- `:span` — TypeError, ParseError, ConfigError, LoadError, StdlibError; most CheckError variants; some RuntimeError variants
- `:location` — 16 CheckError variants (all the `BareLegacy*` + `ScopeDeadlock` + `ChannelPairDeadlock` + `DefRestrictedCallerNotAllowed`)
- `:call-span` — `CheckError::SandboxScopeLeak`, `RuntimeError::SandboxScopeLeak`
- `:join-location` — `CheckError::ProcessJoinBeforeOutputDrain`, `ProcessJoinHoldsStdinSender`
- `:body-span` — `RuntimeError::PostconditionFailed`
- `:prior-loc` / `:current-loc` — `CheckError::DefRedefForbidden`, `DefRedefTypeChange`
- ABSENT — freeze-pair variants `UserMainMissing`, `EvalVerificationFailed`

A consumer pattern-matching on span must special-case each variant. A registered error
record would make the span field name a single-source contract.

### 2.B Span elision policy differs between families

`CheckError` serializer (`src/check/error_edn.rs`) elides unknown spans via `push_span`.
`RuntimeError` serializer (`src/runtime_error_edn.rs`) emits `:span` unconditionally via
`span_val`, producing `{:file "<runtime>" :line 0 :col 0}` for unknown-span variants.
`MacroError` follows the unconditional-emit path.
`TypeError`, `ConfigError`, `LoadError`, `ParseError` all elide via `push_span_field`.

A consumer checking `(nil? (:span err))` will produce different results for CheckError
(nil absent) vs RuntimeError (non-nil but sentinel value `{:file "<runtime>" :line 0 :col 0}`).

### 2.C No canonical `:message` field in any error EDN

None of the error types emit their human-readable Display text as a `:message` key.
The message is derivable from the structured fields but requires the same Display logic.
This makes it hard to display an error without pattern-matching the full variant tree.
A base error surface that mandated `:message: String` would make display trivially one-liner.

### 2.D Three structural shapes across the families

- **Pattern A (struct + kind-enum):** CheckError, TypeError, RuntimeError, MacroError, ParseError, ConfigError, LoadError, StdlibError
- **Bare enum (no outer span):** ResolveError, StartupError
- **Registered wat type (Value::Enum/Aggregate):** ThreadDiedError, ProcessDiedError, Failure

A type system claiming "every error is a record" cannot accommodate bare enums without
first making them Pattern A (or changing `ResolveError` to wrap a span it doesn't currently carry).
`ResolveError` in particular has no outer span because its errors are purely a collection —
the design is sound but incompatible with a single-outer-span base surface.

### 2.E `*DiedError` messages carry serialized EDN-in-a-String

`ProcessDiedError::RuntimeError { message }` and `ProcessDiedError::StartupError { message }`
carry the full `to_wire_edn` output of the underlying error as a `:String` field, NOT as
a nested tagged value. To recover the structured error, consumers call `edn::read` on the
string. This is a seam: the type registry says "String" but the actual content is structured
EDN. Closing this would require changing the variant field type from `:String` to a typed
error record — a non-trivial API change to a registered public type.

---

## 3. The Round-Trip Gap

### 3.A What round-trips today

The type registry (`TypeEnv`) governs which tags `edn_to_value` can lift back into
typed `Value`s. The registry-entry path lives in `src/edn_shim.rs:2265–2308`
(`tagged_to_value` → `reconstruct_record` / `reconstruct_struct` / `reconstruct_enum_tagged`).

**Types that round-trip TODAY:**
- `:wat::kernel::Failure` (Holder::Record) — registered at `src/types.rs:1065`
- `:wat::kernel::Location` (Holder::Record) — registered at `src/types.rs:1010`
- `:wat::kernel::Frame` (Holder::Record) — registered at `src/types.rs:1027`
- `:wat::kernel::ThreadDiedError` (Enum) — registered at `src/types.rs:908`
- `:wat::kernel::ProcessDiedError` (Enum) — registered at `src/types.rs:960`

All five survive `value_to_edn_with` → `edn_to_value` round-trip without structural loss.

**Types that DO NOT round-trip:**
Every error type whose Rust struct/enum is NOT in the TypeEnv:
`CheckError`, `TypeError`, `RuntimeError`, `MacroError`, `ResolveError`, `ParseError`,
`ConfigError`, `LoadError`, `StdlibError`, `AssertionPayload`.

When a consumer calls `(:wat::edn::read "#wat.kernel/UnboundSymbol {:name \"x\" :span …}")`,
the `tagged_to_value` dispatch finds no `":wat::kernel::UnboundSymbol"` in the TypeEnv and
returns `Err(EdnReadError { kind: UnknownTag { … } })`. The EDN can be emitted to disk
but cannot be lifted back.

### 3.B What `edn_to_value` does on an error tag today

`tagged_to_value` (`src/edn_shim.rs:2207+`):
1. Attempts capability-tag reconstruction — rejected (not `wat-edn.cap`).
2. Attempts substrate-emitted special tags — none match (these are `wat-edn.holon/*` etc.).
3. Falls through to the body-shape dispatch.
4. Body is `Edn::Map` → calls `reconstruct_struct` or `reconstruct_record` based on TypeEnv lookup.
5. TypeEnv lookup for `":wat::kernel::UnboundSymbol"` → `None`.
6. Falls to `Err(EdnReadError::UnknownTag { ns: "wat.kernel", name: "UnboundSymbol", body_shape: "map" })`.

Result: a runtime panic/error, not a typed value.

### 3.C What it would take to lift an error EDN with a fresh reader

For a `#wat.kernel/SomeError {…}` to round-trip via `reconstruct_record`:
1. `":wat::kernel::SomeError"` must be in the `TypeEnv` as a `TypeDef::Aggregate` with
   `Holder::Record` (or `Holder::HolonRecord`).
2. Every field named in the EDN map must match a declared field in that record definition.
3. Nested types in those fields (span maps, variant enums) must themselves be reconstructable.

Currently none of the error variants satisfy condition 1. Making them registered records
requires either:
(a) Registering them as builtin types in `register_builtin_types`, OR
(b) Having a user-visible `defrecord` form in a stdlib `.wat` file that registers them.

Option (b) is the open-registry path that preserves user extensibility (same mechanism
as user `defrecord`). Option (a) is the closed-registry path (substrate owns the type forever).

---

## 4. User-Extensibility Assessment

### 4.A The type registry IS open

A user `defrecord` call goes through `register_types` → `TypeEnv::register` →
`TypeEnv::get` at reconstruction time. There is no closed-list check at `reconstruct_record`.
The dispatch path in `tagged_to_value` is purely: look up the tag's path in the `TypeEnv`
and dispatch on the result. A user who registers `:myapp::BadInput` as a Record gets
round-trip reconstruction for free, with no substrate changes.

### 4.B Error-specific machinery that is NOT open today

1. **`ToEdn` trait:** Defined in `src/to_edn.rs`, Rust-only. User wat code cannot add a
   `ToEdn` impl for a wat-defined type today — `ToEdn` is a Rust trait, not a wat surface.
   If error records become registered types, their serialization goes through the generic
   `value_to_edn_with` path (same as `Failure` today) — no user-authored `ToEdn` impl needed.
   This is the path that makes the system open: a user `defrecord` error type uses the same
   generic serializer as `Failure`, not a hand-written `ToEdn`.

2. **The compile wall (`to_wire_edn`):** The wall at `src/to_edn.rs:145–147` is generic over
   `ToEdn`. Currently only Rust structs have `ToEdn` impls. If error records are registered
   wat types (Value::Aggregate), they serialize via `value_to_edn_with` — they must bypass
   the `to_wire_edn` boundary or the boundary must accept `Value` directly. The cleanest
   resolution: add `impl ToEdn for Value` (pass-through via `value_to_edn_with`) — then
   ANY `Value` (including user error records) can cross the wire boundary. This makes the
   wall still hold: a raw `String` or a non-`ToEdn` type still cannot cross; a `Value::Aggregate`
   that is a registered error record CAN.

3. **The `#wat.kernel/` namespace:** Currently hard-coded in every serializer's `tagged()`
   helper. A user error record would serialize under the user's own namespace
   (e.g. `#myapp.core/BadInput`). This is correct — user types should not live in
   `wat.kernel`. The tag derives from the record's registered class name (same as how
   `Failure` emits `#wat.kernel/Failure` from `AggregateValue.class = "wat::kernel::Failure"`).

---

## 5. The Worklist

Ordered strikes toward the strongly-tagged error record system.

---

### Strike 5.1 — Audit and fix span-key inconsistency (prerequisite)

Before any record-ification, the span field key across all families must resolve to one name.
Every variant should emit its primary span under `:span` and secondary spans under descriptive
but consistent keys (`:outer-span`, `:secondary-span`, or domain-named keys documented on
the record). The runtime error EDN should adopt the elide-when-unknown policy consistently.

**Scope:** `src/check/error_edn.rs`, `src/runtime_error_edn.rs`, `src/macros/error_edn.rs`.
**Gate:** No field named `:location` in any error EDN (rename all to `:span`).
**Big decision:** Multi-span variants (ScopeDeadlock, ProcessJoinBeforeOutputDrain, etc.)
keep their secondary-span field names (they are domain-correct) but the PRIMARY span
moves to `:span` for all. The base surface can then declare `span: Option<Location>`.

---

### Strike 5.2 — Define the base error surface (intueri names)

The intersection of what EVERY error carries is:

```
mandatory fields (ALL families have these):
  - kind:    String          — discriminant name (the variant name; maps directly to the
                                current EDN tag's trailing segment)
  - message: String          — the human-readable Display text (single-source; derived
                                from structured fields at construction, stored once)

optional fields (absent in some families):
  - span:    Option<Location>  — primary source location; absent for freeze-pair errors,
                                  StdlibError (baked), SigmaFn, and ResolveError's outer type
```

**Families that LACK a primary span (the floor excludes span as mandatory):**
- `ResolveError` outer type — spans are on items
- `StdlibError` — baked stdlib, no wat-source location
- `StartupError::SigmaFn` — flat message only
- `RuntimeError::UserMainMissing`, `EvalVerificationFailed` — freeze pair (no location)

The base surface therefore:
```
base-error surface fields (all mandatory on the base):
  kind:    :wat::core::String
  message: :wat::core::String

optional on base, but many subtypes mandate:
  span:    :wat::core::Option<:wat::kernel::Location>
```

Naming of the base surface record: DEFERRED to intueri.

---

### Strike 5.3 — Decide the error-record model (big design question)

**The core question:** How does the Rust `enum ErrorKind` map to a registered wat record
WITHOUT a drift seam?

**Option A — One record per error (flat, no kind-enum):**
Each variant of e.g. `CheckErrorKind` becomes its own top-level registered record.
`CheckError::ArityMismatch` → `:wat::kernel::CheckError/ArityMismatch` (or
`:wat::check::ArityMismatch`). `StartupError`-as-sum remains as a union type over these.
Pros: no "kind" discriminant on the base; variants are maximally specific.
Cons: 80+ records to register; `StartupError` becomes a wide type union.

**Option B — Pattern-A record with a kind-map payload:**
One registered record per error FAMILY: `:wat::kernel::CheckError { kind: String,
message: String, span: Option<Location>, data: Map<Keyword, Any> }`.
Pros: small number of registered types; easy to add variants.
Cons: `:data` is untyped — loses the structural guarantee the pattern-A Rust enums
currently provide. Round-trip would require knowing the `kind` to interpret `:data`.

**Option C — One record per family, variant encoded as nested tagged value:**
`:wat::kernel::CheckError { kind: #wat.kernel/ArityMismatch {:callee "…" :expected N :got N},
message: String, span: Option<Location> }`. The `:kind` field holds a tagged variant value.
For round-trip, each variant must ALSO be a registered type.
Pros: preserves structural data while keeping one outer record per family.
Cons: two registration levels (family + variant). Existing EDN format changes.

**Recommended path for the builder to evaluate:** Option A is cleanest for EDN-all-the-way-down
(no "kind" discriminant field — the TAG IS the kind). Each variant record is independently
reconstructable. The base surface is expressed as a wat `typesub` hierarchy: every variant
record is declared a subtype of the base error surface. This is the same mechanism that would
work for user-defined error types: user `defrecord :myapp::BadInput` + registers it as a
subtype of the base.

---

### Strike 5.4 — Single-source record definition (no drift seam)

Currently each error type has THREE separate representations:
1. Rust `struct`/`enum` definition
2. `Display` impl (message text)
3. `ToEdn` impl / free function (EDN serializer)

These three must be kept in sync by hand. Adding a field to `CheckErrorKind::ArityMismatch`
requires updating all three sites. This is the drift seam.

The end-state: a SINGLE macro/attribute/derive derives all three from one definition. Options:

**Option D1 — Rust derive macro `#[derive(WatErrorRecord)]`:**
The derive reads field names + types and generates both the EDN serializer and a registration
call (populating the TypeEnv at startup). The Display impl becomes the `:message` field
(still hand-written once, but the derive enforces it exists). This is a Rust-only mechanism.

**Option D2 — wat `defrecord` + manual bridge:**
Register the error type via a `defrecord` in a stdlib `.wat` file. Write a Rust-side
`From<RustError> for Value` that constructs the `Value::Aggregate`. The EDN serialization
goes through `value_to_edn_with` — no hand-written `ToEdn` body.
Pros: fully open to users; same mechanism as user records.
Cons: two definitions (Rust struct + wat defrecord) — still a drift seam unless the
Rust struct is eliminated (errors become pure `Value::Aggregate` types at construction).

**Option D3 — Eliminate the Rust struct, build error values directly:**
Error construction returns `Value::Aggregate(error_record)` directly. No Rust struct.
The error value IS the record. This closes the seam completely.
Cons: Rust's `?` operator and `std::error::Error` integration becomes awkward; IDE support
for error variant matching disappears.

**Recommended:** D1 (Rust derive) for the substrate-owned families (keeps Rust `?` + type safety);
D2 (defrecord + bridge) for the user-visible extension point. The derive generates the TypeEnv
registration call — single source, no drift.

---

### Strike 5.5 — Rust-error→record bridge at the IPC boundary

At the `to_wire_edn` call sites, errors are converted to EDN text. Once error types are
registered records, this path changes:

**Current path:**
```
RuntimeError → runtime_error_to_edn() → OwnedValue → wat_edn::write → String
```

**Target path:**
```
RuntimeError → runtime_error_to_value() → Value::Aggregate → value_to_edn_with → OwnedValue → write → String
```

OR if the derive generates `ToEdn`:
```
RuntimeError → RuntimeError::to_edn() → OwnedValue (same as today, but from registry) → write → String
```

The wall at `to_wire_edn` still holds: the generic `impl ToEdn` requirement means only
registered-EDN-able types cross. If `Value` implements `ToEdn` (passthrough via
`value_to_edn_with`), then `Value::Aggregate(error_record)` automatically crosses.

**Flag:** `ProcessDiedError`'s `message: String` fields currently carry EDN text.
Changing this to a nested typed error record changes the wat-level type (`String` →
`:wat::kernel::RuntimeError`). This is a breaking change to a registered public type —
it would require a migration arc and bumped registry entries. The assessment recommends
NOTING this as deferred: the `message-as-EDN-string` pattern is a known seam, but the
cost of closing it (breaking the registered type) must be weighed against the benefit.

---

### Strike 5.6 — Compile wall: every error satisfies the base surface

Once the base error surface is defined and each family registers a subtype:

**The wall:** Every Rust site that produces an error value must satisfy:
```rust
let err = MyError { … }; // Rust construction
let val = err.to_error_value(); // → Value::Aggregate(base-surface-compatible record)
```

A Rust struct that does NOT implement `to_error_value()` (or does not implement `ToEdn`)
has no path to the IPC boundary. The current `to_wire_edn` compile fence already provides
this for `ToEdn` — it needs to also hold for the record path.

The enforcement mechanism: the compile-fail doctest at `src/to_edn.rs:131–136` proves the
wall is real. A parallel test must prove the same for the error-record path.

---

### Strike 5.7 — Round-trip tooling: `edn::read` lifting error EDN

Once error types are in the TypeEnv, `(:wat::edn::read "#wat.kernel/ArityMismatch {…}")` 
round-trips back to a `Value::Aggregate` representing the error record. But tooling
(CLI `--check-output`, test runner, process-died path) needs to DECLARE this:

1. A probe test asserting `(= err (edn::read (edn::write err)))` for each registered error family.
2. Documentation in the error surface wat file (the defrecord declarations) that the
   tag is derived from the class name (same as `Failure`).

**Flag — `ResolveError` structural anomaly:** `ResolveError` has no outer span and
is a bare enum with one variant wrapping a `Vec<UnresolvedReference>`. For the round-trip
model, this family may need a wrapper record: `UnresolvedReferences { unresolved: Vector<UnresolvedReference>, message: String }` with its own class registration.

---

### Strike 5.8 — User extension: `defrecord` error types satisfy the base surface

Users should be able to:
```wat
(:wat::core::defrecord :myapp::BadInput
  [field :wat::core::String]
  [span  :wat::core::Option<wat::kernel::Location>]
  [message :wat::core::String]
  [kind :wat::core::String])
```
...and have `:myapp::BadInput` automatically satisfy the base error surface (if it
carries the required base fields).

**The mechanism:** A `typesub` declaration:
```wat
(:wat::core::register-subtype :myapp::BadInput :wat::kernel::BaseError)
```
...or, if the base surface is expressed as a protocol/structural constraint (not a
nominal supertype), the user's record satisfies it by structural subtyping.

Today's type system uses nominal subtypes (`register_subtype` at `src/types.rs`) — the
user must explicitly declare the subtype relationship. Structural checking (implicit
satisfaction) would require the type checker to verify the base field set at definition time.

**Decision needed:** Is the base error surface a NOMINAL supertype (explicit `register-subtype`
required, open but verbose) or a STRUCTURAL constraint (implicit satisfaction if fields match,
fully open but requires checker extension)?

---

## Summary: Key Facts the Worklist Rests On

| Family | Pattern | Span? | ToEdn? | In TypeEnv? | Round-trips? |
|---|---|---|---|---|---|
| CheckError | Pattern A | Yes (elide-unknown) | Yes (`src/check/error_edn.rs`) | No | No |
| TypeError | Pattern A | Yes (elide-unknown) | Yes (inline) | No | No |
| RuntimeError | Pattern A | Yes (ALWAYS-emit) | Yes (`src/runtime_error_edn.rs`) | No | No |
| MacroError | Pattern A | Yes (ALWAYS-emit) | Yes (`src/macros/error_edn.rs`) | No | No |
| ResolveError | Bare enum (no outer span) | Per-item only | Yes (inline) | No | No |
| ParseError | Pattern A (foreign crate) | Yes (elide-unknown) | Yes (orphan impl) | No | No |
| ConfigError | Pattern A | Yes (elide-unknown) | Yes (inline) | No | No |
| LoadError | Pattern A | Yes (elide-unknown) | Yes (inline) | No | No |
| StdlibError | Pattern A (span always unknown) | Structurally yes, semantically never | Yes (inline) | No | No |
| StartupError | Bare enum (union) | Delegates | Delegates | No | No |
| ThreadDiedError | wat Enum (registered) | N/A — EDN via value_to_edn | Via value_to_edn | YES | YES |
| ProcessDiedError | wat Enum (registered) | N/A — EDN via value_to_edn | Via value_to_edn | YES | YES |
| Failure | wat Record (Holder::Record) | N/A — field is `location: Option<Location>` | Via value_to_edn | YES | YES |
| AssertionPayload | Rust struct (internal) | Yes (location field) | Via panic_hook (separate path) | No | No |

**The PRECEDENT is Failure:** it is a registered Record, round-trips through `reconstruct_record`,
derives its tag from its class name, and its fields express exactly the base surface floor
(message, location, frames, actual, expected). The path to the strongly-tagged system is:
make every error family follow `Failure`'s pattern, with the tag namespace derived from the
error type's registered class rather than hard-coded in a free function.
