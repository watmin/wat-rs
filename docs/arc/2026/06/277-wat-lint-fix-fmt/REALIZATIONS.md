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
