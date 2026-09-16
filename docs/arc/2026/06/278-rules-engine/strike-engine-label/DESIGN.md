# DESIGN-STONE — a label that says `(engine)` names the engine function it calls

> **Origin (2026-09-01).** Class **C7** — the *rule* `intueri` proposed — with **C2** as its one
> live instance. Driven at HEAD `7ed71cf12`.

## Why — the class has already recurred once

`b7d9d8e90` is titled *"the benchmark called the wrong arm 'the engine' for eleven days."* It fixed
one mislabel and its message named the class. **One instance still stands.**

All three `(engine)` labels in the tree, each driven to its body:

| label | body calls | verdict |
|---|---|---|
| `V  FxHashMap<Value> (engine)` — `accum_cost.rs:1354` | `intern_val` (`compiled_cond`) | ✅ real |
| `P  PV iter + seen_insert (engine)` — `gather_probe_cost.rs:289` | `super::seen_insert` (`fire/delta.rs:181`) | ✅ real |
| **`S  FxHashSet<Value> insert (engine)` — `gather_probe_cost.rs:176`** | **`s.insert(f.clone())` — a raw set** | ❌ **mislabelled** |

Every cost number this arc has quoted came off this harness. A row that says *engine* and times
something else is not a slow measurement, it is **a wrong one wearing the production label**.

## ⛔ C2's ROW IS WRONG, AND ITS CAUSE IS INSTRUCTIVE

C2 reads: *"`gather_probe_cost.rs:176`; `accum_cost.rs:1383` — **two more** arms labelled `(engine)`
that are not."* Measured: **one**, not two. `accum_cost.rs:1383` is a `const RUNS` inside
`accum_exec_ops_split` and carries no label at all.

**C1's own sweep rewrote 103 accumulators across these same files the day after the vigilia was
cast** — so a row on this list was invalidated by a strike from this list. Line numbers in an
inherited row are the first thing to re-derive, not the last.

## ★ THE ONE CONTRACT DECISION

**An `(engine)` label NAMES the production function it claims to time** — `(engine: seen_insert)` —
and a gate resolves that name to a real production site outside the test tree, in a file that
actually calls it.

**Not an inferred mapping.** The alternative is to parse each table's format rows, match them
positionally to `ms(x)` arguments, resolve `x` to its assigning block, and scan that block — real
parser work, fragile at every step, and wrong in a way no one would notice. **Make the label carry
its own evidence instead.** This is the rung the previous strike taught: *spelling beats a
vocabulary; a rule that makes the author write the claim correctly beats a checker that guesses at
it.*

## ⚠ THE NAMED FUNCTION MUST RESOLVE OUTSIDE THE TEST TREE

A gate that accepts any function of that name would be satisfied by a **test helper** called
`seen_insert` — the label vouching for itself with a fixture. The name must resolve to a definition
under `src/rete/` **outside** `kernel/tests/`. This session has hit the self-vouching shape four
times; it is the default failure of every resolver written here.

## The one mislabelled arm is NOT fixed by naming

`gather_probe_cost.rs:176` times a raw `FxHashSet<Value>` insert. There is no production function
for it to name, because **it is not the production path** — the engine's route is `seen_insert`,
which the `P` arm already times. Its `(engine)` claim is simply false, and the fix is to **drop the
label**, not to invent a name for it.

## Blast radius

Three label sites in two files, plus one gate under `tests/lint/`. No `src/` behaviour change.

## Out of scope — AFFIRMATIVELY CUT

- **C3, C4, C5, C6.** Four *different* instrument defects — a phase mark the engine never emits, a
  production arm handed an empty `bind_only`, a tautological assert, a phase rebuilt from the retired
  interpreter. None is an `(engine)`-label question and none is fixed by this rule. Their own rows.
- **Widening to every label in every cost table.** `(identity)`, `(spec)` and the bare arms make no
  production claim. This gate polices **one word**, which is why it can be exact.
