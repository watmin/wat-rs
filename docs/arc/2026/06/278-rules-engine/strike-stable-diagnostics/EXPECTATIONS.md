# EXPECTATIONS — C19

## The scorecard

| # | what | state AT HEAD (driven) | required after |
|---|---|---|---|
| 1 | ★ same program, same bytes | **5 runs → 5 hashes** | 5 runs → 1 hash |
| 2 | ★ undetermined reads as undetermined | `:?3399` — an allocator counter | `_` / `?` / `?1`, argued |
| 3 | both renderers fixed | 2 sites leak | **mutation 2** — breaking one still REDs |
| 4 | the message still names the right types | `Stream :- [...]` vs `Vector :- [i64]` | unchanged apart from the var |
| 5 | a determinism gate exists | **none** | each `.wat.bad` run twice, byte-identical |
| 6 | no golden changes | 0 pin a literal id | 0 changed — STOP-2 |
| 7 | no inference change | — | zero diff outside the two arms + gate |
| 8 | floor / lints / clippy | 5357, 210/210, rc=0 | green |

## What would make this strike a failure even if every test passes

**A stable but still meaningless counter.** Renumbering to `?0` deterministically satisfies row 1 and
leaves the reader exactly as confused — and the same output still spells one unknown `T` and another
`?0`. Row 2 is the half that is about the human.

**And a gate that only checks the one file that reproduces today.** The defect is *any* source of
run-to-run variance in a diagnostic; a gate pinned to `probe_arc247_hof_coll_first.wat.bad` would go
green the day a different nondeterminism appears elsewhere. **Gate the corpus, not the repro.**
