# SCORE — D2 cured, and it was a shipped wrong answer at the public API

> **Written after the orchestrator's own weighing.** The ★ was that this strike's own framing — *"the
> differential could not have seen this"* — is **false**, and the rider proved it by driving both
> binaries.

## ⛔⛔ D2 WAS A WRONG ANSWER AT THE PUBLIC SURFACE, NOT AN INTERNAL INVARIANT VIOLATION

Every artifact rowed a multiplicity-sensitive grid column as future work on the premise that no
existing observable could see this. **Driven, on both binaries** (`git stash` the cure → rebuild at
HEAD → run → pop → verify 11 files byte-identical, stash list empty):

| | `Hit` | `Hit2` | **chain-rows** |
|---|---|---|---|
| HEAD `f4a271cb3` native | 12 | 6 | **18** |
| HEAD `f4a271cb3` oracle | 12 | 6 | 12 |
| cured native | 12 | 6 | **12** |
| cured oracle | 12 | 6 | 12 |

**A user query returned 18 rows where the spec says 12 — a 50% over-count, shipped.** Reproduced on
the cured tree by the orchestrator: `native … chain-rows=12` / `oracle … chain-rows=12`.

### The fifteenth false claim is this strike's own framing

`:derived` **is** blind — the `Hit`/`Hit2` columns confirm it exactly. **A chain-mirroring query is
not**, and the 2026-08-31 drive *used precisely that observable*. It returned `[1 1 1]` == `[1 1 1]`
because its shape produced the **vacuous partition**, not because the observable could not see.

**The DESIGN flagged the stagger as load-bearing two paragraphs above its own conclusion and then did
not apply it there.** Consequence: the rowed follow-up is scoped wrong. The port check needs **the
stagger**, not a new column — and that is much cheaper than rowed.

## The scorecard, graded

| # | required | result |
|---|---|---|
| 1 ★ | the invariant holds | ✅ `mark == Σ bucket lengths`, every join, every round |
| 2 ★ | the test un-ignored and GREEN | ✅ **attribute deleted** (the three remaining `ignore` mentions are prose recording that the banking was an error). Floor **5420 run / 21 skipped**, was 5419 / 22 |
| 3 ★ | the bypass **unrepresentable** | ✅ **proven by a compiler error** — see below |
| 4 | the cure is what holds it | ✅ mutation 2b REDs naming J6 `12 vs 18`, J11 stays cured, J4/J9 stay clean |
| 5 | the verb really maintains | ✅ mutation 3 REDs — every mark 0, maintainer re-pushing every visit |
| 6 | facts do not move | ✅ `Hit=12 Hit2=6` identical HEAD vs cured, native and oracle; `check-spec-native.sh` **38 families / 315 rows**, rc=0 |
| 7 | controls still discriminate | ✅ J4/J9 unchanged at 12/12; J11's maintainer re-push row is now **absent** |
| 8 | floor / lints / clippy | ✅ **`5420 tests run: 5420 passed, 21 skipped`** (458.7 s), 0 FAIL, lints **265**, clippy rc=0 |

## ★ Unrepresentable, proven by the compiler

`JoinRightIndex` (`session.rs:242`) owns `buckets` and `indexed_n` as **private** fields. The only
door is `writer(join_id) → RightIndexWriter`, whose `push(key, el)` appends **and** increments in one
statement. No accessor hands out `&mut` to the buckets.

**Mutation 2a — the pre-cure bypass written verbatim:**

```
error[E0616]: field `buckets` of struct `session::JoinRightIndex` is private
```

**To run mutation 2 at all the rider had to ADD an escape hatch the cure deliberately lacks.** That is
the difference between the convention rung and the top one — and it is why patching the two known
call sites would have been the wrong cure: this defect already survived `partire` because nothing
structural forbade a third writer.

## The rider's own RED, captured not re-run

```
FAIL wat::lint rete_header_claims_are_asserted::fire_mod_cfg_test_sites_are_exactly_the_documented_set
`fire/mod.rs` has 10 `#[cfg(test)]` sites, expected 9 … left: 10  right: 9
```

Real, caused by the change, and the gate was right. Fixed as the gate instructs — count **and** module
header together — **and strengthened**: `"RIGHT_IDX_SITE_MAINTAINER"` added to the named-set list so
the set cannot swap shape while keeping the count.

## ⚠ A GUARD WAS CHANGED, AND THE RIDER FLAGGED IT RATHER THAN BURYING IT

The acceptance test's non-vacuity check read *"the maintainer visited J"* off `indexed_n[J].is_some()`
— sound only while the maintainer was the mark's **sole** writer. **After the cure every writer
advances it**, so that guard would have kept passing on a workload where the maintainer never ran: a
guard that cannot fail, created by the cure.

Re-based on the maintainer's own census row, with that row excluded from `bypass_appends` so the
maintainer cannot count as its own bypass. **Strictly stronger, and directly observed rather than
inferred.**

## STOP-4 — hot-path shape, reported not sold

Per element: one `*self.indexed_n += 1` against a pointer resolved **outside** the loop. The two
`entry()` calls sit exactly where `right_idx.entry(join_id).or_default()` already sat — **no map
lookup added**. It *removes* one hash lookup per block (the maintainer's `indexed_n.insert`) and
removes the duplicate pushes themselves. **No number is claimed, because none was measured.**

## Honest deltas

- **`index_upto(join_id, &[Element])` is not implementable as written** — both artifacts named it. No
  call site has a ready `&[Element]`; all three compute the `JoinKey` and apply
  `element_with_row_span` per element with `&mut wm.bind_pool` live in the loop. A slice-taking verb
  would drag the keying context into `session.rs`. A writer guard gives the same guarantee with
  `push` as the single door.
- **Line citations drift.** The step-2 site is `:296` in the work list, `:298` in DESIGN/BRIEF, and
  is actually the `entry` at **`:308`**. `session.rs:210` is the doc comment; the alias was `:211`.
  Only `:185` and `fire/mod.rs:799/802/815` were exact.
- **A stale bolded totality in the original row** — *"`indexed_n` … never read back as anything but
  0"*, INCREMENTAL 0 across four workloads. J6 round 1 is `already=6 < len=12`: the incremental
  branch. True of its four workloads, false as stated — **and it is what supported "latent, not
  live."**
- The new probe `wat-scripts/scratch-pad/d2-derived-fact-axis.wat` is kept: it is the only instrument
  that shows D2 at the public API, and both `wat-scripts/` gates were re-run with it present.
