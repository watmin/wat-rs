# EXPECTATIONS — the chaos gate has teeth

**Written BEFORE the strike**, from `b13a4119f`.

## What this stone is graded on

That an **inert injector becomes a red** — and that the gate itself cannot go red on a correct tree. Both
halves matter; the second is the one I got wrong in my own four-questions.

| # | what | check | expected |
|---|---|---|---|
| 1 | ⭑⭑ a floor harness drives chaos | `grep -l disrupt tests/**/*.rs` | **≥1** file — currently **0** |
| 2 | ⭑⭑ `draws > 0` asserted | read the harness | present, at both rates |
| 3 | ⭑⭑ `hits == fires` asserted | read the harness | present, at both rates |
| 4 | `fires == draws` at 100 % | read the harness | present at the 10000 bp scenario only |
| 5 | ⛔ **no threshold on a random quantity** | read the harness | **no** `fires > 0`, no pinned draw count |
| 6 | the entry point the harness drives RETURNS its summary | read `circuit.wat` | `-> :wat::core::String`. ⚠ Whether that is `:user::chaos` changed or a new sibling is the strike's call (STOP-5) — this row does not pin it |
| 7 | ⭑ determinism demonstrated | run the new harness **3×** | three identical results |
| 8 | ⭑ it actually FAILS when the injector is inert | point the harness at a **rate-0** scenario — measured: rate 0 ⇒ `disrupt-draws=0` — and show `draws > 0` going RED | the red quoted verbatim; **no logic or counter edit needed**, so nothing to revert |
| 9 | ⛔ rate unchanged | `git diff` | 200 bp still the shipped value |
| 10 | ⛔ counters unchanged | `git diff` | `disrupt-draws`/`-fires`/`disrupts` untouched |
| 11 | no stdlib/Rust-substrate change | `git diff --stat -- src/ wat/` | **EMPTY** |
| 12 | unperturbed happy path | `2000 4 3 8192 true 1000` | `distinct=8000;dup=0` |
| 13 | floor | `./scripts/floor.sh` → **Summary line** | green, **and the new count stated** (it will not be 5237) |
| 14 | clippy | `-D warnings` | exit 0 |
| 15 | ⚠ floor time delta reported | before/after | reported, not hidden |

## ⭑ Row 8 is the row I will not accept a claim for

A gate that has never been **seen to fail** is a gate nobody has tested. ★ And it needs **no edit**: a
rate-0 scenario gives `disrupt-draws=0` (measured on every happy-path run this session), so pointing the
harness at rate 0 reds `draws > 0` on an untouched tree. Capture that red verbatim. ⭑ This is deliberately
arranged so row 8 does **not** collide with STOP-4 — nothing is forced, nothing is reverted.

## ⚠ What I will reject

- **`fires > 0` anywhere** (row 5). `0.98²⁵⁶ ≈ 0.6 %` — it reds a correct tree about 1 run in 170, and I
  proposed it myself before doing the arithmetic.
- **A pinned draw count** (row 5). 4 and 5 on two runs of one command.
- **The rate tuned** (row 9 / STOP-3) — D2-a was explicitly not taken.
- **A green claimed without row 7's three runs.** A no-probability claim owes repetition.
- **`5237` quoted as unchanged** (row 13). New harnesses move it; a stale count reads as a shrink.

## Runtime prediction

**35–55 minutes.** One returning entry point, one sibling, one harness; the long poles are row 7 (three runs
of an n=2000 chaos scenario, ~25 s each) and row 8's forced red plus revert. Floor ~500–560 s with the new
chaos runs in it.

## Trap-doors, ranked

1. ⛔ **`fires > 0`** — the obvious row, and a flake.
2. ⛔ **Row 8 skipped** — an untested gate.
3. **A pinned draw count** — timing-dependent.
4. **`5237` reported unchanged** — the count moves.
5. **Floor time** — two chaos runs added; report the delta.
