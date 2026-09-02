# EXPECTATIONS — C4

> **Every pre-value below was measured at HEAD `f90d4c126` by the orchestrator, not recalled.**

## ⛔ NO PINNED TEST COUNT

The floor must be **≥ 5,313** (5,312 + the committed probe) plus any arm added here.

## The scorecard

| # | what | pre-value AT HEAD | expected after |
|---|---|---|---|
| 1 | ★ table 1 has a production-faithful row | **absent** — only `A alpha_activate_fact 14.18 ms`, measured with `skip_span` OFF | present, and labelled as the production branch |
| 2 | ★ table 3 has one | **absent** — only `A alpha_activate_fact 13.90 ms` | present |
| 3 | the old rows relabelled | `A alpha_activate_fact` (both tables) | says `skip_span` is forced off |
| 4 | table 1 ladder still nests | `M 12.09 → A 14.18`, `A−M push 2.09` | unchanged — the old `a` is untouched |
| 5 | table 3 ladder still nests | `M 11.61 → H 11.61 → V 12.44 → D 12.61 → A 13.90`, `A−M 2.28` | unchanged |
| 6 | ⚠ no derived row goes negative | `H−M` already prints **−0.00 ms** | still ≥ that; a NEW negative is STOP-3 |
| 7 | the probe still passes | `pool=0` vs `pool=120200`, 3/3 bind-only | unchanged — it is the non-vacuity proof |
| 8 | radius | — | `accum_alpha_cost.rs` only |
| 9 | lints | **196/196** (measured) | green |
| 10 | floor | **5312/5312** + probe (measured) | ≥ 5,313, zero FAIL |
| 11 | clippy | **rc=0** (measured) | silent |

## The mutation proofs

1. **Empty the new arm's map** (`bind_only_prod` left as `HashMap::new()`) → its row must become
   indistinguishable from the old `A` row. If it does not, the new arm is not measuring what it
   claims and the two rows differ for some other reason.
2. **Point the new arm's map at a bogus id** (insert under `id + 1`) → `.get()` misses, `skip_span`
   goes false, the row returns to the old value. Proves the row reads the MAP, not merely "a second
   arm exists".
3. Per arm: **proven** / **reachable but not driven** / **not reachable, and why**.

## Runtime prediction

30–45 minutes. Two arms and two label blocks; the mechanism is already driven and the pattern is
already in the file at `:530`.

## What would make this strike a failure even if every test passes

**A production row that is not actually production-faithful.** The new arm must build the map the
way `fire/delta.rs:339-346` does — via `bind_only_fields` over `arm.compiled_conds` — and not by
copying a literal or reusing the probe's map. Mutation 2 is what separates those.

**And relabelling without adding the arm.** The file is named for `alpha_activate_fact`; if the
honest label lands but no production number does, the table now correctly reports that it measures
something else and still never measures the thing.
