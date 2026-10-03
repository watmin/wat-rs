# AMEND 4 — STONE 255.86: the pins are data, and no gate needs more time

**Drawn 2026-10-03.** **Executor: grok via pulsare, working solo** (it runs the floor). Continues at `be0c894fa`.
Commit locally on `main`; **do not push**.

Amend 3 is accepted in substance: the replacements, the retirement rows and the new homes are tests; `Formattable` and
`Reject` were asked of the registry and respelled; the member-join wall now decides "type" by `TypeEnv::contains`. Two
of its choices conflict with standing rulings:

## 1. The pins compare strings

`tests/cli/probe_stone_25586_one_name.wat` returns `(:wat::core::str value)` per case and the Rust test compares 68
strings. **Equality is data equality, never strings** (the builder, 2026-09-27: *"measuring strings is … so incredibly
anti-wat"*). Pin each value **as data**: an `assert-eq` (or `wat.doctest/matches?`) in the fixture against the expected
value written as a value, so the fixture exits 0 only if every pin holds; or return the values as EDN and compare them
read back as data. Hash-ordered renders (a HashMap or HashSet of several entries) are then compared as sets/maps, not by
their print order, and need no `length`/`get` workaround.

## 2. A timeout was raised to fit one machine

`stone_rows_are_refused_naming_their_replacement` walks 43+ rows, spawning a real `wat` process per row in series:
16.28 s alone, past the 30 s kill under floor load. The cure raised the kill to 120 s (`.config/nextest.toml`). That is
fitting a gate to one machine's speed under load, the class T1 retired (*"i strongly dislike timing based on a single
machine's capabilities"*). **Remove the override** and make the test not slow by structure: one generated test per row
(as `every_probe_runs` generates one per probe, so nextest runs them in parallel and a red names the row), or check every
row's refusal inside **one** process (startup once, then each row's program through the checker). Say which.

Then the floor (no timeout overrides added), clippy, census `--diff`; `git status` clean before the floor.

STOPs as in the brief. A STOP means STOP. Append to the SCORE, commit, **do not push**.
