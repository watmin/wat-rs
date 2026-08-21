# DESIGN — arc 109 Stone ②-i-b: `:-`, the parameterization operator (+ the Tuple arm)

**Status: RE-DRAWN 2026-08-20 after the builder's ruling on `:-`.** Blocks ②-iii.
Written against `c557e34b5`.

## ★ THE RULING — `:-` is one operator with one meaning, in every declaration position

> Builder: *"the symbol ` :- ` is declaring **'this thing on the left is parameterized by the thing
> on the right'**"* … *"this is the same as arg-spec and ret-type in my mind — they declare what
> they are explicitly."*

```clojure
(wat.core/defn user/some-fn
  [n :- wat.type/i64]                       ; arg-spec      — n is parameterized by i64
  :- wat.type/i64                           ; ret-type      — the fn is parameterized by i64
  (wat.core/+ n 1))

(wat.type/Vector :- [wat.type/i64])         ; type args     — Vector is parameterized by [i64]
(wat.type/Vector :- [wat.type/i64] 1 2 3)   ; constructor   — same head, values follow
```

**Why this is not a redundant marker, which is what I argued twice and was wrong about twice.** I
read `:-` as a *position marker* — "a type follows here" — which is redundant wherever position
already decides, and the mandatory type-vector (`3821db4ba`) means position always decides. That
analysis was correct about a delimiter and simply not about `:-`. A **relation** is never redundant
with position, because position can encode at most ONE relation implicitly, and they nest:

```clojure
[xs :- (wat.type/Vector :- [wat.type/i64])]
     └── xs is parameterized by that type
                             └── Vector is parameterized by [i64]
```

Two distinct facts, one operator. That is compositionality, not verbosity — and it is the same axis
FQDN-everywhere sits on: the resolver could always find the bare name; FQDN exists so nothing is
inferred from context. **Juxtaposition is context.**

## Why

②-i (`0422b67ff`) gave `type_expr_to_clojure_form` a head-spelling mode and bracketed the
`Parametric` arm. Its rider scoped ONE arm out and said so plainly:

> *"`TypeExpr::Tuple`'s head (`wat.type/Tuple`) I left OUT OF SCOPE for `mode` — it's not part of
> the 4-way ladder Room 2 scopes to, and nothing in the acceptance criteria or the 8-fixture
> contract suite exercises a COLON-mode Tuple."*

Accurate and correctly reported. ②-ii then walked into it, and the codemod had to grow a
rendered-output guard that SKIPS rather than corrupts. `NOTE-the-Tuple-arm-is-mode-blind.md` files
the measurement. This stone closes it.

## Measured at HEAD, by the orchestrator's own hand

`wat-scripts/scratch-pad/arc109-tuple-arm-faults.wat`:

```
1 nil bare       : (wat.type/Tuple)
2 nil nested     : (:wat::core::Result [(wat.type/Tuple) :wat::core::String])
3 tuple 3-ary    : (wat.type/Tuple :wat::core::i64 :wat::core::i64 :wat::core::String)
4 tuple 1-ary    : (wat.type/Tuple :wat::core::i64)
5 tuple empty    : (wat.type/Tuple)
6 control parm   : (:wat::core::Vector [:wat::core::i64])
```

Three faults: wrong head spelling in COLON mode · mixed spelling inside one otherwise-correct form ·
args spliced FLAT instead of bracketed, at every arity. Row 6 is the control — `Parametric` is
already right.

**Rows 1 and 5 are the finding: `nil` and `:()` render IDENTICALLY.**

## The correction this stone rests on

The seam recorded `nil` and `()` as *"verified distinct at the surface"* because `-> :()` with a nil
body exits 1. The exit code was right; the inference was not. The error text says the opposite:

> `BareLegacyUnitType`: *"bare unit type '()' is retired (arc 109 slice 1d); canonical FQDN form is
> ':wat::core::nil' (arc 153 renamed unit -> nil)"*

`:()` is rejected as a **retired spelling of the same type**, not as a different type. Internally
`nil ≡ TypeExpr::Tuple(vec![])`, and ~30 sites in `check.rs`/`runtime.rs`/`freeze.rs` use
`Tuple(vec![])` *as* the unit type.

Two consequences:

- **It bounds the corpus work.** `:()` appears **0 times** as a type annotation in the corpus (the
  only three `:()` hits are a string fed to the verb in a fixture, a comment, and this stone's own
  probe). The corpus work is the non-empty tuples: **243 occurrences** — `wat/` 52 · `wat-scripts/`
  165 · `tests/` 26.
  ⚠ The NOTE's "30 standalone tuples" is a DIFFERENT measurement — what the codemod's guard skipped
  on the paths it ran, not a corpus census. Neither number is wrong; they answer different questions.

  ⛔ **CORRECTED 2026-08-20, same day, builder's catch.** An earlier draft of this design read that
  census as *"an empty Tuple is unreachable from legal source; the empty case is defensive."*
  **That is false.** The census saw the KEYWORD spelling `:()` and I generalised it to the type.
  The FORM spelling is legal, writable source today — `(wat.type/Tuple [])` type-checks as a param
  type and as a return type, measured in this stone's own reader probe, which I had already run.
  The retirement was of one *spelling*, not of the empty tuple. The empty rung is a first-class
  member of the arity ladder and this stone must treat it as one.
  `[[feedback_state_what_the_instrument_can_see_before_quoting_it]]`
- **It leaves the builder's ruling one step short.** `(Tuple [])` is already WRITABLE; what it is
  not yet is a *distinct type*. Measured: a `nil` argument satisfies a `(wat.type/Tuple [])` param
  AND a `:wat::core::nil` param in the same file, `--check` EXIT 0. Both are `TypeExpr::Tuple(vec![])`
  internally, load-bearing at ~30 checker sites. So `nil != ()` is a **type-identity split**, not a
  syntax question — and it is a separate, larger stone, and the builder's. It does not block this
  one: keeping `nil` as `Path(":wat::core::nil")` at parse time is correct under either future.

## ★★ THE MEASURED PAYOFF — `:-` retires a heuristic that GUESSES

This is the part that makes the operator a correctness change and not an aesthetic one, and it was
written down in the substrate by step ①b's own rider, who named the cure without knowing it had one:

> `src/check.rs:12027`, `is_type_bracket_candidate`'s doc: *"This does not distinguish a bracket
> from a data-vector-of-KEYWORDS, e.g. `[:a :b]` … so the ambiguity is real but currently vacuous;
> **a future literal vector-of-keyword-VALUES in this exact position would need a different
> production to stay unambiguous.**"*

`:-` **is** that production. And the hazard is not hypothetical — measured at HEAD:

```
(:wat::core::Tuple [:a :b])          → ArityMismatch: expected 2 argument(s); got 0
(:wat::core::Tuple "tag" [:a :b])    → ["tag" [:a :b]]        ← same vector, slot 2, fine
```

A 1-tuple whose single value is a vector of two keywords is **unwritable today**. The bracket is
sniffed by `is_type_bracket_candidate` — a function whose entire job is to GUESS whether a bracket
is types or data by inspecting its contents — and guessed wrong. That is the middle rung of the
extirpare ladder: a check that runs and can be wrong. With `:-`, slot 1 is a value ALWAYS unless
`:-` precedes it, the guess has nothing left to decide, and **`is_type_bracket_candidate` can be
deleted outright.**

⚠ **The deletion is ③'s, not this stone's**, and the sequencing is the campaign's existing rhythm:
② ADDS the spelling (dual-read — both `(Head [T])` and `(Head :- [T])` parse), ③ makes the old ones
illegal. While the unmarked bracket is still accepted the heuristic must stay, so
`(:wat::core::Tuple [:a :b])` stays broken until ③. **③'s scope therefore grows by one line:
delete `is_type_bracket_candidate` and its three call sites, and the keyword-vector value becomes
writable as a side-effect of the hard-cut.**

## The change

**(a) The verb stops canonicalizing.** `eval_keyword_to_type_form_impl` (`src/edn_shim.rs:1364`)
calls `parse_type_expr`, which hardcodes `canonicalize=true`; `src/types.rs:4728` then collapses
`:wat::core::nil` → `Tuple(vec![])`, which is why the renderer cannot say `nil`. A `canonicalize:
bool` already exists (`src/types.rs:4625`). The verb gets a non-canonicalizing sibling entry point.

The one other thing that flag governs is the `:wat::type::` → `:wat::core::` alias. Measured: every
`:wat::type::` keyword in the corpus is `:wat::type::Infer`, **all 39 of them**, and the codemod's
`type-shaped-keyword?` never selects it (no matching `<…>`). Preserving its spelling is *more*
faithful, not less. The flip is clean.

**(b) `:-` is ACCEPTED before the type-vector, and EMITTED by the renderer** — in the `Parametric`
arm and the `Tuple` arm alike, in both head-spelling modes (`:-` is a Keyword and is mode-independent).
The unmarked bracket keeps parsing; only the emitted form changes.

**(c) The Tuple arm brackets and honours the mode** — exactly what `Parametric` got in ②-i. The
head always takes a bracket, at EVERY arity including zero. The full ladder, both spellings, as the
builder set it down, now carrying the operator:

```
(:wat::core::Tuple)                                           ILLEGAL — a bare head is not a form
(:wat::core::Tuple :- [])                                     empty
(:wat::core::Tuple :- [:wat::core::i64])                      1-ary
(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String])   2-ary

(wat.type/Tuple :- [])
(wat.type/Tuple :- [wat.type/i64])
(wat.type/Tuple :- [wat.type/i64 wat.type/String])
```

★ **The 1-ary rung is where the form surface is strictly better than the keyword surface,** and it
is worth stating because it is the reason the arity ladder has to be spelled out rather than
summarised as "bracket the args". On the keyword surface `:(A)` is Rust GROUPING and collapses to
`A`; a 1-tuple can only be spelled `:(A,)`, with a trailing comma carrying the entire distinction.
The form surface has no such ambiguity — `(wat.type/Tuple [A])` is a 1-tuple and nothing else.
Measured distinct from a scalar: passing a bare `7` to a `(wat.type/Tuple [wat.type/i64])` param is
a TypeMismatch.

This is the builder's ruling, 2026-08-20:

> *"nil is rust's unit… but `nil != ()` in wat. nil is not an empty list. `(wat.type/Tuple)` is
> illegal, it'd be `(wat.type/Tuple [])` to be an empty tuple."*

## The transitional spelling, named

The corpus still writes `<-` (7,488) and `->` (6,797) for arg-spec and ret-type; `:-` appears 66
times. Arc 251.4a made `:-` a dual-read alias for both and 251.5 hard-cuts the arrows. So a
mid-transition site reads `[xs <- (:wat::core::Vector :- [:wat::core::i64])]` — mixed, and that is
FINE: dual-read holds and 251.5's sweep catches the arrows up. The parametric operator lands as
`:-` directly and never as `<-`, because `<-` contains the very glyph this arc is annihilating.

## The contract decision, pinned

**Two, and they are independent.**

1. The args-tail production in `parse_type_node` (`src/types.rs:4528`) accepts BOTH
   `[Vector(inner)]` (today) and `[Keyword(":-"), Vector(inner)]` (new). Dual-read; ③ cuts the first.
2. `pub fn parse_type_expr_preserving_with_span(kw: &str, span: &Span) -> Result<TypeExpr, TypeError>`
— byte-identical to `parse_type_expr_with_span` (`src/types.rs:4334`) except `canonicalize=false`.
It **still calls `reject_any`**. It returns `Result`, NEVER `Option` — the verb surfaces parse
errors and `parse_type_expr_audit` (the existing `canonicalize=false` path) swallows them, which is
why that one cannot be reused.

## The reader already accepts the bracketed form — proven, not read

`wat-scripts/scratch-pad/arc109-tuple-bracket-reader.wat`, `--check` EXIT=0. `parse_type_node`'s
bracket unwrap (`src/types.rs:4528`) is head-agnostic and the `Tuple` branch (`src/types.rs:4540`)
reads `args` AFTER it, so the bracket rule reached Tuple for free at step ①. The probe pins:
bracketed 2-ary as a param type · bracketed with a nested parametric as a return type · the EMPTY
`(wat.type/Tuple [])` unifying with a nil-returning body · and the FLAT form still reading.

**Non-vacuity control:** perturbing one inner member to `wat.type/Bogus` goes RED —
`":wat::core::Tuple: parameter #2 expects :wat::core::Bogus; got :wat::core::String"`. The bracketed
inner types are genuinely resolved and unified; the green is not free.

So the round-trip is safe and **the writer is the only side that changes.**

## Out of scope — affirmatively cut, with the reason

- **The reflection path still shows a nil return as an empty tuple.** `runtime.rs:13034` and
  `runtime.rs:14649` (`signature-of-defn`) share this renderer but receive an ALREADY-canonicalized
  `TypeExpr` from the stored scheme — there is no source keyword left to preserve, so half (a)
  cannot reach them. After this stone they render `(wat.type/Tuple [])` where they render
  `(wat.type/Tuple)` today. That is the pre-existing `nil ≡ unit` identity, not something this stone
  introduces, and closing it requires the substrate split named above — the builder's call.
  Measured: **no golden currently renders a nil return through that path** (`(wat.type/Tuple)`
  appears in exactly one golden, contract-07, which feeds `:()`).
- **Diagnostics still speak the retiring dialect.** `check::format_type` renders a tuple type in
  the KEYWORD spelling inside error messages — measured: `":user::one-ary: parameter #1 expects
  :(wat::core::i64,); got :wat::core::i64"`. That is a fifth surface, it is not this renderer, and
  after this stone a type the user wrote as `(wat.type/Tuple [wat.type/i64])` will still be quoted
  back at them as `:(wat::core::i64,)`. Out of ②-i-b's scope because it is a different function with
  a different call graph; it belongs with ③, whose whole subject is the checker writing a `remedy`
  in the surface spelling the user is required to use.
- **The corpus migration of the 243 tuple sites** is ②-iii's job, not this stone's. This stone
  unblocks it by making the codemod's guard stop firing.
