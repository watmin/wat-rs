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
