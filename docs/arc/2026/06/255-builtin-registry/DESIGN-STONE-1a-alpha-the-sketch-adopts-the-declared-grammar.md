# DESIGN — STONE 1a-α: the sketch adopts the registry's DECLARED grammar

> Phase 1a of `[[DESIGN-CAMPAIGN-the-registry-becomes-the-sole-authority]]`, governed by
> `[[RULING-the-registry-is-the-sole-authority]]`.
>
> **Builder, 2026-09-01:** *"match........... dropped -> T syntax...... a long time ago......
> only fns declare types......."*
>
> This stone is the stepping stone that must land **before** the 23 registrations, and it is the
> stone that retires the fossil that ruling names.

## ★★★ The finding this stone exists for: reflection has TWO renderers of one question, and they disagree

Both live in the registry's own reflection surface. Both answer *"what is this form's shape?"*
Neither knows about the other.

```
src/intrinsic/reflect.rs:456-470   render-doc      @syntax verbatim  →  else derive from @arg  →  else nothing
src/reflect/verbs.rs:186-212       lookup-form     derive from @arg  →  else special_forms.rs   →  else a sentinel
```

★ **`render-doc` already consults `@syntax` first and has since it shipped.** `lookup-form` never
consults it at all. Same registry, same rows, opposite precedence — the exact duplication the RULING
orders eliminated, sitting inside the authority that is supposed to end duplication.

### What that costs today, measured

The registry DECLARES the correct grammar for all three forms whose sketch is wrong:

```
src/intrinsic/special/binding.rs:28    @syntax (let [<binder> <expr> ...] <body>+)
src/intrinsic/special/fn_form.rs:45    @syntax (fn [<param> <- :T ...] -> :RetType <body>+)
src/intrinsic/special/match_form.rs:38 @syntax (match <scrutinee> (<pattern> <body>) ...)
```

and `lookup-form` serves, from `src/special_forms.rs`, the coarser sketches instead:

```
tests/wat_lang/wat_arc144_special_forms__let.edn     (:wat.core/let <bindings> <body>+)
tests/wat_lang/wat_arc144_special_forms__fn.edn      (:wat.core/fn <params> <body>+)
tests/wat_lang/wat_arc144_special_forms__match.edn   (:wat.core/match <scrutinee> -> <T> <arm>+)
```

⛔ **The third is not merely coarser — it is a SIX-WEEK-OLD FOSSIL.** `src/special_forms.rs:171`
has served `["<scrutinee>", "->", "<T>", "<arm>+"]` since before 2026-07-22, the day arc 278
annihilated `-> :T`. `check.rs`'s `infer_match` refuses that form today with a **named error**
(`":wat::core::match no longer takes -> :T"`, `check.rs:6088`). **Reflection has been teaching users
a grammar the checker rejects, for six weeks, and the correct text was sitting in the registry the
whole time.**

That is the campaign's thesis demonstrated on itself: two authorities drifted, nothing noticed, and
only folding one into the other made it visible.

## The change, in one sentence

`signature_of_defn` gains **one arm, placed first**: a `Binding::Registered` row whose
`entry.syntax` is non-empty renders that string through the substrate's own reader — adopting
`render-doc`'s precedence exactly, so the two renderers stop disagreeing.

## ★★ THE PROBE — committed BEFORE the brief, and it refuted my own prior framing

`wat-scripts/scratch-pad/255-can-the-reader-parse-a-syntax-grammar.wat`

Before compaction I stopped this stone with: *"rendering `@syntax` means turning a grammar string
into a WatAST, and the tokens (`[…]`, `(<pattern> <body>)`) don't survive a naive whitespace split
— the right move is the substrate's own reader, and choosing that is a design decision."*

The probe asks whether the reader can actually do it. It can — all three, clean:

```
#wat.core.ReadOutcome/Forms [((let [<binder> <expr> ...] <body>+))]
#wat.core.ReadOutcome/Forms [((match <scrutinee> (<pattern> <body>) ...))]
#wat.core.ReadOutcome/Forms [((fn [<param> <- :T ...] -> :RetType <body>+))]
```

`<binder>`, `...`, `<body>+`, `<-`, `->`, `:T` all read without complaint. **There is no new
machinery to design.** The Rust-side entry point is `wat_reader::parser::parse_one_with_file`
(`crates/wat-reader/src/parser.rs:238`) — already a dependency of `src/reflect/`, already the
substrate's own reader, and therefore the only honest authority on what a wat form looks like.

★ `[[feedback_a_design_is_unfalsifiable_until_something_consumes_it]]` — the design that said
"new machinery" was never measured. Fifteen minutes of probe deleted the whole objection.

## THE ONE CONTRACT DECISION — pinned

**`@syntax` is rendered VERBATIM, through the reader, with no substitution of any kind — exactly
as `render-doc` renders it.**

⚠ Verbatim means the renderer never edits the string — the row's declaration is the answer. The
temptation is to "fix up" a head at the consumer; **rejected**, because that mints a THIRD rendering
of the same question, authored by the renderer rather than declared by the row.

## ⛔ AMENDED 2026-09-01, AFTER THE STONE SHIPPED — the paragraph that stood here was WRONG

It read: *"`@syntax` names the form with its **short** head (`let`, `match`, `fn`) … `render-doc`
already ships the short head; after this stone the two agree."* Both sentences were true as
descriptions and wrong as a decision.

> **Builder:** *"wait.... we had short hand `if` not `:wat::core::if` nor `wat.core/if` ?.......
> we are grinding through this registry to force ourselves into clojure/edn compliant syntax......
> wat is fqdn.... always.... anything that in not a binder... is illegal.... even bound symbols.....
> are shadow fqdn.... belong to the `$bound` namespace........"*

**A short head is not a rendering style. It is not-wat.** The whole clojure-ination migration exists
to force FQDN compliance, and this stone shipped a doc surface teaching the spelling it is trying to
eliminate. `render-doc` shipping it first made it precedent, not correct.

★ **And my probe measured the wrong thing.** It proved the reader PARSES those strings — it does.
Parsing is not legality: the reader takes a bare head, `resolve`/`check` is what must refuse it (and
today does not — `(zorble [x 1] x)` also type-checks clean, which is this arc's founding target
restated). `[[feedback_a_probe_answers_the_question_you_asked_not_the_one_you_meant]]`

★★ **The contract survived the correction intact, and that is the evidence for it.** Because the
renderer edits nothing, the fix landed in the three DECLARATIONS and the arm never changed —
correcting the row corrected BOTH renderers at once. A consumer that had spliced a head locally would
have left `render-doc` still printing the illegal form. What the correction also exposed:
`render-doc`'s DERIVED path built its head with `identifier::leaf(entry.name)`, a second copy of the
same defect in code rather than in a declaration, and for a nested name (`:wat::core::Bytes::to-hex`
→ `to-hex`) the short head was not merely illegal but **ambiguous**. Both paths now emit the FQDN.

## What changes, exhaustively — three goldens, and nothing else

| row | vehicle after this stone | golden |
|---|---|---|
| `:wat::core::let` | `@syntax` | **CHANGES** → `(let [<binder> <expr> ...] <body>+)` |
| `:wat::core::fn` | `@syntax` | **CHANGES** → `(fn [<param> <- :T ...] -> :RetType <body>+)` |
| `:wat::core::match` | `@syntax` | **CHANGES** → `(match <scrutinee> (<pattern> <body>) ...)` ★ the fossil dies |
| `:wat::core::if` | `@arg` ×3 (no `@syntax`) | unchanged — arm 2 still answers |
| `:wat::core::and` / `or` | `@arg` (no `@syntax`) | unchanged |
| `:wat::core::quasiquote` | still unregistered | unchanged — `special_forms.rs` still answers |
| every `Kind::Intrinsic` row | `@arg` (measured: all 6 SPECIAL_FORMS rows registered as intrinsics carry `@arg`) | unchanged |

**Exactly three goldens move, and every one moves to text the registry already declares.**

## ⛔ THE FAILURE CLASS THIS STONE MUST NOT CREATE — and the rung it is pulled out at

A grammar string that does not parse would make reflection fall silently to the next arm — an
absence recorded as an answer, the defect class this arc has a NOTE family about.

**The fix is not a `match` on the error. It is a gate:** a floor test walks `registry()`, takes every
row with a non-empty `syntax`, and asserts `parse_one_with_file` returns `Ok`. A malformed `@syntax`
becomes a **red floor at the moment it is authored**, not a silent hole discovered six weeks later
— which is precisely how the `match` fossil survived.

★ `[[extirpare]]`'s ladder: the convention rung ("author your `@syntax` carefully") is what we have
now and it is what failed. This is the check rung, and it is the rung the material allows — a
proc-macro cannot parse a wat grammar at compile time without linking the reader into the macro
crate, which is a bigger change than this stone.

⚠ And the gate must be proven able to FAIL. Sabotage: corrupt one `@syntax` to `(let [` and confirm
the floor goes red naming that row. **A gate never seen red is not a gate**
(`[[feedback_a_green_test_can_prove_nothing]]`).

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **A — `@syntax` first, verbatim, + the parse gate** | YES | YES | YES | YES | ✅ **PICKED** |
| B — author `@arg` for `let`/`fn`/`match` instead | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| C — update the three goldens to the fossil-free text by hand | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| D — `@syntax` first, but splice the FQDN head in | YES | **NO** | **NO** | — | ⛔ DISQUALIFIED |
| E — render `@syntax`, no parse gate | YES | YES | YES | **NO** | ⛔ DISQUALIFIED |

- **B Honest? NO.** `@arg` carries a TYPE (`@arg exprs… :wat::core::bool …` — `and_form.rs`'s own
  shape). `<scrutinee>`, `<bindings>`, `<params>` are **syntactic positions with no type**. Filling
  that slot with a type claim mints a lie, and it is the lie the seam explicitly refused to
  improvise. `@syntax` is a grammar, which is what these three actually have.
- **C Honest? NO.** It corrects the symptom at the golden and leaves both renderers, both
  precedences, and the whole drift mechanism in place — the RULING's *"a gate that compares two
  tables is a measurement of the split, not a cure for it."* The next fossil forms the same way.
- **D Simple? NO, Honest? NO.** A third rendering, authored by the consumer, of a string the row
  already declares. It also silently disagrees with `render-doc` again, in the other direction.
- **E Good UX? NO.** It hands the next 23 registrations a way to write an unparseable grammar and
  discover it never — the exact six-week shape this stone is repairing.
- **A Simple? YES** and it is worth stating why, since the campaign's headline shape (FOLD) answers
  NO on this axis: this stone adds ONE match arm and ONE test. It removes a precedence disagreement
  rather than introducing a mechanism.

## Blast radius

```
src/reflect/verbs.rs      + one match arm, placed FIRST in signature_of_defn
src/intrinsic/mod.rs      + one floor test (the @syntax-parses gate), beside the other ratchets
tests/wat_lang/           3 goldens recaptured via UPDATE_EDN
wat-scripts/scratch-pad/  the probe (already committed)
```

No `.wat` corpus change. No registrations. No dispatch change. No checker change. **No form's
behaviour moves** — this stone changes only what reflection SAYS about three forms, and it changes
it to what the registry already declared.

## Acceptance — rows chosen to be unfakeable

| what | command | expected |
|---|---|---|
| `match`'s fossil is gone | the `__match.edn` golden | no `->` and no `<T>` anywhere in it |
| all three render the DECLARED string | each golden vs its `@syntax` line | byte-identical after the head |
| ⛔ `if` did NOT move | `__if.edn` | unchanged — proves the `@arg` arm still wins when there is no `@syntax` |
| ⛔ `quasiquote` did NOT move | `__quasiquote.edn` | unchanged — proves the `special_forms.rs` deferral survives for unregistered rows |
| the two renderers now AGREE | `render-doc` vs `lookup-form` for `let`/`fn`/`match` | same grammar text |
| ⛔ the gate can FAIL | corrupt one `@syntax` to `(let [` | floor RED, naming that row |
| ⛔ the gate is not vacuous | count rows it actually inspects | ≥ 3, and it names them |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5119/5119, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |

⚠ **The two ⛔ "did NOT move" rows are the load-bearing ones.** They are what distinguishes "the
new arm is placed correctly" from "the new arm swallowed everything" — and a golden sweep that
updated all five would have hidden exactly that.

## Out of scope = REJECTED (affirmatively, not deferred)

- **Registering the 23.** That is 1a-β…ε below. This stone deliberately lands first so each of those
  rows knows which vehicle to declare.
- **Deleting `src/special_forms.rs`.** It still answers for the 23 unregistered rows. It has exactly
  **three** consumers (`src/reflect/verbs.rs:204`, `src/reflect/lookup.rs:262`, one test) — measured
  — so it dies cheaply at Phase 4a, and not before the rows that need it are registered.
- **The FQDN-vs-short-head convention anywhere else.** `render-doc` already ships short; this stone
  makes `lookup-form` match it. Any broader ruling on reflection's head vocabulary is not this stone.

## ⬜ What 1a-α unblocks — the 23, decomposed by shared axis argument

Measured this session: `src/special_forms.rs` holds **35** rows; **12** are registered (6 as
`#[wat_special_form]`, 6 as `#[wat_intrinsic]` — the precedent already splits both ways); **23** are
not. **Every one of the 23 is dispatched by hand-written literal arms** (`check.rs`, `runtime.rs`,
`types.rs`, `declare/`, `macros/`, `load/`) and **none has a native handler** — so all 23 are
`#[wat_special_form]` candidates, and registration therefore imposes:

- **no dispatch change** — `Kind::SpecialForm` carries `handler: None`
- **no checker change** — `check.rs:5737`'s registry-arity door is guarded
  `entry.kind == Kind::Intrinsic`, measured, so special forms never reach it
- **two ledgers move**, and they are the falsifiable meter:
  **9 of the 23 are in `GAP_B_CORPUS_CENSUS_121`** (→ `GAP_B` 113 → 104) and
  **10 of the 23 are in `KNOWN_UNREVIEWED`** (→ 29 → 19)

The families, each one axis argument applied N times rather than N arguments:

```
1a-β  the definitional 6   def · defmacro · defstruct · defenum · newtype · typealias
1a-γ  the homoiconic 8     quote · quasiquote · unquote · unquote-splicing ·
                           macroexpand · macroexpand-1 · forms · struct->form
1a-δ  the loaders 4        use! · load-file! · digest-load! · signed-load!     (effectful)
1a-ε  the remainder 5      ann-form · do · stream::lazy · set-redef! · set-eval-redef!
```

⚠ **This is not a mechanical sweep and the design says so.** `and_form.rs` is ~45 lines of dense
per-axis reasoning for ONE row; each of the five axes is a ruling with grounds, not a label. The
family grouping exists precisely because within a family the Purity/Determinism/Totality argument is
**one** argument — which is the only thing that makes 23 rows tractable.
