# DESIGN — STONE: `metadata-of` answers with the whole row

> **Builder, 2026-09-06:** *"so… metadata-of is deficient….. examples and args need to be on
> that….. they are in the metadata-map… yes?"* → then, on the count: *"yeah… draw it… put all five
> on there…."*

**They are.** The code says so in its own comment, and the struct proves it.

## THE DEFECT — the data is carried and silently dropped

```rust
// src/runtime.rs, metadata-of's registry branch
// :doc — the GFM prose body … :added … :ret — the @ret description.
// (Vector-valued keys :args/:examples/:see are CARRIED on the entry
//  but rendered by the iv-b2 verifier seam, not here — SCOPE CUT.)
```

## ⛔ AND IT IS SEVEN, NOT FIVE — the count I gave the builder was wrong

Enumerating `IntrinsicEntry`'s fields against `metadata-of`'s `put` list, rather than trusting the
comment's three plus the two I happened to spot:

```
PUT (13)      :name :kind :layer :defined-in :doc :added :ret :arity
              :purity :determinism :totality :expand-time :category

CARRIED, NEVER PUT (7)
  args         &[(name, type, desc, optional)]     ← doc-row contract
  examples     &[ExampleSubmission]                ← doc-row contract
  see          &[&str]                             ← doc-row contract
  deprecated   Option<(&str, &str)>                ← doc-row contract
  alias_of     Option<&str>                        ← doc-row contract  ★ the one I missed
  ret_type     &str                                ← folds into :ret's PAIR
  syntax       &str                                — carried, but OUTSIDE the doc contract
```

★ **`alias_of` is the sixth and I told the builder five.** Same census failure as three others this
session: I read the comment's list, added what I noticed, and never enumerated the struct against the
emitter. `[[feedback_a_census_of_a_name_must_ask_every_rendering]]`

## ★★ THE PRECEDENT ALREADY RULED THIS EXACT CLASS A DEFECT

The two keys immediately above the scope cut are the same story, closed:

> *":totality / :expand-time — these two axes **have lived on `IntrinsicEntry` since the T3 stones
> but were never `put` here**, so an intrinsic's `metadata-of` … silently OMITTED them — the same
> **"answers in two shapes" defect** this stone closes … per the acceptance bar (**"the other axes
> too … converging one key only MOVES the defect"**)."*

A prior stone hit this shape, named it, wired in two axes, quoted an acceptance bar saying
key-at-a-time only moves the problem — **and left seven behind.** This stone finishes it.

## ⚠ `:ret` MUST BECOME THE PAIR, AND IT COSTS NOTHING — measured

`wat_doc::from_metadata` — the ONE decoder, already the round-trip gate's subject — reads exactly:

```
:added  :alias  :args  :deprecated  :doc  :examples  :ret  :see  :yields
```

and its `:ret` is the **pair** `[type, description]`, which it splits into `ret_type` + `ret`.
`metadata-of`'s `:ret` is the description alone. **Same key, two shapes — the precedent's exact
complaint, one level down.**

Changing it breaks nothing that exists. Every reader, measured across 344 call sites:

```
:arity 48 · :totality 18 · :purity 5 · :determinism 5 · :name 1 · :ret 0
```

**Zero read `:ret`.** So it becomes the pair, matching the decoder.

## ⚠ AND ONE ASYMMETRY THE STONE MUST NAME, NOT PAPER OVER

`from_metadata` reads `:yields`. **`IntrinsicEntry` has no `yields` field**, while `DocComment`
does. So the registry branch cannot emit it and the wat branch can. That is a real gap between the
two sources, it is **not** this stone's to close, and a stone that silently emits `:yields` from one
branch only would recreate the two-shapes defect it exists to remove.

## ★★★ THE CONTRACT DECISION — the acceptance is a ROUND TRIP, not a key count

**`metadata-of <fqdn>` must return a map that `wat_doc::from_metadata` accepts**, so its output IS a
doc-row map rather than a similar-looking one. That is what makes the migration a lookup instead of
a transcription, and it is checkable by the gate that already exists
(`read(print(doc)) == doc`, `crates/wat-macros/src/edn_doc.rs:580`).

**Both branches must emit the same shapes, key for key.** The precedent's whole point was that the
registry branch and the wat-binding branch answer in ONE shape; adding six keys to one branch alone
is the defect, not the fix.

## WHY IT IS LOAD-BEARING NOW

Arc 255's sweep renders a `#wat.doc/Row` **576 times**. Today that needs `metadata-of` for the
scalars **plus a 615-row linear scan** of `:wat::intrinsic::examples` for the vectors — and `:args`
is not reachable from wat by **either** route. `[[PROBE-277-registry-row-pretty]]` measured the scan.

## WHAT THIS STONE IS NOT

- **NOT `:syntax`.** Carried, but outside the doc-row contract; adding it is scope creep.
- **NOT `:yields`.** Named above as an asymmetry, deliberately left.
- **NOT a by-name `examples-of`.** If `metadata-of` carries `:examples`, the enumeration verb stops
  being the only route and a second lookup verb is unnecessary.

## FILES

```
src/runtime.rs        metadata-of, BOTH branches — the registry `put`s and the wat-binding emit
tests/                the round-trip acceptance, and a both-branches-agree test
```
