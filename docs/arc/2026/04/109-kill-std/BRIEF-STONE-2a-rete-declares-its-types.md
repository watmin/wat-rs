# BRIEF — ②a strike A: `wat/rete.wat` declares its parametric types

166 bare parametric annotations in one file — 69% of the corpus-wide population. Design:
`DESIGN-STONE-wall-2-the-unwritable-238.md`. Why this goes before any wall:
`DESIGN-STONE-the-four-walls.md` — Wall 2 is the ONE wall whose errors carry no remedy, so its sites
are fixed first, alone, where they cannot hide inside 4,485 auto-fixable ones.

**Your role: you write the text. The orchestrator builds, floors, and clippies.** No `cargo`, in any
form. `./target/release/wat --check <file>` IS available and IS current — the schemes landed at
`9c82f157`, so it reflects the checker you are writing against. Use it freely on scratch files.
Foreground everything; ending your turn ends you. Do not commit, push, stash, or revert.

## ★ The method: the type is already written, in the comment above the annotation

This is recovery of documented intent, not invention. `wat/rete.wat` documents its own shapes — several
in type syntax already:

```
:237   support: PersistentMap<derived-fact, Support> — the provenance index.   ← literally the type
:188   production-memory: node-id → PV<:wat::core::Record>                     ← literally the type
:180   alpha-memory:      node-id → {join-bindings → [Element …]}
:185   beta-memory:       node-id → {join-bindings → [Token …]}
:178   network:           id → Node (raw node records) — the compiled DAG, id-indexed.
:427   network: the id→Node PersistentMap built so far.
:27    bindings: {?var → value} — variable bindings accumulated left-to-right.
```

**Read the comment, write the type it already describes.** Where the file gives you `PersistentMap<K,V>`
in prose, that is your answer — rendered in the bracket form.

★ **The worked example, already settled** (builder-ruled 2026-08-20):

```wat
bindings <- (:wat::core::PersistentMap [:wat::core::String :wat::core::Value])
```

K is `String`: `rete.wat:146` already declares `result-var <- :wat::core::String` for the same `?var`
names, and every literal key in the corpus is a String (`"?count"`, `"?fact"`, `"?s"`…). V is `Value`
because **rete is TRANSPORT, not a consumer** — it carries bindings between an inserter and a reader
that each know the real type. Measured: equality, presence-match and get→assoc all work on an opaque
`Value`, and `assoc`-ing a concrete `i64` into a `Value` slot works ("UP is free").

## The roles — ~15 real ones behind 38 spellings

Abbreviations collapse: `bm`/`beta-mem`/`beta-memory`/`bmem` are one role; so are `amem`/`alpha-mem`/
`alpha-memory`, `net`/`network`, `pm`/`prod-mem`/`production-memory`.

```
network 24+2 · beta-mem 13+11+1+1 · alpha-mem 10+1+1 · facts 10 · bindings 7 · acc 4+2
prod-mem 3+1+1 · derived 2 · ext 2 · m 2 · support · tokens · rhs · query-memory · progs
params · nodes · nb · km · folds · els · elements · drivers · deps · conds · pv · a · acc-facts · acc-derived
```

Determine each ROLE once; apply mechanically to every site that carries it.

## ⛔ The contract decision

> **When the prose does not say, the site is a FINDING — not a guess.**

`:wat::core::Value` type-checks everywhere and would silence all 166. It is also the weakest type that
compiles, and writing it where a truer type exists cements the heresy in a form that now LOOKS
compliant. **`Value` is correct only where the role genuinely carries opaque payload between two
parties that each know the real type** — which is exactly `bindings`, and is NOT a licence for the rest.

Report every role you could not determine. Twelve named unknowns beat twelve lies that pass the wall.

## ⚠ The hazard: nested parametrics

Several comments describe NESTED shapes — `alpha-memory: node-id → {join-bindings → [Element …]}` is a
map of maps of vectors. The bracket form nests uniformly:

```wat
(:wat::core::PersistentMap [:wat::core::i64
   (:wat::core::PersistentMap [K (:wat::core::PersistentVector [:wat::rete::Element])])])
```

⚠ **If a nested level's own type is undetermined, the whole role is a FINDING** — do not fill the
inner level with `Value` to make the outer one writable. That is the contract decision, one level down.

## ⚠ Additive — nothing becomes illegal in this stone

A bare head is still legal; the walls come later. **The floor must stay green throughout.** Every
annotation you add is now genuinely ENFORCED — the Persistent family got its 13 schemes at `9c82f157`
— so a wrong type will go RED immediately. That is the point of doing this after the schemes rather
than before: the checker validates your work as you write it.

⚠ **`(:wat::core::PersistentMap)` bare-empty constructors are NOT this stone.** Value position, Wall
3's business, ~110 sites in this file. Leave them.

## Blast radius

`wat/rete.wat` only. No `src/`. No other `.wat`. No `tests/`.
⚠ This is a single file, so hand edits are correct — R21's codemod doctrine governs MULTI-file
structural rewrites, which this is not.

## STOP triggers — each rejects; none is a fallback

1. A role's type cannot be determined from its comment, its constructors and its consumers. STOP for
   that role, report it, continue with the others.
2. An annotation you add turns the file RED and the right type is not obvious. STOP; report both the
   type you wrote and the error.
3. The fix needs a type that does not exist (a supertype nobody declared). STOP — do not mint types.
4. You need to touch `src/` or another file. STOP.

## Acceptance criteria

- `grep -cE '(<-|->) :wat::core::(PersistentMap|PersistentVector|HashMap|Vector|HashSet)([^A-Za-z0-9_<]|$)' wat/rete.wat` → **0**, or every survivor named in your report as a STOP-1 finding.
- `./target/release/wat --check` on the corpus stays clean; the floor is the orchestrator's.
- Every role's type traceable to the comment, constructor or consumer that decided it — say which, per role.
- No `(:wat::core::PersistentMap)` bare-empty constructor touched.
