# DESIGN-STONE — the same wire bytes must not be accepted here and fatal there

> **Origin (2026-08-31).** Class A6 of `VIGILIA-2026-08-30-WORK-LIST.md`, found by
> `circumspicere`. Driven here at HEAD `a685a9d8d`. **The recorded severity was wrong** and the
> real one is worse in the way that matters.

## Why

`unpack_expr` (`export.rs:723`) recurses over wire-chosen nesting with no depth counter. Its
`:call`, `:and`, `:or`, `:if`, `:let`, `:match` arms all descend on caller-supplied structure.

### Driven, and the work list's "SIGSEGV" is not what happens

A `[:and [:and … [:lit 1]]]` tower poked into an Export's `progs` and imported:

```
depth=200 / 500 / 1000 / 2000 / 3000   →  import ACCEPTED a N-deep expr tower
depth=5000 / 20000 / 50000 / 200000    →  thread ... has overflowed its stack
                                          fatal runtime error: stack overflow, aborting   (SIGABRT)
```

Not a SIGSEGV — Rust's stack guard fires and **aborts**. It is not catchable: no `catch_unwind`
reaches it, no wat error, no span, and the process dies. (Same correction shape as B1's
`try_with`, where the failure was also an abort rather than the panic the design assumed.)

### ⛔ THE ACTUAL DEFECT: acceptance is a property of the importing THREAD, not of the format

The *same* 20,000-deep Export, driven both ways:

| importing thread | outcome |
|---|---|
| 2 MiB (the test harness's) | `fatal runtime error: stack overflow, aborting` |
| 256 MiB (spawned) | **ACCEPTED** |

So `import` has **no depth criterion at all**. What it accepts today is whatever the current
thread's remaining stack happens to allow — an environmental property that differs between the
main thread, a test thread, and a service worker. Two processes running identical code disagree
about whether the same bytes are a valid network, and the disagreement is resolved by an abort.

This is Class A's root once more: **an invariant proven at one door and assumed at the others.**
`compile-all` builds expressions from a parsed program whose nesting the parser already bounds;
`import_export` builds them from bytes and bounds nothing.

## ★ THE ONE CONTRACT DECISION

**The import door's recursive descent carries ONE depth budget, shared across every mutually
recursive unpack function on the path, and refuses past a STATED constant with `malformed` —
the same refusal the other four walls use.**

After this strike, whether an Export is accepted must be answerable from the declared format
alone, identically on every thread. A bound that only `unpack_expr` counts is the rung below and
it does not hold — see the cycle below.

## ⚠ THE CYCLE — a counter on `unpack_expr` alone is bypassed

Verified on the disk:

- `unpack_expr`'s **`:user` arm (`:779`) calls `unpack_prog`** (`:1011`), whose root calls
  `unpack_expr` again (`:1074`). A tower of `:user` nodes alternates between the two functions
  and never increments an expr-only counter.
- `unpack_expr`'s **`:match` arm (`:973`) calls `unpack_pat`** (`:567`), which recurses on
  `Pat::Variant` (`:616`).
- `unpack_prog` is *also* reached from `unpack_compiled_cond` (`:1357`), `:1401`, `:1452`, `:1512`
  — four more entries into the same cycle.

**One budget, threaded through all of them.** Counting only the function whose name is on the
finding is how a class regrows.

## The bound must be MEASURED, not chosen for roundness

Do not pick a number because it looks generous. **Measure the deepest nesting the existing corpus
actually produces** (instrument the descent, run the floor's export/import tests, report the max),
then state the bound as that maximum times a named multiplier, with both numbers written at the
constant. A bound nobody measured is a second unstated criterion replacing the first.

Ceiling from the drive: the smallest stack observed here dies between **3,000 and 5,000**, so the
bound must sit far below 3,000 to be honest on the smallest thread — and far above anything real.

## Blast radius

`src/rete/export.rs` only (the unpack descent + the constant), plus the probes in
`tests/rete/probe_arc278_export.rs` — which already carries `cool-export`, `import-one`,
`poke_named` and `seq_values`, so **no new fixture is needed**.

## Out of scope — AFFIRMATIVELY CUT

- **Making the abort catchable.** A stack guard abort is not a panic; `catch_unwind` cannot reach
  it. Refusing before the recursion is the only cure, which is this strike.
- **`check_expr_slots` / `check_pat_slots` getting their own bound** — they walk a tree
  `unpack_*` already produced, so a bounded unpack bounds them. **VERIFY that, do not assume it:**
  if any path reaches them on a tree that did not come through the import unpack, that is a
  separate door and its own finding.
- **Bounding the parser or `compile-all`.** A different door with a different budget; this strike
  is the wire.
