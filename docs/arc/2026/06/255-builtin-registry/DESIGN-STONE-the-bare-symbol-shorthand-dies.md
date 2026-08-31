# DESIGN — STONE: the bare-symbol shorthand dies

> ⛔ **THIS DESIGN SAID "BOTH FAMILIES". THERE ARE THREE.** Corrected in place 2026-08-30 after the
> rider found a third live door and the orchestrator verified it. **Two doors are closed; the heresy
> is NOT dead.** Read `## ⛔ THE THIRD DOOR` before treating this stone as complete.

## ⛔ THE THIRD DOOR — the CEK stepper, still live after this stone

`:wat::eval::walk` / `:wat::eval-step!` (arc 068/070, `is_match_canonical` + `try_match_pattern_ast`,
`src/runtime.rs:~23689,23725`) carries its **own** bare-symbol recognition, structurally separate
from the two families below. Verified live, post-rebuild, with both doors closed:

```clojure
(:wat::eval::walk '(:wat::core::match (Some 5) ((Some n) n) (:wat::core::None 0)) 0 :my::v)
;; => #wat.core.Result/Ok [[5 2]]        the pattern fired, n bound to 5
```

★ **And it is the heresy, not generic structural matching** — discriminated with a made-up head:

```clojure
… (Zorble 5) ((Zorble n) n) …   =>  Err "eval-step! has no rule for op: symbol-head:Zorble"
```

`Some` is special-cased where `Zorble` is not. Had I not run that control, "the stepper just matches
forms structurally" would have been a plausible and wrong dismissal.

⚠ **Also correcting my own two mis-citations in the sections below**, both found by the rider
verifying rather than trusting:
- **The population was 4 sites in 2 files, not 5 in 3 plus a Rust fixture.**
  `wat-scripts/perf/grid/where-control.wat`'s "site" is a `;;` comment (its live code is already
  FQDN), and `tests/function/wat_arc170_closure_extraction.rs` contains **no inline wat at all** —
  its fixtures load external `.wat` files already in FQDN form. The same contamination this design
  warned about, one paragraph after warning about it.
- **`src/check.rs:5896,5921,5941,6206` pointed at the wrong code** — those are inside `infer_list`,
  the already-closed CONSTRUCTOR door. The open pattern-door arms are 300–900 lines further down in
  `pattern_coverage` and `check_subpattern`.



> **Builder, 2026-08-30:** *"we should just do that `(Some ...)` NOTE now?.... this is an active
> heresy .... it must go..."*
>
> Ruled. `NOTE-the-bare-symbol-constructors-are-retired-at-the-door-and-live-behind-it.md` recorded
> half of it; the pre-flight for this stone found the other half, which is worse.

## ★ THE RETIREMENT WAS HALF DONE — measured, and this is the finding

Arc 109 slice 1h's own remedy names **two** site kinds:

> *"rename `(Some x)` → `(:wat::core::Some x)` at constructor sites; rename `((Some v) ...)` →
> `((:wat::core::Some v) ...)` at **match-pattern sites**"*

| door | status | measured |
|---|---|---|
| **constructor** `(Some 1)` | closed at check time, **live at runtime** | refused with a remedy; but `(:wat::eval-ast! '(Some 99))` → `Some [99]`, exit 0 |
| **match pattern** `((Some v) …)` | ⛔ **NEVER CLOSED** | `(:wat::core::match (Some 5) ((Some v) v) …)` → `5`, **exit 0, checks clean** |

**The remedy text has been instructing people to migrate pattern sites that nothing ever refused.**
A wall's paperwork claiming a door it did not close.
`[[feedback_a_walls_paperwork_can_claim_a_door_it_did_not_close]]`

## The population — validated, not grepped

```
bare-symbol PATTERN sites in .wat ....... 5   wat-scripts/perf/grid/where-control.wat,
                                             tests/cli/wat_cli__programs_are_atoms.wat,
                                             tests/cli/wat_cli__presence_proof.wat
bare-symbol CONSTRUCTOR sites in .wat ... 0   (the check-time door already forbids them)
Rust inline-wat fixtures ................ tests/function/wat_arc170_closure_extraction.rs
```

⚠ A first pass counted **20** constructor sites. **14 were `;;` comment lines and the rest were
English prose** — `(Ok path)` inside a trailing comment, `Ok(Some v)` in a doc block. The honest
instrument was the checker, not grep: a live bare constructor in a gated `.wat` would make the floor
red, and the floor is green.
`[[feedback_validate_a_search_pattern_before_trusting_its_count]]`

## THE ONE CONTRACT DECISION — pinned

**The pattern door closes exactly the way the constructor door already did** — a check-time error
carrying the `RETIREMENT_TABLE` remedy, so both spellings fail identically and the existing remedy
text becomes true for the first time. No new mechanism: the retirement machinery, the remedy, and
the error shape all already exist and are already proven on the constructor half.

## What ships, in this order

**Order is load-bearing:** closing the door before migrating the corpus makes the corpus illegal.

1. **The codemod** — `wat-scripts/fixes/bare-symbol-shorthand-to-fqdn.wat`, migrating the 5 pattern
   sites. ⛔ R21: the `.wat` corpus moves by codemod, never a hand-edit.
2. **Close the pattern door** — `src/check.rs:5896,5921,5941` (and the `:6206` `matches!`), which
   today *accept* a bare-symbol pattern head, refuse it with the retirement remedy instead.
3. **Delete the runtime arms, both families:**
   - constructors — `src/runtime.rs:5183,5186,5189` (`eval_list`)
   - patterns — `src/runtime.rs:16102,16135,16164,16192,16195,16196` (`try_match_pattern`)
4. The Rust inline-wat fixture moves with them.

⚠ Step 1 and step 2 in one commit means the codemod runs against a checker that still permits the
old form — which is the supported order. If the sequencing fights, `wat/fix.wat`'s header
**BOOTSTRAP / STASH-DANCE** note is the documented path; do not hand-edit `.wat` to escape it.

## What this closes, beyond the heresy

`:wat::core::Some` was homed and ruled this session. The bare spelling was not, so the two disagree
on a reachable path — `(:wat::core::Some 1)` is `pure? true`, `(Some 1)` is `pure? false`. **One
constructor answering two ways is two slots**; deleting the second spelling is what makes it one.
`[[feedback_a_slot_with_two_implementations_is_two_slots]]`

## Out of scope = REJECTED (not deferred)

- **`:wat::core::None`** — not a bare-symbol shorthand of this family; its `eval_list` occurrence is
  a pattern-clause head inside `match`'s own implementation, excluded by `meter-2` with a cited
  reason.
- **THE ROAD's step 4, "every call head a symbol"** — a future arc makes bare-symbol heads the
  NORMAL form. ⛔ **That is not a reason to keep these.** They must die for the right reason — a
  retired shorthand the checker refuses — rather than be kept for the wrong one, that they resemble
  the coming syntax. A future symbol-headed surface will be designed, not inherited from a graveyard.

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **close the pattern door, then delete both families** | YES | YES | YES | YES | ✅ **ADMITTED** |
| delete the runtime arms only, leave the checker | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| keep them for THE ROAD's symbol-headed future | **NO** | YES | **NO** | — | ⛔ **DISQUALIFIED** |

- **runtime-only Honest? NO** — the checker would keep *accepting* a pattern the runtime can no
  longer evaluate: a form that type-checks and then fails at run time is worse than either door
  alone.
- **keep-for-the-future Obvious? NO** (a reader cannot tell live surface from graveyard);
  **Honest? NO** — it preserves a retired spelling under a justification nobody has designed yet.

## Acceptance

| what | command | expected |
|---|---|---|
| the pattern door is closed | `(:wat::core::match … ((Some v) v) …)` under `--check` | check-time error naming `:wat::core::Some` |
| the constructor door still teaches | `(Some 1)` under `--check` | unchanged remedy |
| ⛔ **the runtime path is dead** | `(:wat::eval-ast! '(Some 99))` | **no longer evaluates** — this is the heresy's actual death |
| one slot, not two | `pure?` on both spellings | FQDN `true`; bare form **refused, not `false`** |
| the corpus moved by codemod | `git diff` on the 3 `.wat` files | only the intended rename |
| codemod is idempotent | re-run, `md5sum` | byte-identical |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5109/5109, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |
