# BRIEF — STONE A-2-ii-a: a resolved name gets the same doors as a head

Restore one invariant in `src/rete/purity.rs`: **the classifier's verdict on a name depends on the
name, never on how the name was reached.** Today a record field accessor classifies `true` as a call
head and `false` when reached through an environment binding — the resolved path consults
`intrinsic_meta` alone, while a head gets four doors. DESIGN:
`docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-A-2-ii-a-a-resolved-name-gets-the-same-doors-as-a-head.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or
`scripts/floor.sh`. You may run the pre-existing `target/release/wat` for a fast read, remembering
it does not contain your Rust changes. **You may not spawn sub-agents.** Work only in
`/home/john/work/holon/wat-rs`; verify with `pwd` first. Do not commit, push, stash, revert, or
`git checkout --` anything.

## Read in order

1. The DESIGN above — the invariant, and why two live substrate sites depend on it.
2. `src/rete/purity.rs`, `fn head_ok` — the four doors, in order: `constructor_meta` →
   `accessor_meta` → `sym.has_function`/`classify_fn` → `intrinsic_meta` → deny.
3. `src/rete/purity.rs`, `fn classify_closure` (shipped in A-2-i) — its `FunctionBody::Native` arm
   consults `intrinsic_meta` on `f.name` and nothing else. That is the gap.
4. `src/rete/purity.rs:884`, `fn accessor_meta` — the door the resolved path never opens.
5. `src/rete/purity.rs`, the `sym.has_function` → `classify_fn` call site — read the comment above
   it. It explains why `ClassifyCtx::Static` is FORCED there on scope grounds. Your change must not
   disturb that.

## The work

### 1 — route a resolved NAME through the head ladder

In `classify_closure`'s `FunctionBody::Native` arm: when the resolved `Function` carries a
`name`, classify that name the way a head is classified — through `head_ok` — instead of consulting
`intrinsic_meta` alone. Delegating to `head_ok` keeps this as ONE mechanism; do not copy the
constructor/accessor doors into a second ladder that would drift out of step with the first.

An **anonymous** native (`name: None`) keeps today's behaviour exactly: default-deny.

### 2 — carry the guards across the delegation

Both recursion guards must survive the hand-off: the FQDN `seen: HashSet<String>` and the
`closure_seen: HashSet<*const Function>` pointer set A-2-i introduced. A named native reachable from
its own body must hit a back-edge, not recurse. If the borrow shape makes carrying both awkward,
solve it — do not drop one.

### 3 — the probe

Write `wat-scripts/scratch-pad/255-probe-a-resolved-name-agrees-with-a-head.wat`. It must assert the
invariant directly, and include the negative rows — a probe that only shows the fix is not evidence
it is bounded:

- a record field accessor, **as a head** and **through a binding** → `true` / `true`, agreeing;
- an **effectful** fn through a binding → `false` (no widening);
- an **anonymous** native through a binding, if you can construct one → `false`.

Follow the shape of the two existing probes in `wat-scripts/scratch-pad/` (`…-follows-a-capture.wat`,
`…-cannot-see-through-a-closure.wat`) — a header comment recording what was measured and why, then
`:user::main` printing one row per line.

## Blast radius

`src/rete/purity.rs` · new `wat-scripts/scratch-pad/255-probe-a-resolved-name-agrees-with-a-head.wat`.
No new types. No changes to `src/intrinsic/rete.rs`, `src/freeze.rs`, `src/collection/transform.rs`,
or any `sort` site. No verb's registration changes.

## STOP triggers — each rejects; ship nothing further on that point and report

**STOP-1 — one ladder, not two.** If delegating to `head_ok` proves impossible (a borrow conflict, a
signature cycle), STOP and report the exact shape you hit. Do NOT copy `accessor_meta` /
`constructor_meta` calls into `classify_closure` as a parallel ladder — two ladders drifting apart
is the defect this stone exists to remove, not a fallback.

**STOP-2 — the guards do not get dropped.** If carrying both `seen` and `closure_seen` across the
delegation fights the borrow checker, STOP and report. Dropping either one to make it compile
reintroduces an unguarded cycle.

**STOP-3 — no widening to make a row pass.** If the accessor rows will not agree without admitting a
name you cannot actually resolve, STOP and report the shape.

**STOP-4 — the forced `Static` stays forced.** The `sym.has_function` → `classify_fn` site passes
`ClassifyCtx::Static` on a scope argument recorded in the comment above it. If your change appears
to require forwarding a caller `ctx` there, STOP and report — that would be a scope bug, not a fix.

## Report

Per-file diff summary; the exact output of your new probe and of both existing probes as your
pre-existing binary reports them (noting it lacks your Rust changes); how you carried the guards
across the delegation. Then the part the orchestrator cannot reconstruct: what surprised you — a
door whose behaviour differed from its name, a resolution path the design did not predict, or a
place where delegating to `head_ok` read wrong.
