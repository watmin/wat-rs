# DESIGN — STONE: keyword arguments, one pair per line, values aligned

> **Builder:** *"(user/some-fn :arg1 val1 :arg2 val2 ...) that should be our form?"* — and, shown the
> two real corpus shapes: **"both of these... they are very good"**

```wat
(:wat::service::defservice :wat-tests::recorder     ← the POSITIONAL rides the head line
  :satisfies :wat-tests::Recorder
  :durable   [total <- :wat::core::i64]             ← one PAIR per line
  :ephemeral []                                     ← VALUES ALIGNED in a column
  :impls     [...])

(:wat::grep::Unreadable                             ← no positional -> nothing rides
  :file   path
  :reason (:wat::core::Error/message __cause)
  :line   (:wat::kernel::Location/line (…))
  :col    (:wat::kernel::Location/col  (…)))
```

**The builder's own sketch falls out as the zero-positional case.**

## THE RULE

> **A keyword argument run: the first `:key` starts a new line; each `:key value` PAIR occupies one
> line; the values are PADDED so they begin in a common column.** Positional arguments before the
> first keyword ride the head line, per the existing leading-atom rule.

## ⛔ ALIGNMENT IS A CAPABILITY THIS ENGINE DOES NOT HAVE

`Break {id, kind}` says **where a line starts.** It cannot say **what column a token inside a line
lands on.** Three separately-ruled things all need that one missing capability:

```
kwarg VALUES in a column          this stone
the `<-` binders in an arg-spec   builder's defn ruling — `self    <-` / `work-fn <-`
trailing `;;` comments aligned    R8, ruled in spirit since the first style table
```

★ **One capability, three ruled consumers.** That is what makes it worth building rather than
hand-waving, and it is why R8 has sat unbuildable since the table was written.

## THE MECHANISM — the emitter aligns, exactly as it indents

`[[DESIGN-STONE-indent-is-structural]]` took columns away from rules and gave them to the emitter,
because a rule that names a column cannot compose with a rule that moves its form. **Alignment is
the same kind of quantity and gets the same answer.**

```wat
(:wat::core::defrecord :wat::fmt::AlignPairs
  [form <- :wat::core::i64])    ;; this form's broken children align their SECOND token
```

A rule ASKS for alignment on a form; the emitter computes the width from the children it is actually
emitting. **No rule ever names a column** — the wall from that stone (`grep -c 'col' rules/*` = 0)
must still hold.

⚠ **A separate fact, not a third `Break.kind`** — the same four-questions that put `BlankBefore` in
its own fact (4/0). Vertical separation, horizontal padding and line-start are three axes; a node may
carry all three without any of them changing another's meaning.

## ⛔ LEVEL 2 IS OUT OF SCOPE, AND IT IS IN THE BUILDER'S OWN EXAMPLE

```
  :line   (:wat::kernel::Location/line (…))
  :col    (:wat::kernel::Location/col  (…))
                                    ↑ the INNER arguments of two sibling calls, also padded
```

Level 1 is a form aligning **its own children** — the emitter has everything it needs.
**Level 2 aligns across SIBLING CALLS**, which is `[[REFUTE-claim-the-forms-you-position-not-the-subtree]]`'s
R12 — the only rule in the table that reasons past one form's own children, and the one that would
need a neighbour fact.

**Named, not smuggled in.** The builder said both examples are good and level 2 is inside one of
them; this stone delivers level 1 and reports level 2 as still open. **A stone that quietly delivered
half of an approved example would be the dishonest kind of green.**

## THE ACCEPTANCE

```
1  ★ a kwarg call with no positional: nothing rides; one pair per line
2  ★ a kwarg call WITH a positional: the positional rides (defservice's shape)
3  ★★ the VALUES are aligned in a column
4  ★★ `grep -c 'col' wat-scripts/fmt/rules/*.wat` -> 0 — no rule names a column
5  idempotent — alignment must not drift on pass 2
6  a ONE-PAIR call is not made worse (report the shape; the builder may want a carve-out)
7  ★★★ run the 615 real doc examples through it and report: how many changed, how many
   still exceed 120, and the worst remaining shape
```

⛔ **Row 7 is new and it is the standing correction from
`[[NOTE-the-formatter-had-never-met-a-real-file]]`: a fixture proves a rule fires; only real input
proves the formatter works.** Every stone from here ends this way.

## OUT OF SCOPE

- **Level-2 / R12 cross-call alignment** — above.
- **The `<-` binder alignment and R8's comments** — they need this same capability and become
  cheap once it exists, but each is its own rule.
- **The emitter's comment-indent defects** — corpus-only, and the builder has re-scoped corpus files
  to a TEST BED rather than a cleanup: *"we can use the tooling on existing files to see how they
  transform to confirm we're ready."* Still real, still next after this.
