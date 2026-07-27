# NOTE — pre-world decode is hand-written, everywhere it happens

> **Surfaced 2026-07-26**, arc 170's execve work, from a builder question about a
> three-tag boot decoder: *"you need to register these shapes as edn records?"*
> The answer was yes, and following it found the same gap this arc already owns.
>
> This is a pointer, not a decision. It records the mechanism and the measured
> scope; the fix's shape is 296's call.

## The finding

`#[derive(Edn)]` submits an `EdnSchema` into a **link-time** inventory
(`wat-edn/src/lib.rs:271`), and `register_builtin_types` drains it
(`types.rs:1718`) so `reconstruct_record` can read the type back. One derive,
both faces.

**The inventory is link-time static state — it does not need a world.** It is
drained *into* a `TypeEnv`, but the entries themselves are available before any
world exists.

The read path is not. `reconstruct_record` takes a `&TypeEnv`, so **every
decoder that must run before a world exists is hand-written**, and each one is a
second copy of a shape the derive already knows.

## Where it costs, measured

| site | hand-written decode |
|---|---|
| `types.rs:1748` — `register_runtime_error_variants` | **104 lines**, incl. hand-built `TypeExpr::Parametric` for `Option<String>` / `Vector<String>` |
| `src/process/boot/mod.rs` — `BootFrame::from_wire` | 3 tags; the child has no world, the frames are what BUILD it |
| any future pre-world decode | the same, again |

The substrate names the root itself at `types.rs:1748`:

> *"That derive is WRITE-ONLY (no `EdnSchema` submit) — so STRICT `edn_to_value`
> hit `UnknownTag` and the cause was string-wrapped. Here we **hand-register the
> DECODE schema** for each variant…"*

And the 24o seam already counted the full scope: **123 variant-tags across 10
error enums, none registered**, with the ruling that the work-unit is *~10 enums,
not 123 hand records* — if the derive does it.

## Why it hasn't been done

The 24o seam names the blocker: `#[derive(Edn)]` carries a **scalar-only
field-type wall** (its STOP-2). The error enums carry non-scalar fields
(`Option<Span>`, `Vector<String>`, nested causes), so they use the write-only
derive and pay for decode by hand.

Corroborating evidence that the mechanism works when the wall is not hit:
`types.rs:1075-1080` records that `:wat::core::Pos` **is** registered via the
`EdnSchema` drain, so an `Option<Pos>` field decodes. And arc 170's boot frames
derived and registered on the first attempt — because they are scalar-only.

## The two shapes, for whoever takes it

1. **Lift the derive's field-type wall** so the error family submits schemas like
   any other round-trippable type. Deletes the 104-line hand registry outright;
   this is 296.3's stated plan (*"register via the DERIVE, not hand-authored"*).
2. **A pre-world decoder driven by the inventory** — resolve a tag against
   `inventory::iter::<EdnSchema>()` and decode by `schema.fields` without a
   `TypeEnv`. Narrower, and it serves the boot case and any future one directly.

They are not exclusive: (1) removes the hand registry, (2) removes the need for a
world at decode time. A site like `boot` wants (2) whether or not (1) lands.

## What arc 170 did in the meantime

Nothing structural — deliberately. `src/process/boot` keeps a small hand reader
(three tags, structural tag comparison, unknown tag = a located refusal, never a
guess) plus a compile-time exhaustiveness guard
(`_every_boot_frame_variant_is_covered`) so a new variant breaks the **build**
rather than failing at runtime.

That is the highest rung reachable without opening this arc. Closing it properly
inside the execve stone would have been building 296.3 in the wrong folder, for
3 tags, while 123 wait next door.

## Status

**OPEN.** Filed against 296 because 296 owns *"Error → EDN, unified under ONE
trait: every diagnostic is structured EDN by construction."* A hand-written
decoder is that unification incomplete on the read side.
