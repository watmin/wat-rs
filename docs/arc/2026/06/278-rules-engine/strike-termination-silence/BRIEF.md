# BRIEF — give the termination verdict a name for every state it can be in

`refuse_non_terminating` returns `Result<(), EvalBreak>`, and its `Ok(())` means three different
things: proven terminating, the rules were not a vector, and this rule had no AST to analyse.
Driven: a `Rule` with empty `:lhs`/`:rhs` — exactly the shape an imported Export's rules have —
makes `compile-all` answer `"Compiled"`. Split the verdict by type and make the one caller match
every arm; then qualify the sentence at `arm.rs:1294`, which claims a door that `import_export`
never calls. Read `DESIGN.md` beside this file first — its ★ pins the contract and states plainly
that **behaviour does not change**, and its "out of scope" cuts three shapes with reasons.

## Read in order

1. `src/rete/kernel/stratify.rs:833-845` — `refuse_non_terminating`'s signature and its first
   `Ok(())` (`:838`, the not-a-vector arm).
2. `src/rete/kernel/stratify.rs:849-855` — the `continue` whose comment says *"saying so is the
   honest outcome rather than passing it as proven"* and then says it to nobody. This is the arm
   the driven repro rides.
3. `src/rete/kernel/stratify.rs:891-895` — the nothing-computes early exit. **This one is a real
   proof** (371 of 381 corpus rules take it) and must come back `Proven`, not lumped in.
4. `src/rete/kernel/stratify.rs:983-989` — the graph-closed exit. Also a real proof.
5. `src/rete/kernel/arm.rs:1293-1305` — the **only** caller, and the sentence to qualify. Note it
   deliberately does not use `?`: *"the verdict must reach the converter, not unwind past it."*
   Your change must preserve that.
6. `src/rete/kernel/stratify.rs:334-342` — the module doc that already states the import gap
   correctly, and names `rules_lack_ast` (real, `fire/rules.rs:814`). `arm.rs:1294` must stop
   contradicting it.
7. `wat-scripts/scratch-pad/a5-termination-silence.wat` — the driven repro, in the tree.

## Sketch

```rust
pub(crate) enum TerminationVerdict {
    Proven,
    NotAnalysable { rules: usize },
    Refused(crate::runtime::EvalBreak),
}
```

Each `Ok(())` becomes the arm that names what it actually knew; `:853`'s `continue` increments the
count rather than vanishing. `arm.rs:1301` matches all three — `Refused` reaches the outcome
converter exactly as today, `Proven` and `NotAnalysable` both proceed.

## Blast radius

`src/rete/kernel/stratify.rs` and `src/rete/kernel/arm.rs`. Nothing else — one caller, enumerated.

## Traps named in advance — each with its step

1. **A mixed set is the interesting case.** If EVERY rule lacks an AST, `edges` is empty and the
   `:894` early exit fires before the `continue` is ever reached — so a naive probe proves the
   wrong arm. **Step:** the probe for `:853` needs at least one rule that DOES compute alongside
   the AST-less one, or it is measuring `:894`.
2. **`:894` and `:988` are proofs, not skips.** Lumping them into `NotAnalysable` would invert the
   finding and make 371 of 381 corpus rules read as unverified. **Step:** map each `Ok(())` site
   individually against DESIGN's table before you write the arms.
3. **The `?` is absent at `arm.rs:1301` on purpose.** *"The verdict must reach the converter, not
   unwind past it."* **Step:** keep `Refused` flowing to `outcome`, not through `?`.
4. **Behaviour must not change.** `NotAnalysable` proceeds. **Step:** if a currently-green test
   goes red, you have made it fatal — STOP and report rather than adjusting the test.
5. **New test code trips `wat::lint`.** Two strikes ago the floor went red on a `contains` in a new
   probe. **Step:** run `cargo nextest run --release -E 'binary_id(wat::lint)'` before reporting,
   and prefer exact `assert_eq!` on any deterministic value.
6. **The scratch `.wat` must keep loading.** `wat-scripts/**` is parsed and type-checked by
   `every_wat_scripts_file_loads`. **Step:** if you change the repro, re-run that gate.

## STOP triggers

- **STOP-1** — if `NotAnalysable` turns out to be unreachable from any test you can write, STOP and
  say so. A variant nothing can reach is a claim, and I would rather know than ship it.
- **STOP-2** — if any currently-green test goes red, STOP and report which.
- **STOP-3** — if qualifying `arm.rs:1294` requires a claim you cannot verify about a THIRD door
  (a hand-assembled `Session`), STOP and report what you found. Say only what the disk shows.

## Shape to copy

`docs/arc/2026/06/278-rules-engine/strike-acc-head/` — the strike immediately before this one, and
`strike-silent-zero/` for the split-by-type shape (A2b: one `Option`, two facts, same cure).

## The one thing worth more than the fix

**Tell me where this brief was thin.** Thirteen riders before you each returned a prescription of
mine that did not survive contact. The last one found that a trap I wrote had *authorized a
regression* — it told the rider not to add a refusal that this file's own module law requires —
and it cited the law back at me rather than shipping it silently. That was worth more than the
code. If a step here is wrong, unnecessary, or impossible, say it plainly.
