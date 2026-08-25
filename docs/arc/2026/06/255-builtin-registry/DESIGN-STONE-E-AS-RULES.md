# DESIGN — STONE E, as RULES: the rename is a structural question

> Supersedes the codemod half of `DESIGN-STONE-E-the-string-home.md`. That stone's SCOPE, its
> door table, its ruling on the rete mirror, and its corrections all stand. What changes is HOW the
> `.wat` corpus moves — and the char-walk prerequisite stone drawn against `wat/fix.wat` is
> **withdrawn before it was briefed**, because the gap it patches stops existing.

## Why this replaces the char-walk fix

Builder: *"the rule… should be made into rules? i wanted wat-fix to be as much rete as we could
be… the reasoning for if something is observed."* And: *"the if/else chaining IS ALWAYS BRITTLE."*

The boundary apparatus in `rename-valid-match?` is not a rule that needs widening. **It is
compensation for not having a parse.** `fix.wat` walks raw text, so it must discover where a name
begins and ends — hence a left-boundary rule, a right-boundary rule, an ident-char alphabet, and a
char-walk. The fact base already knows: the reader tokenized the file, and `ast-name` hands back
`":wat::core::string::length"` as **one whole token**.

Four things therefore do not shrink — they disappear:

| what | why it is gone |
|---|---|
| left boundary | the fact IS a whole name; there is no "middle of a name" to guard |
| right boundary | the remainder is what we WANT; it was never a real question |
| type-arg reach | a type arg is a `Named` node like any other — free, not special-cased |
| comment-faithfulness | prose is not a node, so a rule CANNOT touch it, by construction |

★ **And the silent no-op becomes unrepresentable.** A char-walk that matches nothing returns its
input and reports `[renamed]` — which is how this stone nearly shipped as a corpus-wide migration
that moved zero bytes. **A rule that matches nothing produces no `Match` facts, and you can COUNT
them before applying anything.** The dry run stops being "run it and diff 1559 files"; it is a query
with a number.

## MEASURED — the whole composition, end to end

Run this session against `wat/string.wat`, verbatim:

```
old :wat::core::string::capitalize  →  new :wat::string::capitalize    span 17:19 → 17:49
old :wat::core::string::length      →  new :wat::string::length        span 20:35 → 20:61
old :wat::core::string::concat      →  new :wat::string::concat        span 22:6  → 22:32
```

**15 sites found. The char-walk tool found 0.** And 15 is exactly the count the rider measured by
hand as CODE occurrences in that file — 22 total minus 7 in comments. Two independent instruments,
one structural and one manual, agreeing on a population the third instrument could not see at all.

**The rule computes its own replacement.** It does not merely locate a site:

```clojure
(:wat::rete::defrule :rr::rename
  :when [(:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::core::String/starts-with? ?n ":wat::core::string::"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "core-string-to-string"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)
                       (:wat::grep::Capture :name "new"
                         :value (:wat::rete::core::String/concat ":wat::string::"
                                  (:wat::rete::core::string::subs ?n 20
                                    (:wat::rete::core::string::length ?n)
                                    :undefined "")))))])
```

⚠ **`subs` is a FALLBACK op and takes FIVE args** — `(s start end :undefined <default>)`. The
mandatory undefined-point is what makes a partial op total on the rete surface
(`vocabulary.rs:1333`). Omitting it fails at `compile-all` with *"wants 5 args"*, not at `--check`.

## THE ADAPTER — and it is the only new machinery

A `Match` already carries what an edit needs. `fix-text-apply` already takes edits. They were built
to meet at this interface by two different authors who never coordinated:

```
Match{line,col,end-line,end-col} → offset  = (fix-text-offset-of {:line :col} lines)
                                   old-len = (fix-text-span-len start end lines)
capture "new"                    → replacement
                                 → (Tuple offset old-len replacement)  → fix-text-apply
```

`fix-text-apply` / `fix-text-offset-of` / `fix-text-span-len` are **untouched**. They are the honest
half of `fix.wat` — span→string surgery, applied right-to-left so offsets stay stable — and they have
never lied. This stone does not make edit application a logical problem, only edit FINDING.

## ⛔ E CANNOT BE DIFFERENTIALLY VERIFIED, AND THAT IS THE POINT

The project's method is annealing: the old implementation's last job is to be the ORACLE for its
replacement — `native_agrees_with_the_oracle_*`, shipped in today's merge, is that method named.
**E cannot use it.** Its oracle is the char-walk, and the char-walk is a total no-op for this shape.
There is nothing to agree with.

So E is verified by **construction and controls** instead, and the controls must be stated because
the usual one is unavailable:

- the 15-vs-15 agreement above (structural finder vs manual count), already in hand
- idempotence as a QUERY: after applying, re-run the finder and get **0 matches**
- the type is untouched: `:wat::core::String` count identical before and after
- the build and the floor

**The differential discipline applies to the OTHER 30 recorded migrations**, whose char-walk
versions DO work and can serve as oracles when they are upgraded. E is the exception, and it is the
exception precisely because its oracle is broken — which is why it is also the honest place to prove
the new mechanism.

## ACCEPTANCE

1. **The finder reports the expected population** — the two rules over the whole corpus, counted,
   before anything is written. Compare against `:wat::core::string::` occurrences excluding comment
   lines. A discrepancy is a finding, not a rounding.
2. **Dry run: apply to a `/tmp` copy and `diff`.** Byte-level, not "it said renamed" — the failure
   this stone exists to prevent.
3. **Idempotent as a query** — after applying, the finder returns **0**.
4. ★ **The TYPE is untouched.** `:wat::core::String` identical before and after. The old stone's
   central control, and it survives the mechanism change because it is about the population, not
   the tool. Capture the number BEFORE starting, on a quiescent tree.
5. **The seven Rust doors** — per-door, per-OCCURRENCE (not per-line; the old table's ambiguity
   fired a STOP). `purity.rs` 5, `wat/string.wat` 22, `runtime.rs` 44, `check.rs` 31,
   `macros/eval.rs` 18, `expr_ir.rs` 10, `vocabulary.rs` 8.
6. **The rete mirror moved**, and its wall fires when broken deliberately (`vocabulary.rs:1565`).
7. **`(:wat::deporder::verify-stdlib)` → `[]`**; floor green accounted BY NAME; clippy 0.

## OUT OF SCOPE — affirmatively cut

- **The `wat/fix.wat` char-walk prerequisite** — WITHDRAWN, undrawn, unbriefed. Its gap stops
  mattering. The char-walk stays alive underneath the 30 recorded migrations until they are
  upgraded; nothing forces that now.
- **Upgrading the other 30 migrations.** They are the reasoning corpus the builder wants, and the
  ones worth writing carefully are the NON-renames (`strip-arrow-ascription`, `wrap-calls-in-match`,
  `first-of-drop-to-nth`, `fix-macro-param-types`) — thirty near-identical prefix renames teach one
  pattern thirty times. Their own arc.
- **Prose.** 7 comment occurrences in `wat/string.wat` cannot be reached by a fact-based finder, by
  construction. Whether the corpus's comments are migrated is a scope question with its own answer,
  not a tool problem.
- **wat-lint / wat-fmt inheriting the finding half.** Real, next, and out of this stone — though
  note `wat/lint.wat:50`'s `Finding [rule file line col severity message fix]` is already a `Match`
  plus three fields, and its auto-fix seam is blocked by a claim (`lint.wat:8`) that arc 281 made
  false.
