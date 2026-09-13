# EXPECTATIONS — the malformed census

**Written BEFORE the strike**, from `04726854e`.

## What this stone is graded on

Whether the **builder can apply *"service items only"* by reading a column** — and whether the instrument
earned the right to be quoted. A census is worth nothing if its numbers cannot be trusted, and five of my own
greps were wrong this session.

| # | what | check | expected |
|---|---|---|---|
| 1 | ⭑⭑ CONTROL A — migrated arms **absent** | search the report for `circuit.wat:5xx` / the sns-fanout twin | **not present** — they return `"malformed"` |
| 2 | ⭑⭑ CONTROL B — a known raiser **present** | search for a `wat/` `UNMIGRATED PLACEHOLDER` arm | **present** |
| 3 | both controls stated in the report | read it | named explicitly, not just passing silently |
| 4 | ⭑ every row carries the enclosing head | read the report | `defservice`/`defsurface`/`defn`/`deftest`/other — **no blanks** |
| 5 | ⭑ every row carries the home | read the report | `wat` / `wat-scripts/service` / `scratch-pad` / `tests` / `docs` |
| 6 | all three raise forms recognised | grep the census source | `assertion-failed!` **and** `raise!` **and** `panic!` |
| 7 | form-walking, not line-matching | read the source | walks `ast->children`; **no** line-based matching of arms |
| 8 | the 487 → findings gap explained | read the report | the difference accounted for, not glossed |
| 9 | ⛔ nothing migrated | `git diff -- wat-scripts/{queue,topic,fanout} wat/` | **EMPTY** |
| 10 | ⛔ no stdlib rule | `git diff --stat -- src/ wat/` | **EMPTY** |
| 11 | ambiguous rows marked | read the report | any judgement call flagged, **not resolved** |
| 12 | run twice, identical | two runs, diff them | byte-identical. ⚠ **Requires a SORTED path list** — an unsorted glob makes row order filesystem-dependent and would red a correct census |
| 13 | floor | `./scripts/floor.sh` → **Summary line** | green at **5239**, 0 FAIL, no `ARM.txt` |
| 14 | clippy | `-D warnings` | exit 0 |

## ⭑ Rows 1–3 gate every other row

I will read no count from this report until the controls are stated and passing. ⛔ **And row 1 is the
subtle one:** the migrated arms share a physical line with a `Stopped` arm that *does* raise, so a
line-oriented census will flag them and look correct while being wrong about its central distinction.

## ⚠ What I will reject

- **Any arm migrated** (row 9 / STOP-1). The builder sequences; the census reports.
- **A `wat/lint.wat` rule** (row 10 / STOP-2) — a permanent gate over a deferred population is noise.
- **A count quoted with a failing or unstated control** (rows 1–3 / STOP-3).
- **Rows without both axes** (rows 4–5). A row that still costs a file-open has not done the job.
- **"Service item" resolved for me** (row 11 / STOP-4).
- **`5237` as the floor count** — D2 moved it to **5239**.

## Runtime prediction

**40–65 minutes.** The walker is the bulk and there are two worked examples to copy; classification is
mechanical once the walk works. Floor ~500–560 s.

## Trap-doors, ranked

1. ⛔ **Line-oriented matching** (rows 1, 7) — flags the migrated arms and looks right.
2. ⛔ **`assertion-failed!` only** (row 6) — loses 11 sites silently.
3. **A count before controls** (row 3).
4. **Migrating "the obvious ones"** (row 9) — the two already done were *tears mislabelled*, a fact no rule
   can know.
5. **Blank axis columns** (rows 4–5) — the census's entire purpose.
