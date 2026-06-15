# BRIEF — Arc 267: parametric protocol bounds (one arm in `assignable`)

Executor: Shadowdancer (sonnet). Anchor: `/home/watmin/work/holon/wat-rs/` (verify `pwd`; ONLY here;
ignore `.claude/worktrees/`). Run `cargo` PLAINLY (no setsid/timeout). Trust your own clean build over
rust-analyzer (its mid-edit snapshots lie). **Do NOT commit — the Inquisitor weighs.** Full rationale:
`DESIGN.md` (this dir).

## Work in one paragraph
A `Parametric` type (`Box<i64>`, `Thread'<I,O>`) is not accepted where a `Path` protocol bound (`:P`)
is expected, even though its constructor `extend-type`s `:P`. Fix: add ONE arm to `assignable` so a
`Parametric { head }` actual consults `is_subtype(head, ep)` against a `Path(ep)` expected. That's the
whole change — one function, a few lines.

## The room (one site)

**`src/check.rs:13673` — `fn assignable`.** Current body:
```rust
let a = reduce(&walk(actual, subst), subst, types);
let e = reduce(&walk(expected, subst), subst, types);
if let (TypeExpr::Path(ap), TypeExpr::Path(ep)) = (&a, &e) {
    if ap != ep && crate::types::is_subtype(ap, ep, types) {
        return true;
    }
}
unify(actual, expected, subst, types).is_ok()
```
Add, immediately after the existing `(Path, Path)` block (before the `unify` fallthrough):
```rust
// Arc 267 — a parametric type satisfies a plain protocol bound iff its CONSTRUCTOR
// extend-types the protocol. Edge keys carry the leading colon (types.rs:1402);
// Parametric.head does not — reconcile with `format!(":{head}")`.
if let (TypeExpr::Parametric { head, .. }, TypeExpr::Path(ep)) = (&a, &e) {
    if crate::types::is_subtype(&format!(":{head}"), ep, types) {
        return true;
    }
}
```
That is the entire fix. Type args are irrelevant (the edge is on the constructor). `is_subtype` already
walks the DAG transitively.

## Gate (run all; report verbatim from YOUR runs)
```
cargo test --release -p wat --test probe_arc267_parametric_extend_type                  # 1 passed (RED→GREEN: Box<i64> satisfies :t::Tagged)
cargo test --release -p wat --test probe_arc209_handle_protocol -- --test-threads=1     # 1 passed (end-to-end: Thread' satisfies :wat::kernel::Spawned)
cargo test --release -p wat --test probe_arc232_2_protocol_assignable                   # passes (non-parametric path UNBROKEN)
cargo test --release -p wat --test probe_arc232_3_protocol_dispatch                     # passes (dispatch UNBROKEN)
cargo test --release -p wat --lib -- --test-threads=1                                   # zero NEW vs baseline 917/36
cargo test --release -p wat --test nursery -- --test-threads=1                          # zero NEW vs baseline 895/4
cargo test --release --workspace --no-run                                               # compiles
```

## STOP triggers (REJECT — surface; do not improvise)
1. The `format!(":{head}")` reconciliation doesn't match the registered edge key (the probe stays RED
   after the arm) → STOP; report the actual edge-key string vs the head string (don't guess another
   form blindly — surface the mismatch).
2. The new arm breaks any 232 non-parametric case (probe_arc232_2/3 regress) → STOP.
3. The fix would require touching `unify`, `is_subtype`, or `register_subtype` → STOP (the design says
   the change is one arm in `assignable` only).
4. Any lib/nursery test that was green at baseline goes red → STOP and report which.

## Blast radius
`src/check.rs` — `fn assignable` ONLY (one added arm). NO changes to `unify`/`is_subtype`/
`register_subtype`/the 232 forms. The probes are already committed.

## Return
Report: the exact arm added (file:line); every gate command's counts from YOUR runs; confirmation the
232 non-parametric probes still pass; any honest delta. If a STOP fires, STOP and report. Do NOT commit.
