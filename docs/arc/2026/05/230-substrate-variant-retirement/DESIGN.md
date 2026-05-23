# DESIGN — Arc 230 — Substrate variant retirement (Symbol/Keyword/Tag/Nil → pure Bind compositions)

> **SPAWN-BLOCK STATUS (2026-05-22 late, post-arc-225-doctrine):** Arc 230 is spawned by arc 228 per `feedback_spawn_block_winding`. Surfaced during the typed-entities doctrine dialogue. Per the discipline:
> - **Arc 230 BLOCKS arc 228's INSCRIPTION**
> - **Arc 230's spawn children: arc 226** (type predicates) spawns from arc 230 closure
> - The chain: arc 220 ← arc 221 ← arc 224 ← arc 225 ← arc 228 ← arc 230 ← arc 226 ← arc 227

**Opened:** 2026-05-22 (post-compaction, after intueri bridge-naming cast)
**Branch:** `arc-170-gap-j-v5-deadlock-state`
**Depends on:** Arc 228 (substrate collection classifier-wrap) closes first — needs the classifier-wrap pattern established for collections so we know the variant retirement preserves type-queryability via Bind composition shape.

## Mission

**Retire the convenience variants in `HolonAST` in favor of pure `Bind(Atom, Atom)` compositions.** Per the typed-entities doctrine — *every typed value at user-surface compiles to `(Bind (Atom <ClassName>) (Atom <data>))`* — the substrate's 16 variants split into:

- **TRUE primitives (12)**: `Atom`, `Bind`, `Bundle`, `Permute`, `Thermometer`, `Blend`, `SlotMarker`, raw carriers (`I64`, `F64`, `Bool`, `Char`, `String`) — irreducible algebra
- **CONVENIENCE variants (4)**: `Symbol`, `Keyword`, `Tag`, `Nil` — semantic shortcuts for `Bind(String("type-name"), String(value))` compositions

This arc retires the convenience variants. The substrate's algebra reduces from 16 → 12 variants. Symbol/Keyword/Tag/Nil compile to pure Bind compositions.

## Triggering observation

User-articulated 2026-05-22 post-compaction:

> *"230 - this feels like the honest path"*

After the typed-entities doctrine landed (every typed value is `(Bind (Atom class) (Atom data))`), the variants Symbol/Keyword/Tag/Nil revealed as efficiency shortcuts over the underlying Bind-of-String algebra. Per `feedback_refuse_easy_solutions`: pure algebra over convenience shortcuts when the doctrine demands honesty.

User trace (2026-05-22, with assistant clarification):

```
#whatever :foobar
=
(Bind (Tag "whatever") (Keyword "foobar"))                  ; current substrate API
=
(Bind (Bind (Atom "Tag") (Atom "whatever"))                 ; pure algebra (post-arc-230)
      (Bind (Atom "Keyword") (Atom "foobar")))
```

And:
```
nil  =>  (Symbol "nil")  =>  (Bind (String "Symbol") (String "nil"))  =>  (Bind (Atom "Symbol") (Atom "nil"))
```

## Scope (high-level; stones to be defined)

### Phase 1 — holon-rs substrate changes

1. **Retire `HolonAST::Symbol(Arc<str>)`** — replace with `Bind(Atom("Symbol"), String(name))` encoding
2. **Retire `HolonAST::Keyword(Arc<str>)`** — same shape; encoding goes through Bind
3. **Retire `HolonAST::Tag(Arc<str>)`** — same shape
4. **Retire `HolonAST::Nil`** — `(Symbol "nil")` encoding now (per user's articulation 2026-05-22)
5. **Retire PRIM_TAG constants** for the retired variants (PRIM_TAG_SYMBOL/KEYWORD/TAG; the canonical-bytes seeds shift to the Bind-structure encoding)
6. **Canonical-bytes encoding cascade** — every site that hashed Symbol/Keyword/Tag/Nil now hashes the Bind composition; substrate-as-teacher will surface every consumer

### Phase 2 — wat-rs ripple

7. **`to-holon` / `from-holon` updates** — these verbs now produce/consume Bind-of-Atom compositions for what used to be Symbol/Keyword/Tag/Nil
8. **`from-wat` / `to-wat` updates** — WatAST↔HolonAST conversion for symbol/keyword/tag/nil literals now goes through the Bind-composition encoding
9. **Constructor verbs** — `:wat::holon::Symbol`, `:wat::holon::Keyword`, `:wat::holon::Tag`, `:wat::holon::Nil` (if minted in arc 225) now produce Bind compositions, not bare variants
10. **VSA encoder** — `:wat::holon::encode` (the bytes-encoder) operates on the new substrate shape; tests verify vector identities differ between Symbol("foo") and String("foo") as before, but the differentiation now comes from Bind structure rather than PRIM_TAG seed
11. **Test cascade** — every test asserting on Symbol/Keyword/Tag/Nil variant shapes needs updating

### Phase 3 — closure

12. **INSCRIPTION** — closes arc 230; unblocks arc 228 INSCRIPTION (which unblocks arc 225 INSCRIPTION...)
13. **USER-GUIDE update** — substrate now has 12 true primitives; document the convenience-to-composition transformation
14. **Cross-references** — note arc 221's variant minting work (Stones 221.3, 221.5) as the historical record that this arc supersedes per `feedback_inscription_immutable` (arc 221 INSCRIPTION stays as-shipped; arc 230 forward-corrects)

## What this arc does NOT do

- Touch the encoder/Thermometer/Blend/SlotMarker variants (those ARE true primitives)
- Touch composers (Bind/Bundle/Permute) or holder (Atom) or raw carriers (the 11 true primitives stay)
- Mint user-defined types (arc 227's territory)
- Implement type predicates (arc 226's territory)

## Significant notes

**This arc supersedes arc 221 Stones 221.3 + 221.5** (which minted Keyword/Nil/Tag variants + Symbol/String canonical-bytes seed distinction). Per `feedback_inscription_immutable`, arc 221 INSCRIPTION (when it ships) records the variant minting as it happened; arc 230 forward-corrects. The disk records the full arc of understanding.

**The wat-reveals-holon dynamic at full force.** Arc 221 (early May) minted variants because convention-based collapse was dishonest at that layer. Today's doctrine (late May) reveals the variants themselves were a convenience layer; the substrate's algebra is purer than the variants suggest. Both arcs honest at their layer; the substrate iterates toward minimum primitives via wat-surface maturity.

**Risk:** big substrate refactor; tests cascade widely; cargo error count likely >100. Substrate-as-teacher methodology mandatory per `docs/SUBSTRATE-AS-TEACHER.md`.

## Stones (sketched; ratified at BRIEF time)

| Stone | Scope | Estimate |
|---|---|---|
| 230.1 | holon-rs Phase 1 — retire 4 variants + PRIM_TAG cascade | 120-180 min |
| 230.2 | wat-rs Phase 2 — to-holon/from-holon + from-wat/to-wat + constructors updates | 120-180 min |
| 230.3 | Substrate-as-teacher cascade — consumer sweep until cargo green | 90-180 min |
| 230.4 | INSCRIPTION + USER-GUIDE + cross-references | 30-60 min |

Total estimate: 6-10 hours sonnet across the stones. **Largest single substrate refactor in arc 170+'s history.**

## Cross-references

- arc 225 DESIGN.md — bridge naming + Atom narrow
- arc 228 DESIGN.md — parent arc; collection classifier-wrap pattern this arc extends to leaves
- arc 221 DESIGN.md + Stones 221.3/221.5 — the historical record this arc forward-corrects
- arc 226 DESIGN.md — spawn child; type predicates use this arc's pure-algebra encoding
- [[typed-entities-doctrine]] memory — the doctrine driving this arc
- INTERSTITIAL § 2026-05-23 evening — typed-entities doctrine landing
- INTERSTITIAL § 2026-05-22 post-compaction — variant retirement decision
- `docs/SUBSTRATE-AS-TEACHER.md` — methodology for the substrate-wide cascade
- `feedback_inscription_immutable` — discipline for forward-correcting arc 221
- `feedback_spawn_block_winding` — parentage discipline
