# DESIGN — STONE: `edn::write` emits keywords `edn::read` refuses. Two renderers; one is wrong.

> **Builder, 2026-09-05:** *"wut... can we fix this now?...."*

## THE PROOF — it ships, and it is frozen in the corpus

```
(:wat::edn::write :wat::holon::Hologram/make)   ->  ":wat.holon/Hologram/make"      TWO slashes
(:wat::edn::read  ":wat.core/__internal/type-decl")
  ->  EDN parse error: invalid keyword: more than one / in wat.core/__internal/type-decl
```

**Five committed golden files already hold such keywords** — output the substrate's own reader
cannot parse:

```
tests/reflection/wat_arc144_uniform_reflection__type_defstruct.edn
tests/reflection/wat_arc144_uniform_reflection__special_form.edn
tests/reflection/wat_arc144_uniform_reflection__primitive_empty.edn
tests/wat_lang/wat_arc144_hardcoded_primitives__length_primitive.edn
tests/wat_lang/wat_arc144_lookup_form__struct_head.edn
```

**82 registered names** carry the `Type/method` shape (`grep -cP '::[A-Z][A-Za-z0-9]*/'`), so the
population is not the five goldens — it is every one of those names whenever it crosses the wire.

## THE ROOT — a validity check that does not fire

`src/edn/render.rs:4154`, `keyword_from_wat_path` — the arm `value_to_edn_with` uses for every
keyword VALUE:

```rust
let ns   = identifier::path(stripped).replace("::", ".");   // "wat.holon"
let name = identifier::leaf(stripped);                       // "Hologram/make"  ← slash retained
Keyword::try_ns(&ns, name)                                   // SUCCEEDS
```

`Keyword::try_ns` does not reject a name containing `/`, so it mints a keyword EDN cannot spell.

Meanwhile `src/edn/render.rs:3264`, `wat_keyword_to_clojure_symbol`, does it **correctly** —
folding `Type` into the namespace when the final segment has a receiver:

```
:wat::holon::Hologram/make  ->  wat.holon.Hologram/make      ONE slash. Reads back.
```

**Two renderers for one question, and the value path uses the wrong one.**

## ★ AND THE COMMENT ABOVE THE BUG IS THE LESSON

`keyword_from_wat_path`'s `Err(_)` arm carries a measured census:

> *"Measured over the corpus, this arm fires for 10 distinct keywords out of 72,510 — all of them
> trailing-`::` namespace-prefix markers … whose EDN name is empty."*

That census is **true and blind**. It counted what `try_ns` REJECTS. The `Type/method` case is not
rejected — it is *accepted and wrong* — so it could never appear in that number. **A census built
on a check that does not fire under-reports exactly the class the check misses**, and the author
reasonably concluded the arm was well understood. This is
`[[feedback_a_pass_answers_only_the_question_the_instrument_asks]]`, in a comment that reads as
diligence.

## THE FIX — two parts, and the second is the one that matters

**Part 1 — one renderer.** `keyword_from_wat_path` folds `Type/method` the way
`wat_keyword_to_clojure_symbol` already does. Share the implementation; do not re-derive it. Output
becomes readable, and the five goldens are regenerated — **from unreadable to readable, which is a
fix, not churn.**

**Part 2 — the wall.** `Keyword::try_ns` REFUSES a name containing `/`. Then the class cannot recur
silently: a future caller that forgets to fold gets an `Err`, and `keyword_from_wat_path`'s existing
`Err` arm already carries such a keyword verbatim rather than lying about its type.

| | Obvious? | Simple? | Honest? | Good UX? |
|---|:---:|:---:|:---:|:---:|
| **Part 1 alone** | YES | YES | **NO** | YES |
| **Part 1 + Part 2** | YES | YES | YES | YES |

**Part 1 alone fails Honest** — it fixes the one caller and leaves the constructor still able to
mint an unspellable keyword for the next one. The bug was *possible* because `try_ns` permits the
state; removing only the instance is patching the stem. Part 2 is the root.

⚠ **Part 2 touches `wat-edn`.** The docstring stone's B2 forbade changing `wat-edn`'s *writer* to
serve one consumer's formatting. This is a different act: making a CONSTRUCTOR reject a value the
format cannot represent. It narrows what can be built, it does not change how anything is written,
and every currently-VALID keyword is unaffected. If it turns out any live caller depends on
constructing a slash-bearing name, that is a finding and Part 2 stops.

## SCOPE

**In:** `keyword_from_wat_path` · `Keyword::try_ns`'s validity rule · the five goldens, regenerated
· a witness proving `write` → `read` round-trips for a `Type/method` name.

**Out:** the `Type::method` five (`Bytes::to-hex`, `HandlePool::{new,pop,finish}`) — they fold to
the same EDN as a `/` method and are a SEPARATE ambiguity, already recorded in
`[[SCORE-STONE-the-printer-and-the-round-trip-gate]]`. This stone makes the wire *readable*; it does
not make it *unambiguous*. Say so plainly rather than implying the wire is now total.

---

## ⛔ AMENDED 2026-09-05 — "UPPERCASE MEANS TYPE" IS NOT A RULE THIS LANGUAGE HAS

> **Builder:** *"uppercase is not a type declaration…. `:wat::core::i64` is a type…. it becomes
> `wat.type/i64` later... once we prepare for the clojure/edn syntax cutover."*

The record backs it. `[[251-types-as-forms/DESIGN-STONE-251.2]]`: *"bridges `:wat::core::T` (old) ·
`:wat::type::T` (new FQDN) · **`wat.type/T` (surface symbol)**"*, with
`ns_to_wat_path("wat.type","i64")` = `:wat::type::i64`. **251.5 — the internal canonical flip
`:wat::core::` → `:wat::type::` — is designed and unshipped.** The cutover is real and parked, not
hypothetical.

**Two things follow, and one is a retraction.**

**1 — A proposed lint is RETRACTED before it was written.** In discussion I recommended renaming the
five `Type::lowercase` names and then forbidding that shape with a lint. That rests entirely on
"uppercase marks a type", which is false. **It would have frozen a discriminator the language does
not have into a wall** — worse than no wall, because a wall carries authority. It never reached a
brief; it is recorded here so it cannot be re-proposed.
`[[feedback_i_cited_a_rule_instead_of_measuring_whether_it_applied]]`

**2 — `fqdn_of`'s rationale in `crates/wat-macros/src/edn_doc.rs` is FALSE, and its answers are
RIGHT.** Committed at `0582f1919`, it reads: *"a method name does not start uppercase, a type (and
an enum variant) does."* That is not the rule. What the code actually keys on is **the last
NAMESPACE segment being uppercase** — which correlates with record types (`Hologram`, `Bytes`,
`HandlePool`) and never touches `i64`/`String`, because those sit in the NAME position, not the
namespace. So every answer it gives today is correct and its stated reason is not, and no test can
catch that. **Ninth comment-caused defect of this campaign; the cure is the same as the other
eight — say what it keys on, and name what it cannot see.** That becomes Part 3.

## ★ AND THE HARD HALF DISSOLVES RATHER THAN BEING SOLVED

The reverse transform (`:wat.holon.Hologram/make` → `/method` or `::method`?) is ambiguous **only
while wat and EDN are different syntaxes**. After 251's cutover, `wat.type/i64` IS the wat surface —
there is no reverse transform, because there is nothing to translate back to.

⛔ **So the reverse discriminator must NOT be solved here.** Any heuristic built for it is machinery
whose whole purpose is to be deleted, and a wall built on it would outlive its own premise. This
stone makes the wire **readable**. 251 makes the question **not exist**.

## PART 3 — the comment (added by this amendment)

Correct `fqdn_of`'s doc to state the rule it actually implements (last namespace segment uppercase +
name not uppercase ⇒ it was a `/` method) and to name its limit plainly: a record type spelled
lowercase, or a method spelled uppercase, defeats it — and that limit is acceptable **only because
251's cutover retires the reverse direction entirely.** Cite 251 so the next reader finds the exit
rather than hardening the heuristic.
