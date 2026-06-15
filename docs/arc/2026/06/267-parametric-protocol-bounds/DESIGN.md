# Arc 267 — parametric protocol bounds (parametric `extend-type` satisfaction)

> Opened 2026-06-14 as the arc-232-named follow-on. Arc 232 shipped protocol bounds for
> NON-parametric extenders and explicitly scoped out parametric ones — *"Parametric protocols — OUT
> of v1 unless a strike proves them load-bearing … if/when [a caller] does, a NEW arc opens"*
> (232 DESIGN.md:99 / INSCRIPTION.md:41). The arc-209 host seam is that caller; the FM-2-bis probe
> below is the strike. Grounded against HEAD `f5f206e5`.

## The gap (grounded)

`assignable` (check.rs:13681) consults `is_subtype` ONLY in the `(Path, Path)` arm:

```rust
if let (TypeExpr::Path(ap), TypeExpr::Path(ep)) = (&a, &e) {
    if ap != ep && crate::types::is_subtype(ap, ep, types) { return true; }
}
unify(actual, expected, subst, types).is_ok()
```

A `Parametric` actual (`Box<i64>`, or the real `Thread'<I,O>`/`Process'<I,O>` handles) against a
`Path` protocol bound (`:t::Tagged`, `:wat::kernel::Spawned`) never reaches the subtype check → falls
to `unify` → rejected, even though the constructor `extend-type`s the protocol.

## The fix (the one contract decision)

**A `Foo<…>` satisfies `:P` iff the CONSTRUCTOR `Foo` extend-types `:P`.** Add one arm to
`assignable`: a `Parametric { head }` actual against a `Path(ep)` expected consults
`is_subtype(head, ep)`. The type ARGS are irrelevant to satisfaction — the edge is registered on the
constructor (`extend-type :wat::kernel::Thread' :P`), so any instantiation satisfies it.

**Form reconciliation (load-bearing):** subtype edges are keyed WITH the leading colon
(`register_subtype(":wat::holon::Record", ":wat::Record")`, types.rs:1402), but `Parametric.head` is
stored WITHOUT it (`head: "wat::kernel::Listener'"`, check.rs:10299). So the head must be reconciled
as `format!(":{head}")` before the `is_subtype` lookup.

```rust
if let (TypeExpr::Parametric { head, .. }, TypeExpr::Path(ep)) = (&a, &e) {
    let head_path = format!(":{head}");
    if crate::types::is_subtype(&head_path, ep, types) { return true; }
}
```

`is_subtype` already walks the multi-parent DAG transitively, so a head with several `extend-type`
edges (or a chain) is handled unchanged.

## Scope / out (affirmative cuts)

- **Parametric PROTOCOLS** (`:P<T>` — a generic protocol) — still out; no caller. This arc is the
  inverse: a parametric *extender* of a *plain* protocol.
- **`unify` changes** — none; satisfaction is a subtype relation, it belongs in `assignable` (mirrors
  the existing `(Path, Path)` arm). `unify` stays structural.
- **`Path` actual vs `Parametric` expected** — not needed (protocol bounds are plain `Path`); not built.

## Probe

`tests/probe_arc267_parametric_extend_type.rs` (committed RED) — a parametric struct `Box<T>`
extend-typing a plain `:t::Tagged`; a fn `[x <- :t::Tagged]` fed a `Box<i64>`. RED at HEAD on exactly
the `assignable` gap; GREEN when the arm lands. (`tests/probe_arc209_handle_protocol.rs` is the same
fix proven end-to-end on the real opaque handles — a second, leg-level witness.)
