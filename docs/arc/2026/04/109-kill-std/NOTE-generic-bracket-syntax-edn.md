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
*required, significant* separator and actual whitespace is *forbidden*. The EDN-natural form —
`SomeThing<Foo Baz>` (whitespace-separated, comma optional) — is exactly the form the lexer
rejects.

The user's framing: **`SomeThing<Foo,Baz>` ⇒ `SomeThing<Foo Baz>`** — let the angle brackets read
like every other EDN collection.

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
3. Only if step 1 keeps types as keyword tokens for the foreseeable future do we treat
   "allow whitespace in `<>`" as a standalone `lex_keyword` change (push whitespace at
   `angle_depth/paren_depth > 0` instead of erroring; teach the keyword-body splitter that comma ≡
   whitespace).

## Open questions to resolve in the deciding arc

- What does a type lexeme become post-keyword? (symbol? a `#`-tagged read form? a dedicated
  type-expression grammar?) This determines whether brackets stay in the lexer at all.
- EDN says comma ≡ whitespace. Do we make comma **optional** (`<Foo Baz>` and `<Foo, Baz>` both
  read identically), or **retire** the comma in type args entirely (one canonical path — the
  whitespace form)? Lean: optional-but-discouraged now, lintable later (mirrors the threading
  depth≥2 convention, arc 249), or retire outright per one-canonical-path.
- The "solutions we have" for the current non-compliance are **comma-required workarounds**, not
  the EDN-true form — they are tolerated debt, not the destination.

## Refs

- `src/lexer.rs` `lex_keyword` (:605); whitespace hard-error (:632–636); angle-open disambiguation
  (:694–717); comma handling (:729–730); stale arc-072 comment (:612–615).
- Sibling NOTEs (same queue-marker shape): `NOTE-type-decl-def-prefix-renames.md`,
  `NOTE-reconsider-atomize-materialize.md`.
- The dialect-faithfulness telos this serves: arc 247 (fn-first HOFs) + arc 249 (threading) —
  "be what you claim (Clojure/EDN); immediate knowability to a model that has seen it."
