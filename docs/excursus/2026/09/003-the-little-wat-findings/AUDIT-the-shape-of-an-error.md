# AUDIT — the shape of an error: which fields earn their place

Excursus 003. Written 2026-09-27 at `9283c4560`, after envelope steps 1–4.

The builder asked: *"wat's error backtraces has been growing like an organic thing … which fields are
half baked and possibly handled by another form / field?"*

The envelope work (D1–D4) made errors structured, and it never asked whether each piece of that
structure is right. This audit asks it, one field at a time. For each field:
- **who writes it**;
- **who reads it**: accessor calls and match keys in tracked `.wat`, and by-name reads in Rust;
- **what values it actually takes**, measured over the tracked `.edn` goldens.

**Contamination, stated up front.** The counts come from text greps over the tree. A grep that finds
nothing is evidence about the query, not the behaviour. Where a zero carries a verdict, the row says
so. "wat readers" counts accessor calls and match keys. It cannot see generic walkers (EDN printed
and inspected by a human or a tool), and that is the main consumer of a backtrace. A backtrace field
with no wat reader is therefore not evidence against it. A **floor** field with no reader, or a field
whose value never varies, is.

## The inventory

The **one declared error floor** is `:wat::core::Error` (`wat/core.wat:2169`), a surface with
`message`, `location <- Span` and `causes <- Vector<Error>`. Around it, and not all of it conforming:

| record | fields | writer | observed |
|---|---|---|---|
| `:wat::core::Fault` | message location causes | `Fault/of`, Rust `fault_value` | the minimal Error |
| `:wat::runtime::<40 kinds>` | floor + kind fields | `RuntimeError::to_record` (3a) | |
| `:wat::kernel::Failure` | error frames actual expected frames-elided | death constructors (3b) | |
| `:wat::kernel::Frame` | symbol span kind | `RuntimeError::new` (2), D4 | |
| `:wat::kernel::LociDiedError` | 6 variants carrying `Failure`; 2 bare | death paths | |
| `:wat::kernel::AssertionFailure` | thread message location actual expected frames upstream-chain | `src/panic_hook.rs` (**hand-built**) | an **unhandled assertion's stderr** |
| `:wat::core::EvalError` | kind (**String**), message (String) | `runtime_error_to_eval_error_value` | eval-family `Err` |
| `:wat::kernel::StartupError` | message | **nothing** | 0 goldens, 0 wat readers |
| `:wat::runtime::ValueSnapshot` | type-name rendered provenance | (3a) | |
| `:wat::{cache,query,sqlite}::Fault`, `:wat::doctest::Failure` | domain fields, **no** location/causes | their modules | domain values, not the floor |

## Findings, most severe first

### F1 — Two death shapes on stderr, and one of them contradicts its own declaration
- A process that dies from a **runtime error** writes `[#wat.kernel/LociDiedError.RuntimeError {:failure …}]`.
- One that dies from an **unhandled assertion** writes `[#wat.kernel/AssertionFailure {…}]`
  (e.g. `tests/diagnostics/probe_ex003_lru_new_refuses_as_a_value__lru_new_zero_unhandled_stderr.edn`).
- They are different top-level records for the same event, a peer died, so a consumer needs two
  parsers.

Worse, `AssertionFailure`'s writer is hand-built (`payload_to_edn`, `src/panic_hook.rs:121`) and emits
`:actual nil :expected nil :upstream-chain nil` and a bare `Span` for `location`. The declaration
says `(Option :- [String])`, `(Vector :- [LociDiedError])` and `(Option :- [Span])`. Everywhere else
an absent Option is the tagged `#wat.core/Option.None {}`.

**Verdict:** retire `AssertionFailure`. An unhandled assertion is a `LociDiedError.Panic` whose
`Failure` carries the assertion. `thread` becomes a field of that envelope, if it earns one (see F6).

### F2 — `Failure.actual` / `Failure.expected` duplicate the assertion's own record
- Measured: `Some` in **4** goldens, `None` in **92**.
- **4** read sites in wat for `actual`, **3** for `expected`.
- They mean something only for an assertion.
- The 3a record `:wat::runtime::AssertionFailed` already carries `actual`/`expected` as its own
  fields.
- On every other failure they are noise, e.g. C-114's overflow carries both as `None`.

**Verdict:** move both into the assertion's error record, the panic path's too, and drop them from
`Failure`. `Failure` becomes `{error frames frames-elided}`.

### F3 — `causes` is written everywhere and read almost nowhere, and 3c gave it a second meaning
- Readers: **0** in wat (no `Error/causes` or `Fault/causes` call, no `:causes` match key). **2** in
  Rust, both navigating the aggregates (`src/rete/validate/mod.rs:1527` and one test).
- Of 438 golden occurrences, **285** are the literal `[]`. The rest are the checker aggregates, one
  REPL `Fault`, and nothing else.
- 38 of the 40 runtime kinds always write `[]`.
- **Two meanings under one name.**
  - The two wrapping kinds (`EvalVerificationFailed`, `MacroExpansionFailed`) use it for *causation*:
    this error happened because of that one.
  - Step 3c moved the checker aggregates' items into it. An aggregate's items are its *members*:
    three type errors are not the cause of "3 type-check errors". That was my ruling in 3c, and this
    audit finds it wrong.
- **Three causal mechanisms coexist.**
  - `causes`: error caused by error.
  - The `LociDiedError` chain vector: peer died because a peer died.
  - `AssertionFailure.upstream-chain`: **0** of 2 goldens non-nil.
  - Measured with a weak instrument: **0** goldens show a chain longer than one, and a grep found no
    wat code reading past the chain's head. The cascade tests (`wat_arc113_cross_fork_cascade`) may
    exercise longer chains through a path the grep cannot see. This needs a behavioural check before
    anything is removed.

**Verdict, as a question for the builder:** does wat want causation at all? If yes, it wants **one**
mechanism, not three, and aggregates should get their own `errors` field back rather than borrow
`causes`. If no, `causes` leaves the floor. The two wrapping kinds then carry their inner error as a
named field, and the floor becomes `{message location}`.

### F4 — `EvalError` is a stringly parallel error type with a large reader base
- `{kind <- String, message <- String}`. `kind` is one of 7 kebab-case strings plus a
  `"runtime-error"` catch-all (`runtime_error_to_eval_error_value`, `src/runtime.rs`).
- It predates the 40 runtime records and now **duplicates them lossily**: every structured field
  (op, operands, location, frames) is flattened into `message`.
- Readers: **204** `EvalError/kind`, **215** `EvalError/message`; **453** constructions and mentions in
  the wat corpus. So this is the most-read error type in the language, and the least structured.

**Verdict:** the eval family's `Err` should carry the `:wat::core::Error` the evaluation raised: the
runtime record, or a check/parse error record. `kind` is then its class, never a string. This is a
large codemod, and it is also where users meet errors most.

### F5 — `ValueSnapshot.provenance` is almost always "unknown", and has two encodings of it
- `ValueSnapshot::of` (provenance always `Unknown`): **503** call sites.
- `of_tracked`: **3**.
- 22 direct constructions of `Literal`/`SymbolBound`/`RuntimeBuilt`.
- In goldens: `:provenance nil` **22** (the old derive writer), `Option.None` **19** (the 3a record),
  and a real value in **5**.
- Arc 233 retired `Value::Tracked`, which fed it.

**Verdict:** half-baked by attrition. Either re-fund it (a real tracking source) or drop the field;
the known cases fold into the error's own `location`. Dropping is the honest default: a field that is
`None` for 503 of 506 sites is not information.

### F6 — `Frame`: the wrong pairing, a derivable `kind`, and a fabricated entry
Readers: **0** wat readers of `Frame/kind`, `Failure/frames` or `Failure/frames-elided`; **1** of
`Frame/symbol`; **28** of `Frame/span`. Frames are read by people and tools, which is right for a
backtrace.
- **Pairing.** Frames pair (callee, call site). Clojure/Java pair (function, where inside it). Read
  conventionally, C-114's trace says `:user::main` called `+` at line 3, which is inside
  `:user::grow`.
- **Fabricated frame.** D4's raise frame `{:symbol ":wat::i64::+" …}` names an intrinsic that never
  had a frame.
- **Non-wat span on a wat frame.** `:user::main`'s span is `src/freeze.rs`, the Rust site that calls
  main.
- **Rust frame out of place.** The Rust frame is listed outermost (D3's "user first"), but it is the
  innermost activation.
- **`kind` is derivable** from `span.end` (D1: wat spans have an end, Rust spans don't), and it
  already disagrees with the span on the `:user::main` frame.
- **`<rust>`** is a placeholder where a function name belongs.
- **Frame elision.** `frames-elided` was never non-zero in any golden. The cap is real and untested
  at scale; that is not a reason to drop it.

**Verdict:** reshape `Frame` to `{fn at}`, where `at` is the location inside `fn`:
- the innermost frame's `at` is the raise site, so D4's fabricated frame goes;
- the Rust frame takes its true innermost position;
- `kind` goes.

The tail-call question changes cost under this shape. Rule on it together with this.

### F7 — Dead or near-dead declarations
- `:wat::kernel::StartupError` (`{message}`): registered (`src/types.rs:3071`), constructed nowhere,
  read nowhere, in 0 goldens. Dead since 3b gave `LociDiedError.StartupError` a `Failure`.
- `LociDiedError.EntryFormFailure` and `.BadReturn`: no producer (measured in 3b).

**Verdict:** retire all three.

### F8 — Four domain `Fault`s that are not the `Fault`
- `:wat::{cache,query,sqlite}::Fault` and `:wat::doctest::Failure` share names with the floor's
  `Fault`/`Failure`.
- They carry no `location` or `causes`, so they do not satisfy `:wat::core::Error` and cannot be a
  cause, a `Failure.error`, or anything the envelope carries.

**Verdict:** either they conform to the floor (they are errors) or they are renamed (they are domain
outcomes). Their shared names currently imply a conformance they don't have.

## What survives untouched
- `message`: 204 floor reads, 685 `Failure/message`, 443 `LociDiedError/message`. It is the human
  headline and heavily used.
- `location`: a `Span`, mandatory since 3c, and derived to the user's line since D4. The one
  location shape (D1) holds.
- Every runtime record's kind fields (`op`, `a`, `b`, …): the reason the envelope work was worth
  doing.
- The `LociDiedError` variant: it says *how* the locus died, which the error's tag cannot.

## A target shape, for discussion (not ruled)

```
[#wat.kernel/LociDiedError.RuntimeError           ; how the locus died
 {:failure #wat.kernel/Failure
   {:error  #wat.runtime/IntegerOverflow          ; what went wrong, structured
             {:message "…" :location <Span: the user's line> :op … :a … :b …}
    :frames [{:fn "<rust fn>"     :at <Span arith.rs:93>}   ; innermost first, true order
             {:fn ":wat::core::+" :at <Span wat/core.wat:66>}
             {:fn ":user::main"   :at <Span …>}]
    :frames-elided 0}}]
```

Gone:
- `actual` and `expected` (F2);
- `causes` unless it is ruled in (F3);
- `Frame.kind`, the fabricated frame, and the Rust-site main span (F6);
- `provenance` (F5);
- `AssertionFailure` (F1).

## Questions for the builder

1. **Causation (F3).** Keep `causes` as the one causal mechanism, retiring the other two? Or remove
   it from the floor?
2. **Frame reshape (F6).** Adopt `{fn at}`, with the Rust frame innermost? This reverses D3's "user
   first" ordering, which was ruled as tweakable.
3. **EvalError (F4).** Should the eval family carry the real error? This is the largest change here,
   and the one users meet most.
4. **Provenance (F5).** Drop the field, or re-fund it?
5. **Retire** `AssertionFailure` (F1), `kernel::StartupError`, `EntryFormFailure` and `BadReturn`
   (F7), and resolve the domain `Fault`s (F8)?

## RULING 2026-09-27 — all five, as recommended

Builder: *"let's do it — we'll add stuff back in later if we choose to."* The bias is **removal**: a
field is re-added when a real consumer asks for it, not kept in case one does.

1. `causes` leaves the floor → `:wat::core::Error` is `{message location}`. Aggregates get their own
   `errors` field back (reverting 3c's move); the two wrapping kinds carry a named `cause`; the
   peer-death chain stays its own thing; `upstream-chain` goes with `AssertionFailure`.
2. `Frame` becomes `{fn at}` — `at` is the location INSIDE `fn`; innermost first in true order, the
   Rust activation innermost; `kind` and D4's fabricated frame go.
3. The eval family's `Err` carries the real `:wat::core::Error`; `EvalError`'s string `kind` goes.
4. `provenance` goes.
5. `AssertionFailure`, `kernel::StartupError`, `EntryFormFailure`, `BadReturn` retire; the domain
   `Fault`s are resolved (conform or rename — measured in their own strike).

## Delivery — in order, each on a green floor

| strike | scope | why this position |
|---|---|---|
| **A** | one death shape: `AssertionFailure` → `LociDiedError.Panic`; `Failure` → `{error frames frames-elided}` (actual/expected move into the assertion's error record, kept readable by DERIVED accessors); retire `kernel::StartupError`, `EntryFormFailure`, `BadReturn` | smallest; fixes the worst finding (F1) |
| **B** | the floor → `{message location}` | touches every `WatError` impl and every golden once |
| **C** | `provenance` out of `ValueSnapshot` | independent, small |
| **D** | `Frame` → `{fn at}` | needs the tail-call ruling alongside |
| **E** | `EvalError` carries the real error | largest; its own design note first |
| **F** | the domain `Fault`s | needs a naming measurement |

## RULING 2026-09-27 (2) — the hard work now: declare every startup-error taxonomy before B

Strike B found `causes`' one real job: carrying a FOREIGN checker diagnostic (undeclared
`#wat.check/…`, `#wat.resolve/…` tags) under a `Fault` with a fabricated `<runtime>:0:0` location, at
four decode sites. Offered an interim wrapper, the builder: *"should we just do the hard work now?..
deferral usually backfires"*. So the SWEEP (`BRIEF-shape-sweep-every-startup-error-is-a-declared-record.md`)
runs first — ~130 kinds, three strikes S1–S3, pure declaration (zero golden changes) — and B follows
with no wrapper left anywhere. Delivery becomes **A ✓ → S1 → S2 → S3 → B → C → D → E → F**.

## Strike B worklist — wire defects the sweep found (pure declaration may not fix them)

Each is recorded where found; B changes the wire, so B takes them.

| from | defect | the cure B owes |
|---|---|---|
| S1 | `NoMatchingClauseAtCallSite.attempted-clauses` — each attempt an UNTAGGED `{:arity :param-types}` map (`src/check.rs:442`, `clause_attempts_to_edn`); the one check kind left undeclared | tag it (a declared attempt record), then declare the kind |
| S1 | sum-typed sub-values ride FLAT per-variant tags (`#wat.kernel/NotFnForm`, not `EnsureFnInvalidReason.NotFnForm`), so wat cannot declare them as one `defenum`; S1 declared five unrelated records joined by `reason <- :wat::core::Value`. Same shape as 3a's `ClauseFailureReason` gap | a sum type's variants carry DOTTED tags on the wire; the field then types as the enum |
| S1 | `EnsureFnInvalidReason` tags `wat.kernel` by the derive's default namespace, though it is a check diagnostic | name its namespace deliberately |
| S2 | `LoadFetchError` (3 variants, `src/load/loader.rs:192`) and `HashError` (8 variants, `src/hash.rs:471`, shared with `RuntimeErrorKind::EvalVerificationFailed`) ride FLAT `wat.kernel` per-variant tags; declared as 11 flat records under `:wat::core::Value`-typed fields | dotted enum tags on the wire; fields type as the enum |
| S2 | cost, not wire: each declared stdlib record adds ~0.8ms per world freeze (hello-world 615–676ms → 682–717ms, 6 samples each); floor 1103s → 1205s (+9%) across S1+S2; two near-margin tests widened (`d0f126e18`, `42cca492a`) | measure where freeze time goes BEFORE S3 |
| S3 | `ParseErrorKind::Lex.cause` — a lex failure rides inside a parse error as OPAQUE PROSE (`LexError`'s `Display`), no tag at all; the offending character, byte position and `LexErrorKind` variant are discarded at that one site though Rust still holds them. No `:wat::lex::*` declared — nothing produces a lex tag (gate `g_lex_never_produces_a_tag`) | give `LexError` a tagged wire form (or route it through `ParseError`'s floor); then declare `:wat::lex::*` |
| S3 | cost, continued: a third near-margin test crossed with S3 — `nested_program_literals_start_on_the_child_path` (123 freezes) 157–160s before S1 → killed at 180.03s; widened `645d3ebae`. Floor 1103s → 1266s (+15%) across S1–S3 | the stdlib-freeze excursus after B |

## RULING 2026-09-28 — finish the shape work first; the stdlib-freeze cost is its own excursus after

`MEASURE-where-a-freeze-spends-its-time.md` (`63bfc0054`): every freeze re-expands (~198ms) and
re-checks (~217ms) the whole fixed stdlib, 5834 freezes per floor, nothing cached. Builder chose to
land S3 now (~+45ms, known and measured; any test crossing its limit is surfaced with its history,
then widened under the timing ruling), finish strike B while the shape work is hot, and THEN open a
separate excursus — the stdlib frozen once — with the MEASURE doc as its baseline.

## Strike B1 landed (`7b0b3acb1`, `6f8f1ca8f`) — and what it found

The floor is `{message location}`; 0 `:causes` in goldens, stdlib records, or Rust; aggregates hold
`errors`; the four wrapper sites carry the typed diagnostic; `NoMatchingClauseAtCallSite` is tagged
(`:wat::check::AttemptedClause`) and declared. Floor 6339/6339.

| from | finding | the cure owed |
|---|---|---|
| B1 | **strict decode does not check a field's VALUE against its declared type.** An untagged `Map` decodes as a generic `HashMap` wherever it sits (`edn_to_value_caps`), so the sweep's G-strict gates (S1–S3) prove every TAG is registered — not that every field's shape matches its declaration. B1's own GB4 had to assert on the writer directly for this reason | typed decode checks each field against its declared type (a record-typed field refuses an untagged map); then re-run the sweep's gates as real shape proofs |
| B1 | `HashError`'s S2 records carry no floor, so they cannot satisfy `:wat::core::Error`; `EvalVerificationFailed.cause` holds an interim `Fault` | B2: give `HashError` the floor (it is an error) — then `cause` holds it |
| B1 | `MacroExpansionFailed.cause` holds an interim `Fault`: a typed `cause` needs a `TypeEnv` that `RuntimeError::to_record` does not have on the peer-death paths | thread the type registry to `to_record`, or build the cause from the already-typed `MacroError` record |
| B1 | `read-json` / `read-foreign` failures are `.to_string()`'d upstream (`eval_edn_read_json`/`_foreign`) before reaching `tagged_read_outcome_malformed`, whose tag (`JsonReadError`/`ForeignReadError`) is never a declared type — structure lost at the source | declare JSON/foreign read errors and carry them structured |
| B1 | `fault_value(message, None)` still synthesizes a `<runtime>` file at line 0 (`src/runtime.rs:~11963`; 1 caller passes `None`) — the fabricated-location class B1 removed from the wrapper sites | the caller supplies a real span (the Rust raise site via `rust_caller_span!` at worst) and the `Option` goes |
