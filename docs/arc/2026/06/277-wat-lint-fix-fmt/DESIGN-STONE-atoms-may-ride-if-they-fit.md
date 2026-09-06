# DESIGN — STONE: a pair collection of ATOMS may stay inline, if it fits

> **Builder:** *"maps composed of atoms can be oneliners ... anything that requires an eval gets its
> own line"*
> ```
> {:a 3 :b 42}                  ;; legal unless it exceeds the line length limit
> {:a (wat.core/+ 1 2) :b 42}   ;; this isn't a oneliner
> {:a (wat.core/+ 1 2)          ;; one pair per line — there's an expr, not a value
>  :b 42}
> ```

## ONE RULE, THREE OPEN QUESTIONS CLOSED

> **A pair collection — a map literal OR a trailing pair run — stays INLINE if every value is an
> ATOM and it fits the budget. Otherwise one pair per line.**

```
{:a 3 :b 42}                  atoms, fits      -> inline
{:a (+ 1 2) :b 42}            compound value   -> one pair per line
{:a 3 :b 42 …very long…}      atoms, too wide  -> one pair per line
(:p::T :a 1)                  atoms, fits      -> inline     ← DEFECT 4, answered
```

It settles **map literals** (156 in the corpus, 77 files), **one-pair constructors**, and the
**`:examples` shape** that made a doc row over-explode. Consistent with every prior ruling:
`(assoc m :k x)` keeps its pair because `x` is an atom; `(Unreadable :file p :reason (Error/message
c))` explodes because a value is compound.

## ⛔ THIS UN-RETIRES THE WIDTH FACT — stated, not slipped in

When the exploded form was ruled, R15 (120) demoted from a **driver** of layout to a **check** on it,
and `[[NOTE-width-is-a-fact-not-a-rule]]` left the width fact off the critical path. *"unless it
exceeds the line length limit"* puts it back.

★ **The distinction that keeps this safe, and it is real:**

```
width of SELF        "am I too wide?"           LOCAL — composes
width of NEIGHBOURS  "did my sibling break?"    what was argued against at the table stone
```

This is width-of-self. Nothing here asks a form about its siblings' widths.

⛔ **And rete still cannot derive it.** `NOTE-width-is-a-fact-not-a-rule` measured the refusal
(*"stratify: negation cycle detected"*) and isolated it: an aggregate over the relation the rule
derives. So width comes from **the walk**, as a fact — and
`wat-scripts/scratch-pad/277-width-as-a-fold.wat` is the proven implementation, checked over
**18,408 single-line forms** with a control that caught its own bug.

## ★ THE ONE DESIGN DECISION — a rule GRANTS permission; the EMITTER exercises it

"Does it fit" depends on the **indent the form lands at**, which only the emitter knows. A rule
cannot decide it without naming a column, and no rule may name a column.

```
the RULE asserts   AllAtoms {form}      a purely STRUCTURAL fact: every value is an atom
the EMITTER decides  indent + width <= 120  ->  inline;  else  explode
```

⚠ **This is the same shape as `AlignPairs`** — the rule asks, the emitter computes from what it is
emitting. It is NOT the emitter growing a style opinion: the emitter never decides *whether a form
may be inline*, only *whether the permission the rule granted can be honoured here*.

## THE WIDTH FACT — and its known hazard

```wat
(:wat::core::defrecord :wat::fmt::Width
  [id <- :wat::core::i64  w <- :wat::core::i64])   ;; rendered-on-ONE-LINE width
```

```
leaf      width = length(ast->source node)
interior  width = 2 delimiters + Σ children + (n-1) separators   ==   Σ + n + 1
```

⚠ **The control from the original fold must come with it:** for a form already on one line, derived
width must EQUAL its span width. It agreed on 18,408 forms **except reader-synthesized nodes**
(`~x`, `` `x ``, `\c`), whose spans do not cover their rendered text — `wat/service.wat:2949` is
literally `~handle-record`, derived 19, actual 1. **The control must exclude them** (`grep.wat`'s
`Written` fact marks a span that holds its node's own text) or it reports a defect that is not one.

## THE ACCEPTANCE

```
1  ★ {:a 3 :b 42}                     INLINE
2  ★ {:a (+ 1 2) :b 42}               one pair per line
3  ★★ a long all-atom map             one pair per line — the WIDTH path, shown firing
4  ★★ (:p::T :a 1)                    INLINE — defect 4, answered
5  ★ (Unreadable :file p :reason (…)) still explodes
6  ★★ the width CONTROL               derived == actual on single-line forms, EXCLUDING
                                       synthesized nodes; CHECKED and MISMATCH both printed
7  idempotent throughout
8  no rule names a column             grep -c 'col' rules/*.wat = 0
9  ★★ the 614 doc examples            over-120 still 0; report how many became INLINE
10 the sibling table still wins        a table row stays a table row, not re-inlined pair-by-pair
```

⛔ **Row 3 is the width path shown firing.** Rows 1 and 4 pass without any width at all — a strike
that never computes a width satisfies them. **Only row 3 proves the budget is consulted.**

⚠ **Row 6 prints CHECKED as well as MISMATCH.** A control reporting 0 mismatches over 0 forms
examined is indistinguishable from success, and that exact vacuous green was published once in this
arc.

## OUT OF SCOPE

- **R15 as a lint** — this makes width available; reporting an over-long line is separate.
- **Level-2 alignment inside a value** — still open.
- **The emitter's comment-indent defects** — corpus-only.
