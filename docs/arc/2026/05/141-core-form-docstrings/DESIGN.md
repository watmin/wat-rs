# Arc 141 — Core form docstrings

**Status:** DESIGN settled 2026-05-03 — implementation deferred.

This arc closes as DESIGN-only before arc 109 is marked resolved
(same shape as arc 118 lazy-seqs). The decision is locked:
**docstrings are first-class citizens in six core forms** —
`define`, `lambda`, `defmacro`, `typealias`, `struct`, `enum`.
Backwards-compatible (existing forms without docstrings still
valid). Implementation work happens in a future session, alongside
or driving the eventual `wat-doc` consumer crate.

**Scope of THIS arc:** the SUBSTRATE change only. The `wat-doc`
crate (a documentation generator that consumes the docstring slot)
is captured in `scratch/2026/05/006-wat-doc/` and will get its own
arc when it ships. Arc 141 unblocks that future arc by minting the
slot in the substrate.

## Why first-class, not comments-above

User direction (2026-05-03):

> *"we could just add doc strings as first class citizens in our
> forms.... that's probably the honest thing to do?...."*
>
> *"i totally agree with this... we'll update our core forms as
> we approach this... we'll have a linter flag when you didn't
> declare a doc string... this is an excellent arc to record"*

Rust takes the comments-above path (`/// docstring` parsed by
rustdoc; tooling-convention rather than language feature).
Clojure takes the strings-in-form path (`(defn name "doc" [args]
body)`; the docstring is part of the form's structure).

Wat is a Lisp; morphology-over-position; the form carries its own
documentation as substrate-guaranteed structure, not tooling
convention. The Clojure path is the honest move:
- The docstring is part of the form's IDENTITY; same hash discipline.
- Tooling never has to guess "is this comment for this form or the
  one above?"
- The form's signature self-documents its public contract.
- Comments retain their separate role — author's inner monologue
  (see Convention section below).

## The convention — comments vs docstrings

Two channels, two audiences, complementary:

| Channel | Audience | Purpose |
|---|---|---|
| `;;` comments | author / future-author | inner monologue — *"this is what I was thinking when I made this"* |
| docstrings | consumer | instructions — *"this is how to use this thing"* |

The discipline this convention encodes:
- Comments are first-person internal ("rejected approach Z because
  W"; "TODO: revisit when N").
- Docstrings are second-person external ("call this with X to get
  Y"; "raises Err on empty input"; "see also `:other-fn`").
- Neither replaces the other; they serve different readers.
- A form may have just comments, just a docstring, or both — they
  don't compete.

The substrate doesn't differentiate semantically (both are valid
wat). The convention is enforced by tooling (eventual `wat-doc`
reads docstrings only; eventual `wat-lint` flags missing
docstrings on function-shaped public forms) and by the discipline
articulated here.

## The substrate change

**Six forms get an optional docstring slot:**

### Function-shaped — docstring between signature and body

```scheme
;; define — old shape (still valid; backwards-compat)
(:wat::core::define (sig) body)

;; define — new shape with docstring
(:wat::core::define (sig) "Doc string." body)

;; lambda — same pattern
(:wat::core::lambda (sig) "Doc string." body)

;; defmacro — same pattern
(:wat::core::defmacro (sig) "Doc string." body)
```

Concrete example:
```scheme
(:wat::core::define
  (:my::helper (x :wat::core::i64) -> :wat::core::i64)
  "Compute the thing. First sentence is the summary.

   Long-form explanation continues across lines per Rule 31."
  (:wat::core::i64::* x x))
```

### Type-shaped — docstring trails the structural body

These forms don't have a "body" position separate from their
structure; the docstring follows:

```scheme
;; typealias with docstring (justifies the alias's existence)
(:wat::core::typealias :MyMap
  :wat::core::HashMap<:wat::core::Symbol, :wat::core::i64>
  "A map keyed on symbol; values are signed counters.
   Used in the metrics layer; never construct directly —
   call :wat::metrics::make-bucket instead.")

;; struct with docstring (explains the type's domain)
(:wat::core::struct :Position
  ((file :wat::core::i64)
   (rank :wat::core::i64))
  "A position on a chess board. file ∈ [0, 7]; rank ∈ [0, 7].
   Origin (0, 0) is a1; (7, 7) is h8.")

;; enum with docstring (explains the variant set's purpose)
(:wat::core::enum :ParseResult<:T>
  ((Ok (value :T))
   (Err (error :ParseError)))
  "Parser output. Ok carries the parsed value; Err carries
   diagnostics with location information.")
```

### Type-checker recognition rule

For each form, the docstring slot is OPTIONAL. The type checker:

- **Function-shaped** — if 3 args, args[1] must be
  `:wat::core::String`. If 2 args, no docstring (body is args[1]).
  3 args + non-String middle → type error with span.
- **Type-shaped** — trailing String is the docstring; absent
  trailing String means no docstring.
- **Runtime** — docstring is metadata; the evaluator ignores it.
  Body evaluation unchanged. No runtime semantics impact.

### Why between signature and body (function-shaped)

Three reasons:
1. **Signature stays purely about types.** Cleaner separation
   between "what the function takes/returns" and "what it does."
2. **Lambda symmetry.** Lambda has no name in the signature;
   placing the docstring outside the signature works identically
   for define/lambda/defmacro. Inside the signature, lambda would
   have an awkward "first arg vs first docstring" ambiguity.
3. **The docstring documents the WHOLE form, not just the
   signature.** It belongs adjacent to both — between sig and body
   — rather than nested inside one or the other.

### Asymmetry — function vs type lint pressure

When the eventual `wat-lint` ships:
- Function-shaped forms (`define` / `lambda` / `defmacro`) get
  LINT-FLAGGED for missing docstrings (high signal — functions DO
  things; document what they do). Default L1.
- Type-shaped forms (`typealias` / `struct` / `enum`) get the slot
  but no lint pressure (lower signal — types NAME things; the
  name often suffices). The docstring is opt-in; useful when a
  typealias justifies its existence or a struct's invariants need
  explanation.

Arc 141 mints the slot for ALL six. Lint asymmetry is downstream
(wat-lint arc).

## The four questions

| Question | Answer |
|---|---|
| Obvious? | ✅ — strings-in-form is the Lisp idiom; the slot is between sig and body where readers expect documentation |
| Simple? | ✅ — three simple parser changes (one per function-shaped form) + three trailing-string parses (per type-shaped form); no new types; no runtime semantics; no Display changes |
| Honest? | ✅✅ — the form carries its own documentation as structure; tooling never has to guess "is this comment for this form?"; comments retain their separate role |
| Good UX? | ✅ — docstrings travel with the form through any AST walker, hashing, signing; same shape across all six forms; backwards-compat preserves every existing wat file |

## Estimated implementation surface

Slice 1 (the substrate change) — citing the scratch SLICE-PLAN's
"~100-200 LOC of Rust changes":

- **`src/runtime.rs::parse_define_form`** — accept 3 elements (sig
  + docstring + body) in addition to the existing 2 elements;
  validate the middle is `WatAST::StringLit`. Same pattern at
  `parse_lambda_form` (currently in `eval_lambda` body) and
  `register_defmacros` (in `src/macros.rs`).
- **`src/types.rs::register_types`** — typealias / struct / enum
  registration accepts an optional trailing String literal; store
  alongside the type def (or simply ignore at the substrate level
  if no consumer surfaces yet — the wat-doc crate consumes it).
- **`src/check.rs`** — type-error case for "3-arg function form
  with non-String middle" surfaces a clear message naming the
  expected shape. Span carries via arc 138's discipline (call_span
  + the offending element's span).
- **Backwards-compat** — every existing wat-rs `.wat` file parses
  unchanged. Test sweep: `cargo test --release --workspace` exit=0.

Future slices (NOT this arc):
- The wat-doc consumer crate (per scratch design).
- wat-lint missing-docstring rule.
- STYLE-RULES.md updates (post-wat-fmt).
- Migration sweep: substrate stdlib + wat-rs source picks up
  docstrings on public forms.

## Cross-references

- **Scratch arc** at `scratch/2026/05/006-wat-doc/` — full
  architectural capture for the wat-doc crate. Read `README.md` +
  `DESIGN.md` for the consumer side; `SLICE-PLAN.md` for the
  4-slice trajectory (substrate change is slice 1).
- **Arc 118** at `wat-rs/docs/arc/2026/04/118-lazy-seqs-vs-threaded-streams/`
  — sibling DESIGN-only arc, same status shape ("settled, deferred,
  before 109 closure").
- **Arc 138** at `wat-rs/docs/arc/2026/05/138-checkerror-spans/` —
  the spans-on-errors discipline this arc inherits when shipping
  the type-checker error for non-String docstring.
- **Arc 109** — § B (`Vec<T>` rename) and § D' (Option/Result method
  forms) both touched the parser's form-grammar; arc 141 extends
  the same area without breaking those rules.
- **Clojure codox** — the cleanest Lisp-family doc generator;
  established the strings-in-form precedent the wat-doc crate
  draws from.

## What this arc does NOT decide

- The wat-doc crate's specific output formats (HTML / Markdown /
  EDN / JSON) — captured in scratch DESIGN; refined later.
- Multi-line docstring rendering rules (STYLE-RULES amendments) —
  refined when wat-fmt sees doc-bearing fixtures.
- Cross-reference resolution mechanics in docstrings — wat-doc
  crate's concern.
- Code-example syntax inside docstrings — refined alongside
  wat-doctest if that future arc opens.
- Whether docstrings flow into the substrate's AST hash — TBD;
  default assumption is YES (substrate hash is structural; the
  docstring IS structure), but this is a real architectural
  question to revisit at implementation time.

## What's earned to ship now

Per proposal 058's discipline (ship only what's earned by cited
use): nothing. The cited use is `wat-doc`, which doesn't exist yet.

Per the user's direction: open the arc record now so the design is
durable across sessions; ship the substrate change when wat-doc
opens. Arc 141 sits parked alongside arc 118 as a settled
substrate change waiting for its consumer.

## Sequence

```
Arc 141 OPEN as DESIGN (this arc) — locks the substrate change
Arc 109 closes (eventually) — its checklist sees 141 marked DESIGN
wat-fmt arc opens — STYLE-RULES.md sees the docstring slot referenced
wat-doc arc opens — consumes the docstring slot (slice 1 of scratch
                    plan IS arc 141's implementation; slices 2-4 are
                    pure consumer crate work)
wat-lint arc opens — adds missing-docstring rule on function-shaped
                     public forms (default L1; "annoying")
Migration sweep — substrate stdlib + wat-rs adds docstrings to
                  public forms; lint pressure drives adoption
```

## Why this lands as DESIGN now, not as code

- The substrate change is small (~100-200 LOC) but has process
  overhead (parser changes + type-checker validation +
  backwards-compat sweep).
- The cited consumer (`wat-doc`) doesn't exist yet; without it,
  the docstring slot is dead weight in the substrate.
- Arc 109's closure wants the design captured (parallel to arc
  118's role); the implementation can land alongside wat-doc when
  that arc opens.
- This arc unblocks: any future session that decides to ship
  wat-doc has the substrate change pre-designed and pre-bargained
  with the four questions.
