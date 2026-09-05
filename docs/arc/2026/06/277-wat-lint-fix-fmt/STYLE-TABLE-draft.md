# STYLE TABLE — draft. **EVERY UNRULED ROW IS MINE AND MEANT TO BE SHOT AT.**

> Builder: *"let's just work through the rules ..... we need stuff to critique.... i have no doubt
> that we can easily eye ball good from bad"*

Status column: **RULED** = the builder said it. **MINE** = my proposal, no authority, argue with it.
**OPEN** = I could not pick a side and the reason is stated.

---

## R1 · `defn` — **RULED** 2026-09-05

```
(wat.core/defn user/some-fn :- [I O]   ;; head + name + PARAM-SPEC on line 1
  [x :- wat.type/i64                   ;; ARG-SPEC own line(s), one arg per line,
   y :- wat.type/i64]                  ;;   continuations aligned under the first
  :- wat.type/i64                      ;; RET-TYPE own line
  (wat.core/+ x y))                    ;; BODY own line
```
Empty `[]` is **not** an exception — own line like any other arg-spec.
Blast radius ~4,202 `defn` sites. Practiced already at `wat/bracket.wat:32`.

---

## ⛔ R2 · `fn` — **OPEN, AND IT IS THE FIRST REAL FIGHT**

R1 applied verbatim to `fn` turns this — the single most common lambda shape in the corpus —

```
(:wat::core::fn [acc <- :wat::core::i64  x <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::+ acc x))
```

into this:

```
(:wat::core::fn
  [acc <- :wat::core::i64
   x   <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::i64::+ acc x))
```

**1,106 lambdas**, and `fn` is already the most contested form in the language (32 styles, no
shape above 37%). Most of them are one-expression reducers living inside a `foldl` argument list,
where four lines of ceremony wrap one line of work.

Three positions, and I do not think it is mine to pick:
- **(a) `fn` follows R1 exactly.** One rule, no exceptions to remember; the formatter is dumber
  and the reader always knows where the return type is.
- **(b) `fn` gets a budget.** Rides one line if it fits, breaks to R1's shape when it does not.
  Introduces the first WIDTH rule in the language, and therefore a number to argue about.
- **(c) `fn` is a distinct form with its own rule** — arg-spec stays on the head line, only the
  body breaks. Preserves what the corpus does; costs an exception.

⚠ Whichever wins decides something bigger: **whether this style system has a width budget at all.**
R1 has none — it is purely structural. (b) would be the camel's nose.

---

## R3 · `let` — **MINE**: binding vector on its own line, one binding pair per line

```
(:wat::core::let
  [m2   (:wat::hashmap::assoc m :bar 99)
   name (:wat::core::first xs)]
  (:wat::hashmap::length m2))
```
**Why:** a `let` binding vector is the exact analogue of an arg-spec — a bracketed list of
name/value pairs — so R1's treatment transfers with no new idea. The corpus is already 56% here.
⚠ **Weakest point:** 36% put the vector on the head line (`(let [x 1]` …), which is genuinely
tighter for a single binding. If R2 lands on a budget, this probably inherits it.

## R4 · `match` — **MINE**: scrutinee on the head line, ONE ARM PER LINE

```
(:wat::core::match msg
  ((:user::Op::Compute n) (:user::compute n))
  ((:user::Op::Halt)      nil))
```
**Why:** the scrutinee is not an arm and reads as part of the question. Arms are peers and get
peer treatment. Corpus is already 63% here. **88 distinct styles — the second most fragmented
form — so this rule earns its keep more than most.**
⚠ Unresolved: what happens when ONE arm's body is long. Sub-rule needed; a rider is looking.

## R5 · `if` — **MINE**: condition on the head line, then-branch and else-branch one per line
```
(:wat::core::if (:wat::kernel::stopped?)
  nil
  (:demo::loop))
```
**Why:** exactly R4's shape (a scrutinee and its peers) and the corpus is 76% here already. The
two branches must be visually parallel — that is the whole readability of an `if`.

## R6 · `do` — **MINE**: one form per line, always
89% already, 5 styles. Nearly settled; the rule just ratchets it.

---

## R7 · keyword arguments — **MINE**: one `:key value` PAIR per line when the form breaks

```
(:wat::grep::Match
  :file     ?f
  :line     ?l
  :end-line ?el
  :rule     "layout")
```
**Why:** a keyword-arg call is a record of named slots, and a named slot is a conceptual unit —
R1's own principle. **And the values want a column**, which is the same instinct as your
*"comments are aligned"*.
⚠ This is the rule most likely to be wrong in its details, and a rider is sampling it now.

## R8 · trailing comment alignment — **RULED in spirit** (*"comments are aligned"*), UNSPECIFIED in detail
Aligned to which column — the widest line in the block? a fixed stop? and what is a "block"?
⛔ **Cannot be built yet regardless**: comments only became visible to the reader this week
(`lex_with_comments`, 9a16b68e6) and `wat/grep.wat` does not assert a comment fact at all.

## R9 · no trailing whitespace — **MINE**, and it should be uncontroversial
Real instance: `tests/comms/probe_arc209_bound_listener.wat:20` ends `(:wat::core::match msg `.
Not a layout shape; a hygiene rule that costs nothing.

## R10 · indent unit — **MINE**: 2 spaces, and indentation is STRUCTURAL, never alignment-to-a-paren
```
BAD   (:wat::core::fn [acc <- …  i <- …]
                      -> …               ← indent depends on the length of "fn "
        body)
GOOD  (:wat::core::fn [acc <- …  i <- …]
        -> …
        body)
```
**Why:** an indent that depends on a name's length re-indents the whole form when the name
changes — the diff noise that makes a formatter worth having. Real instance of the bad form:
`tests/rete/probe_arc278_concurrent_retes.wat:90` (indents at +1, +15, +18).
★ I believe this is the single highest-value rule after R1, because it is the one that makes
every other rule's output STABLE under renaming.

---

## ⛔ WHAT NO RULE HERE CAN YET EXPRESS

- **Alignment of any kind** — R1's aligned binders, R7's aligned values, R8's aligned comments.
  The layout probe records a child's indent COLUMN; it cannot see siblings AGREEING on a column.
  Three ruled-or-proposed rules rest on a fact nothing emits yet.
- **Comments** — see R8.
- **Whether a width budget exists at all** — R2 decides it.

---

# RIDER FINDINGS — 4 riders, 4 form families, 2026-09-05. Verified against the disk.

## ⭐⭐⭐ R11 · SIBLING BREAKING IS ALL-OR-NOTHING — **MINE, and it is the most valuable rule here**

**The corpus's monster lines are not caused by width. They are caused by half-broken sibling sets.**

`tests/services/probe_arc170_m1_teeth_revoked.wat:95` — **1,096 columns, verified**. It is a `match`
whose INNER arms were each given their own line and whose OUTER arms were then appended to the tail
of the last inner one, closing parens and all:

```
…(:wat::kernel::assertion-failed! "unexpected RequestMalformed" …)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::…
                                                                   ↑ four closes, then the NEXT ARM, same line
```

Whoever formatted it broke the inner `match` and never came back to the outer one. The rider found
the same shape at `:78` (800 cols), `probe_arc170_m1_teeth_admitted.wat:67` (778), and in ~6 more
arc-170 probes — copy-pasted, so it is ONE authoring mistake replicated, not six judgements.

**The rule:** if ANY sibling in a form's child list is on its own line, EVERY sibling is. Arms of one
`match`, branches of one `if`, pairs of one `let`, fields of one record. No form is half-broken.

⛔ **THIS IS THE ANSWER TO R2's WIDTH QUESTION, AND IT ARGUES AGAINST A BUDGET.** A width budget
would not have caught line 95 — the author was already breaking lines; they just stopped partway.
R11 catches it structurally, with no number to argue about. **A width rule is neither necessary nor
sufficient for the corpus's worst damage.**

## R12 · CROSS-CALL TABLE ALIGNMENT — **OPEN. Real, deliberate, and nobody wrote it down.**

`wat-scripts/scratch-pad/rules-corpus-02-gates-and-unknowns.wat:176-180`, verbatim:

```
(:m::Member :id 1 :prefix "String" :base "concat"    :style "slash")
(:m::Member :id 2 :prefix "string" :base "length"    :style "colons")
(:m::Member :id 3 :prefix "i64"    :base "to-string" :style "colons")
```

Values padded into columns **across sibling calls** — a fact table, not a value list. Also at
`rules-corpus-01-node-facts.wat:121-125`, and the same instinct in a data literal at
`wat-scripts/fixes/reclaim-service-fixture-names.wat:40-58`.

⚠ **Every rule in this table so far reasons about ONE form's own children. This one reasons about a
form's NEIGHBOURS** — it needs a fact no rule has ("my sibling is a call to the same head with the
same keyword sequence"). If wat-fmt does not learn it, canonical formatting **destroys** these three
hand-built tables. That is a real cost of "canonical", and it should be paid knowingly or not at all.

## R13 · NO BLANK LINE BETWEEN A HEAD AND ITS FIRST ARM — **MINE**
59 occurrences of a `match` head followed by a blank line before arm 1 (`wat/query/sqlite-store.wat`
×5, `wat/rete/acc.wat:151`), and the same tic on `if` (`wat/core.wat:1687,1694,1706,1793`). Consistent
within a file, absent everywhere else — a local habit, not a convention.

## R9 · trailing whitespace — **MEASURED: 941 lines across the corpus.**
Includes a distinct flavour the riders named: a space left *before* a deliberate wrap
(`(:wat::core::Option/expect  ⏎`) at `wat/rete/acc.wat:63,82,115,147`. Free to fix, invisible in most
diff viewers, and it will touch many files on the first run.

## ★ R14 · `defservice` — clause keyword per line, values aligned. **20/20 UNIFORM in the corpus.**
```
(:wat::service::defservice :wat-tests::recorder
  :satisfies :wat-tests::Recorder
  :durable   [total <- :wat::core::i64]
  :ephemeral []
  :impls     [...])
```
The most uniform named-slot form found. It already IS R1's principle (one conceptual slot per line,
values in a column) applied to keyword clauses — so R7 should be stated as *generalising* R14, not
inventing something.

## ⛔ THE CENSUS THAT TESTS THE DESIGN'S CENTRAL CLAIM — head-symbol dispatch

`wat-scripts/scratch-pad/277-head-kind-census.wat`. The DESIGN rests extensibility on *"a layout
rule dispatches on HEAD SYMBOL"*, which presumes every form HAS one.

```
OF 89,968 FORMS WHOSE PARENT IS A LIST:
  literal keyword head    81,722   90%   ← head-dispatch works
  LIST head                4,840    5%   ← cannot dispatch on a name
  bare symbol head         3,185    3%   ← nameable, but no registry row
```

⚠ **The 5% is still confounded and I am not publishing it as the answer.** A `match` ARM
`((:wat::core::Ok v) body)` is a list whose child-0 is a list, and arms are counted here. The v1 rule
was worse — it fired on child-0 of *every* node, counting the first element of every vector and map,
and reported **9,115**. That number is retracted; the amended rule joins the parent's kind.

What survives: **90% of forms dispatch cleanly on a literal head, and the remainder is real but
small.** Match arms need dispatch on their PARENT's head, not their own — which the fact base
already supports (`Node.parent`) and no rule has used yet.

## THE SMALL FINDINGS WORTH KEEPING

- **`:wat::core::when` is DEAD** — zero call sites corpus-wide; its only mention is a probe naming it
  an orphan (`255-probe-are-the-orphans-live.wat`). Not a formatter matter; a registry one.
- **`wat/sqlite.wat:52-55` is the best comment-alignment exemplar in the corpus** — better than
  `bracket.wat`'s. Variant names padded so every `[` lines up AND every `;;` lines up.
- **The stdlib already violates R1** — `wat/fix.wat:1028` and `:1083` disagree with each other about
  crowding two args, 55 lines apart in one file.
- **`wat/core.wat:1405` vs `:673`** — `defn`'s own defining macro splits `name` and `& rest` onto
  separate lines; `->`'s puts `acc` and `& steps` together. Same file, same era, two answers for the
  fixed-arg-plus-variadic shape.
