# DESIGN — the boot cache is possible, or it is not

**Drawn 2026-09-17**, builder-directed (*"sounds like there's only one real choice here... 83%
reduction or not?"* / *"meh - this is just another member of the excursus"*). **NOT STRUCK.**

⛔ **THIS IS A FEASIBILITY PROBE, NOT THE CACHE.** It answers whether the 83 % can be precomputed at
all, and at what load cost — so the decision to build it is made on numbers. Building it here would be
a prescription, and `3f159b0c5` already charged this campaign for prescribing on this exact subject.

## What is established

`boot-names-where-its-time-goes/SCORE.md`, reproduced independently by the orchestrator:

```
macro expansion   206.5 ms  51.0 %        ← stdlib-expand alone 199.5 ms
type-checking     129.7 ms  32.0 %        ← and it is checking the STDLIB, not user code
registration       30.4 ms   7.5 %
parsing            29.3 ms   7.2 %
freeze              4.8 ms   1.2 %
process itself      2   ms                ← the floor, measured via the --help path
```

**83 % re-derives, from identical inputs, state that is fully determined at build time.**

## The three questions, in order of what kills the idea fastest

### 1. ⛔ CAN THE CHECK BE CACHED, OR ONLY THE EXPANSION?

This decides the prize before anything else. Caching **expansion only** attacks **51 %**; caching
**expansion + check** attacks **83 %**. `check:body-infer(ALL fns)` is 109 ms by itself and walks
**registered `Function`s, not source forms** — so its output may not be a thing that *has* a
serialisable form at all. **Answer this first**; the rest is wasted if the answer is "expansion only".

### 2. WHAT IS ACTUALLY IN THE STATE, AND WHAT CAN ROUND-TRIP?

The 83 % lives in `TypeEnv` + `MacroRegistry` + `SymbolTable` + the expanded `Vec<WatAST>`
(`FrozenWorld::freeze`'s inputs, `src/freeze.rs:517`). Measured today: **`FrozenWorld` carries no
`Serialize`, and the tree contains no serde / bincode / rkyv / postcard at all.** So this is new ground,
and the interesting part is the split:

- **pure data** — parseable, expandable, checkable forms; type entries; macro definitions;
- **handles that cannot serialise** — native/builtin functions, `Arc<dyn …>`, loaders, anything holding
  a Rust fn pointer.

⭑ **A handle does not have to be cached — it has to be RE-REGISTERABLE BY NAME at load.** Report which
parts fall on which side, and whether the non-data side is re-registerable cheaply. That distinction,
not raw serialisability, is the design.

### 3. HOW FAST DOES A ROUND-TRIP OF THIS SIZE GO?

Prototype the round-trip of the **largest serialisable component** at **representative size** — not a
toy — and time both directions, warm and cold.

⚠ **The asymmetry that makes a crude prototype worth building**: a crude serializer is a *pessimistic*
speed estimate. If crude is already fast enough, **GO is proven**. If crude is slow, that proves
**nothing** — a zero-copy/mmap format could be an order faster. Say which conclusion your number
supports and which it does not.

## The deliverable

A **FINDING that says GO or NO-GO with numbers**, and if GO, the shape: what is cached, what is
re-registered by name, what load costs, and what fraction of the 430 ms it would actually remove.

## Trap-doors

1. ⛔ **Boot must not change.** Any prototype lives behind a test, a feature, or an env gate. Three runs
   proving the default path is untouched, as the census stone did.
2. ⛔ **Optimise nothing else**, and do **not** touch `.config/nextest.toml`.
3. **Cold vs warm is real here**: `stdlib-parse` moves 29 → 48 ms cold. Report both for any number that
   will be quoted later; this campaign's earlier hand-timings are ambiguous by ~15 ms because nobody did.
4. **Do not cache what can be re-registered.** A cache that serialises native handles is both impossible
   and unnecessary.
5. **A payload that is not representative proves nothing.** If you cannot build the real thing, build
   the biggest real component and say so.
6. ⛔ **Name no fix beyond the shape the measurement supports.** GO/NO-GO plus the numbers is the job.

## Out of scope

- **Building the cache.** That is the next stone, and only if this one says GO.
- **The two sharpening follow-ons** the profile named — attributing `check:body-infer`'s 109 ms to files
  (impossible today: no span to stamp), and per-form expansion inside `telemetry/journal.wat` (~100×
  outlier). Useful for tuning a fix, not for deciding one.
- **The queue promotion**, parked on `queue-promotion-blocked-on-startup-cost` (`6136d144f`), and its
  topic/bracket successors. They are blocked *by* this cost and are the reason it matters.
