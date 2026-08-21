# BRIEF — arc 109, β-ii-c: `type-params-used-in`, and `defservice` stops over-stamping

Two halves, in this order. **(1)** a new pure-total intrinsic that answers *"which of these type
params appear anywhere in this AST?"* **(2)** `defservice` uses it, so each generated type carries
only the params it consumes.

Design: `DESIGN-STONE-binder-beta-ii-c.md`. Ruling: option **A**, 4/4, recorded there with the table.

## Why the intrinsic must exist (do not try to write this in wat)

A program-body macro **cannot** perform an arbitrary-depth AST search: no recursion, no helper
`defn` (F5 refuses at DEFINITION — measured, 3029 tests red), no `mapv` over a bare primitive
(measured). Read `NOTE-the-F5-allow-list-and-what-a-macro-body-may-call.md` before writing anything.

## Half 1 — the intrinsic

**Home it in `src/intrinsic/reflect.rs`** as a `#[wat_intrinsic]`. That is the modern registry (arc
255's rehoming direction) and it already holds `:wat::core::show-source` and `:wat::core::render-doc`
— source/AST-adjacent siblings. Do **not** add it beside `ast->children` in `src/edn_shim.rs`; that
is the old home.

```
:wat::core::type-params-used-in
  (params <- Vector<wat::WatAST>, node <- wat::WatAST) -> Vector<wat::WatAST>
```

Returns the subset of `params`, **in the order given**, that appear anywhere in `node`. Pure, total,
deterministic; no allocation-order dependence.

⚠ **A type param can be INSIDE a token.** With today's keyword spelling, `:wat::cache::Lru<K,V>` is
ONE node whose name carries `K` and `V` as text. So the search must inspect the token's own text,
not merely walk children — a walk alone finds nothing and the whole stone silently reports "consumes
nothing" for every clause. Match on type-parameter boundaries (`<`, `,`, `>`, and the end), never a
bare substring: `K` must not match inside `Key` or `KV`.

★ **The runtime dispatch and the F5 allow-list are TWO INDEPENDENT LISTS.** Measured: zero
`#[wat_intrinsic]` verbs currently appear in `src/macros/eval.rs`'s `is_pure_total`. Registering the
intrinsic does **not** admit it to macro bodies. **Add it to `is_pure_total` as well**, beside the
existing `:wat::core::ast-*` entries — it meets that list's stated criterion (pure-total) and is
useless to this stone without it.

## Half 2 — `defservice` stops over-stamping

`wat/service.wat` builds each generated type's name by appending `fqdn-tp` (the bracketed param
string). Six sites, measured:

```
:525 {b}::State{p}   :528 {b}::Record{p}   :807 {b}::Op{p}
:855 {b}::Handle{p}  :915 {b}::Admin{p}    :1080 {b}::Op{p}
```

For each, compute that type's OWN params — `type-params-used-in(fqdn-tp-syms, <its field/member
vector>)` — and build its suffix from that subset instead of from `fqdn-tp`. `fqdn-tp-syms` already
exists (β-ii-a′). The empty result must produce `""`, giving a monomorphic name.

The reference sites for each generated type must use the SAME subset, or a declaration and its
references will disagree. Derive the per-type suffix ONCE, beside where its name is built.

## STOP triggers

1. **STOP-1** — if the intrinsic cannot be reached from a macro body after registering it, you have
   hit the second list. Add it to `is_pure_total`. If it still cannot, STOP and report.
2. **STOP-2** — no helper `defn`, no `mapv` over a bare primitive keyword, no new wat-level walker.
   F5 refuses at definition and takes the stdlib down.
3. **STOP-3** — if a generated type's field vector is not available at the point its name is built,
   STOP and report which. Reordering that macro's `let` is a real change and not yours to guess at.
4. **STOP-4** — do not edit `wat/cache.wat` or any other service definition. Services are the
   acceptance targets, not the work.

## Acceptance — the wall proves this, not a scorecard I invented

`docs/arc/2026/04/109-kill-std/PATCH-param-spec-consumption-wall.patch` (291 lines) is the parked
consumption wall. It is what FOUND this defect. **The orchestrator re-applies it after your edits**;
if `defservice` still over-stamps, it refuses the stdlib and names the offending generated type.

You do not apply the patch and you do not run the floor.

## Blast radius

`src/intrinsic/reflect.rs` (one intrinsic) · `src/macros/eval.rs` (one allow-list entry) ·
`wat/service.wat` (six per-type suffix derivations). No other service. No `.wat` corpus edits.

## How this lands

You are a rider. **Text edits only.** Do not build, commit, stash, or revert. `wat/service.wat` is
baked into the binary by `include_str!` at RUST-compile time, so `--check` reflects the LAST BUILD
and will warn it is stale — expected. Trace by reading; report what you verified by reading versus
what you could not verify at all.

Report: the diff; the intrinsic's exact signature and boundary-matching rule; which six suffixes you
derived and what each computed for `lru-svc<K,V>`; and anything on disk contradicting this brief.
