# BRIEF — 293: eight fixtures were hollowed by another arc's migration, and they are GREEN

> Builder, 2026-08-16: *"these should go first... these are worse than ignore."* He is right — an
> ignored test is **visibly** absent; a hollowed fixture is a **passing test that proves nothing**, and
> it inflates the floor's own number. Baseline HEAD `e9068f0f`, tree clean, floor
> **4566 run / 4566 passed / 122 skipped**, clippy 0.

## THE FINDING

`3cd00fbb` (2026-07-10, *"arc 170: the `:user::main` wall — a useless/illegal main is now
UNCOMPILABLE"*) was a correct wall. The migration made 25 fixtures compliant, and for some of them
**compliance meant deleting the `main`** — which was the **driver**. The call site went with it.

**Eight fixtures lost a real driver.** The deleted body IS the call site, verbatim from the diff:

| fixture (`tests/types/`) | the call that was deleted |
|---|---|
| `probe_arc293_ctor_parity_newtype.wat` | `(:my::Amount 42)` |
| `probe_arc293_ctor_parity_struct.wat` | `(:geo::SPt/x (:geo::SPt 3 4))` |
| `probe_arc293_holder_bound_accept.wat` | `(:env::wants-holon (:env::HEnv 1))` |
| `probe_arc293_holder_bound_reject.wat` | `(:env::wants-holon (:env::CEnv 1))` |
| `probe_arc293_record_surface_core.wat` | `(:geo::describe (:geo::Circle "red" 2.0))` |
| `probe_arc293_record_surface_holon.wat` | `(:geo::describe (:geo::HCircle "red" 2.0))` |
| `probe_arc293_structtype_primitive.wat` | `(+ (:my::Point/x (:my::Point 3 4)) (:my::Point/y …))` |
| `probe_arc293_structural_surface.wat` | `(:geo::accepts-shape (:geo::Circle "red" 2.0))` |

Confirmed hollow **today**, not inferred from the diff. `probe_arc293_ctor_parity_struct.wat` is now
**one `defstruct` and nothing else** — a *constructor-parity* probe with no constructor call in it. Its
test asserts that a `defstruct` parses.

## ⛔ WHY NOBODY NOTICED — the mechanism, and it generalises

**Every one of the eight is a `.wat`, not a `.wat.bad`.**

- A `.wat.bad` that loses its driver stops producing its expected error → **goes RED** → gets noticed.
- A `.wat` that loses its driver **still loads clean** — trivially, because it only declares things —
  and its test's `is_ok()` **passes**.

**The migration only hollowed the positive fixtures, and positive fixtures fail by passing.** That is
the whole reason this survived 37 days inside a green floor.

## NOT IN SCOPE — classified, leave them alone

Five other files net-lost a main and are **fine**. Do not "restore" them:

- `wat_arc170_slice_1e_user_main_nil_{legacy_3arg,slice2_4arg,wrong_return}.wat` — arc 170's **own**
  main-signature fixtures. Deleting/changing the main **is** their subject.
- `probe_arc296_pure_surface_field.wat` — its own comment says *"the purity fix is exercised at
  REGISTRATION time"*. No driver required.
- `probe_arc170_c1_kwargs_bracket.wat`, `probe_arc293_acceptance_demo.wat` — the deleted body was
  empty/comment-only.

`probe_arc296_record_in_surface_vector.wat` and `probe_arc170_parametric_surface.wat` are
**UNCLASSIFIED** — the second's deleted comment reads *"POSITIVE: `:user::main` returns `(Holds/get b)`
directly"*, which sounds like a driver. **Classify both by measurement and report; restore only if the
subject is genuinely unexercised.**

## THE WORK

Restore a driver to each of the eight, **in a form the 170 main-wall accepts.** A non-`main` driver fn
works — measured this session:

```clojure
(:wat::core::defn :env::drive [] -> :wat::core::bool
  (:env::wants-holon (:env::CEnv :slot 1)))
```

Use kwargs construction (bare-positional is retired — arc 294 item 9a; the corpus was migrated this
session). Keep each restored call **semantically identical** to the deleted one; you are restoring
coverage, not writing new tests.

## ⛔ THE PROOF — restoring is not the deliverable, NON-VACUITY is

A restored driver that still can't fail has changed nothing.

1. **Per fixture, prove the test can now fail.** Break the subject the fixture exists to probe —
   e.g. in `record_surface_core`, make `:geo::Circle` stop satisfying `:geo::Shape` — rebuild, confirm
   the test goes **RED**, revert, confirm green. `git diff` must show no residue. **This is the
   deliverable.** Without it we have swapped one vacuous green for another
   (`[[feedback_a_green_test_can_prove_nothing]]`).
2. **Report, per fixture, what the test could NOT see before and can see now.** One line each.

## ⛔ A RESTORED FIXTURE THAT GOES RED IS A FINDING, NOT A REGRESSION

These subjects have been **unverified since 2026-07-10**. If restoring a driver makes a test fail, the
thing it was written to prove has been broken or unbuilt for 37 days and nobody could tell.

**Capture it verbatim, report it, and do NOT fix it in this strike.** That is STOP-1 and it is the most
valuable outcome this brief can produce.

## STOP TRIGGERS

- **STOP-1 — a restored fixture goes red.** A finding. Capture verbatim; do not fix; do not weaken the
  fixture to make it pass.
- **STOP-2 — you cannot restore a driver without violating the 170 main-wall.** Report the shape that
  defeats it; do not add a `main`.
- **STOP-3 — an "unclassified" file turns out to need a driver you cannot reconstruct** from the diff.
  Report it rather than inventing a call the original never made.
- **STOP-4 — you are tempted to change a test's assertion** rather than restore the fixture's driver.
  The assertion is not the defect.

## BLAST RADIUS

The 8 `tests/types/probe_arc293_*.wat` fixtures. **No `src/`. No test-`.rs` changes** unless a restored
fixture reveals one is needed — and that is STOP-4 territory, report first. Do not touch `.wat.bad`
fixtures, the 32 captured goldens, or any ignored test.

## OUT OF SCOPE, RECORDED FOR LATER

**A wall for this class is desirable but not trivial**: "a positive fixture whose declarations are
never exercised" would have caught this on 2026-07-10 — but `probe_arc296_pure_surface_field.wat` is a
*legitimate* declaration-only fixture (registration-time subject). A naive gate would condemn it. The
rung exists; the exemption set is the design question. **Do not build it here.**

## VERIFY

`cargo build --release --tests`, then `cargo clippy --workspace --all-targets --release -- -D warnings`
(0), then `scripts/floor.sh` — read the **Summary line**, never a piped exit code.

Baseline `4566 / 4566 / 122 skipped`. **Test count should not change** — these fixtures gain drivers,
not new tests. A changed count needs explaining. A red needs reporting, not fixing.

**On any red you did not intend: do NOT re-run.** Copy the whole stdout+stderr block **verbatim** —
never a `| head` window — name the exact assertion, and report.

## HOW TO WORK

You are a rider. **Ending your turn ENDS you.** ⛔ **Run every build and test in the FOREGROUND and
block on it. Do NOT use `run_in_background`. Do NOT set a Monitor. Do NOT poll and stop.** Four riders
on these arcs died exactly that way.

Anchor at `/home/watmin/work/holon/wat-rs`; `pwd` first. **Leave the work uncommitted.** Never
`git commit`/`push`/`stash`/`revert`/`checkout --`; `stash@{0}` holds unrelated work.

## REPORT

- the 8 restored drivers, each compared to the call `3cd00fbb` deleted
- **the non-vacuity proof per fixture** — how you broke the subject, the red it produced, and that the
  revert is clean
- one line per fixture: what its test could not see before, and can now
- the classification verdict on the 2 unclassified files
- any restored fixture that went red — **verbatim**
- the floor Summary line verbatim with the arithmetic
- every STOP that fired
- **the honest deltas — especially anywhere this brief did not match the disk.**
