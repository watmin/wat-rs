# NOTE — a finite f64 at or above 1e16 does not round-trip through EDN

**Found 2026-08-05**, incidentally, while a rider built a scratch probe for the f64 fallback rows.
Orthogonal to that stone and deliberately not fixed there. Filed here because 278 is the active arc
and this is where it surfaced; the **owner is `crates/wat-edn`**, and the topical ancestors are arc
**086-edn-roundtrip-and-natural** (INSCRIBED — do not amend it; this is a new finding, not a
correction to what shipped) and arc **218-wat-edn-impeccable**.

## The defect

`crates/wat-edn/src/writer.rs`'s `write_float`:

```rust
// Rust's default formatter elides ".0" for whole floats which would
// round-trip back as integers. Force a fractional component.
if f == f.trunc() && f.abs() < 1e16 {
    write!(out, "{:.1}", f).unwrap();
} else {
    write!(out, "{}", f).unwrap();
}
```

**The `if` exists to stop a whole float being written as an integer. Above `1e16` control falls to
the `else`, which does exactly that** — `{}` is Rust's `Display`, and `Display` never uses scientific
notation for `f64`. So the guard's own stated purpose is defeated by its own bound, at precisely the
magnitudes where the resulting integer is unreadable.

Proven by run, 2026-08-05:

```
(:wat::kernel::println (:wat::core::f64::* 1.0 1e200))
  -> 1000000000000000000000000000000000000000000000000000000000000000000…  (201 digits)
```

No `.`, no `e`. An EDN reader takes that as an **integer**, and it is ~183 orders of magnitude beyond
`i64::MAX`, so the parse fails. The value writes successfully and fails on read — the asymmetry that
makes it worth writing down.

Non-finite values are fine and are NOT this defect: `write_float` returns early with
`#wat-edn.float/{nan,inf,neg-inf}` sentinel tags, which do round-trip (verified same session).

## How it surfaced, which is the part worth keeping

The rider put a `1e200` literal in a scratch `.wat` to exercise overflow-to-Inf. That turned
`probe_arc170_edn_bridge_unspellable::c03_the_whole_corpus_crosses_the_wire` RED — a gate requiring
every `.wat` under `wat-scripts/` to survive `program_to_edn → edn_to_program`.

**The gate caught a defect nobody was looking for, in a file written for an unrelated reason** —
`QVOD TVEBAMVR, NOS TVETVR` (R64) recurring, and the same mechanism: the corpus's only exposure to
this requirement was a throwaway, because no production `.wat` happens to carry a literal that large.
It stayed invisible for as long as nobody wrote one.

The rider worked around it correctly for its own purposes — it built the overflow value at *runtime*
by repeated squaring from `9.9e15`, so no huge literal appears in source, and documented that in the
probe's header. **The workaround is right for the probe and wrong as a resolution**; the writer is
still broken for any large float that crosses the wire.

## Why it matters beyond a scratch file

`records-are-EDN` is a project law and the wire is EDN everywhere — `defservice` messages, the
durable `{facts, rules}` snapshot (R5: the blob IS the program), telemetry rows, stdio. Any computed
`f64` that grows past `1e16` and then crosses a boundary is written in a form the far side cannot
read. Nothing in the corpus does this today, which is exactly why it has never fired.

## What closing it needs

1. **Decide the rendering.** `{:?}` (Rust `Debug`) does use scientific notation for large magnitudes
   and always emits a `.` or an `e`, so it round-trips; `{:e}` forces exponent form always. Either is
   a candidate — **this is a wire-format decision, not a formatting nicety**, since it changes bytes
   that cross a boundary and land in durable stores.
2. **A round-trip property test over the float domain**, not another point fixture: the existing
   coverage is point cases and this class hid under them. Include `1e16` itself (the boundary), a
   value just below, `1e200`, `f64::MAX`, and the negatives.
3. **Check the reader's side too.** This note grounds the WRITER only. Whether the reader accepts
   scientific notation on input is unverified here — do not assume it from the writer's fix.

## Related, on the disk

- `crates/wat-edn/src/writer.rs` — `write_float`, the site.
- `tests/.../probe_arc170_edn_bridge_unspellable.rs` — `c03_the_whole_corpus_crosses_the_wire`, the
  gate that caught it.
- `wat-scripts/scratch-pad/probe-f64-fallback-rows.wat` — carries the runtime-squaring workaround and
  a header comment explaining why the literal is not there.
- `docs/arc/2026/04/086-edn-roundtrip-and-natural/INSCRIPTION.md` — the arc that owns this topic and
  is closed; per the immutability rule it stays as it shipped.
