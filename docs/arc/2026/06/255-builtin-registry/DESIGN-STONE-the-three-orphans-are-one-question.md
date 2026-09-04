# DESIGN — STONE: the three orphan `core_name` targets are ONE question with three answers

> **Builder, 2026-09-04:** *"we step forward.."* — executing item 1 of
> `[[SEQUENCING-the-only-chain-that-gates-the-founding-target]]`.
>
> Governed by `[[RULING-the-registry-is-the-sole-authority]]`. Three RETE_OPS rows point at a
> `core_name` the registry cannot vouch for. **Three rows gate twenty-nine**, and through them the
> corpus and Phase 3a — the arc's founding target.

## The gate, and why it is latent rather than red

`no_dangling_or_chained_aliases` (`src/intrinsic/mod.rs:2119`) **panics** on an alias row whose
target is not a registered row. It is GREEN today — measured — because the three rete rows are
themselves unregistered, so the gate never sees them. **It fires the moment Phase 1b tries to
register them.**

## The three, measured — and they are NOT three problems

```
:wat::core::Vector   a bare TYPE CONSTRUCTOR
:wat::core::cond     a stdlib DEFMACRO         wat/core.wat:1455
:wat::core::reduce   a wat-side DEFALIAS       wat/seq.wat, since Stone 1c-f (this session)
```

Each is a name whose authority is **not** the intrinsic registry. But they do not resolve the same
way, and the difference is the whole stone.

### ★★★ `Vector` is not an exemption — it is an INCONSISTENCY, and it SHIPS

`purity.rs`'s own residue comment calls the bare type constructors *"unhomed"* by design. **Measured,
that rule is already broken 4 ways:**

```
:wat::core::List               REGISTERED   src/intrinsic/list.rs
:wat::core::PersistentVector   REGISTERED   src/runtime.rs   ← Stone 1c-b-i
:wat::core::PersistentMap      REGISTERED   src/runtime.rs   ← Stone 1c-b-i
:wat::core::Tuple              REGISTERED   src/runtime.rs   ← Stone 1c-a-ii
:wat::core::Vector             —
:wat::core::HashMap            —
:wat::core::HashSet            —
```

The campaign has been registering these one at a time for stones. `Vector`/`HashMap`/`HashSet` are
simply the remainder, and `PersistentVector` — same shape, same file — is a shipped precedent to copy
(`@Purity Pure` · `@Determinism Deterministic` · `@Totality Total` · `@ExpandTime Legal` ·
`@Category Transform`). **All three are also in `intrinsic_meta`'s `pure_det` residue**, so
registering them makes the 1c-c residue gate demand their deletion — the campaign's own mechanism
drives the cleanup rather than a hand-list edit.

⚠ **The axes are NOT to be copied blind.** `PersistentVector`'s grades are its own; each of the three
gets its own measured argument against its own constructor. A copied `Total` is exactly the class of
claim this arc has corrected four times.

### ⛔ `cond` and `reduce` are the FOURTH REGISTRY, and they do not ship here

```
:wat::core::cond     defmacro, wat/core.wat:1455.   51 corpus call sites.
                     registry row NO · checker scheme NO · infer arm NO · runtime dispatch NO
:wat::core::reduce   defalias, wat/seq.wat.
                     `register_defalias` writes to `sym.functions`, NEVER `registry()`
                     (proven by the precedent `:wat::core::count`, a wat alias with no row).
```

Neither can hold a registry row under any mechanism that exists today. That is
`[[NOTE-there-is-a-FOURTH-registry-and-it-holds-defn]]` — 41 stdlib macros invisible, and every
wat-defined verb with them. **This stone does not open that fork; it records that two of the three
orphans ARE it**, which is new: the fork was known as a reflection gap and is now also a measured
blocker on the chain to the founding target.

⚠ And `reduce` is a blocker **this session created** — Stone 1c-f turned it from a `defclause` into a
`defalias`, correctly, and a `defclause` was equally unregisterable, so the blocker predates the
stone. Recorded because a later reader will otherwise date it wrong.

## THE FOUR QUESTIONS — on the shippable half

| | Obvious? | Simple? | Honest? | Good UX? |
|---|:---:|:---:|:---:|:---:|
| **register Vector · HashMap · HashSet** | YES | YES | YES | YES |

- **Obvious? YES** — four of their seven siblings are already registered by this campaign.
- **Simple? YES** — one shipped precedent in the same file; no new mechanism.
- **Honest? YES** — the "stays unhomed" comment describes a rule the campaign stopped following
  four stones ago. Leaving three rows out while their siblings are in is the inconsistency.
- **Good UX? YES** — `Vector` stops being an orphan target, and the residue shrinks by three.

## What this stone changes on the chain

```
BEFORE   3 orphan core_name targets gate ~29 RETE_OPS rows gate the corpus gate Phase 3a
AFTER    1 orphan — :wat::core::Vector — CLEARED.
         2 orphans — cond, reduce — REMAIN, and are now NAMED as the FOURTH-registry fork
         rather than as three unexplained absences.
```

★ **The chain is not unblocked by this stone, and saying so is the point.** It converts "three
mysterious orphans" into "one shipped, two blocked on a named fork" — which is what makes the next
decision possible instead of guessed at.

## Scope

**In:** `:wat::core::Vector` · `:wat::core::HashMap` · `:wat::core::HashSet` registered, each with
its own measured axes · their three `pure_det` residue entries removed as the gate demands · the
`purity.rs` comment that calls them "unhomed" corrected · a NOTE recording the three-way split.

**Out, affirmatively:** `cond` and `reduce` — no mechanism exists · the FOURTH-registry fork itself ·
the ~29 RETE_OPS rows (still blocked by the two) · the six non-verb artifacts (item 2 of the
SEQUENCING map, an independent prerequisite).
