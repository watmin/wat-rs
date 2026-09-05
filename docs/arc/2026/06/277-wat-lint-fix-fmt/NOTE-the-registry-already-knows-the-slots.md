# NOTE — the registry already knows the slots. Measured, 2026-09-05.

> **Builder:** *"ret-spec is a single line... i will not accept otherwise... we express what we must
> in the rules to make this hold true"* · and on the mechanism: *"absolutely this"* —
> **wat reading wat's own declared grammar, the same move as wat-fix and wat-grep.**

## THE PROBLEM THIS ANSWERS

R11's exploded rule split a ret-spec across two lines, because `:wat::core::fn` **has a grammar and
no rule**, and the default cannot see that `-> T` is one slot:

```
    (:wat::core::fn
      [acc <- :wat::core::i64 x <- :wat::core::i64]
      ->
      :wat::core::i64          ⛔ the ruling says ONE line
```

## MEASUREMENT 1 — how much grammar does the registry hold?

`wat-scripts/scratch-pad/277-does-the-registry-know-slots.wat` — **asked, not grepped.**

```
registry rows = 572        rows carrying a non-empty @syntax = 36
```

```
fn · rete::fn · let · rete::let · match · rete::match · do · if · and · or · quote · quasiquote
def · defmacro · defenum · defclause · defsurface · structtype · newtype · typealias · derive
defalias · forms · ann-form · macroexpand · macroexpand-1 · struct->form · use! · load-file!
signed-load! · digest-load! · lazy · holon::literal · set-redef! · set-eval-redef!
```

## MEASUREMENT 2 — what the registry does NOT hold, and why it does not matter

`277-does-defn-have-a-row.wat`:

```
:wat.core/defn · :wat.core/defrecord · :wat.core/defstruct · :wat.test/deftest   ->  0 ROWS
```

They are wat-level MACROS, not intrinsics — no registry row at all, so no grammar.

⚠ **Control:** the same predicate with `:wat.core/fn` added returns **1 row**
(`kind=SpecialForm`, syntax present). The zeros are a finding, not a broken probe.
`[[feedback_a_green_from_a_mis_aimed_probe_is_indistinguishable_from_a_working_gate]]`

★ **And the coverage is COMPLEMENTARY, not partial.** The forms WITH grammars are exactly the ones
nobody has written rules for — and exactly where R11 is mangling. The forms WITHOUT rows are the big
user-facing macros that already have hand-written rules (`defn` → R1, which is why `defn`'s ret-spec
was never damaged). **@syntax covers the gap, not the middle.**

## MEASUREMENT 3 — can `read-string` eat a grammar string?

The strings carry placeholders real code never contains: `<param>` `:T` `...` `<body>+` `<exprs>+`.

`277-can-wat-read-its-own-grammar.wat`, over **all 36, not one**:

```
GRAMMARS=36   UNREADABLE=0
```

**Every grammar string is readable wat source.** No tolerant reader needed; `read-string` is enough.

## MEASUREMENT 4 — is the SLOT actually derivable? ★ THE ONE THAT DECIDES IT

`277-locate-the-slot-in-a-grammar.wat` walks the parsed grammar:

```
GRAMMAR OF :wat.core/fn                    GRAMMAR OF :wat.core/let
  idx 0  keyword  :wat::core::fn             idx 0  keyword  :wat::core::let
  idx 1  vector   [vector]                   idx 1  vector   [vector]
  idx 2  symbol   ->        ← THE ARROW      idx 2  symbol   <body>+
  idx 3  keyword  :RetType  ← ITS TYPE
  idx 4  symbol   <body>+                    (no arrow -> no ret-spec slot. Correct.)
```

> **THE RULE: if a head's grammar has `->` at index i, children i and i+1 of any form with that head
> are ONE SLOT — withhold the break between them.**

Derivable, positional, and it needs nothing the fact base cannot already carry.

## ⚠ WHAT IS NOT MEASURED — named so it is not assumed

- **Index alignment under variadics.** `fn`'s pre-arrow part is a single vector, so grammar index ==
  form child index. A grammar with a VARIADIC before the arrow would break that correspondence.
  None of the 36 has one today — but **that was read off the list, not computed**, and the rule
  should refuse rather than guess if it ever meets one.
- **`defclause`'s arrow is NESTED** (`:name [-> :T] (…)`) — inside a vector, not a top-level child.
  A top-level scan correctly finds no slot there. Whether that is right for `defclause`'s layout is
  unasked.
- **How a `Slot` fact reaches a rule.** The natural shape is a fifth fact beside `Node`/`Named`/
  `Span`/`Written`, built by the same walk that already reads the registry. Not built, not measured.
