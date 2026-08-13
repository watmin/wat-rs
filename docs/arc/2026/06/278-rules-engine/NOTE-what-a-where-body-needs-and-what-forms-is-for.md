# NOTE — what a `where` body needs, and what `forms` is actually FOR

**2026-08-12. Two routes proposed, BOTH REFUTED BY RUN in one session. Filed so neither is
re-derived.** This is not a design. It is the ground a design has to stand on, and the two places
the ground gave way.

## The question

`resolve::boundary::quote_boundary` carries **library grammar** — `:wat::rete::make-rule` and
`:wat::form::matches?` sit beside `quote`/`quasiquote`/`match`, plus `is_where_form()`, a whole
function for one library form. rete edits the compiler's list; a third-party DSL author cannot.

The builder's frame: *"suppose i was a third party developer — how would i express a rete like dsl
to all wat users… you keep landing on 'the only option the universe allows is making rete special'."*

What rete needs and the language cannot say: **"this subtree inside my quoted data is CODE — resolve
it, expand it, and leave it a FORM."** The `where` body must stay a form, not become a value and not
become a closure, because the Rust jump table (task **#49** `compiled_where`) compiles the form.

## REFUTED (1) — quasiquote's escape cannot carry it

For RESOLUTION an unquote escape means *"resolve this subtree in place"* — fits.
For EVALUATION it means *"evaluate and splice the VALUE"* — `runtime.rs:10891` evaluates the escape
and `value_to_watast`s the result (`:11074`, which accepts primitives / keyword / nil / `WatAST`).
A `where` body like `(> ?c 50)` is none of those: it either dies on unbound `?c` or collapses to a
`bool`. **The form is destroyed either way.**

## REFUTED (2) — `forms` is NOT an undesigned asymmetry; it is the child-program constructor

I claimed `forms` was "expand-yes / resolve-no, an undesigned asymmetry" and proposed making it
resolve-transparent. **Wrong, and the corpus says so with a purpose-built fixture.**

Imposed the check (removed `forms` from `Boundary::AllData`), built, ran the floor:

- **build GREEN** — the whole baked stdlib froze fine.
- **positive control confirmed the imposition was live** — the same probe went from 1 unresolved
  reference (bare call only) to 2 (bare call + the one inside `forms`).
- **floor: `4391 tests run: 4367 passed, 24 failed, 262 skipped`.**

The failing 24 cluster, and the clustering IS the finding:

| cluster | n | what it is |
|---|---|---|
| `wat_core_forms::*` | 6 | the entire `forms` specification module |
| declaration-lift | 9 | `closure_body_prelude_lift`, `declaration_form_lift`, `def_not_special` |
| **services** | 5 | arc209 service loop, arc272 state-over-lineage, arc170 bracket — **child-program assembly** |
| kernel | 2 | counter-actor process proof, recv budget |
| loader gate | 1 | `every_wat_scripts_file_loads` |

`tests/macros/probe_resolver_quote_awareness.rs:19`'s assertion message IS the specification —
*"startup must succeed: forms arguments are data, not live call heads"* — and its fixture
(`probe_resolver_quote_awareness_forms_data.wat`) names `:my::probe-f2::ghost-inner` and
`:my::probe-f2::ghost-other`: **deliberately chosen not to exist.** Its two siblings
(`probe_quote_argument_is_data`, `probe_quasiquote_unquote_resolves_correctly`) still passed, so the
three are one designed specification of all three boundaries.

> ★ **`forms` builds AST for ELSEWHERE.** It must not resolve locally, because the universe it names
> is not this one. `spawn.wat:559` — `(forms (def :user::spawn::service-locus (process)))` — is
> constructing the CHILD's program. That is why the services cluster broke: the experiment died on
> precisely the mechanism this arc exists to serve.

Reverted; `boundary.rs` is byte-identical to HEAD. **Do not re-propose making `forms` resolve.**

## The distinction that survives, and it is the useful output

| | resolves HERE? | stays a form? | means |
|---|---|---|---|
| `quote` | no | yes | data |
| `forms` | **no — by design** | yes | code for *another* universe |
| `quasiquote` + `~` | yes | **no — becomes a value** | data with computed holes |
| **what a `where` body needs** | **yes** | **yes** | code for *this* universe, not yet run |

That fourth row has no form in the language. Everything else follows from it.

## Where the privilege actually came from — rete's own macro says so

`wat/rete.wat:2403`, three lines above `defrule`'s template:

> *"The macro is kept **TRIVIAL**: it quotes both vectors as-is… `make-rule` does the per-element
> split at **runtime**."*

`defrule` performs no transformation. It quotes patterns and `where` bodies together and defers to
runtime — so code ends up inside a runtime `quote`, and the compiler needed a private door to reach
back in. **Clara's macro does not punt**: it splits its own conditions at expansion time and emits
real `fn`s, which is why Clojure needs no new form. The builder's challenge — *"why doesn't clojure
need some new form?"* — is answered by that comment.

**The split at expansion time is the move.** It deletes `Boundary::MakeRule`, `is_where_form`, and
the three hand-rolled `make_rule` descents in `walk`/`normalize`/`expand`. What is UNSETTLED is what
the body becomes once split — a `fn` (Clara's answer, opaque to #49's jump table, though `fn-forms`
recovers forms) or a form (needs the missing fourth row).

## MEASURED, and load-bearing for the main line

Refusing `Boundary::MakeRule` in `closure_extract` has a real cost, proven with a non-vacuity
control (`probe_where_body_dep.wat`, three arms, one shared helper `:usr::big?`):

```
POSITIVE-CONTROL (ordinary call)        forms=6     ← the collector finds it fine
BASELINE (where, no user dep)           forms=5
SUBJECT  (where CALLS :usr::big?)       forms=5     ← IDENTICAL — NOT collected
```

**A fn referenced only inside a `where` body is never collected into the shipped forms.** The rule
ships; the function it calls does not; the child names the missing symbol at startup. That is the
"failing to deliver rules to install-rules" symptom, mechanised.

Also proven and NOT fixed: the refusal removes **no** privilege — `quote_boundary` still returns
`MakeRule`, and `walk`/`normalize`/`expand` all still honour it.

## Reproductions — inline, because both probes MUST FAIL

Neither can live under `wat-scripts/` (the `every_wat_scripts_file_loads` gate would go red on a
file that is supposed to be refused), and a `.wat.bad` with no paired `.rs` is an inert file nobody
runs. So the sources live here, where the claim lives.

**(A) `forms` is not resolve-transparent.** Same unresolvable head in two positions; only the bare
one is reported. Run `target/release/wat --check <file>`:

```clojure
(:wat::core::defn :probe::bare [] -> :wat::core::i64
  (:nosuchns::vanished 1))                                    ;; line 14 — REPORTED

(:wat::core::defn :probe::in-forms [] -> :wat::core::Vector<wat::WatAST>
  (:wat::core::forms (:nosuchns::vanished 1)))                ;; line 17 — NOT reported

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
```

→ `1 unresolved reference`, line 14 only. (With `forms` removed from `Boundary::AllData` it becomes
`2 unresolved references`, lines 14 **and** 17 — that is how the imposition was validated as live
before the corpus census was believed.)

**(B) A `Rule` cannot hold a fn — 293.W containment.** A record field of fn type is refused
outright:

```clojure
(:wat::core::defrecord :probe::Cond
  [pred <- :wat::core::Fn(wat::core::i64)->wat::core::bool])
```

→ `#wat.type/ImpureFieldInPureAggregate` — *"pure aggregate ':probe::Cond' may only hold pure
fields… A struct cannot be reconstructed from EDN bytes across a comms boundary; a record or holon
holding a struct field could never cross — it must not exist."*

The fn-field precedents that made this look viable — `:wat::spawn::ThreadOpts` (`spawn.wat:51`) —
are **`defstruct`**, not `defrecord`; `bracket.wat:34` is a defn **parameter**, not a field at all.
`:wat::rete::Rule` is a `defrecord` (`rete.wat:52`).

## Kin

R60 `QVOD FAVET PRIMVM CADIT` (two of my own premises died here and the answer improved) ·
R59 `NISI FRANGAS NIHIL PROBAS` (the imposition was the break; the census was the proof) ·
`feedback_impose_the_check_and_read_the_screams` (the census cost one build, not a survey) ·
`feedback_ask_whether_a_constraint_was_ever_chosen` — asked, and this time it **was** chosen.
