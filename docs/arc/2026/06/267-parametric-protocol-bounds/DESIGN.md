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

## The fix — TWO parts (corrected after the strike found the runtime half)

**A `Foo<…>` satisfies `:P` iff the CONSTRUCTOR `Foo` extend-types `:P`.** This must hold at BOTH the
check layer (so a `:P`-typed param accepts the value) AND the runtime-dispatch layer (so
`(:P/method recv)` finds the impl by the receiver's concrete type). The first strike applied part 1
and the probe — running end-to-end, not just type-checking — caught that part 2 was missing
(runtime.rs:4953 only recognized `Record` receivers; an explicit `// STOP-2: only Record variants…`
comment confirms 232.3 scoped dispatch to Records knowingly).

**Part 1 — check (`assignable`, check.rs:13681).** A `Parametric { head }` actual against a `Path(ep)`
expected consults `is_subtype(head, ep)`. Args are irrelevant (the edge is on the constructor). Edge
keys are keyed WITH the leading colon (`register_subtype(":wat::holon::Record", …)`, types.rs:1402),
but `Parametric.head` is stored WITHOUT it (`head: "wat::kernel::Listener'"`, check.rs:10299) — so
reconcile as `format!(":{head}")`:
```rust
if let (TypeExpr::Parametric { head, .. }, TypeExpr::Path(ep)) = (&a, &e) {
    if crate::types::is_subtype(&format!(":{head}"), ep, types) { return true; }
}
```

**Part 2 — runtime dispatch (`runtime.rs:4953`).** The `concrete_type_fqdn` match recognizes only
`Value::wat__Record`/`wat__holon__Record` (via `class_fqdn`, which it colon-prefixes). Extend it to
the other receiver shapes — both already carry the colon-prefixed FQDN, so use them directly:
```rust
Value::Struct(sv) => sv.type_name.clone(),            // e.g. ":t::Box" — already colon-prefixed
Value::RustOpaque(inner) => inner.type_path.clone(),  // e.g. ":wat::kernel::Thread'" — already colon-prefixed
```
(Grounded: `StructValue.type_name` is stored colon-form — runtime.rs:18753 `":wat::kernel::Bound"`;
`RustOpaque.type_path` is colon-form — `THREAD_PEER_TYPE_PATH = ":wat::kernel::Thread'"`; both match
the `extend:<P>:<T>` key, whose `<T>` is the colon-form FQDN the Record path also produces.) The
`other_val =>` error arm stays as the genuine fallback (a non-dispatchable receiver). No over-
acceptance: a Struct/opaque that doesn't extend the protocol still fails the `extend_key` lookup →
the existing clean "does not extend protocol" error.

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
