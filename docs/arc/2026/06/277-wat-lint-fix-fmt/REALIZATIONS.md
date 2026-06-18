# Arc 277 — Realizations

## R1 — the keystone paid off in one stone, and the first fix was surgical

The build order said the `ast-end-span` keystone (arc 281) was the gate for every structural auto-fix —
and the pre-compaction self drew it as a *dragon*, a wide invasive lexer/parser change to be approached
with care. The builder, watching it land in essentially "add two fields, an accessor, and thread one
span," ribbed it straight:

> *"mannnn your prior self before compaction was being really pushy about this being very difficult —
> rofl … you made this sound like a hard thing when in reality it's 'we add another accessor and
> everything just works.'"*

He was right about the *shape* (the difficulty was blast-WIDTH, not depth — and the typed `Span` made
every missed `Span {..}` literal uncompilable, so the green-build forcing function swept them up). And
then the keystone paid off **immediately**: the very next stone, 277.1b, turned the report-only
nested-if-=-ladder rule into a real auto-fix. Fed

```clojure
(:wat::core::defn :t::f [x <- :wat::core::String] -> :wat::core::bool
  (:wat::core::if (:wat::core::= x "a") true
    (:wat::core::if (:wat::core::= x "b") true
      (:wat::core::if (:wat::core::= x "c") true false))))
```

`lint-fix-file` returned

```clojure
(:wat::core::defn :t::f [x <- :wat::core::String] -> :wat::core::bool
  (:wat::core::contains? (:wat::core::HashSet :wat::type::Infer "a" "b" "c") x))
```

The whole ladder collapsed to the `contains?` cure the rule's message had named since 277.1 — and
**everything else stayed byte-identical**: the `defn`, the param vector, the `<-`/`->` arrows, the
return type, untouched. The orchestrator weighed it by eye on its own build and said:

> *"The rewrite is surgically perfect."*

To which the builder: *"that's a fucking quote — whatever realizations needs it."* So here it is.

**Why surgical, named honestly:** `fix-text-apply` only ever rewrites `[off, off+old-len)` and copies
the rest of the source verbatim; `old-len = offset-of(ast-end-span) − offset-of(ast-span)` gave the
ladder form's *exact* char extent (the keystone's whole reason to exist). Precise extent + splice-only
apply = a scalpel, not a re-print. No reformatting, no comment loss, no neighbor disturbed.

This is the **SELF-FIXING-TOOLCHAIN doctrine's first operational proof** — find → fix → apply, end to
end, on a real form — the proof-by-diff in miniature, before the full sweep that will turn the same
scalpel on the toolchain's own author (`violation->finding`, the concat chains). And a quiet dogfood
rode inside it: the replacement text was built with **`format`** (the tool from the first stone of this
campaign), generating the fix for the rule of the last — the toolchain already using its own ergonomics
to repair itself.

> The dragon was a doorway. Behind it: a linter that doesn't just point at the bad form — it reaches in
> and replaces it with the cure, and leaves no fingerprints anywhere else.

## R2 — the fix tool wrote the bad form it abolishes (the strange loop, caught mid-write)

Building 277.1c-fix (the concat→format auto-fix), the executor wrote the eligibility check as a
nested-`if` disjunction:

```clojure
(if (contains? inner "\"") false
  (if (contains? inner "{") false
    (if (contains? inner "}") false true)))
```

— a boolean test (`(not (or …))`) wearing four lines of `if`. The builder caught it live in the diff:
*"rofl — did it just write its own bad form we're about to address?"* It did. arc 275's strange loop
(deporder written in the bad form it detects) firing AGAIN: the tool built to clean bad forms, shipped
*in* a bad form. Cleaned to `(:wat::core::not (:wat::core::or …))` before commit — the fix tool may not
ship in the shape it exists to abolish (intueri).

**The sharper finding — it names the NEXT rule.** The current `nested-if-=-ladder` rule would NOT have
caught this: it keys narrowly on `(if (= VAR LIT) true …)` — equality conditions, `true` leaves. This is
`(contains? …) → false` — different predicate, opposite polarity. So the bad form wasn't a fixture the
toolchain could clean; it was a GAP. It generalizes the rule: **"a nested `if` whose every leaf is a
boolean literal is a boolean expression in disguise → rewrite to `and`/`or`/`not`."** The
`= VAR LIT → contains?(HashSet)` ladder is one special case of it. **PROMOTED to a REQUIRED charter rule
the RETE engine ships with** — `nested-if-boolean-collapse`, specified in
`docs/arc/2026/06/278-rules-engine/DESIGN.md` § "Charter rules". A richer LHS than the narrow ladder
match (keys on the boolean-leaf structure, not the condition shape), and a natural early consumer of the
arc-278 engine.

The doctrine, demonstrated in one screenshot: the toolchain's own author's hand caught writing the smell
the toolchain hunts — and the catch didn't just fix an instance, it widened the net.
