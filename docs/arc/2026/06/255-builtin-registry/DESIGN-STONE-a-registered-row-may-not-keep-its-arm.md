# DESIGN — STONE: a registered row may not keep its dispatch arm

> **Builder, 2026-09-01:** *"the dead arm sweep... more megafile lines die...."*
>
> ⛔ **The premise needs correcting first, and the correction is mine to make.** The sweep is
> **5 lines**, not the bonanza the framing implies. The real deliverable is the wall that makes
> every future registration sweep its own arm.

## What is actually dead — measured three times, wrong twice

```
464 registered rows.  FIVE have a surviving dead dispatch arm:
   :wat::core::record?  1975      :wat::core::u8    2321      :wat::core::bool::to-string  2377
   :wat::core::not      2427      :wat::core::show  2670
```

One line each. The ~65 lines those arms *span* are almost entirely historical retirement
commentary (*"Arc 255 Stone C — the old `:wat::core::i64::*` arms that lived here are RETIRED…"*) —
Bucket C context that FM 14 says to keep, not code.

⚠ **Two wrong measurements got there first, and both are the same class:**

1. A first probe asked *"does this name appear as an arm anywhere in `runtime.rs`?"* and returned
   **6**, including `:wat::holon::Blend`. Blend's only arm is in **`step_list`** — a different match
   thousands of lines away, not the registry-first door. `[[feedback_a_census_without_attribution_is_not_a_census]]`
2. That same probe would have called `fn`, `if`, `let` and `match` dead. **They are not.** The
   registry door is `registry().lookup(head)` → *handler*, and a `Kind::SpecialForm` row carries
   `handler: None`, so it never dispatches through it. **Sweeping those four would have deleted the
   dispatch for the language's core syntax.**

★ Three matches in `runtime.rs` dispatch on these names — `dispatch_keyword_head_value` (registry-first
lives here), `eval_tail`, and `step_list`. `u8` has an arm in two of them: one dead, one live.
`[[feedback_a_slot_with_two_implementations_is_two_slots]]`

## ★★★ The real finding: the convention held for 459 rows, and I broke it

Only five rows carry a dead arm, and **all five are from the stone immediately before this one.**
Every earlier registration swept its own arm by hand. The convention has been working — and my brief
broke it, deliberately: I scoped it *"attributes only, no bodies"*, the rider obeyed, and then
flagged the residue rather than cleaning up outside its blast radius.

**That is the convention rung failing exactly as `extirpare` predicts:** it holds until one hand
doesn't know it, and nothing in the tree can tell.

## THE ONE CONTRACT DECISION — pinned

**A registered row that carries a handler may not also have a literal arm in the registry-first
dispatch function. The gate asserts it; the five deletions are its first payment.**

The predicate needs BOTH halves, and each was learned by getting it wrong:

```
entry.handler.is_some()                         ← a SpecialForm row has none; its arm is LIVE
∧  the arm lies inside dispatch_keyword_head_value   ← not eval_tail, not step_list
```

## ⛔ What this stone does NOT do

- **It does not delete the historical comments.** They record what was retired and why; FM 14
  Bucket C.
- **It does not touch `eval_tail` or `step_list` arms.** Different matches, different reachability.
- **It does not sweep `fn`/`if`/`let`/`match`.** No handler, live arms, and the gate must exempt
  them by *measuring* `handler.is_some()` rather than by naming them.

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **gate + the five deletions** | YES | YES | YES | YES | ✅ **ADMITTED** |
| delete the five, no gate | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| gate only, defer the deletions | YES | YES | YES | **NO** | ⛔ DISQUALIFIED |
| exempt `fn`/`if`/`let`/`match` by NAME | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| also sweep the historical comments | **NO** | YES | **NO** | — | ⛔ DISQUALIFIED |

- **no-gate Honest? NO** — it fixes five instances of a class that just proved it recurs the moment a
  brief forgets to mention it. The convention rung already failed once, measurably.
- **gate-only Good UX? NO** — a gate that has never been satisfied by a real deletion is a gate
  nobody has proven can go green; the five are the proof.
- **exempt-by-name Honest? NO** — a name list rots the moment a fifth special form registers.
  `handler.is_some()` is derived from the row itself and cannot drift.
- **sweep-the-comments Obvious? NO, Honest? NO** — they are the record of what was retired; deleting
  them is the revisionism FM 11's corollary forbids, and a future reader would re-add the arms.

## Acceptance — rows chosen to be unfakeable

| what | command | expected |
|---|---|---|
| the five arms are gone | `grep -c '":wat::core::\(record?\|u8\|bool::to-string\|not\|show\)" =>' src/runtime.rs` | drops by 5 |
| ⛔ the LIVE four are untouched | `fn`/`if`/`let`/`match` arms in `dispatch_keyword_head_value` | still present |
| ⛔ `step_list` + `eval_tail` untouched | their arms for the same names | unchanged |
| the gate is derived, not listed | it reads `handler.is_some()` + the enclosing fn | no exemption name list |
| ⛔ the gate can FAIL | re-add one deleted arm | red, naming that row |
| ⛔ the gate does not fire on a SpecialForm | it stays green with `fn`'s arm present | green |
| the comments survive | the Stone C/D retirement notes | present, unchanged |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5118/5118, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |

★ **The megafile gain is 5 lines and the stone says so.** Its worth is that the next twenty
registrations cannot leave their arms behind, and the sweep stops depending on whoever writes the
brief remembering to ask.
