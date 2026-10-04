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

## RULING 2026-09-30 — the decoder hole first, then B2

Builder: *"let's continue in the order you've expressed"*. Delivery: **T** (typed decode checks each
field value against its declared type — the hole B1 found) → **B2** (dotted enum tags for sum types,
a tagged lex error, `HashError` gains the floor, `fault_value`'s `<runtime>` synthesis goes) → C → D
→ E → F → the stdlib-freeze excursus.

## Strike T + T2 landed (`f24e16037`) — and the hole T2 reopened

T closed the decoder hole (every field checked, undeclared keys refused, in all four reconstructors
plus `coerce_struct_path`); T2 made `recv'`'s decode failures structured (`:wat::edn::*` read-error
records). But T2 restored arc 278's `:RequestMalformed` by a LENIENT RE-DECODE: on a shape refusal,
`decode_client_message_event` discards the strict error and decodes again the pre-T way so
`:wat::edn::validate` has a value to check. Driven by the orchestrator (a temporary probe, not kept):

```
wire     #t.edn2.Bag/Op.Put {:req #t.edn2.Bag/PutRequest {:items ["a"] :stray 1}}
strict   Err(UnknownField { type_path: ":t::edn2::Bag::PutRequest", key: "stray" })
event    ServiceEvent::Message                       ← the lenient retry
validate sees  #t.edn2.Bag/PutRequest {:items ["a"]} ← the stray key already dropped
validate Ok (ACCEPTED)
```

An undeclared key is ACCEPTED on the process-tier service wire — strike T's hole reopened at the one
place a real attacker reaches. The lenient door is also a second answer to one question.

## RULING 2026-10-01 — a wrongly-shaped request frame replies `:RequestMalformed`

Builder chose to keep arc 278's contract: no lenient re-decode; the serve loop identifies the op from
the frame's outer tag and replies that op's `:RequestMalformed`, built from the STRUCTURED refusal.
Strict decode becomes the wall's first line. Strike T3.

## Strike B2 landed (`6da5efd3b`, `929f0c0cf`, `fcd4741d8`, `62de4f19a`, `2e827fed7`)

The derive learned `#[to_edn(namespace = …, qualified)]` (`#<ns>/<Enum>.<Variant>`). Every flat-tagged
sum type the sweep found is one dotted `defenum`: `ClauseFailureReason`, `HashErrorKind`,
`LoadFetchError`, `EnsureFnInvalidReason` (moved to `wat.check`), `LexErrorKind`. `HashError` is an
Error (`{message location kind}`) — `EvalVerificationFailed.cause` holds it, closing B1's interim
`Fault`. Lex failures are structure (`#wat.lex/LexError {:position :kind}`), not prose. **Zero
holder fields remain typed `:wat::core::Value`.** Floors green at each commit (6364 → 6369). The lex
gate's mutation (drop `qualified`) was run by the orchestrator: RED, restored.

| from | finding | the cure owed |
|---|---|---|
| B2 | `:wat::core::char` is a runtime `Value` but deliberately NOT a `TypeEnv` member (`src/types.rs`, `TABLE-STONE-Q`), so no record can declare a `char` field; the lex kinds carry their offending character as a one-character `String` | builder's call: register `char` as a declarable type, or keep the hole and its `String` stand-in |
| B2 | `LoadFetchError`'s writer stays hand-written: its `Other` variant renames its tag to `LoadOther`, and the derive has no variant-level tag rename | a `#[to_edn(tag = …)]` variant directive, or rename the Rust variant to match its tag |

## Strike B3 landed (`8e2710ea9`, `9828cefbc`, `5ddbb44e9`, `4feb8982f`, `1808af90e`) — strike B is done

`MacroExpansionFailed.cause` holds the real `MacroError` (decoded against the builtins registry;
`single_cause_fault` retired). `read-json`/`read-foreign` failures carry `:wat::edn::*` records, not
prose. `fault_value`'s location is a mandatory `Span` (compile error proven); 0 goldens carry
`<runtime>`. Goldens no longer hold stale `.rs` line numbers (2 fixed; a capture-normalization gate
added) — the orchestrator's brief misdiagnosed the mechanism (capture already normalized since
`8c1392ca4`); the executor found and reported the contradiction. The serve loop's unreachable-op
fallback replies `Reply::Failed`, not a panic. Floor 6372/6372.

| from | finding | the cure owed |
|---|---|---|
| B3 | ~24 `assertion-failed!` sites in `wat/service.wat` (and `wat/spawn.wat`'s shared `launch`) are owner↔child self-peer protocol invariants — measured NOT client-reachable (the admin channel is the owner's private fd 0/1 or in-process channel; only generated `start`/`resume` ever send on it, always `Init`/`Resume`). `dispatch-admin` runs once, before `serve`, with no error channel (`-> State`) | builder's call: accept owner-only protocol violations as panics (the repo-wide `launch` precedent), or a follow-on that gives `dispatch-admin`/`Locus::launch` a failure value |

## Strike C landed (`38ccf65f7`)

`ValueSnapshot` is `{type rendered}`; `:wat::runtime::Provenance` retired (lost its only holder);
`of_tracked` collapsed into `of`. 44 goldens recaptured; GC1 lint (anchor: 44 files, 46 occurrences
= F5's 22+19+5) mutation-proven. Floor 6373/6373.

| from | finding | the cure owed |
|---|---|---|
| C | **a real cost of the removal, measured:** in the 5 goldens where provenance was KNOWN, the error's `:message` lost its suffix — e.g. `got wat::core::keyword \`:ns::nonexistent-verb\` (built by :wat::keyword::from-string at …p2.wat:…)` is now just `got … \`:ns::nonexistent-verb\``. That suffix answered "where did this bad value come from?", which `location` (where it was USED) does not | builder's call, under the removal bias: leave it gone, or re-add provenance only where it is known (a field present iff known — not an `Option` that is `None` 503 times) |
| C | **the `Provenance`/`TrackedValue` machinery is now dead end to end**: every producer (literal eval, `keyword`/`ast`/`edn`/`holon` intrinsics, the `wat_intrinsic` shim) writes it; the one reader (`Environment::lookup`, `src/value/environment.rs:~203`) re-wraps it into a value nobody reads; `provenance_to_edn` has no production caller. Kept untouched, per scope | builder's call: retire the machinery, or re-fund it (see the row above) |

## RULING 2026-10-03 — four decisions, each by the four questions (all YES)

Builder: *"you've got your 4 YES paths"* (after rejecting a menu: *"we don't use the menus here"*).

1. **Tail calls (strike D).** When a tail call replaces a frame, the surviving frame records the
   REPLACED callee (the true owner of its call site) and a count of collapsed tail calls — O(1), a deep
   tail-recursive loop stays constant-space. C-114 then reads `{fn :user::grow at c114.wat:3 tail}`.
   Rejected: silence (Obvious/Honest NO — it pins line 3 on `main`); a ring buffer (Simple NO — hot-path
   bookkeeping, an N chosen by symmetry).
2. **Provenance machinery: retire it.** Re-funding is Simple NO (18 files, `TrackedValue` threaded
   through the environment, for 3 of 506 known cases). Strike after F.
3. **Owner-only admin protocol errors: make them unrepresentable.** The service's FIRST admin message
   gets its own type admitting only `Init`/`Resume`; Stop-before-Init has no form. The panics stand
   until that lands. Strike after F.
4. **`char` is declarable; `LoadFetchError`'s variant is renamed.** Register `:wat::core::char` as a
   leaf (the Stone-Q census pins the hole precisely so this is a deliberate edit); lex kinds carry a
   `char` again. Rename the Rust variant `Other` → `LoadOther` so the derive covers it and the
   hand-written writer retires. Strike after F.

## Strike D, first boundary (`0df5038e2`) — and the four-questions answer to its STOP

`FrameInfo` carries the tail record (`entry_call_site`, `last_tail_caller`, `tail_hops`); the pure
`reconstruct_frames` → `{fn at tail-elided}` is proven at the Rust level (GD1–GD4, each mutated RED;
a 10⁶-hop tail loop stays one physical slot). Floor 6379/6379. The executor STOPPED at item 4: 10 of
40 `RuntimeErrorKind`s carry nothing that names the Rust activation that raised them (`DivisionByZero`,
`NotCallable`, `BadCondition`, `PatternMatchFailed`, `AssertionFailed`, `MacroAbort`,
`UserMainMissing`, `EvalVerificationFailed`, `WriteStopped`, three `ReteCeiling` variants).

**Answer (orchestrator, by the four questions):** a frame's identity is a property of the STACK, not
of the error's content. One thread-local slot names the current native activation — set by the
intrinsic dispatcher (the registered `#[wat_intrinsic(":…")]` name), the special-form evaluator (the
form's head), and, before any wat activation exists, the freeze pipeline (its `pass_order` phase
names) — and `RuntimeError::new` reads it, as it reads `CALL_STACK`. Obvious YES, Simple YES (one
slot, O(1), zero error-kind changes), Honest YES (names what was executing; never inferred from the
error), Good UX YES. Rejected: threading name fields onto ~10 kinds (Simple NO — ten wire changes
for a framing concern, braiding *what went wrong* with *where it ran*); any fallback name (Honest NO).

Side finding: `src/numeric/arith.rs:~88`'s doc claims `DivisionByZero`'s `op` names the caller's
spelling — `DivisionByZero` has no `op` field. A false comment; correct it.

## Strike D landed (`0df5038e2`, `f4a873146`, `0dfcdb5ea`)

`Frame` is `{fn at tail-elided}`, innermost first in true order, the Rust activation innermost and
named by the `CURRENT_ACTIVATION` slot (intrinsic/special-form dispatch + freeze phases); `kind`,
`<rust>` and D4's synthesized frame are gone; tail calls name their last caller and count the rest.
C-114 reads `{:fn ":wat::i64::+" :at arith.rs} {:fn ":wat::core::+" :at core.wat:66}
{:fn ":user::grow" :at …:10 :tail-elided 1}`. 70 goldens recaptured (nothing outside Frame fields
moved); GD5 lint anchor 95 violations / 70 goldens; corpus migrated by codemod. Floor 6381/6381.

Orchestrator check (5 real producers through the binary, since no test asserts the Rust frame is
PRESENT): DivisionByZero → `:wat::i64::/`, IntegerOverflow → `:wat::i64::*`, MalformedForm ×2 →
`:wat::keyword::from-string` / `:wat::core::first`, NotCallable → `:wat::core::let`. All named.

| from | finding | the cure owed |
|---|---|---|
| D | the Rust frame is `Option`: OMITTED when the activation slot is empty. The floor cannot see an omission (no assertion of presence); the only known empty shape is a bare `RuntimeError::new` in a unit test. The per-kind census of all 40 kinds was not built | build the census as a standing gate: every kind's real producer carries a NAMED Rust frame — a presence assertion, so an omission goes RED |
| D | a call whose head is a SYMBOL (`(f 1)`, `f` a local) sets no activation, so its raise is named by the enclosing form (`NotCallable` → `:wat::core::let`) | the application path names its own activation (the callee's resolved name, or the symbol) |

## Strike D2, item 1 landed (`c003ac8fa`) — the census is partial, and it found E's defect from below

A symbol-headed call names itself (`(f 1)` → `f`, not the enclosing `let`); `apply_function` names
the resolved callee, replaced in place on tail continuation (GD2b, mutation-proven). Floor 6382/6382.
The 40-kind census reached 26 kinds: 17 real producers, all named; 9 with no real producer.

| from | finding | the cure owed |
|---|---|---|
| D2 | **`BadCondition`, `PatternMatchFailed`, `EffectfulInStep`, `NoStepRule` are reachable ONLY through `:wat::eval-ast!` / `:wat::eval-step!`**, whose `wrap_as_eval_result` flattens every `RuntimeError` into `EvalError {kind message}` (`runtime_error_to_eval_error_value`, `src/runtime.rs:~12476`) — the raise happens with a correctly-named frame, then the frames are discarded before any observer | strike E (the eval family carries the real error) — so E runs BEFORE the census is finished |
| D2 | `ParamShadowsBuiltin` has zero producers (registration only, `src/types.rs:~2563`) — a dead variant | retire it |
| D2 | `NoEncodingCtx`/`NoSourceLoader`/`NoMacroRegistry` are unreachable from a frozen world (the pipeline always installs the capability); `UserMainMissing`'s producer was not found from the CLI or `spawn-peer` | the census lists them with reason; a kind with no producer from any real path is a candidate for retirement, measured not assumed |
| D2 | `AssertionFailed` from `assertion-failed!` travels `AssertionPayload`, never `RuntimeError::new` — a sibling path; the census asks it the wrong question | the census checks each kind on the path it actually takes |
| D2 | `src/freeze/pass_order.rs`'s `record()` doc says `UserMainMissing`/`EvalVerificationFailed` are the only freeze-phase producers; six more raise during `6-register-defines`/`7-resolve-references` | correct the comment when the census lands |

Sequencing: **E → finish the census (GD2a as a standing gate, 40 kinds, on the path each actually takes) → the `Option` decision → F → the post-F strikes.**

## CORRECTION 2026-10-04 — F4's headline number was contaminated

F4 said `EvalError` was "the most-read error type in the language" — 204 `EvalError/kind` reads, 215
`EvalError/message`, 453 mentions. Re-measured at `f2d4375ca`: those counts include
`wat-scripts/scratch-pad/` (16 files carry nearly all the `kind` reads — stone-probe scratch).
Outside scratch-pad: **32 mentions; 2 readers** — `wat/doctest.wat` and `tests/value/wat_eval_result.wat`.
The verdict (carry the real error) stands; its blast radius is a fraction of what F4 claimed.
