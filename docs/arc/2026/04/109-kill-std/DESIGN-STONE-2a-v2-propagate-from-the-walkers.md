# DESIGN — ②a v2: propagate the memory types OUTWARD FROM THE WALKERS

Supersedes `DESIGN-STONE-wall-2-the-unwritable-238.md` for the rete population. That stone was drawn
against a `wat/rete.wat` that no longer exists (merge `fe7700f0e` split it into `wat/rete/**`), and its
method — *"read the doc comment"* — was shown wrong twice. See
`SCORE-STONE-2a-rete-declares-its-types.md` for the failure it is drawn from.

## What the merge changed, and it changes everything

`723936550` split `walk-sorted-ids` into four monomorphic walkers, **each carrying the concrete `acc`
type this arc could not previously write**:

```wat
walk-alpha-ids   acc <- :wat::core::PersistentMap<wat::core::i64,wat::core::PersistentVector<wat::rete::Element>>
walk-beta-ids    acc <- …PersistentVector<wat::rete::Token>>
walk-filter-ids  acc <- …PersistentVector<wat::rete::Token>>
walk-prod-ids    acc <- …PersistentVector<wat::core::Record>>
```

`wat/rete/oracle/fire.wat:18,36,54,77`. Floor green at **4855/4855**.

**The coupling that killed ②a v1 is gone**, and something better replaced it: four load-bearing
declarations, written by the engine's own author, that the compiler already accepts.

## ★★ THE METHOD — propagation, not prose

v1 said *"read the doc comment above the annotation."* That method failed twice: the
`alpha-memory`/`beta-memory` comments describe a nested `{join-bindings → …}` shape that exists
nowhere (`join-bindings` appeared exactly twice in the old file, both in those comments), and the
`bindings` V it produced was wrong.

**v2 does not read prose at all.** It propagates from a typed caller to its bare callee:

```
walk-alpha-ids   acc <- PersistentMap<i64, PV<Element>>     ← TYPED (ring 0, in the merge)
   └─ calls  activate-alpha  [alpha-mem <- PersistentMap]   ← BARE (ring 1)
        └─ calls  activate-fact  [alpha-mem <- PersistentMap] ← BARE (ring 2)
```

Ring 1's `alpha-mem` **is** ring 0's `acc`; it can be nothing else. Ring 2's is ring 1's. Each
annotation is derived from a declaration that already exists and already type-checks — and the
compiler proves each step, because a wrong propagation is a `TypeMismatch` at the call site.

★ **This is why it will work where v1 failed.** v1 wrote types from an authority (prose) that could be
stale and that the compiler could not check. v2 writes them from an authority (a caller's declared
type) that the compiler checks on every build.

## Scope — ring by ring, and STOP at the first ring that does not resolve

| ring | members | source of truth |
|---|---|---|
| **0** | the four walkers | ✅ already typed — the merge |
| **1** | `activate-alpha` · `root-join-pass` · `accumulate-pass` · `filter-pass` · `hash-join-pass` · `production-pass` | ring 0's `acc` at each call site |
| **2** | `activate-fact` · `append-token` · and whatever ring 1 hands a memory to | ring 1 |
| **3+** | outward until no bare memory parameter remains | ring 2 |

Do rings **in order**, and floor between them. A ring that goes red has propagated wrongly and its
predecessor is the evidence.

## The population, re-measured after the merge

```
244 bare annotations corpus-wide  (was 242 pre-merge — the split moved them, barely changed the count)

wat/rete/oracle/pass.wat        90   ← rings 1-2 live mostly here
wat/rete/oracle/fire.wat        28
wat/rete.wat                    16
wat/rete/oracle/accum-pass.wat  16
wat/rete/oracle/explain.wat      9
wat/rete/compile.wat             7
wat/rete/syntax.wat              2
tests/rete/**                  ~50   ← strike B, after the wat/ side settles
wat/query.wat · a types test     2   ← strike C, independent
```

`pass.wat`'s roles: `network` 15 · `beta-mem` 10 · `alpha-mem` 9 · `bm` 8 · `facts` 6 · `bindings` 4 ·
`prod-mem` 2 · `ext` 2 · and singletons.

## ⛔ Held OUT of this stone, both already paid for

**1. `bindings` — still unresolved, still not `Value`.** rete compares binding values with `<`/`>`,
uses them as `PersistentMap/get` keys, and `conj`s them into vectors; an opaque `Value` refuses all
three (arc 278 R7, `src/types.rs:1160`). ⚠ The merge introduced a `$native`/`$oracle` dual-impl —
**whether the consuming sites moved is UNMEASURED.** Leave every `bindings`, `ext`, `m`, `params`,
`km`, `nb` annotation bare. Roughly 10 of the 244.

**2. `bm` is NOT always `beta-mem`.** In the old file two `bm` sites were local *bindings*
accumulators despite 25 siblings being beta memory. A rider caught it by reading bodies rather than
the name census. `pass.wat` now has 8 `bm` sites — **read each body; the name is not evidence.**

## The contract decision

> **Every annotation is derived from a CALL SITE, and the derivation is named in the report.**

Not from a comment, not from a name, not from a sibling that looks similar. If a bare parameter never
receives an already-typed argument, it is not in this stone's scope — it is a finding, and it waits
for a ring that reaches it.

## The four questions

- **Obvious?** YES — each annotation's justification is a call site the compiler already checks.
- **Simple?** YES, and it is ordered: rings, floored between.
- **Honest?** YES, and *only* because propagation is compiler-verifiable where prose was not. v1's
  method could not fail loudly; this one cannot fail quietly.
- **Good UX?** YES — the engine's hottest structures stop being untyped in the reader's face.
