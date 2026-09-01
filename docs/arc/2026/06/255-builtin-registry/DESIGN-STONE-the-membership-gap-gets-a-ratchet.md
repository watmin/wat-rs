# DESIGN — STONE: the membership gap gets a ratchet, and `fn`/`match` prove it moves

> **Builder, 2026-09-01:** *"we continue onwards... the registry unblocks many things... the onslaught
> against the megafiles wages on.... we plan our strikes and execute, ruthlessly..."*
>
> Target: `[[NOTE-the-registry-is-not-yet-the-largest-membership-set]]`. The arc's founding
> deliverable — delete the reserved-prefix blanket-accept — blocked on the registry not knowing
> what the rest of the system knows.

## ⛔ Why a registration stone CANNOT go first

Registering `fn` changes nothing observable. `resolve`'s `if is_reserved_prefix(head) { return true }`
short-circuits **before** the registry is consulted, so a new row is invisible until the flip — and
the flip cannot happen until the gap is closed. **A stone whose effect nothing can see is
unfalsifiable, and this campaign has shipped that before.**
`[[feedback_a_design_is_unfalsifiable_until_something_consumes_it]]`

So the first strike is the **instrument**: the thing that makes every later registration visibly
move a number, and that fails when the number moves and nobody updates it.

## The two populations, measured

```
GAP A — has a checker TypeScheme, no registry row ......... 68   structurally derivable
GAP B — a corpus call head the registry cannot vouch for .. 121  measured by experiment
```

★★★ **Gap A is the builder's sentence made mechanical:** *"the registry is not even the largest
membership set."* When A is empty, `registry ⊇ check_env`. It needs no hand-list at all — it is
`{n : check_env.get(n).is_some() ∧ registry().lookup_entry(n).is_none()}`, computed in-process.

Gap B is the real finish line (121 → 0 lets `resolve` flip), and it was measured by patching
`resolve` to consult the registry and running all 599 corpus files: **578 failed**, and the deduped
unresolved names ARE the list. That experiment is the re-measure procedure; it is recorded here so
the number can be re-derived rather than trusted.

## THE ONE CONTRACT DECISION — pinned

**Both gates freeze NAMES and are bidirectional. Neither freezes a count.**

A count cannot tell "+1 new, −1 fixed" from "nothing happened", and its failure message cannot name
the offender — `[[feedback_a_gate_freezes_names_never_a_count]]`. ⚠ And this campaign proved the rule
applies to the floor's own total three stones ago: a test was silently disarmed while the floor read
5114 both before and after, because a new test replaced it one-for-one.

So, exactly as `checker_skip_debt_is_named_and_frozen` already does:

- a name in the gap but NOT frozen → **NEW**, named, fail.
- a frozen name no longer in the gap → **STALE**, named, fail until deleted.

**Every registration stone deletes names from a frozen list, or it goes red.** That is the ruthlessness
made structural: a stone cannot claim progress it did not make, and cannot make progress silently.

## Scope — the instrument, plus two registrations that prove it moves

1. **`REGISTRY_MEMBERSHIP_GAP_A`** — the 68, with a gate deriving the population in-process.
2. **`REGISTRY_MEMBERSHIP_GAP_B`** — the 121 corpus names, frozen, each asserted still unregistered.
3. **Register `:wat::core::fn` and `:wat::core::match`** as `#[wat_special_form]` rows in the
   existing `src/intrinsic/special/` home, and delete both from Gap B — **which the gate will
   REQUIRE**, because leaving them frozen after registering them fails as STALE.

★ `fn` and `match` are chosen on measurement, not convenience: both already carry **named impls on
both sides** (`crate::function::{eval_fn,infer_fn}`, `eval_match_tail`/`infer_match`), so the
registration is annotation, not extraction — and they are the two largest names in the corpus
(**3,952** and **1,613** call sites). ⚠ `map`/`mapv` also measured "ready" but are ordinary HOFs
needing `#[wat_intrinsic]`, a different attribute; they are not in this stone.

## ⛔ What this stone does NOT do

- **It does not flip `resolve`.** Gap B is 121, not 0. Flipping today fails 578 of 599 corpus files —
  measured, not feared.
- **It does not touch the blanket-accept.** That is the LAST stone of this thread, not the first.
- **It does not register the other 119.** Each is its own strike; the ratchet is what makes them
  countable.

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **the ratchet + `fn`/`match`** | YES | YES | YES | YES | ✅ **ADMITTED** |
| register `fn`/`match` first, ratchet later | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| flip `resolve` now and meet the cascade | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| ratchet alone, no registrations | YES | YES | YES | **NO** | ⛔ DISQUALIFIED |
| one gate over the union of A and B | YES | **NO** | YES | — | ⛔ DISQUALIFIED |

- **registrations-first Honest? NO** — nothing observes them; the stone's own claim would be
  unfalsifiable, which is the defect the ratchet exists to remove.
- **flip-now Honest? NO** — 578/599 measured. Shipping that is not ruthlessness, it is a red tree.
- **ratchet-alone Good UX? NO** — Obvious/Simple/Honest hold, so this is a real cut: a gate that has
  never been moved by a real stone is a gate nobody has proven can be satisfied.
- **one-union-gate Simple? NO** — A is derived structurally and needs no list; B is a frozen corpus
  snapshot with a re-measure procedure. Fusing them hides which half a failure came from.

## Acceptance — rows chosen to be unfakeable

| what | command | expected |
|---|---|---|
| Gap A is derived, not listed | the gate computes it from `check_env` × `registry()` | 68 names, no hand-list |
| Gap B is frozen by name | `REGISTRY_MEMBERSHIP_GAP_B` | **119** after this stone (121 − `fn` − `match`) |
| ⛔ both gates fail BOTH ways | sabotage each: drop a name; add a resolved one | 4 sabotages, each names the offender |
| `fn` and `match` are rows | `registry().lookup_entry` each | `Some`, `Kind::SpecialForm` |
| the special-form contract holds | the existing `#[wat_special_form_impl]` check+eval gate | green — both roles annotated |
| ⛔ nothing observable changed yet | `wat --check` on `(:wat::holon::Bogus 1 2)` | still ACCEPTED — the flip is a later stone |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5116+ passing, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |

★ **The last acceptance row is the important one.** This stone must leave `Bogus` accepted. A stone
that quietly closed the hole early would be a stone whose cascade nobody measured.
