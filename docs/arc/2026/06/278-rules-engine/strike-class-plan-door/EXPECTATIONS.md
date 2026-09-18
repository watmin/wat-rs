# EXPECTATIONS — A8: the class-plan door

> Written BEFORE the strike. Graded by the orchestrator's own re-run.

| # | what | expected |
|---|---|---|
| 1 ★ | one door | push-or-demote is ONE act; no site can do one without the other |
| 2 ★ | `has_mixed` is DERIVED | no `any_mixed` field survives; the predicate reads the map |
| 3 ★ | the bypass is a COMPILE ERROR | re-introduce a demote past the door → `error[E0616]` or equivalent. **Quote it** |
| 4 ★ | the batch fast path is unmoved | every `*_cost` gate green, **numbers unchanged** — especially `accum_alpha_cost`'s `alpha_elements == 80_200` |
| 5 | packed-arm-first survives | `observe` branches on `packed` before the lookup; a packing fact runs today's sequence |
| 6 | floor | **0 failed** |
| 7 | clippy | rc=0 |
| 8 | blast radius | `src/rete/kernel/` only |

★ load-bearing. **Row 6 is the deliverable, row 4 is the one most likely to bite.**

## Trap doors, named in advance

- **A cure that taxes the batch path.** The site's header forbids it in its own words. STOP-1.
- **`has_mixed` cached "for speed".** That reintroduces exactly the stored-summary defect being
  cured. If a walk is genuinely too costly, STOP-3 and report both options rather than choosing.
- **A `pub` map or a `&mut` accessor** — defeats the strike, same as a `beta_mut()` would have.
- **This is LATENT.** Do not manufacture a "before" red. The proof is the compiler, not a failing
  test. If you construct a real violation, that is a finding — report it before curing.
- **Re-run the floor at FINAL state.**

## What would make me reject the result

- Row 3 answered with anything but a compiler error.
- `any_mixed` surviving as a field.
- A moved cost number explained away rather than surfaced.
- A red floor of any size.
