# DESIGN — STONE expand-1: three lies leave the expand-time allow-list, and it gets its real name

> **Builder, 2026-08-30:** *"this is the kind of audit i've been waiting for… we never applied
> strict scrutiny to these… now we are doing it. get these lies out."*

## The audit

`src/macros/eval.rs`'s `is_pure_total` is a 202-name **default-deny allow-list** gating what may be
called inside a `defmacro` body at expand time. Every blessed name was compared against its own
registration:

```
LISTED_BUT_EFFECTFUL          0
LISTED_BUT_NONDETERMINISTIC   4    macro-call-site · fresh-symbol · hashmap::keys · hashmap::values
LISTED_BUT_TOTAL_PARTIAL      1    :wat::i64::/
```

★ **Zero effectful verbs are blessed.** The default-deny half of the discipline has held perfectly
across 202 entries — worth stating, because what follows is about the other half.

## ⛔ THE THREE ARE NOT ONE DEFECT, AND THEY DO NOT GET ONE DISPOSITION

### `hashmap::keys` · `hashmap::values` — DRIFT. They leave.

Added in `110335bd5` while believed deterministic. `afc9f776b` — *"CORRECT(255): keys/values were
classified DETERMINISTIC and are not"* — corrected the ruling **and did not sweep this list.** A
seventh table, stale for the same reason the other six were.

★ **And nondeterminism here is disqualifying for a specific reason, not by category.** A macro must
expand **reproducibly**: the same source must generate the same code every run. `keys`/`values`
iterate in hash order, which the substrate says outright is *"deliberately NOT part of the
contract"* — so a macro body folding over them emits **different code on different runs**. That is
not a purity concern; it is a build-reproducibility one, and it is why these two cannot be blessed.

### `:wat::core::fresh-symbol` · `:wat::kernel::macro-call-site` — PRINCIPLED. They stay.

Also nondeterministic; also blessed; **correctly.** `fresh-symbol` mints a capture-proof gensym —
*"the SAME `base` argument mints a [different symbol each time]"* — and that nondeterminism **is the
point**, and is exactly what makes hygienic expansion possible. `macro-call-site` is the
expand-time twin of a source-location reader.

⚠ **So nondeterminism is not the discriminator.** Two nondeterministic verbs stay and two go, and
the axis separating them is whether the nondeterminism makes the EXPANSION vary. That is the
sharpest evidence yet that expand-time legality is **an independent property**, not a coarser view
of purity.

### `:wat::i64::/` — NOT A LIE. **The list's NAME is the lie.**

`i64::/` is pure, deterministic, and `@Totality Partial` (undefined at a zero divisor — transcribed
from its own doc in stone total-T2b). A list called `is_pure_total` blessing a partial verb is a
contradiction **only because of what it calls itself.**

★ **Dividing by zero at expand time is not a defect — it is a compile-time error instead of a
runtime one, which is better.** The verb is legitimately expand-time-legal *and* legitimately
partial. The two claims never conflicted; one name pretended they were the same claim.

**So `i64::/` stays and the FUNCTION is renamed.** This is the fix that dissolves the contradiction
I could not resolve this morning when the two lists appeared to disagree about `i64::/`: they were
never disagreeing. They were answering different questions under one name.

## What ships

```
REMOVE   :wat::hashmap::keys · :wat::hashmap::values      irreproducible expansion
KEEP     :wat::i64::/                                     partial AND legal; no conflict
RENAME   is_pure_total  ->  is_expand_time_legal          and rewrite the header's claim
```

The header currently reads *"ONLY the **pure-total subset** of `dispatch_keyword_head`…"* — which is
false in two directions at once: it blesses a partial verb and two nondeterministic ones, and it
omits 174 pure∧deterministic verbs. The rename is not cosmetic; it is what lets the list be judged
against what it actually decides.

## ⚠ AND A CLAIM IN THAT HEADER THAT THIS STONE MUST TEST

> *"The suite teaches completeness: a false-refusal (a pure head missing from this list) makes a
> stdlib test RED. Add it here."*

**That is testable, and it has never been tested.** Removing `keys`/`values` either breaks a macro
body that calls them — proving the mechanism works — or it does not, proving the mechanism only
covers verbs the corpus happens to exercise. **Either outcome is a finding**, and it is the same
"is this gate real?" question that caught the `p3b` probes and the `open-file` tests today.

## Out of scope = REJECTED

- **Minting `@ExpandTimeLegal`.** The next stone. This one makes the list honest so the axis lands
  on something that is not lying.
- **The 174-verb gap.** Real, measured, and untouched here — closing it needs the declared axis,
  not more hand-curation.
- **`i64::/`'s totality.** Settled in T2b; this stone does not revisit it.

## Calibration

Predicted 25–40 min. Three entries and a rename; the header rewrite and the completeness-claim test
are the work.
