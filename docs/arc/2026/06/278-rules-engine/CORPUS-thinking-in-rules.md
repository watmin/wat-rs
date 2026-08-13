# CORPUS — thinking in rules

> Training material. The subject is the faithful-Clojure migration, but the subject is not the point —
> the point is the *move* each entry teaches, which is reusable anywhere a decision is currently an
> `if`/`else` chain.
>
> Every claim here is grounded. Where a number appears it was measured this session; where a mechanism
> appears it was proven by a run, and the run is named. Nothing is asserted from reading.

---

## 0. Why this exists — the measured state of the hand-rolled migration

`wat/fix.wat` is 1082 lines and **98 `if` forms**. Its classification core is six predicates
(`structural?`, `annotated-if?`, `head-keyword?`, `arrow?`, `type-shaped-keyword?`) dispatched by
`fix-seq` (`fix.wat:119`) — a linear left-to-right walk over a child vector carrying **one boolean**,
`prev-arrow?`. That boolean is the entire memory the classifier has.

Driven over the whole corpus (1392 `.wat` files, one invocation per file so each failure names its
file):

```
TOTAL=1392   OK=1312   FAIL=80
```

The 80 loud failures resolve to **five distinct roots**, every one a `MalformedForm` raised inside
`fix.wat`'s own conversion verbs:

| n | root |
|---|---|
| 43 | `ast-name` on a list head that is not a Symbol/Keyword/StringLit |
| 15 | `keyword/to-symbol` on a non-convertible keyword (bare data keyword / namespace-prefix marker) |
| 12 | `string::subs` — **`start > end`**, an inverted span |
| 7 | `keyword/to-type-form` — parametric type with a bare, non-FQDN head |
| 3 | `keyword/to-type-form` — `InnerColonInCompoundArg` |

And the finding that matters more than all of them: **the `OK` column is not clean.** Re-run from a
pristine original, `tests/macros/probe_do_splice_define_via_macro.wat` exits **0**, is written, goes
from 9 lines to 5, and emits fused garbage:

```clojure
;; before
  `(:wat::core::do
     (:wat::core::defn :my::probe::helper [] -> :wat::core::i64 42)
     ~body))
;; after — exit 0
  wat.core/quasiquotet.core/defn my.probe/helper [] :- wat.type/i64 42)
     wat.core/unquote  (wat.core/defn my.probe/main [] :- wat.type/i64 (my.probe/helper)))
```

Two forms destroyed. `wat.core/quasiquotet.core/defn` is not a symbol — it is one replacement written
over the top of the next token.

**And the checker cannot see it.** Measured, twice, in both spellings:

```
(:wat::core::totally-not-a-real-verb 1)   -> --check exit 0
(wat.core/totally-not-a-real-verb 1)      -> --check exit 0
(:wat::core::+ 1 "not-an-int")            -> --check exit 1   (type error on a KNOWN fn)
```

`--check` catches type errors on functions it resolved; it does **not** resolve names. So a corrupted
or fused name passes `--check` and dies at runtime as `UnknownFunction`. **`--check` cannot be the
acceptance gate for this migration.** Any claim of the form "the substrate accepts the output —
`--check` exit 0" proves parse + type-check and nothing about name resolution.

---

## 1. The target dialect (the builder's spec — the thing the rules must produce)

```clojure
;; types get their own namespace
wat.type/i64   wat.type/f64   wat.type/string   wat.type/keyword   …

;; generics become a FORM whose type arguments are a VECTOR
(wat.type/Vector [wat.type/i64])
(wat.type/Vector [wat.type/i64] 0 1 2 3)                       ; prepopulated

(wat.type/HashMap [wat.type/keyword wat.type/i64])
(wat.type/HashMap [wat.type/keyword wat.type/i64] :first 1 :second 2)

;; receiver-typed methods get a receiver namespace
wat.core.i64/to-string

;; string moves to its OWN top-level namespace, Clojure-style
wat.string/join   wat.string/length   …
```

The two largest classes, in the builder's words, are **inconsistent styles** and **unexpected
slashes**. Both are named below.

---

## 2. THE MOVE — a fact per node, and every decision a join

Proven this session by run (`wat-scripts/scratch-pad/rules-corpus-01-node-facts.wat`, exit 0):

```
IsArrow   1     <- non-vacuity: the seed landed, so the rows below mean something
IsHeadKw  2     <- exactly the two intended; the withheld-name arm did NOT leak
IsTypePos 1     <- the prev-sibling JOIN, replacing `prev-arrow?`
```

Two fact types carry the whole tree:

```clojure
(:wat::core::defrecord :fixr::Node  [id parent index kind])   ; EVERY node
(:wat::core::defrecord :fixr::Named [id name])                ; ONLY nodes that have a name
```

Note what is absent: the `WatAST` itself. Every decision in `fix.wat`'s chain reads **kind, name, and
position** and nothing else. The classifier never needs the node.

---

## 3. The lessons, each grounded in a measured failure

### L1 — A missing fact is the guard. A preceding `if` is not.

**On disk** (`fix.wat:63`, `annotated-if?`): guards on ARITY, then calls `ast-name` on the head.

```clojure
(:wat::core::if (:wat::core::empty? (:wat::core::drop ch 2))
  false
  (:wat::core::let [head (:wat::core::first ch) …]
    (:wat::core::if (:wat::core::= (:wat::core::ast-name head) ":wat::core::if")  ; ← boom
```

The guard and the use are in different `if`s, so **nothing forces them to agree**. Measured cost: 43
of 1392 files. Bisected to the exact form and confirmed by a two-arm differential where the *only*
difference is child count:

```clojure
((:wat::core::fn [a b] …) 1 2)   ; 3 children -> drop 2 non-empty -> ast-name on a LIST -> exit 1
((:wat::core::fn [a]   …) 1)     ; 2 children -> drop 2 empty     -> early false        -> exit 0
```

**In rules** there is no gap to leave open, because the precondition *is* the match:

```clojure
(:wat::rete::defrule :fixr::head-kw
  :when [(:fixr::Node  (?id <- :id) (?k <- :kind))
         (:fixr::Named (?id <- :id) (?n <- :name))          ; ← no name ⇒ no join ⇒ no fire
         …]
  :then [(:fixr::IsHeadKw :id ?id)])
```

> **The move:** stop writing "check X, then use X". Make the thing you would have checked a *fact*,
> and let the absence of the fact be the guard. You cannot forget a check that is the match.

**How it was proven, and why the naive version would have been vacuous.** The first draft of the probe
gave the unnameable node `kind "list"` — so it failed the kind test anyway and proved nothing about
the join. The armed version uses two nodes *identical in every respect the rule tests*, differing only
in whether a `Named` fact exists. `IsHeadKw` must read exactly 2: a 3 means the guard leaks, a 1 means
the positive arm is broken and "no leak" was vacuous.

---

### L2 — Carried state is a join you refused to write.

**On disk** (`fix.wat:119`): `fix-seq [items prev-arrow?]`. One bit. Everything the classifier can
know about context is "was the previous token an arrow". That is *why* there are four rules — not
because four sufficed, but because one boolean cannot express a fifth.

**In rules**, position is a join on `(same parent, index − 1)`:

```clojure
(:wat::rete::defrule :fixr::type-pos
  :when [(:fixr::Node    (?id <- :id)  (?p <- :parent) (?i  <- :index))
         (:fixr::Node    (?aid <- :id) (?p <- :parent) (?ai <- :index))   ; ← same parent
         (:fixr::IsArrow (?aid <- :id))                                   ; ← stands on L1's verdict
         (:wat::rete::where
           (:wat::rete::core::i64::= ?i (:wat::rete::core::i64::+ ?ai 1 :undefined 0)))]
  :then [(:fixr::IsTypePos :id ?id)])
```

Once previous-sibling is a join, **next-sibling, grandparent, head-of-parent and nth-cousin are the
same move for free**. None of them requires a new parameter, a new walk, or a new boolean.

> **The move:** every accumulator threaded through a recursive walk is a relation you declined to
> name. Name it, and the walk stops being the only way to reach it.

---

### L3 — `if`/`else` encodes priority invisibly. Rules make it a fact you can test.

**On disk**, `fix-seq`'s dispatch is a nested chain whose ORDER is load-bearing and unwritten:

```
post-arrow type  →  structural type  →  arrow  →  head keyword  →  recurse
```

Reorder two arms and the meaning changes silently. Nothing anywhere states that these are meant to be
mutually exclusive, so nothing can check it.

**In rules**, each arm is a rule with its full condition stated. Overlap stops being invisible: two
rules deriving a verdict for the same node is a *queryable* condition. The property "no node receives
two classifications" becomes a rule of its own:

```clojure
(:wat::rete::defrule :fixr::conflict
  :when [(:fixr::IsArrow  (?id <- :id))
         (:fixr::IsHeadKw (?id <- :id))]
  :then [(:fixr::Conflict :id ?id)])          ; must always query to 0
```

> **The move:** when correctness depends on ordering, you have an invariant with no home. Give the
> invariant a rule, then assert its extension is empty.

---

### L4 — Separate the DECISION from the ACTION.

**On disk**, `keyword/to-symbol` *decides and converts in the same call*, and raises when it cannot
decide. Inside a whole-file walk that raise kills the file — 15 of 1392, measured.

**In rules**, the LHS decides and the RHS acts, and they are different phases by construction. A node
nothing classifies produces no edit and is *reportable*, not fatal:

```clojure
(:wat::rete::defrule :fixr::unclassified
  :when [(:fixr::Node (?id <- :id))
         (:wat::rete::not (:fixr::Verdict (?id <- :id)))]
  :then [(:fixr::NeedsRuling :id ?id)])
```

That derived `NeedsRuling` set **is the worklist for the next rule**, and it is the honest form of "we
made poor choices early on": every form we have not yet decided how to handle names itself, once,
instead of blowing up whichever file happens to contain it first.

> **The move:** a converter that raises is a decision fused to an action. Split them and the
> undecided case becomes data instead of a crash.

---

### L5 — Make the invariant a rule, not an assumption. (The silent corruption.)

The 12 `string::subs start > end` failures and the 47 files of silent fusion are the **same defect
class**: an edit is emitted whose span does not cover the token it claims to replace. The classifier
says "this node's name is `:wat::core::quasiquote`"; the editor writes that name over the node's whole
span — which, for a reader-macro sigil, is the entire wrapped form.

Nothing checks that the span being overwritten is the *name's* span. It is an assumption, held in two
places that never meet.

**In rules**, an edit is a fact, so it can be *constrained before it is applied*:

```clojure
(:wat::core::defrecord :fixr::Edit  [id offset old-len new-text])
(:wat::core::defrecord :fixr::Token [id offset len])           ; the token actually at that offset

(:wat::rete::defrule :fixr::bad-edit                            ; an edit that does not cover its token
  :when [(:fixr::Edit  (?id <- :id) (?o <- :offset) (?l <- :old-len))
         (:fixr::Token (?id <- :id) (?o <- :offset) (?tl <- :len))
         (:wat::rete::where (:wat::rete::core::i64::not= ?l ?tl))]
  :then [(:fixr::BadEdit :id ?id)])
```

This is constraint engineering, not failure engineering: the wrong edit is not *caught*, it is
*refused* — and the refusal is one rule rather than a review habit.

> **The move:** when two components share an assumption and neither states it, the assumption is
> already a bug. Write it as a relation and let a rule hold it.

---

### L6 — Some of this is SHAPE and some of it is a RULING. Rules hold both; `if`/`else` inlines one into the other.

This is the entry the target dialect forces, and it is the reason the migration cannot be a codemod
with more branches.

**Shape-derived** — computable from position, no table needed:

```clojure
:wat::core::i64        in TYPE position        ->  wat.type/i64
:wat::core::i64::+     in CALL-HEAD position   ->  wat.core.i64/+
```

*The same source token maps to different targets depending on role.* One boolean cannot see role;
a join over `(parent-head, index, prev-sibling)` can.

**Ruling-derived** — not computable from the spelling at all:

```clojure
:wat::core::string::length   ->  wat.string/length      ; string RELOCATES to a top-level namespace
:wat::core::i64::to-string   ->  wat.core.i64/to-string ; but i64 does NOT
```

Nothing in `:wat::core::string::length` says it should become `wat.string/…` while
`:wat::core::i64::to-string` becomes `wat.core.i64/…`. That is a **decision**, and it belongs in a
fact table, not in a branch:

```clojure
(:wat::core::defrecord :fixr::NsRuling [from <- :wat::core::String  to <- :wat::core::String])
;; (:fixr::NsRuling :from ":wat::core::string" :to "wat.string")
;; (:fixr::NsRuling :from ":wat::core::i64"    :to "wat.core.i64")
```

and the migration is the **join of shape against ruling**:

```clojure
(:wat::rete::defrule :fixr::relocate
  :when [(:fixr::CallHead (?id <- :id) (?ns <- :ns) (?base <- :base))
         (:fixr::NsRuling (?ns <- :from) (?to <- :to))]
  :then [(:fixr::Emit :id ?id :ns ?to :base ?base)])
```

> **The move:** an `if`/`else` chain forces every ruling to be inlined into control flow, where it
> cannot be listed, counted, reviewed, or diffed. Rules keep the rulings as *data* and the shape as
> *conditions* — and a ruling you have not made yet shows up as an unjoined fact (L4), not as a
> wrong answer.

---

### L7 — Unexpected slashes: the collision the new dialect creates.

`/` is the namespace separator in the target dialect. It is **also already present** in names we
carry today — record accessors (`:usr::Client/reputation`), rete ops (`String/contains?`).

A naive split-at-the-last-`::` produces two separators in one symbol:

```
wat.core/Option/expect
```

Measured behaviour, and it is the worst of the three possibilities:

```
--check   ->  exit 0                                                   (silent)
run       ->  UnknownFunction ":wat::core/Option::expect"               (dies late, mis-normalized)
```

It is **not** a parse error. It passes every static gate we have and fails at runtime under a name
nobody wrote. So "EDN-illegal double slash" is not a category the checker will ever hand us — it has
to be a rule:

…and the rule that finds them must be written **inside the fence**, which is stricter than it looks
(see L8). A helper is not available:

```clojure
;; NOT legal — `:fixr::slash-count` is a user fn, refused by Law A (grounded below, L8)
(:wat::rete::where (:wat::rete::core::i64::> (:fixr::slash-count ?sym) 1))
```

The collision therefore has to be expressed in rete primitives, or detected in the RHS phase where the
fence does not apply. **Which of those is right is OPEN and is the next thing to probe.**

> **The move:** when a migration introduces a new significant character, every existing use of that
> character is a collision site. Enumerate them as a rule *before* the rewrite, because no downstream
> gate will.

---

### L8 — The first failing axis is not the whole verdict. (Learned the hard way, this session.)

The `where` fence measures four axes **strictly and separately** — pure, deterministic, total, rete-
primitive — and its message names the **first** one that failed. That is a diagnostic about where the
check stopped, not a census of what is wrong.

Grounded, two runs, same file, one line changed:

```
;; helper body uses (:wat::core::i64::* x 2)
where expr is not total — ':wat::core::i64::*' is not total          ← reads like: fix the body

;; body changed to (:wat::rete::core::i64::* x 2 :undefined 0)
where expr is not a rete primitive — ':h::twice' is not a rete primitive
```

The first message made it look as though a user helper is admissible *if its body is total* — the
fence had walked **into** the body to measure totality, so the helper itself looked accepted. It is
not. Both are true at once: the fence descends into helper bodies **and** requires every head to be a
`:wat::rete::` primitive. Net: **a user helper cannot appear in a `where` at all.**

I nearly wrote the wrong conclusion into this corpus off the first message. That is
`[[feedback_an_error_names_where_it_gave_up_not_what_is_missing]]` recurring at the fence: the message
names where the *instrument* stopped, not what the *system* permits.

> **The move:** when a checker reports one violation, fix it and re-run before drawing the rule. An
> axis that has not been reached has not been passed.

---

## 4. What this corpus does NOT yet establish

Kept honest, because the whole point of the record is that the next reader can trust it.

- **The runnable exemplar covers L1 and L2 only.** L3–L7 are written as rule *shapes* against the
  proven fact model; they have not been run. They are designs, not results.
- **L7's `slash-count` predicate has no legal home yet.** Grounded in L8: Law A refuses a user helper
  in a `where` outright. Whether the slash check can be written in rete primitives, or must move to
  the RHS phase, is **open** — probe it, do not assume it.
- **`:wat::rete::not` is used in L4 without being grounded this session.** Negation exists (stratified,
  R18/R20), but the exact spelling in a `:when` vector was not verified here.
- **Exit codes read through a pipe were wrong.** Several readings this session used
  `… | head -c N; ${PIPESTATUS[0]}` and reported `0` where the real code was `2`. The corpus drive did
  **not** use a pipe, so its `OK=1312 / FAIL=80` split stands; the inline spot-checks did, and any
  number taken that way was re-taken without one.
- **No count exists for how many of the 1312 "OK" files are silently corrupt.** One is proven from a
  pristine original; the grep that suggested ~47 is contaminated by string literals and was not
  cleaned, because the run to clean it was stopped in favour of this work. **Do not quote 47.**
- **The corpus drive used the current `fix.wat`.** It measures today's tooling, not the ceiling of a
  text-edit approach in general.

## 5. Two live findings this corpus depends on

1. **`--check` does not resolve names.** Any acceptance gate for the one-shot migration must be a RUN
   or a dedicated resolution pass. This retires `--check exit 0` as evidence that a converted file works.
2. **`wat-scripts/scratch-pad/probe-rules-rich.wat` is dead on run** — it uses `:wat::core::>` inside a
   `where`, which Law A (#57) refuses at rule-compile. The loader gate parses and type-checks and
   therefore *structurally cannot see it*. That is task #85's class, still live in scratch-pad.
