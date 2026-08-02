# EXPECTATIONS — the scalar-`def` door

Fixed before the strike so the result cannot move the goalposts.

| # | what | command | expected |
|---|---|---|---|
| 1 | the def registration asks the gate | `grep -n 'resolve::gate' src/check.rs` | a call in the `":wat::core::def" if is_top` arm |
| 2 | `Existing::Equivalent` on presence, unchanged | read the arm | presence ⇒ `Equivalent`, so `infer_def` keeps redef authority |
| 3 | fourth taxonomy entry | `grep -n 'UnnamespacedName' src/check.rs src/check/*.rs` | `CheckErrorKind::UnnamespacedName` + Display |
| 4 | build | `cargo build --release --all-targets` | exit 0, **zero warnings** |
| 5 | clippy | `cargo clippy --release --all-targets` | **zero warnings** |
| 6 | **the RED probe goes red before** | run it at `72a1ac3d` behaviour | bare scalar def **accepted** (that is the bug) |
| 7 | **and red for the RIGHT reason after** | run it with the change | located `UnnamespacedName`, naming the fix |
| 8 | redef discipline untouched | `cargo test --release --test wat_lang -- arc157_def` | same pass/fail set as before |
| 9 | `Reserved` question answered | a run, not a reading | `(def :wat::core::pi 3.14)` from user source — policed or not, stated |
| 10 | no corpus def newly rejected | the central floor (mine) | zero new failures |
| 11 | **the floor** (orchestrator, after) | `cargo nextest run --release` | **4266/4266**, and +1 or +2 for the new probe |

Rows **6 and 7** are load-bearing and they are a pair. Row 4 alone proves the variant is *handled*, not
that the door is ever *reached* — that was the exact narrowness that let me report the wall "green" while
32 violations stood. `NISI FRANGAS, NIHIL PROBAS`.

## Independent prediction

**15–30 min.** One call site, one enum variant, one fixture. The variance is row 3 (a fourth error
taxonomy may want plumbing the other three already have) and STOP-3 (if `:7798` turns out not to be the
real binding site).

## Trap-doors named in advance

- **The corpus can no longer prove this.** The six `wat_arc157_def_*` fixtures that held bare scalar defs
  were namespaced in `72a1ac3d`. A green floor is therefore *silent* on this hole, not supportive of it —
  which is why the probe must be new and mutation-proven.
- **A `.wat` probe under `wat-scripts/` would go permanently RED** (the loader gate type-checks every file
  there). It belongs in `tests/` as a `.wat.bad`.
- **Gating in two places.** `infer_def` owns the redef *decision*; `:7798` owns the *registration*. One
  gate call, at the registration. A bespoke `if` in `infer_def` would be the convention rung wearing a
  wall's clothes.
- **`Duplicate` must never reach the user here.** If presence maps to anything but `Equivalent`, a benign
  redef becomes a gate error and masks `DefRedefForbidden` — arc-157 semantics broken. STOP-1.

## Affirmatively out of scope — not deferred

- The `(:wat::core::forms …)` second-top-level class: closed for the codemod in `72a1ac3d`; whether the
  *gate* sees a def inside a child's forms payload is a separate question and is only in scope here if
  STOP-2 surfaces one.
- `probe-bare-defrule-name.wat`'s retirement — mine, separate, its own weigh.
- A name **constructor** that makes a malformed name unspellable (arc 109
  `NOTE-macro-minted-names-are-unvalidated-string-concatenation.md`). That is the top rung; this stone is
  the check rung.
