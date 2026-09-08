# NOTE — the blocker NOTE's two load-bearing facts have BOTH moved

**Measured 2026-09-08.** `NOTE-the-registry-is-not-yet-the-largest-membership-set.md` (2026-09-01)
is the document that ordered arc 255's remaining plan. **Neither of the two facts it rests on is
true any more.** Recorded here, beside it, without editing it — what is written stays written.

## Fact 1 — the corpus failure rate

```
the NOTE     "578 of 599 .wat files FAIL — 96%" · "121 distinct names"
measured     2026-09-07:  143 of 833 FAIL — 17% · 35 distinct names
```

## Fact 2 — the special forms, which are why the ordering was FORCED

```
the NOTE     "The registry holds exactly TWO special forms — `let` and `if`. `def`, `fn`, `do`,
              `defn`, `quote`, `match` are not rows."
             "…a corpus that cannot resolve `fn` cannot be measured for anything else."
measured     2026-09-08, asked of the attribute set:
             if · let · fn · match · def · quote · do · defclause    ALL REGISTERED
```

★ **The bucket that made the order forced is gone.** The NOTE's sequence —
*registry becomes complete → resolve asks the registry → the undefined-func class dies as a side
effect* — was correct reasoning on its evidence, and the evidence moved underneath it. Nobody aimed
at 255; the special forms were registered in the course of other work, exactly as the founding
DESIGN predicted the class would die: *"as a side effect."*

## ⛔ What this does NOT mean

- **Not that the blanket can be deleted today.** The 2026-09-07 re-census measured 17% / 35 names
  with the blanket lifted — a real worklist, four families, rete numerics ~87% of sites at ~11 names.
  That is much smaller than 96%, but it is not zero.
- **Not that the registry is now complete.** Stone Q measured the TYPE side: three mechanisms
  (`TypeEnv.types` · `builtin_names` · `is_builtin_primitive`), and *"the largest membership set is
  the union, and no current verb asks it."* Stone Q2's census measured the VERB side: 133 unique
  `check.rs` arm FQDNs, **23** with no registration. Both are small; neither is nothing.
- **Not a criticism of the NOTE.** It was true when written and its reasoning was sound. It is the
  third instance this week of 296 R20 `HAERESIS EST ITERVM ROGARE` — *the record's settled numbers
  expire, and the ones that hurt are the RIGHT ones read past their date.*

## ⛔ AND THE BLANKET IS NOW A CORRECTNESS DEFECT, NOT HYGIENE

`296/NOTE-the-dot-spelling-silently-builds-the-wrong-variant.md`, same day:

```
(:wat::core::Option.Some {:value 7})   check=0  ->  #wat.core/Option.None {}   ⛔ WRONG VALUE
(:usr::Box.Full          {:payload 7}) check=1  ->  UnresolvedReference        correctly refused
```

The dot spelling — **the spelling the head migration is moving toward** — is accepted under
`:wat::*` and silently builds the wrong variant. A user namespace catches it; the blanket does not.

So the blanket is no longer *"a typo could ship."* It is *"a plausible, intended future spelling
produces the wrong answer today."* And 255's own NOTE observed the registry had **one consumer, and
that consumer was the orchestrator**. It has a real one now.

## The order this implies — for the builder to rule

```
the blanket   PROMOTED. Correctness, gates the dot flip, and its blocker's two facts have moved.
Q2's 22       a short list, named, not this stone's.
P-1           the annotation position validates — Q built its authority (is-type?)
P-2           a variant is a type — 4/4, unblocked
```
