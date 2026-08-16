# 296 · DESIGN STONE K — `#[ignore]` means ONE thing, and the ledger empties

> **STATUS: DRAWN 2026-08-16. STRIKE-READY.** Builder: *"is there another word we can use instead of
> ignore?... or do we need to rely on the ignored string for context?"*
>
> Neither. The word was never the problem and the string is a crutch. **This is 296's closure work**,
> not a successor to it — the arc set itself a gate and that gate is still open.

## ⛔ WHY THIS IS 296's STONE, AND WHY I ALMOST MISSED IT

`IGNORE-LEDGER.md:12`, this arc's own words:

> **Gate**: this ledger MUST be EMPTY before arc 296 closes.

Measured today: **`296-recapture-pending` ignores = 0**, six waves, 115 → 0. But the ledger document
still lists **246 rows**, and 296 has **no `INSCRIPTION.md`**. The *count* reached zero; the *gate* did
not. I drew this as a new arc first — because I had been treating the campaign as finished at the
number I liked, which is the same error as reading a floor Summary and calling it green. **A condition
the arc set for itself is not satisfied by a number moving.** The builder retracted the new arc; the
work belongs here.

Emptying the ledger honestly requires answering the question this stone answers: **what does a
remaining `#[ignore]` MEAN?** Until that has one answer, "empty" is undefined.

## THE DEFECT

`#[ignore]` answers two incompatible questions with one attribute:

- *"this test is **blocked or broken**"* — debt, must trend to zero
- *"this is **deliberately outside the floor**"* — an instrument, correct forever

**You cannot drive to zero a population that contains things which must never reach zero.** So the
count has been lying pessimistically for months, and the lie was invisible because the two kinds are
textually identical.

Measured on disk after the campaign closed:

```
total #[ignore] attributes   31
ON-DEMAND (not debt)          7
ACTUAL DEBT                  24
```

The 7 were re-labelled in `96146e94` with a greppable `ON-DEMAND (not debt)` marker. **That was the
CONVENTION rung of the ladder, and this stone exists because a convention is where we stopped, not
where we should have landed.** A string every future hand must write and read correctly is a failure
class waiting for a tired afternoon — this arc has spent all day proving that about strings.

## ★ THE FINDING — the 7 are THREE kinds, not one

| kind | n | what it actually is | real home |
|---|---|---|---|
| **benchmark** | 3 | measures throughput; asserts nothing meaningful | **`benches/` + `cargo bench`** |
| **diagnostic** | 3 | prints a gap for a HUMAN to read | **`wat-scripts/`** (loader-gated) |
| **slow test** | 1 | a genuine test, excluded on RUNTIME not on kind | **nextest `default-filter`** |

The perf items are **not tests**. The diagnostics are **not tests** — their own reason strings say
*"run with `--ignored` to read the gap"*, which is a script's job description. Only the boundary probe
is a test, and it is excluded for an entirely different reason from the other six.

Three categories collapsed into one attribute is why the count could not be reasoned about.

## THE THREE MOVES

### 1 — Benchmarks to `benches/` (3)

```
src/runtime.rs                          :: dispatch_keyword_head_value_perf
tests/rete/perf_arc278_fire_baseline.rs :: fire_throughput_baseline
tests/rete/perf_arc278_fire_baseline.rs :: native_fire_once_join_scaling
```

There is **no `benches/` directory** and no `[[bench]]` in `Cargo.toml` — cargo has had this concept
the whole time and we have never used it. A benchmark in `benches/` **cannot inflate the ignore count,
because it is not an ignored test.** The category becomes structural, not textual.

⚠ `dispatch_keyword_head_value_perf` is a `#[cfg(test)]` unit test inside `src/runtime.rs`, so moving
it means reaching whatever `super::` internals it uses. If those are not reachable from `benches/`,
that is STOP-1 — a finding, not a licence to leave it.

### 2 — Diagnostics to `wat-scripts/` (3)

```
tests/macros/probe_arc249_threading_in_wat.rs :: diag_first_over_form
tests/macros/probe_arc249_4_rehome_in_wat.rs  :: diag_first_over_vector_form
tests/macros/probe_arc249_4_rehome_in_wat.rs  :: diag_keyword_to_string_over_form
```

`wat-scripts/` is the sanctioned home for durable, loadable, type-checked wat, and
`every_wat_scripts_file_loads` parses and type-checks **every** file under it on the current runtime.
A relocated diagnostic therefore **cannot rot into a graveyard that reads like live code** — strictly
stronger than an `#[ignore]`d test, which nothing exercises at all.

⚠ **The move most likely to fail.** If a diagnostic leans on Rust test scaffolding with no wat
expression, it does NOT get forced across — fall back to move 3 for that one and say why. A diagnostic
mangled into a shape it does not fit is worse than one left alone.

### 3 — The slow test stays a test, excluded by CONFIG (1)

```
tests/function/tco.rs :: self_recursion_via_if_at_million_depth
```

It asserts, it can fail, it belongs in the suite. It is excluded on **runtime**, and that policy
belongs in the runner, not hidden in an attribute on line 51 of one file. nextest is **0.9.138**
(verified) — `default-filter` is available:

```toml
[profile.default]
default-filter = 'not test(self_recursion_via_if_at_million_depth)'

[profile.slow]
default-filter = 'all()'
```

The `#[ignore]` comes **off**; it becomes a plain `#[test]`. `.config/nextest.toml` already carries
profiles and per-test `filter` overrides, so this extends a present mechanism rather than inventing one.

## THE LEDGER — 296's gate, closed in the same motion

With `#[ignore]` meaning one thing, `IGNORE-LEDGER.md`'s 246 rows can be honestly retired: every row
was a `296-recapture-pending` mute, and that population is **0**. The ledger becomes a closed record —
its gate met, stated as met — rather than a live tracker of a thing that no longer exists.

**Do NOT delete it.** It is the record of a 241-test quarantine and its drain. Retire it in place: a
header stating the gate is satisfied, the final 115 → 0 trajectory, and a pointer to this stone for
what `#[ignore]` means afterwards.

## WHAT THIS BUYS — the no-form rung

After this stone `#[ignore]` means exactly one thing: **blocked or broken.** Not by anyone
remembering — the other two kinds have nowhere to hide. A benchmark is not in the test binary; a
filtered test carries no attribute to misread.

Two things become possible, and neither is worth building first:

1. **The count is honest by construction** — `#[ignore]` count == debt, no subtraction, no marker to parse.
2. **An ignore-justified lint gets simple** — one category to justify (reason + `unlock:`), armable at
   the true debt number, and it joins the existing family (`retired_name_justified`,
   `span_substitution_justified`, `unused_span_justified`). **Deliberately NOT built in this stone:**
   built before the categories separate, it would enshrine the confusion it exists to end.

## THE GATE

| # | assertion |
|---|---|
| 1 | `benches/` exists; the 3 perf items run under `cargo bench` and produce their numbers |
| 2 | `grep -rn 'ON-DEMAND (not debt)'` → **0**. The marker is GONE, not satisfied — it was scaffolding |
| 3 | total `#[ignore]` == **24**, down from 31, with **no test's behaviour changed** |
| 4 | the boundary probe carries **no `#[ignore]`** and does not run under `--profile default` |
| 5 | it DOES run under `--profile slow`, and **passes** — prove it, do not assume |
| 6 | the 3 relocated diagnostics type-check under `every_wat_scripts_file_loads` (or are reported un-relocatable per move 2) |
| 7 | `IGNORE-LEDGER.md` retired in place: gate stated as met, 115 → 0 recorded, not deleted |
| 8 | floor green, clippy 0, and the run/skip **arithmetic accounted for** — moved items leave the test count, they do not vanish silently |

**Row 2 is the point of the whole stone.** If the marker survives, the convention survived, and the
convention is what is being replaced.

**Row 5 is the trap.** A test excluded by config and never run again is worse than an ignored one — at
least `#[ignore]` is loud about not running.

## STOP TRIGGERS

- **STOP-1 — a benchmark cannot reach the internals it needs from `benches/`.** Name the symbol and its
  visibility. **Do not make things `pub` to move a benchmark** — that is an API change wearing a
  chore's clothes.
- **STOP-2 — a diagnostic has no honest wat expression.** Fall back to move 3, say why.
- **STOP-3 — any test's BEHAVIOUR changes.** This stone relocates and re-declares; it does not fix,
  rewrite, or delete. A benchmark that starts asserting is out of scope.
- **STOP-4 — tempted to `#[ignore]` something to make a move work.** That is the defect re-entering
  through the door being closed.
- **STOP-5 — a red you did not intend. Do NOT re-run.** `scripts/floor.sh` keeps the untruncated log:
  copy the failing block verbatim, name the arm, report.

## Kin

- `IGNORE-LEDGER.md` — this arc's gate; row 7 closes it.
- `96146e94` — the ON-DEMAND markers this stone deletes; the worklist, always a waypoint.
- `CAMPAIGN-the-recapture-cascade.md` + waves B1–B6 — drove pending 115 → 0 and made the remaining
  count worth reasoning about at all.
- `docs/arc/2026/05/122-per-test-attributes` — **INSCRIBED/closed**; cited as lineage, not a host.
  This is that arc's unfinished half, landing where the ledger lives.
- `.config/nextest.toml` — already carries profiles and per-test `filter` overrides.
