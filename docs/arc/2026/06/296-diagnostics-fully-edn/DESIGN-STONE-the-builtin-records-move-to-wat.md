# 296 · DESIGN STONE — the builtin records move to wat, and Rust consumes them

> Builder's cut, 2026-08-15: *"i say we define as much as we can in wat files..... then we use
> the rust macros to consume those wat files to do whatever necessary work.... there's no drift
> because the source of truth is a single wat expression."*
>
> And: *"find the heretics and correct them.... define it all in wat.. then use those wat
> defined values where they must be used... there is no forgetting..."*

## WHY — this is `b2136b02` for records

`b2136b02` made **wat the source of truth for six Rust enums** — `Kind` `DefinedIn` `Layer`
`Category` `Purity` `Determinism` — via `wat_enum_from!`. No Rust enum mirrors a `defenum`
by hand any more.

**The records never got the same treatment.** `src/types.rs` hand-writes **16 aggregate type
declarations** (`register_builtin(TypeDef::Aggregate(AggregateDef { … }))`) — wat's own record
types, declared in Rust, in a language that is not wat. That is exactly the pre-`b2136b02`
state, one shape over.

It surfaced through arc 296's `field-N` hunt: 16 Rust construction sites were about to
hand-transcribe field names that a `defrecord` already states, and the builder stopped it —
*"we did that exact move recently?"*. The transcription was the symptom. **Rust holding the
declaration at all is the cause.**

## THE TRAP-CHECK — RUN, WITH A WORKING NEGATIVE CONTROL

The entire risk was one question: a type declared in wat AND registered by Rust — does the
loader no-op or collide?

`TypeEnv::register` (`src/types.rs:594`) classifies an existing entry three ways, and the
middle arm exists for precisely this, per its own comment: *"a byte-equivalent re-declaration
is a no-op — Arc 054, e.g. an in-crate shim delivered both via `wat_sources()` and on-disk."*

```rust
Some(e) if e == &def => Existing::Equivalent,   // → Registration::NoOp
```

**Measured, both directions** (`wat-scripts/scratch-pad/probe-arc296-builtin-redeclared-in-wat.wat`):

| | form | result |
|---|---|---|
| POSITIVE | `defrecord :wat::kernel::Location [file line col]`, transcribed to match the Rust literal exactly | **clean** — `Equivalent` → NoOp |
| NEGATIVE | same name, `column` instead of `col` | **`#wat.type/DuplicateType`** |

The negative control is what makes the positive mean anything: without it, a clean run could
have meant the check never looked. It looked.

**So the migration is mechanical, and the collision I feared does not exist.** Arc 054 built
the path years before we needed it.

## THE INVENTORY — 16, and none are excluded

`register_builtin(TypeDef::Aggregate(…))`, in registration order:

| | type | nature | fields |
|---|---|---|---|
| 1 | `:wat::core::Struct` | Struct | 0 |
| 2 | `:wat::holon::CapacityExceeded` | Struct | 2 |
| 3 | `:wat::core::EvalError` | Struct | 2 |
| 4 | `:wat::kernel::Location` | Record | 3 |
| 5 | `:wat::kernel::Frame` | Record | 3 |
| 6 | `:wat::core::Span` | Record | 4 |
| 7 | `:wat::kernel::Failure` | Record | 4 |
| 8 | `:wat::kernel::AssertionFailure` | Record | 6 |
| 9 | `:wat::kernel::StopAccepted` | Record | 1 |
| 10 | `:wat::kernel::StopFailure` | Record | 2 |
| 11 | `:wat::kernel::StopFailed` | Record | 1 |
| 12 | `:wat::kernel::StartupError` | Struct | 1 |
| 13 | `:wat::holon::CoincidentExplanation` | Struct | 5 |
| 14 | `:wat::holon::Match` | Record | 2 |
| 15 | `:wat::core::Record` | Struct | 0 |
| 16 | `:wat::holon::Record` | Struct | 0 |

None has `type_params`; none has `restrictions: Some(…)`. Every one is a plain monomorphic
aggregate — the easiest possible shape to express in wat.

⚠ **CORRECTION, builder 2026-08-15: *"records are allowed to have no fields... that's legal."***
An earlier draft of this stone was about to exempt rows 1/15/16 — the nature ROOTS — on the
grounds that a zero-field record could not be written in wat. **That is false.** A zero-field
`defrecord` is legal, so there is no *legality* barrier to any of the sixteen. Whether the three
nature roots SHOULD live in wat is a question about what they ARE (subtype roots, not data), and
it is the builder's to rule — but it is not the question I nearly filed it as, and an exemption
argued from a false premise would have stranded three types in Rust forever with a reason that
does not hold.

**The count is 16, not the 23 reported earlier.** 23 is every `AggregateDef {` literal in
`types.rs`; 7 of those are in other contexts (synthesis paths, tests) and are not builtin
registrations. The earlier number was a grep, not a census — the same error this arc keeps
paying for.

## THE SHAPE

1. Each of the 16 gets a `defrecord` / `defstruct` in the corpus, in the file its namespace
   names (`wat/core.wat`, `wat/kernel.wat`, `wat/holon.wat`).
2. `wat-source-derive` gains **`wat_record_from!`** — the record sibling of `wat_enum_from!`,
   reading the same `.wat` file at build time via `wat-reader`, emitting the
   `register_builtin(TypeDef::Aggregate(…))` row.
3. The hand-written literal in `types.rs` is **deleted**, replaced by the macro invocation.
4. Equality with the corpus-parsed def then holds **by construction** — generator and loader
   read one expression — so the `Equivalent` arm is guaranteed, not hoped for.

## THE CONTRACT — one decision, pinned

**`wat_record_from!` emits the REGISTRATION, not a Rust struct.** It is not the mirror of
`wat_enum_from!` (which emits a Rust `enum` because Rust code matches on those variants).
Nothing in Rust needs a `struct Failure`; what Rust needs is for the TypeEnv to contain the
declaration before the corpus loads. So the generated artifact is the registration row and, for
arc 296's construction sites, the field-name constant.

## WHAT THIS UNBLOCKS

With the 16 declared in wat, **no Rust site needs to state a field name.** That deletes the
`static_field_names!` macro from the reverted G patch entirely, and G becomes plumbing over a
single source rather than a second transcription of it.

## OUT OF SCOPE — affirmatively cut, not deferred

- **The 7 non-builtin `AggregateDef` literals** (synthesis paths, tests). They construct defs
  from already-parsed input rather than declaring types; they are not a second source of truth.
- **`:wat::spawn::Bound<S,R>`** and the other corpus-declared parametric types. Already in wat;
  nothing to move.
- **G itself** (`AggregateValue.names`). Lands after this stone, on the single source this one
  creates.

## STOP TRIGGERS

- **STOP-1 — a type whose wat form will not compare `==`.** Transcribe, `--check`, and if it
  raises `DuplicateType`, STOP and report the divergence. Do NOT edit the wat form until it
  matches: **the divergence is the finding.** A hand-written builtin that disagrees with what
  wat's own reader produces for the same declaration is a defect that exists today, and it is
  worth more than the migration.
- **STOP-2 — a type needed before its corpus file loads.** The generated registration still runs
  before any wat parses (it is Rust by then), so this should not arise. If a load-order failure
  appears anyway, STOP: the premise that build-time generation is order-free is wrong and the
  whole stone needs re-drawing.
- **STOP-3 — `restrictions` or `type_params` needed by any of the 16.** Measured as none today.
  If one appears, STOP rather than inventing a wat spelling for it.
