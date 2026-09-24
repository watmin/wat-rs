# DESIGN — STONE D: shard the wat-scripts load gate

**Drawn 2026-09-23.** Builder: *"that thing has needed some love for quite a bit.. i don't mind
carving it up for some concurrency.."* Not a the-little-wat finding — infrastructure. Every stone
on this branch pays it at least once; stone B paid it twice.

## The problem, measured

`tests/lint/wat_scripts_fixes_load.rs::every_wat_scripts_file_loads_on_the_current_runtime` is
**one `#[test]` over 769 files**: **425.736s isolated** on a quiet box, and the config's own
comment says *"THE GATE IS 98% OF THE FLOOR'S WALL-CLOCK."* Its budget has been raised **three
times** (45/90 → 120/240 → 300/600 → 1500/3000). The corpus went 327 → 769 in five weeks.

## ⭐ The cost is FLAT — measured per file, all 770

```
median 985 ms · heaviest 1120 ms · top 10 files = 1.4% of the total
scratch-pad 424 (55%) · fixes 113 · probes 83 · fmt 74 · perf 61 · grep 7 · demos 3 · …
```
(Timed with `wat --check` per file, 8-way parallel. The in-process gate is cheaper per file —
425.7s / 769 = **0.554 s** — but the SHAPE is the same: no file dominates.)

Two consequences drive the design:

1. **Stride sharding balances near-perfectly.** No long pole exists to strand a shard.
2. ⛔ **The config comment's per-DIRECTORY split does NOT balance.** `scratch-pad/` alone is 55% of
   the corpus, so its shard would still cost ~230s. Rejected in favour of stride.

## The cure — the in-tree precedent, copied

`tests/lint/every_wat_bad_fixture_actually_fails.rs` stride-shards its corpus
(`paths.iter().skip(shard).step_by(N_SHARDS)`) into `N_SHARDS` generated `#[test]`s, each asserting
its own slice is non-empty, and its header states the rule this gate broke: *"IF THIS EVER NEEDS
RAISING, SPLIT INSTEAD."* Same shape here.

## The ONE contract decision — naming

⛔ **Every shard name keeps `every_wat_scripts_file_loads_on_the_current_runtime` as a PREFIX**
(`…_shard_00` … ). Seventeen files cite the gate by name — `CLAUDE.md`, three lint headers,
`rete_compile_gate.rs`, `.config/nextest.toml`, a census script — and nextest `test(...)` filters
are SUBSTRING matches. A prefix-preserving name keeps every one of them true with zero edits.

## How many shards — an invariant, not a number

⛔ **Do not pick N by symmetry with the sibling's 16.** Derive it:

- per-file in-process cost **0.554 s** (measured);
- the config's recorded contention band **3.5×–4.4×**;
- the config's own flake line: a loaded cost above **69% of the kill** is *"a timeout flake waiting
  to happen."*

**Invariant: a shard's loaded cost (isolated × 4.4) sits under 69% of its kill.** Either size
shards to fit the DEFAULT 15s-warn / 30s-kill with no override (the precedent's choice), or keep
an override sized to the shard — but whichever, **measure one shard isolated and show the
arithmetic.** N=16 would give ~48 files / ~27s isolated / ~117s loaded: it needs an override.

## Out of scope — REJECTED

- **Making each file cheaper.** The flat ~0.55 s/file is almost certainly the stdlib re-frozen
  per file. Freezing once and checking each file against it is the DURABLE cure, and a different,
  larger stone. Sharding buys concurrency, not less work.
- **Changing what the gate checks.** Same loader (`FsLoader`, never `InMemoryLoader` — the header
  says why), same verdict per file, same failure text naming the file.
