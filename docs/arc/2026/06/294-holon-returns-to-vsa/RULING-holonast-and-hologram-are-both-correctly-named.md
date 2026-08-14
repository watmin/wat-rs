# RULING — `HolonAST` and `Hologram` are BOTH correctly named. R1's keystone rename is VOID.

> **Builder's ruling, 2026-08-14.** Recorded by the apparatus as scribe — the ruling is HIS, on HIS
> own realization. R1 is NOT edited; this note stands beside it and supersedes its flaw #6.

## The ruling, verbatim

> *"`:wat::holon::Hologram` is currently correctly defined.... the hologram is the thing you hold and
> can get objects out who point to more objects.... **the hologram is made from holons**"*

> *"**HolonAST needs no change**..... it is an AST for building holons.... its far more restrictive
> than WatAST.... but it can hold everything a WatAST can hold....
> **HolonAST is an edn**.... whatever you can express with edn... you can build in holon-ast...."*

## What it voids

**294 R1 flaw #6 and the keystone are VOID.** R1 says:

> *"strip HolonAST's borrowed roles (code-AST → WatAST, wire → EDN) and **what remains is not a syntax
> tree at all** … It was a **Hologram wearing an AST's coat** … **`HolonAST` reduces to `Hologram`**."*

The stripping was right; the naming of what remained was one step too far. It **is** a syntax tree — an
AST *for building holons* rather than for code. And the destination name was never vacant.

## The layering, and every name in it is right

```
EDN  ≅  HolonAST        the AST. Restrictive vs WatAST; expresses everything EDN can.
          │ encode
          ▼
       a holon          a whole-that-is-also-a-part (Koestler); one point in hyperspace.
          │ stored in
          ▼
       Hologram         made OF holons — you hold it, get an object out, and it points to more.
```

**The incumbent's name is provable from its own definition** (`src/hologram.rs:63`):

```rust
pub struct Hologram {
    slots: Vec<HashMap<HolonAST, HolonAST>>,   // ← made OF holons, keyed BY holons
    capacity: usize,                            //   floor(sqrt(d)) — Kanerva cells
    …
}
```

`HashMap<HolonAST, HolonAST>` in coordinate slots. The 074 INSCRIPTION says the same in prose:
*"unbounded coordinate-cell store … **HolonAST-keyed coordinate stores**"*, ten store verbs
(`put`/`get`/`find`/`find-best`/`remove`/`len`/`capacity`/…).

And **"HolonAST is an EDN"** is R1's own *"edn goes in and vectors get built … holon can host all of
edn"* seen from the other end. 294's "EDN is canonical" therefore does **not** require HolonAST to go —
it requires HolonAST to be recognized as **the EDN-shaped input to `encode`**.

## R1's six flaws, re-sorted after the ruling

| flaw | status |
|---|---|
| #1 construction split-brain | ✅ landed — `aggregate-new` (`f301a6fc`) |
| #2 hologram-as-identity (data demoted to a cache) | ✅ landed — identity = EDN data (`ed7ecd50`) |
| #3 `#wat-edn.holon/*` tags — scar tissue | **OPEN — 44 sites** |
| #4 `HolonRepresentable` redundant with `EdnRepresentable` | **OPEN — 11 impls** |
| #5 HolonAST doing **code**-AST duty | **OPEN — a scoping cleanup, NOT a rename** (task #91) |
| #6 the rename / the strange loop closing in it | **VOID — the name was right** |

**"The remaining holon junk" is therefore three items, all in wat-rs, all on legal ground** — not 1263
sites across two repos. The cross-repo migration that looked unavoidable an hour ago does not exist.

**Flaw #5's sharpest tell, measured 2026-08-14** — files where HolonAST rivals or beats WatAST:

```
special_forms.rs   HolonAST=17   WatAST=2      ← special forms are CODE. The clearest code-duty site.
lower.rs           HolonAST=29   WatAST=34     ← lowering is CODE.
comms/mod.rs       HolonAST=81                 ← the wire = flaw #4's territory, not #5's
hologram.rs        HolonAST=32                 ← LEGITIMATE: the store is made of them
```

## How this was found, kept because the method is the lesson

Not by measurement. The apparatus had measured `Hologram`'s 116 refs an hour earlier and reported them
as *"the destination name"* — i.e. as progress toward the rename — when they were **the collision**. It
then built a merit argument that the incumbent was misnamed (*"a store is not a hologram"*), leaning on
arc 078's `HologramCache` as precedent. The builder cut it twice:

1. *"078 is very old and we changed away from that name in 109 or later"* — and the crate
   `crates/wat-holon-lru` is **GONE**; the cache is `:wat::cache::Lru` in `wat/cache.wat`. The
   precedent had been dead for months and was cited as live.
2. Then the semantic correction above, which inverted the merit argument entirely.

**The taste-first read beat the measurement-first read to the finding, again.** Kin: 294 R6
`DOLOR INDEX EST` — *"a cluster's name in any map is only as good as the assertion underneath it,
including, and especially, a name the apparatus wrote itself."*

## Debris found while grounding this — four dead pointers to the deleted crate

```
src/hologram.rs:3      "the bounded sibling `HologramCache` adapts on top"
src/hologram.rs:207    "Used by bounded variants (HologramCache) to drop entries"
src/runtime.rs:17887   "HologramCache calls this on LRU eviction"
wat/cache.wat:246      "Study oracle: crates/wat-holon-lru/wat/holon/lru/HologramCache.wat"
```

The last one instructs a reader to study a file in a crate that no longer exists. Small, unruled, and
exactly the graveyard-pointer class — tracked here, not silently swept.
