# NOTE (arc 109 vocabulary) — EDN-compliant generic brackets + the keyword-as-type coupling

**Filed 2026-06-04. A POINTER, not a decision.** Queue marker for a syntax stickler that
surfaces as we complete the "clojure-ination." No four-questions verdict locked yet — this
records the problem, the grounded current mechanism, the proposed direction, and (the load-bearing
part) **why it is coupled to killing keywords-as-types and probably should not be patched in
isolation.**

## The stickler

Parametric type arguments today are **comma-separated with NO whitespace**:

```
:wat::core::HashMap<wat::core::String,wat::core::i64>     ← legal
:wat::core::HashMap<wat::core::String, wat::core::i64>    ← LEX ERROR (space inside <>)
:wat::core::HashMap<wat::core::String wat::core::i64>     ← LEX ERROR (space, no comma)
```

This is **not EDN-compliant.** In EDN, commas **are** whitespace (insignificant); a reader splits
on whitespace and treats `,` as a no-op. Our brackets do the **inverse**: the comma is the
*required, significant* separator and actual whitespace is *forbidden*.

## The direction — pipe-separated: `<First|Second|Third>` (updated 2026-06-04)

The user has been playing with syntax that **parses in Clojure**, and the direction is
**`<First,Second,Third>` ⇒ `<First|Second|Third>`** — pipe-separated.

**Why pipe — and why NOT whitespace (correcting this note's earlier `<Foo Baz>` proposal):** the goal
is that the whole parametric type reads as **one Clojure token**. Clojure's *reader* treats both
whitespace **and** comma (EDN-whitespace) as token *separators* — so `Thing<Foo Baz>` *and*
`Thing<Foo,Baz>` each read as **two** tokens (`Thing<Foo` + `Baz>`), defeating "parses in Clojure."
`|` is a valid Clojure/EDN **symbol-constituent** character with no whitespace, so
`Thing<First|Second|Third>` reads as a **single** symbol/keyword. Pipe is the separator that keeps
the generic type *atomic to Clojure's reader* — exactly what the earlier whitespace idea missed
(whitespace is EDN-natural for *collections*, but it *splits* a *single-token* type).

**It likely lexes cleanly today.** `|` is not in `is_symbol_break` (lexer.rs:428), so
`:wat::core::HashMap<wat::core::String|wat::core::i64>` already lexes as one keyword token — no
whitespace, no `UnclosedBracketInKeyword`. The change is then mostly in the keyword-body *parser*
(read `|` as the arg separator instead of `,`) — a **smaller, more standalone** change than the
whitespace form (which needed the lexer to stop erroring on spaces). So the pipe direction is *less*
coupled to the keyword-as-type retirement than the whitespace idea was, though the broader cleanup
(below) still applies.

**One tension to weigh:** `|` conventionally reads as **union/or** in type syntax (`First | Second`
= "First or Second"). Here it is the **positional** arg separator (`HashMap<K|V>` = key-type `K`,
value-type `V` — *not* "K or V"). The pipe-as-separator choice trades that connotation for
Clojure-reader-atomicity; the deciding arc should weigh whether that's acceptable, or whether `|`
should be reserved for actual union types and a different glyph chosen for positional args.

## The current mechanism (grounded 2026-06-04)

The whole apparatus lives in **`src/lexer.rs` `lex_keyword` (line 605)** — because a parametric
type is crammed into a **single keyword token**, the lexer hand-rolls a mini bracket-parser to
keep `<...>` / `(...)` inside that one token:

- `angle_depth` / `paren_depth` tracked across the keyword body (lexer.rs:611, 627).
- `<` opens a type-head only when preceded by an alphanumeric (`Vec<`), so operator keywords
  `:wat::core::<` / `:wat::core::>=` (which follow `::`) don't false-open (lexer.rs:694–717).
- **Whitespace inside `<...>` or `(...)` is a HARD ERROR** — `LexErrorKind::UnclosedBracketInKeyword`
  (lexer.rs:632–636). This is *the* limit the user named.
- Comma at depth 0 in a keyword body is rejected (`CommaInKeywordBody`, lexer.rs:729–730); comma
  *inside* brackets falls through and is preserved — i.e. comma is the in-bracket separator.

**Reconciliation debt found while grounding:** the arc-072 comment at lexer.rs:612–615 claims
"(with the user's intuitive whitespace) `:Result<(i64,i64), i64>` … don't silently truncate at the
space" — but the current code at :632–636 **errors** on that space. The comment describes a
superseded behavior (push-the-space) that a later hard-error arm overrode. Stale arc-archaeology
(cf. the class arc 245 warded against); fix it in whichever arc touches this.

## Why this is coupled to killing keywords-as-types — and the sequencing call

The bracket-lexing complexity **exists only because the type is a keyword token.** If a type were
a single atomic lexeme, the reader would never need to peer inside it for `<`/`>`/`,`/whitespace.

The user has flagged that **keywords-as-types die in the near future** (types are currently
`:wat::core::Thing`; the clojure-ination moves away from that). That is the *bigger* structural
decision, and it likely **dissolves this stickler for free:**

- If parametric types become a normal **read structure** (a list/vector/`Tag`-style form that the
  EDN reader parses with ordinary whitespace + comma-as-whitespace rules), then `<Foo Baz>` reads
  like any other collection — **no `lex_keyword` bracket machinery at all**, and EDN-compliance is
  automatic, not a patch.
- Patching `lex_keyword` *now* to allow in-bracket whitespace would be **scaffolding we are about
  to delete** — the same "don't ward/build what you're about to rewrite" call that deferred
  `stream.wat` → lazy-seqs in arc 245.

**Recommended sequencing (for the deciding arc, not locked here):**
1. Decide the **keyword-as-type retirement** first — what a type lexeme/form *becomes*.
2. Let the generic-bracket syntax **fall out of** that representation (whitespace-separated EDN
   brackets should be a byproduct, not a separate lexer stone).
3. Even while types stay keyword tokens, the **pipe separator is a tractable standalone change** —
   `|` already lexes (not a symbol-break), so it's a keyword-body *parser* change (read `|` as the
   arg separator), not a lexer rewrite. This is the one piece that can land *before* the
   keyword-as-type retirement without being throwaway scaffolding — the parser survives the
   representation change. (Contrast the old whitespace idea, which needed a lexer rewrite that the
   retirement would then delete.)

## Open questions to resolve in the deciding arc

- What does a type lexeme become post-keyword? (symbol? a `#`-tagged read form? a dedicated
  type-expression grammar?) This determines whether brackets stay in the lexer at all.
- Separator: **`|` (pipe)** is the direction (it reads as one Clojure token; comma/whitespace split).
  Do we **retire** comma entirely (one canonical path — `<First|Second>`) or allow both during a
  transition? One-canonical-path → retire comma.
- The **`|`-as-union tension**: positional-pipe (`HashMap<K|V>`) vs reserving `|` for union types
  and choosing a different glyph for positional args. The deciding arc's call.
- The current comma-required form is **tolerated debt**, not the destination.

## Refs

- `src/lexer.rs` `lex_keyword` (:605); whitespace hard-error (:632–636); angle-open disambiguation
  (:694–717); comma handling (:729–730); stale arc-072 comment (:612–615).
- Sibling NOTEs (same queue-marker shape): `NOTE-type-decl-def-prefix-renames.md`,
  `NOTE-reconsider-atomize-materialize.md`.
- The dialect-faithfulness telos this serves: arc 247 (fn-first HOFs) + arc 249 (threading) —
  "be what you claim (Clojure/EDN); immediate knowability to a model that has seen it."

## ★ ADDENDUM 2026-06-06 — BUILDER DIRECTION: parametrics become FORMS (the stickler dissolves)

The builder has now internalized Typed Clojure's `ann-form` model — `(ann-form xs (t/Vec t/Num))`,
`(t/HashMap t/Str t/Int)`, `(t/All [x] ...)` — and the direction is **strongly set**: wat's
parametric/generic types move from string-encoded keywords to **ordinary read forms**:

```
:wat::core::HashMap<wat::core::String,wat::core::i64>    ;; today: ONE keyword token,
                                                          ;; hand-rolled bracket lexer
(wat.type/HashMap wat.type/String wat.type/i64)           ;; the direction: a FORM —
                                                          ;; whitespace separates, comma is
                                                          ;; EDN-whitespace, the reader is enough
```

**This DISSOLVES this note's entire question rather than answering it:**
- The separator debate (comma vs pipe vs whitespace) is MOOT — forms separate args by
  whitespace like every other form; EDN-compliance is automatic, not a patch.
- The `lex_keyword` bracket machinery (angle_depth/paren_depth tracking, the whitespace
  hard-error, the operator-`<` disambiguation, the comma rules — lexer.rs:605-730) DELETES
  WHOLESALE. The stale arc-072 comment dies with it.
- The pipe-separator interim (this note's step 3) is now scaffolding-about-to-be-deleted —
  per this note's own doctrine, likely SKIP it and go straight to forms in the deciding arc.

**Live evidence from arcs 249/245 (the week this clicked) that keyword-encoded parametrics
are the wrong representation:**
1. `keyword/of` (wat/core.wat) constructs parametric types by STRING CONCATENATION —
   `(string::concat head-text (string::concat "<" (string::concat joined ">")))`. Building
   types out of strings inside a homoiconic language is the smell at its purest. With
   types-as-forms, keyword/of's whole job becomes `(wat.type/Head arg1 arg2)` — a quasiquote
   template, no strings.
2. The run-threads macros DESTRUCTURE parametric types via `Bundle/children` + positional
   `get` + string round-trips (`keyword/to-string` → concat → `keyword/from-string`) just to
   extract `I`/`O` from `ThreadPeer<I,O>`. With types-as-forms, that is `first`/`rest`/`get`
   over an ordinary form — the total-pure macro engine's NATIVE vocabulary.
3. The deepest win: **types join homoiconicity.** The 249 engine made macro bodies total-pure
   programs over forms; if types ARE forms, the same engine computes over types with the same
   blessed combinators — type-level macro tooling (the reusable-tooling thread, 2026-06-06)
   for free, no reflection verbs doing string surgery.

**Status upgrade:** was "a POINTER, not a decision." Now: **builder-directed DESTINATION**
(types-as-forms for parametrics/generics), still owed its deciding arc + four-questions at
strike for the concrete grammar (see the sibling NOTE's three coupled moves + ann-form as the
local-ascription precedent). The sequencing recommendation stands and sharpens: the
keyword-as-type retirement and the forms representation are ONE decision now, not two.
