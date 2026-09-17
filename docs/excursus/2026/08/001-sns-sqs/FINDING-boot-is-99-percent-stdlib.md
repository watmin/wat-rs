# FINDING — boot is 99.5 % stdlib, and it has tripled in three weeks

**Filed 2026-09-17**, from the blocked queue promotion. Builder, on being shown the first numbers:
*"whoa - you found where to attack slow boot?..... i think yes..... we've been raising timeouts"*.

**This is a measurement and a fork, not a plan.** Arc 255 already charged this campaign once for
prescribing a remedy in the same breath as a mechanism (`3f159b0c5`), so what follows deliberately
stops at *where*, and names the experiment that would find *which*.

## Where boot goes: essentially all of it is the stdlib

```
binary start, stdlib NOT loaded (--help path) ......  0.002 s
binary start, do-nothing program ..................  0.425 / 0.432 / 0.438 s
```

⭐ **99.5 % of a `wat` process's startup is re-deriving the stdlib** — parsing, macro-expanding and
type-checking 22,979 lines across 55 files. The process itself costs **2 ms**. Every invocation pays it:
every test, every probe, every child a service spawns, every user program.

## ⛔ AND IT IS GETTING WORSE SUPERLINEARLY

| | 2026-08-26 (`3f159b0c5`) | 2026-09-17 | Δ |
|---|---|---|---|
| trivial-program boot | **147 ms** | **430 ms** | **+193 %** |
| stdlib | 54 files / 19,672 lines | 55 files / 22,979 lines | +17 % |

**+17 % source bought +193 % boot — superlinear by ~2.5×, in three weeks.** The queue promotion
supplies an independent marginal datapoint in the same direction: +1,910 lines containing one
`defsurface`+`defservice` pair cost **+126 ms**, i.e. **≈66 µs/line against a 19 µs/line average — ~3.5×
the average per line.** Both readings point the same way: the cost is concentrated in **macro
expansion**, not in bytes parsed.

## The accommodation trail — the builder's own observation, in the config

`.config/nextest.toml` today:

```
[profile.default]  slow-timeout = { period = "15s",  terminate-after = 2 }   # kill at 30 s
[profile.ci]       slow-timeout = { period = "20s",  terminate-after = 3 }
override test(a_counter_increments_across_turns)                 period = "120s"
override test(r2_drop_)                                          period = "90s"   # kill 180 s
override test(every_wat_scripts_file_loads…)                     period = "300s"  # kill 600 s, priority 100
```

`git log --follow` on that file shows the pattern by name — `e372c8d1c` (2026-08-16) is literally
*"the loader gate's deadline is raised"*. ⛔ **Every one of those overrides is a startup bill paid in
wall-clock and then absorbed by a larger number.** The queue promotion did not break the floor so much
as exhaust the last of the headroom: the loader gate had **8 %** left against its 600 s kill and arm A
**13 %** against its 35 s.

## ★ WHAT IS ALREADY KNOWN NOT TO WORK, so it is not re-proposed

**Batching the gate.** `3f159b0c5` tried it and measured the refusal: one program containing all 98
rows reported *"1 type-check error"* and **stopped** — the check walk short-circuits, so 97 rows went
unexamined. A batched gate would have looked 100× faster **while testing one row**. Three independent
well-formed calls do all report, so the abort is shape-dependent and nobody has established which
shapes collect and which stop.

⚠ I reached for exactly this idea today before reading that commit, and it was already dead. **Read the
prior finding before attacking a known-slow thing.**

**Raising timeouts.** It is what has been happening, it is why a 29 % regression could land unnoticed
until the queue stone, and it converts a measurable cost into folklore.

## The fork

Boot is dominated by work that is **identical in every process** and already known at build time. The
frozen stdlib embeds **source**; nothing precomputes the expansion or the checked environment.

| option | what it does |
|---|---|
| ⭐ **precompute the freeze** — serialise the expanded, checked environment at build time; a start maps it instead of re-deriving it | attacks the 99.5 %; makes promotion nearly free; every wat program starts faster; removes the reason timeouts creep |
| profile first, then decide | the honest prerequisite — see below |
| keep raising timeouts | the status quo, with the cost hidden and compounding |

## ⛔ THE NEXT MEASUREMENT, and it is NOT the fix

**Nobody has measured WHICH PHASE owns the 430 ms** — parse, macro-expand, type-check, freeze. Two
independent readings say "expansion", but both are inferences from marginal cost, not a profile. The
first stone of any attack is a **per-phase breakdown of one boot**, plus a per-file marginal cost curve
across the 55 manifest entries (the queue experiment shows the method: toggle one row, rebuild, time,
restore sha256-verified).

★ Arc 255's lesson, quoted because it applies directly to whoever picks this up: *"I described a
mechanism and prescribed a remedy in the same breath, having measured neither, and committed both. A
prescription is a claim."*

## Provenance

- `3f159b0c5` (2026-08-26) — the startup-bound finding and the batching refutation.
- `e372c8d1c` (2026-08-16) — "the loader gate's deadline is raised".
- `the-queue-matures-into-the-stdlib/FINDING-promotion-costs-every-startup.md` — the +29 % measurement
  and the promotion parked on `queue-promotion-blocked-on-startup-cost` (`6136d144f`).
