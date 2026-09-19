# DESIGN — every sweep names what it reads

**Drawn 2026-09-18**, builder-ruled: *"A -> B -> C as you've named it"*, after the four questions scored
**A four-YES, B four-YES, C not-YES-alone**. **NOT STRUCK. A is report-only: no behaviour changes.**

## Why this exists — the last attempt died of an incomplete door list

Tier B's phase 1 was *"prove the closure by reading the seven doors the spike instrumented."* Those seven
are **all read by one sweep** — the probe window wraps `P_CHECK_BODIES`. `check:restricted-call` reads an
**eighth** door nobody had instrumented (`CheckEnv::get_binding_metadata`, `check.rs:1679`), and one
user metadata-map reddened **9 stdlib bodies across 6 files**.

★★ The executor's line, and the reason this stone exists:

> **"A proof whose premise is a door census is only as good as the door list."**

⛔ **The eighth door was found by READING `check.rs`, not by the instrument built to find doors.** An
instrument finds only what it was pointed at. **This stone is a reading, not an instrument.**

## The work

For **each** of the four `ALL fns` sweeps —

```
check:retired-syntax(ALL fns)    11.13 ms
check:restricted-call(ALL fns)    3.07
check:def-position(ALL fns)       1.72
check:body-infer(ALL fns)       108.73        ⇒ 124.65 ms total, 67 % of accounted boot
```

— enumerate **every piece of state it consults**, and for each one answer: **can user code write it?**

Deliverable: one table, `sweep × input × user-writable? × how it is closed (or not)`.

## ⛔ THE COMPLETENESS ARGUMENT IS THE STONE

A list is worthless without a reason to believe it is complete. **Name your method and its stopping
rule.** The expected shape is a **transitive callee closure** from each sweep's entry point — every
function reachable, every `env`/`self` field read, every global or thread-local touched — carried until
the frontier is only leaf operations on owned data.

⚠ **Do not stop at a function signature.** The eighth door was reached *through a helper*, and
`check_program`'s own signature (`&[WatAST], &SymbolTable, &TypeEnv`) names none of it.

⭑ **If the closure cannot be completed** — a dynamic dispatch, a trait object, a registry keyed at
runtime — **say exactly where it becomes unbounded.** That is a finding, and it tells B what it must
wall off.

## What counts as "user-writable"

A piece of state is user-writable if **any** program a user can legally write can change it:

- directly — a `def`/`defn`/`defclause`/`defmacro`/`deftype`/`extend-type` they may declare;
- indirectly — metadata on **their own** name (the eighth door's mechanism), a load-file, an entry
  program, a REPL turn;
- ⛔ **and mind the reserved wall is a HALF-answer**: `:wat::` is unforgeable, so a door keyed only by
  `:wat::` names is closed *by construction*. A door keyed by **any** name — as
  `get_binding_metadata` is — is **not**, regardless of prefix.

## Out of scope

- **B** (the structural gate) and **C** (the elision). This stone changes no behaviour and takes no
  perf.
- Fixing anything it finds. A ninth door is a **finding**; C dying again is a cheap, correct outcome.
- The open items from earlier stones: the top-level-mention hole, the blame-inversion diagnostic, the
  rendezvous message, `Existing::Equivalent`, `check.rs:6068`, `env.rs:456`.

## Trap-doors

1. ⛔ **An instrument cannot prove completeness.** You may *use* one to cross-check, but the list comes
   from reading. Say which lines you read.
2. **The probe instruments ONE sweep.** `src/spike_probe.rs`'s seven doors are body-infer's. The other
   three sweeps have never been enumerated at all.
3. **`check:restricted-call` is the known-hardest** — it already surprised two stones. Its walker fires
   on every keyword leaf and consults metadata keyed by arbitrary names.
4. **Report the four sweeps separately.** They are different passes with different inputs; a merged
   table hides exactly the asymmetry that bit Tier B.
