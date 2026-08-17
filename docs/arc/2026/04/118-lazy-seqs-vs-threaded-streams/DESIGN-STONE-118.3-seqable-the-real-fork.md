# DESIGN STONE — 118.3 · `Seqable`: the real fork, drawn on measured ground

**Builder, 2026-08-17: *"we're building it."*** Drawn the same day the two-month blocker was found
stale, and re-drawn twice within the hour as probes killed my first two framings. Everything below
is measured; where it is not, it says so.

## The ground, measured today

| claim | probe | result |
|---|---|---|
| surfaces admit builtins | `probe-seqable-is-spellable-today.wat` | ✅ **RUNS**, prints `"3,4"` — `Vector` + `PersistentVector` both satisfy one surface, dispatch routes by runtime type |
| parametric surface **declares** | `probe-seqable-parametric-all-four.wat` | ✅ `--check` 0 — `Seqable<T>`, four `extend-type`s, generic `defn` |
| parametric surface **is callable** | the same + call sites | ⛔ **RED ×4** |

```
:sq::count-of: parameter #1 expects :sq::Seqable<?454>;
               got :wat::core::Vector<wat::core::i64>
```

`?454` is a fresh unification variable. **The surface's type parameter never binds.**

★ **This is the blocker, and it is none of the three on record** (`infer.rs:638`). Those three name
surfaces, `:nature`, and unions — all refuted. The real one is **parametric satisfaction**, and it
was never written down anywhere.

## ⊘ OPTION A IS DEAD — recorded because I nearly recommended it

*"Use a NON-parametric `Seqable` — it's proven to run, and chain-D says `T` is unconstrained anyway."*

**Non-viable.** The working probe returns `:wat::core::Vector<wat::core::i64>` — a **hardcoded**
element type. A non-parametric surface can only ever serve one fixed element type, and there is no
erased fallback: `types.rs:78` — *"`:Any` is banned — the type universe is closed per 058-030's
rejection of the escape hatch. `parse_type_expr` refuses it at the parse layer."*

The "T is unconstrained" quote is about `join` not needing a **bound** on `T`. It still needs `T` to
*exist*. I conflated the two.

## The four questions — flat, on the three live options

### B — fix parametric satisfaction in the checker

Make `Builtin<Concrete>` unify against `Surface<T>`, binding `T`.

- **Obvious? YES** — `Seqable<T>` is what every doc, the chain, and the builder have been asking for
  for two months; it is the thing the error message is already trying to do.
- **Simple? YES** — one concept: satisfaction checking learns to bind the surface's type params.
  Not a new mechanism; an existing one that stops at a wall.
- **Honest? YES** — it makes `Seqable` a **real, user-spellable type**. Every other option leaves
  the concept checker-internal while the docs keep calling it a type.
- **Good UX? YES** — a user can write their own `Seqable` and extend their own containers onto it.
  Nothing else here gives that.

**ALL FOUR — and it is the only option that does.** ⚠ **Size unknown.** I have not read the
unifier's satisfaction path. That is the first thing the stone must scope, and it is a genuine
type-system change, not a widening.

### C — widen the intrinsics to the four-head set (no user-nameable type)

Give `join` an `infer_string_join` reusing `extract_lazyable_elem`, exactly as `map`/`take`/`drop`
already do, and as `interpolate` proves the custom-inference-hook pattern (`check.rs:3124`).

- **Obvious? YES** — `(join "," some-list)` works; identical in shape to `map`.
- **Simple? YES** — one function, an existing helper, a proven pattern.
- **Honest? NO.** ⛔ **It ships the behaviour while leaving `infer.rs:638`'s "the type wat cannot
  currently spell" TRUE — and adds a fifth hand-rolled consumer of the four-head set.** It is the
  *workaround*, and building the workaround again is precisely how we got seven `-stream` twins and
  a two-month stall. Honest fails on the arc's own history.
- Good UX — not reached.

### D — do nothing

- **Honest? NO** — chain-D stays shipped-narrower-than-specified with the record saying "closed".

## ★ THE RULING I RECOMMEND — B, and C only as an explicitly-labelled bridge if B is large

**B is the stone.** It is the only option that passes all four, and it is the only one that makes
`Seqable` exist rather than simulating it a fifth time.

**But B's size is unmeasured**, and I will not brief an unmeasured type-system change. So the stone
opens with a **scoping probe, not a rider**:

> Read the satisfaction path in the checker. Answer one question: *when a concrete
> `Builtin<Concrete>` is checked against a `Surface<T>` parameter, where does `T` fail to bind, and
> is that site a missing case or a missing mechanism?*

A missing **case** → B is small, brief it immediately.
A missing **mechanism** → B is a real arc, and **then** C becomes defensible as a named bridge with
its dishonesty stated on the record — never as a silent substitute.

## Out of scope — affirmative cuts

- **The seven `-stream` twins.** `dedupe` · `distinct` · `interpose` · `keep` · `keep-indexed` ·
  `map-indexed` · `reduce` (`wat/seq.wat`). Collapsing them is B's *payoff*, a separate stone, and
  it must not be bundled into the mechanism that enables it.
- **`into`'s missing `(Vector<T>, List)` clause.** Found by the probe; a sibling of task #45's
  already-shipped `(PersistentVector, Vector)`. Small, real, independent.
- **Per-element dispatch COST.** `join`/`map`/`filter` walk every element; a surface dispatch per
  element is unmeasured and could reverse the whole design. **It must be measured inside B's scoping
  probe, before any migration.**
- **Whether a Rust intrinsic's `TypeScheme` can name a wat-defined surface.** Measured: **zero**
  existing `check.rs` schemes name any wat-defined type (`Dialable`, `TypedCapability`, `Cache` — all
  0), so there is no precedent. `join` must stay an intrinsic (the bootstrap cycle). Unprobed, and it
  decides whether `join` can take the surface at all or the first consumer must be wat-defined.
- **Correcting `infer.rs:638` in place.** Its three blockers are refuted and it is the most expensive
  stale sentence found this session. A `src/` edit needing a floor run — ship it with B.

## What this stone does NOT get to claim

It does not claim `Seqable` is nearly free. Three framings died today — "the blockers are real",
then "the design type-checks", then "use the non-parametric form" — each killed by a probe that took
minutes. **The honest position is that the blockers on record were wrong, one new blocker is now
precisely located, and its size is unknown.** That is progress, and it is not a green light.
