# EXPECTATIONS — 118.3-B · parametric surface satisfaction

**Written before the strike, 2026-08-17, against `4603e900`** (floor 4698/4698, clippy 0,
ignores 13). Fixed here so the result cannot move the goalposts.

## The scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 0 | ★ the 4 REDs exist **before** the change | `--check` the parametric probe + 4 call sites, pre-edit | **4 × `TypeMismatch`**, each `expects :sq::Seqable<?N>; got :wat::core::<Container><…>` |
| 1 | after: all four containers dispatch | run the same file | prints `3,4,5,2` |
| 2 | ★ the bare-surface path is untouched | run `probe-seqable-is-spellable-today.wat` | `"3,4"`, unchanged |
| 3 | parametric non-surface bounds untouched | named call site exercising `Vector<T>` as a bound | unchanged |
| 4 | ★ the arm's existing tenants unmoved | floor rows for `Dialable` / `TypedCapability` / `Handle` | all green |
| 5 | no new hardcoded letters | `git diff src/types.rs` | `transport_satisfier_heads` **unchanged** |
| 6 | the test is KEPT | `git status --short tests/` | added, not deleted |
| 7 | floor | `scripts/floor.sh` → Summary | **≥4699 passed, 0 failed, 0 timed out** |
| 8 | clippy | `cargo clippy --release --all-targets` | **0** |
| 9 | ignores held | the `#[ignore]` grep | **13** |

Row 7's baseline: `4603e900` ran **4698**. The kept test adds at least one row → **4699+**. Lower is
a finding, not a rounding.

## Independent prediction

**Runtime: 35–60 minutes.** Two release builds dominate; `check.rs` is large. The edit itself is one
match arm. The uncertainty is not the typing — it is row 4, because three prior arcs already built
on this arm and the rider may need a round of reading before touching it.

**Time-box: 120 minutes** (2× upper bound).

## Trap doors — named before the strike

1. **★ The arm has three existing tenants.** `Dialable`, `TypedCapability`, `Handle` — arcs 267,
   170 C2, 293.W.2f. This is the single most likely source of a red, and unlike the last two stones
   the blast radius is *inside* the type checker rather than beside it. Row 4 exists for this.
2. **Variance.** The arm states args are **invariant**. If binding a surface's params forces a
   covariance choice, that is a design ruling the rider must not make — STOP-1.
3. **Fixing it in the wrong place.** `transport_edge_keys` / `transport_satisfier_heads` look
   inviting and are the *existing workaround* for this same disease. Touching them is STOP-2; row 5
   is the check.
4. **Row 0 skipped.** Change `src/` first and rows 1–2 prove nothing. Ordered first in the brief.
5. **Breaking arm 3 while fixing arm 5.** The bare-surface probe runs *today*; if it moves, the
   wrong arm was edited. Row 2 catches it.

## What would make me call this Mode B

- Row 2 or row 4 red — the fix works for `Seqable` by breaking the arm's existing tenants.
- `transport_satisfier_heads` gaining another hardcoded letter. That ships the behaviour by
  deepening the exact defect this stone exists to remove.
- Row 0 unreported, or reported as reasoning rather than captured output.
- Any `#[ignore]` added, for any reason.

## What I will re-run myself before committing

Rows 1, 2, 4, 7, 8, 9 — independently, on my own invocation, per FM 9. Row 0 cannot be re-run after
the fact by construction, which is exactly why the brief requires its output captured **verbatim**.

## What this stone is NOT allowed to claim when it lands

That `Seqable` exists. It will make `Seqable` **possible**. Minting it in the stdlib, extending the
four containers, and pointing `join`/`map` at it is the next stone — and the seven `-stream` twins
are the one after that.
