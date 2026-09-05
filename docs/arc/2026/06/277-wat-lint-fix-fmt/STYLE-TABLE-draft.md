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
