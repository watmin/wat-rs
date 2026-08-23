# SCORE — set the angle form ablaze

**Mode C, and the STOP that mattered was one my brief did not contain.** The rider executed exactly
as briefed, the wall fired, and the census came back **7 sites in ONE role** against a population of
~18 across four. The gap is a defect in the brief, not the work.

## What it delivered

```
Summary [ 38.475s ] 4882 tests run: 1865 passed, 3017 failed, 19 skipped     exit 100
```

Seven distinct refused keywords, all **DECL-NAME**, classified without ambiguity. Both halves of the
reachability test the brief demanded were satisfied:

- **written** — `src/types.rs:5960`, `:my::Container<T>`, literal text in a unit-test fixture
- **minted** — `wat/cache.wat:195`, `:wat::cache::lru-svc::State<K,V>`, assembled at expand time by
  `wat/service.wat:626`'s `string::interpolate "{b}::State{p}"` and handed to `keyword-node`,
  appearing in no file anywhere

So the design premise held: **a parse-door wall does see minted names.** That was the thing worth
proving and it is proven.

Wall reverted, `git status` clean, nothing staged, no conversions attempted.

## ⛔ MY BRIEF'S FIRST DEFECT — I aimed at a DECL-ONLY door and asked it to census every role

The brief called `src/types.rs:4608` *"the single place a keyword becomes a parametric type."* It is
not. It is inside `parse_declared_name`, which has **five call sites, every one a declaration
registration** — `defstruct`/`defrecord` (:4062), `newtype` (:4258), `typealias` (:4308),
`typeunion` (:4367), `defsurface` (:4493).

**Type ANNOTATIONS never reach it.** They resolve through the separate `parse_type_expr*` family
(38 references). So `bracket.wat:448`'s `[self <- ~runner-self-kw]` — the worked ANNOTATION example
I put in the brief myself — **cannot be caught by this wall, no matter how many times it runs.**

★ Sixth instance today of `[[feedback_a_slot_with_two_implementations_is_two_slots]]`, and the most
avoidable: I had `parse_type_node` described in my own notes as *"the substrate's one door that reads
all four type node shapes"* and transplanted that confidence onto a **different function** whose name
also starts with `parse_`. There are two parse families — one for a declaration's own NAME, one for a
type REFERENCE — and the roles I asked the census to distinguish map exactly onto them.

## ⛔ MY BRIEF'S SECOND DEFECT — a fail-fast wall CANNOT enumerate

`parse_declared_name` returns `Err` on the first violation, and the stdlib boot is fail-fast, so
**every** test process aborted at the same site: `wat/cache.wat:195`, the first `defservice` in load
order. 3017 failures, seven distinct raw values, one root.

The rider proved the masking is total rather than assuming it — `grep -oP ':raw "[^"]*"'` over the
whole captured log (not a window), and an independent `--check wat/service.wat` that produced the
identical `cache.wat:195` error because it too bootstraps the stdlib first. It also traced the minted
site far enough to find the sibling `{b}::Record{p}` at `service.wat:634` **that never got a turn**,
because registration aborts on the first `?`.

★ **A checker that raises on the first violation enumerates exactly one thing per run.** I asked a
raising mechanism for a census. The substrate already draws this line and I did not look:

```
src/check.rs    275  local_errors.push(…)      COLLECTS — reports every violation
src/types.rs     52  return Err(TypeError…)    ABORTS   — reports the first
```

**The census wall belongs in the layer that collects.** That is not a tuning detail; it is the
difference between a list and a single name.

## The rider's judgement call, and why I think it was slightly under-called

It declined STOP-1 on the grounds that the wall was not vacuous — 3017 failures, both name kinds
caught. That is a fair reading of STOP-1 as I wrote it ("floor green, or only a handful scream").

But the *purpose* was a census of ~18 sites in four roles, and what came back is one role, structurally.
By purpose it was STOP-1; by the letter I wrote, it was not. **That is my wording, not its judgement**
— and it reported the structural reason clearly enough that the correction is obvious, which is what a
STOP is for.

## What survives, and what the corrected stone is

Survives: the door mapping (`parse_declared_name` = DECL-NAME; `parse_type_expr*` = reference), the
proof that a parse wall catches minted names, the seven DECL-NAME sites, and the `service.wat:634`
sibling that never fired.

The corrected stone needs **two changes to the instrument**, not to the target:

1. **Walls at BOTH doors** — `parse_declared_name` for DECL-NAME, `parse_type_expr*` for every
   reference and annotation.
2. **COLLECT, do not raise.** Accumulate every violation and report at the end, or the stdlib's
   fail-fast boot hands back one site per run forever.

⚠ And a warning for whoever writes it: `parse_type_expr*` is on the hot path for every type in the
substrate. A wall there is not free, and "does this change what the KEYWORD path accepts" is the row
that decides it — same as the six guards.
