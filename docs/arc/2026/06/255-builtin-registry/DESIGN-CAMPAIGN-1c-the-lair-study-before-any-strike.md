# DESIGN — PHASE 1c: the lair study. **No strike is drawn in this document.**

> Cast under `examinare`. The builder: *"we study the enemy... their lair... we find the traps...
> slow is smooth, smooth is fast."* Everything below is measured on disk 2026-09-03, after
> Stone 1a-ζ. Nothing is briefed until the traps are on the table.

## ⛔ THE HEADLINE: "68 names" IS NOT 68 UNITS OF WORK, AND NINE OF THEM ARE NOT VERBS

The corpus experiment's remainder has been read as a registration worklist all campaign. It is
not one. Asked of every one of the 68 — *does a definition for this name exist anywhere, in
`src/` or as a `wat/` `defn`?* — **nine have none at all:**

```
    9  :wat::type::Tuple          ⎫
    7  :wat::type::i64            ⎬ NOT VERBS — a second RENDERING of a type path
    5  :wat::type::String         ⎭
    1  :wat::rete::f64::>X          A DELIBERATE NEGATIVE CONTROL — do not touch
    2  :wat::core::println        ⎫
    2  :wat::core::edn::write     ⎬ DEAD CALL SITES the whitelist launders
    1  :wat::core::tuple-get      ⎬
    1  :wat::core::reduce-walk    ⎭
    1  :wat::spawn::process/grants
```

### ① The `:wat::type::*` four are a RENDERING, not a population

**Zero occurrences of `type::Tuple` / `type::i64` / `type::String` / `type::Vector` exist in
`wat/`, `wat-scripts/`, `wat-tests/` or `tests/`** — measured, not assumed. Yet the sweep reports
23 call sites and all four sit on **GAP_A *and* GAP_B**, counted as unregistered verbs.

`src/types.rs:5172` and `:5443` explain them: `wat::type::` is an **arc 251.2 dual-read alias**
that strips to `wat::core::`, with the flip to `:wat::type::` as canonical *"deferred to the 251.5
hard-cut"*. The audit walk (`canonicalize=false`) preserves the source spelling. So these are the
same names the census already counts under `:wat::core::` — `Tuple` appears at 271 sites as a verb
and 9 more as a type path.

★ `[[feedback_a_census_of_a_name_must_ask_every_rendering]]` was written in this arc, about this
arc, and the ledgers still carry both renderings as if they were different names.
**⬜ OPEN — I have not proven the exact production path**, only that no corpus text spells them
and that a dual-read alias exists which would explain it. That question must be answered before
any stone touches these four, and it may belong to arc 251 rather than 255.

### ② `:wat::rete::f64::>X` is a committed negative control. **Fixing it would destroy evidence.**

`wat-scripts/scratch-pad/probe-f64-comparator-bogus-head.wat` exists on purpose, and its own
header states the finding this campaign was founded on:

> *"`--check` does NOT validate `:wat::*` heads at all — a bogus rete keyword is opaque to the
> checker exactly like any other unregistered `:wat::` symbol, so this file TYPE-CHECKS
> (`target/release/wat --check` on it exits 0) despite `:wat::rete::f64::>X` never having been
> minted."*

★★★ **This is `is_reserved_prefix`'s indictment, already written down, already committed, by a
prior self.** Phase 3a does not merely delete a redundant authority — it turns this file from
green to a compile error, which is the *point*. **3a must decide this probe's fate as part of its
own scope**, and the honest move is to convert it into a compile-fail expectation, never to
quietly delete the witness.

### ③ Four `:wat::core::` names and one `:wat::spawn::` name are DEAD CALL SITES

`println`, `edn::write`, `tuple-get`, `reduce-walk`, `process/grants` — no handler, no arm, no
scheme, no `wat/` definition, **and not on the `RETIREMENT_TABLE`**. They are called from
`wat-scripts/scratch-pad/` and `wat-scripts/probes/` files that pass the loader gate today
**only because the prefix whitelist accepts anything under `:wat::`.**

⚠ These are not registration work. Registering them would be *manufacturing verbs to satisfy dead
callers.* They are corpus rot, and the honest dispositions are: fix the call site (`println` is
almost certainly `:wat::kernel::println` — the very same probe file that calls
`:wat::core::edn::write` calls `:wat::kernel::println` two tokens earlier), or delete the file, or
retire the name properly. **That is its own small stone and it should run BEFORE 3a**, because
3a turns every one of them into a build failure.

## THE REAL 1c POPULATION, decomposed by MECHANISM

With the nine non-verbs removed, `:wat::core::` holds **31 names / 3,407 call sites**, and it
splits on the same axis that cut 1b — *does a `CheckEnv` scheme already exist?*

```
                              names   ledger effect of registering
has a scheme (on GAP_A)         11    GAP_A ↓ · GAP_B ↓ · DEBT unchanged   ← the 1b-i shape
   foldl · Tuple · get · map · apply · filter · contains? · stream->vec ·
   mapv · find-last-index · conforms?

no scheme (not on GAP_A)        20    GAP_B ↓ · DEBT ↑                     ← the 1b-ii shape
   = · PersistentVector · first · second · PersistentMap · extend-type ·
   str · < · derive · > · >= · not= · defclause · <= · third · None · …
```

★ The cut that has worked twice works again, and it is measured rather than assumed.

## ⛔⛔ THE TRAP THAT MAKES 1c A CAMPAIGN AND NOT A STONE

**24 of the 31 carry a live literal arm in the eval door.** Registering any one of them mints a
handler, and `registry_first_door_owns_every_handler_row_no_literal_arm_survives` — which filters
on `entry.handler.is_some()` — then *demands that arm be deleted*. This is not optional and it is
not a preference: it is the gate that has already outranked one of my STOPs.

So 1c is **not** "author 31 doc rows." It is **31 dispatch migrations out of a 1,696-line
function**, each one authoring five grounded axes on the way past. The door's current state:

```
fn dispatch_keyword_head_value  —  1,696 lines,  54 live literal arms
   :wat::core::  33   ← 1c's own population is 24 of these
   :wat::rete::   9
   the eval-* family 11   (eval-ast! alone is 330 corpus call sites)
   program · stdlib   2
```

★ Measured capacity, from this campaign's own record: **6 authored rows + 11 edits ≈ 375K tokens,
near a rider's ceiling.** 31 authored rows with arm surgery is therefore **five to six riders
minimum**, and a single brief covering them would fail the way 1b's original scoping failed.

## The five traps, named

1. **A name that is not a verb.** Nine of them. Registering one manufactures a verb for a caller
   that should not exist. *Ask "what defines this?" before "what should its axes be?"*
2. **The arm cascade.** Every registration with a handler forces a deletion in a 1,696-line
   function. The gate is right and will not be argued with.
3. **Two renderings, two ledger rows.** `Tuple` is counted as a verb (271) and as a type path (9)
   and sits on GAP_A/GAP_B twice over. A stone that "registers Tuple" will move one and not the
   other, and the ledger will look wrong for a reason no one wrote down.
4. **The un-schemeable HOFs.** `foldl`/`map`/`filter`/`reduce` are polymorphic over the *container
   constructor* — which is precisely why `OpClass::Redispatch` was invented ("cannot be stated as
   a rank-1 `TypeScheme` at all"). Four of them are on GAP_A, so a scheme *does* exist; that
   contradiction must be read before they are touched, not after.
5. **A negative control that looks like a defect.** `>X` is evidence. `[[feedback_a_negative_control_that_can_be_kept_must_be_kept]]`.

## What the crawl says the ORDER must be

```
1c-0   the corpus-rot stone — dispose of the 5 dead call sites and rule on the
       :wat::type:: rendering. SMALL, and it must precede 3a, which turns each
       of them into a build failure.
1c-a   the 11 that already have a scheme — the 1b-i shape, drains two ledgers.
1c-b…  the 20 without, in mechanism-sized batches, NOT one brief.
```

⬜ **Nothing is briefed by this document.** The next act is the builder's ruling on the order
above, and on whether the `:wat::type::` question belongs to arc 251 rather than here.
