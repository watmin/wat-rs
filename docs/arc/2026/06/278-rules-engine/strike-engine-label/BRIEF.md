# BRIEF — make the `(engine)` label name its evidence, and gate it

Three arms in the cost harness carry `(engine)`. One of them times a raw `FxHashSet` insert and is
not the engine at all — and this class already recurred once (`b7d9d8e90`, *"the benchmark called the
wrong arm 'the engine' for eleven days"*). Make the label state the production function it calls, and
gate that the name resolves outside the test tree. Read `DESIGN.md` first — its ★ explains why the
label carries its evidence instead of a parser inferring it, and its ⚠ names the self-vouching trap.

## Read in order

1. `src/rete/kernel/tests/gather_probe_cost.rs:170-182` — the table whose `S` row claims `(engine)`,
   and `:90-106` where its body does `s.insert(f.clone())`. **This is the false one.**
2. `src/rete/kernel/tests/gather_probe_cost.rs:289` and the `super::seen_insert` calls above it — a
   **true** one.
3. `src/rete/kernel/tests/accum_cost.rs:1354` and `:1101`'s `intern_val` — the other true one, and
   C7's cited model.
4. `src/rete/kernel/fire/delta.rs:181` (`seen_insert`) and `src/rete/compiled_cond.rs` (`intern_val`)
   — the production definitions your gate must resolve to.
5. `tests/lint/rete_citation_resolves.rs` — landed two strikes ago. Its resolver **strips comments
   and excludes its own file**; both problems recur here. Read how it did it.
6. `tests/lint/every_walking_gate_declares_non_vacuity.rs` — your gate is in its population.

## The work

1. **Convert the two true labels** to `(engine: <fn>)` — `seen_insert` and `intern_val`.
2. **Drop the label** on `gather_probe_cost.rs:176`. It has no production function to name because it
   is not the production path; the `P` arm already times that. **Do not invent a name for it** — say
   in the report what you relabelled it to and why.
3. **The gate**: every `(engine: X)` resolves `X` to a definition under `src/rete/` **outside**
   `kernel/tests/`, and the file carrying the label must call `X`.

## Traps named in advance — each with its step

1. **★ The named fn must resolve OUTSIDE the test tree.** A test helper called `seen_insert` would
   otherwise satisfy the label with a fixture. **Step:** resolve against `src/rete/` minus
   `kernel/tests/`, and drive it — put a decoy `fn seen_insert` in a test file and confirm the gate
   still reds if the real one is gone.
2. **★ A bare `(engine)` with no name must RED.** Otherwise the old spelling silently survives and the
   gate polices only the sites already converted. **Step:** mutation — restore a bare `(engine)` and
   confirm.
3. **Population is 3. Volume cannot validate this gate.** **Step:** every arm mutation-proven
   individually, plus a non-vacuity floor — a gate over three sites that finds zero must RED, not
   pass.
4. **C2's line numbers are stale** — `accum_cost.rs:1383` names no label; C1's sweep moved them.
   **Step:** re-derive every site yourself; report what you find, not what the row says.
5. **Your gate is a walking gate.** **Step:** `NON-VACUITY` declaration with a measured floor.
6. **`binary_id(wat::lint)` is not clippy.** Three riders have been green there and had clippy RED on
   new test code. **Step:** run the lint binary; keep the test code idiomatic.

## STOP triggers

- **STOP-1** — if the `S` arm turns out to be a real production path under some spelling I have not
  found, STOP and report. DESIGN's claim that it is not the engine rests on `seen_insert` being the
  engine's route, which the `P` arm times.
- **STOP-2** — if any currently-green test goes red, STOP and report which.
- **STOP-3** — if a fourth `(engine)` site exists that I did not name, STOP and report it before
  converting anything.

## Shape to copy

`docs/arc/2026/06/278-rules-engine/strike-rune-vocabularies/` — last strike, same directory: a closed
set, a gate, and an honest statement of what the gate does *not* check.

## The one thing worth more than the fix

**Tell me where this brief was thin.** Twenty-five riders before you each returned a prescription of
mine that did not survive contact. The last refuted **both** of my premises — the vocabulary I called
undefined was authored upstream, and the six sites I called mislabelled were correct. If a step here
is wrong, unnecessary, or impossible, say it plainly.
