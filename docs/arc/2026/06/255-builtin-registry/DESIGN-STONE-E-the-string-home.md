# DESIGN — STONE E: `:wat::core::string::*` → `:wat::string::*`

> The last link of `CHAIN-rendering-before-the-string-home.md`, and the thing arc 255's whole
> detour was for. A → D are on disk; this is E.

## The chain is clear

```
A  EdnRepresentable        the trait exists; Process<I,O> is bound by it
B  #wat-edn.* → #wat.*     ZERO tags remain (the one grep hit is prose in a println)
C  str goes TOTAL          25d9d0158
D  join widens to Seqable  78bed2e3f
E  the string home         ← this stone
```

Ruled twice already and never executed: `109/NOTE-stdlib-namespace-homing.md` names `wat.string/`
as the home; `278/SEAM.md:82` — *"`wat.string/join` ; string RELOCATES, Clojure-style"*. **And the
file already moved while the namespace did not** — `wat/string.wat` is top-level, not
`wat/core/string.wat`. That divergence is on disk right now.

## THE BUILDER'S RULING — the rete mirror MOVES

2026-08-24: *"i say we mirror — wat-rete's dsl is meant to be a restricted clone of wat's lang…
just with (purity, deterministic, totality) imposed… and it'll induce confusion with it being an
odd ball."*

```
:wat::core::string::*        →  :wat::string::*
:wat::rete::core::string::*  →  :wat::rete::string::*
```

The reason is what the mirror IS, and it belongs in the record: rete's DSL is **a restricted clone
of wat's language with three axes imposed**, not a separate vocabulary that happens to overlap. A
mirror tracks its subject exactly or it stops being a mirror — and after this stone there is no
`core::string` left for `rete::core::string` to mirror. Its name would point at a thing that does
not exist.

Builder, on the horizon this sits under: *"long term… we may not have a rete mirror… it's a proxy
to how to build a total wat. we're not there yet, but we can rename the string namespace now."*
**The mirror is scaffolding for a total wat, not a permanent fixture.** That is why it follows
rather than diverges: a scaffold that drifts from the building is worse than no scaffold.

## MEASURED TODAY, because the chain's numbers are ten days old

The chain says **1,617 sites / 22 verbs** as of 2026-08-14. On disk now:

| area | sites |
|---|---|
| `wat-scripts` | 865 |
| `wat` | 533 |
| `tests` | 222 |
| `src` | 151 |
| `wat-tests` | 107 |
| **TOTAL** | **1,878** |

**+261 sites in ten days**, most of it `wat-scripts` (today's wat-grep corpus is part of it). Plan
off this number, not the chain's.

⚠ **26 distinct grep hits is NOT 26 verbs.** Four are artifacts: `:wat::core::string::*` and
`:wat::core::string::` are prose, and `::=` / `::not=` **do not exist** — probed this session,
`(:wat::core::string::= …)` raises `UnknownFunction`. Those two are the verbs stone E's sibling
(`DESIGN-STONE-string-equality-closes-the-family.md`) INTRODUCES; they are not sites to rename.
The real count is 22, which is what the chain said — arrived at by a different route, which is the
only reason to trust it.

## ⛔ SEVEN RUST DOORS — this is the stone's real difficulty

A namespace rename reads like a text sweep. It is not: the name is spelled in seven independent
Rust readers, and a rename that misses one leaves a verb that type-checks and does not dispatch, or
dispatches and is not admitted by rete.

| door | hits | what it decides |
|---|---|---|
| `src/runtime.rs` | 44 | dispatch — the verb runs or does not |
| `src/string_ops.rs` | 32 | the implementations themselves |
| `src/check.rs` | 31 | the type registry — the verb exists or does not |
| `src/macros/eval.rs` | 18 | the macro-body allow-list |
| `src/rete/expr_ir.rs` | 10 | rete RHS lowering (`StrLen` &c.) |
| `src/rete/vocabulary.rs` | 8 | the `RETE_OPS` mirror rows |
| `src/rete/purity.rs` | 3 | axis classification |
| `wat/string.wat` | 19 | the wat-side defns |

★ **ONE WALL ALREADY GUARDS THE RETE HALF.** `src/rete/vocabulary.rs:1565` asserts every `RETE_OPS`
row is admitted by the `RETE_MODULES` set. A half-moved mirror **screams** rather than silently
dropping rows. The other six doors have no such wall — which is exactly where a rider will lose a
verb, and why the acceptance rows below are per-door.

## THE BOOTSTRAP — and it is the thing to get right

The `.wat` corpus migrates by **wat-fix codemod** (R21), prior art
`wat-scripts/fixes/rename-kernel-to-spawn.wat`, which re-parented a namespace exactly this way and
documents the discipline this stone reuses verbatim: **the prefix is the FULL name**, because the
parent segment is shared. Here that guard is load-bearing in a specific way —

`:wat::core::String` (capital S, the TYPE) and `:wat::core::string::` (lowercase, trailing `::`)
share the parent `:wat::core::`. The trailing colons are what make the rename unable to touch the
type. `wat.type/String` and `wat.string/join` never collide.

**The codemod is itself written in wat and calls string verbs**, so the ordering looks like a
STASH-DANCE. It is not, and the reason is a property of the tool measured this session:

> `rename-prefix-edits` rewrites **"for every keyword LEAF"** (`wat/fix.wat:716`). A verb CALL is a
> keyword node; the codemod's own arguments — the strings `":wat::core::string::"` and
> `":wat::string::"` — are STRING LITERALS. **The tool can migrate itself**: its calls move, its
> arguments do not.

So the sequence is ordinary, and it is the documented atomic-commit-across-coordinated-sweeps
pattern (recovery doc § "Atomic commit"), not a special dance:

1. Write the codemod using the OLD verb names — it must load against today's binary.
2. Run it over the whole corpus, **including itself**. The tree is now broken against the current
   binary: it calls names that do not exist yet. That is mid-sweep brokenness and it is fine.
3. Rename the seven Rust doors + `wat/string.wat`.
4. Rebuild. Floor. Commit **once**, when the tree is green.

**Mid-sweep brokenness is acceptable; on-disk-committed brokenness is not.**

⊘ **CORRECTED 2026-08-24, same session, before briefing.** This section first prescribed registering
BOTH prefixes in Rust as an alias window, then deleting the old one — three commits. That was
over-engineering: it doubles the seven-door edit to buy a property the codemod already has. The
alias only earns its cost if the tool cannot survive its own migration, and it can. Rejected in
favour of the above. Hand-editing stays rejected for the original reason (R21, and 1,878 sites).

## ACCEPTANCE

1. **Per-door, not aggregate.** For each of the seven Rust files: zero `:wat::core::string::`
   remain. A total that reads zero while one door still holds the old name is the failure this row
   exists to catch — count them separately or the count lies.
2. **`(:wat::deporder::verify-stdlib)` returns `[]`** — `wat/string.wat`'s position is unchanged by
   a rename, but the gate is the authority, not this sentence.
3. **The rete wall fires if the mirror is half-done** — deliberately break one row, watch
   `vocabulary.rs:1565` scream, restore. A wall nobody has seen fire is a claim.
4. **Idempotent** — re-running the codemod yields zero changes, exactly as
   `rename-kernel-to-spawn.wat` promises of itself.
5. **The TYPE is untouched** — `:wat::core::String` count is identical before and after. This is
   the negative control for the shared-parent hazard.
6. **The old name resolves to nothing** — probe `(:wat::core::string::length "x")` and get
   `UnknownFunction`. The negative control for "did the rename actually land, or is the old name
   still quietly working somewhere".
7. Floor green, accounted BY NAME; clippy 0.

## OUT OF SCOPE — affirmatively cut

- **Home #4** — the `core::string` carve into the intrinsic registry. It lands on the FINAL names,
  which is the whole reason it was moved to last in the chain. Its own stone, after this.
- **`=` / `not=` for String.** They do not exist yet; introducing them is
  `DESIGN-STONE-string-equality-closes-the-family.md`, not a rename.
- **`concat`.** `string::concat` is String→String and `Vector/concat` is Vector→Vector — genuine
  receiver dispatch, a separate question. `join` is not: it always returns a String, which is why it
  stays in `wat.string/`.
- **The dotted `wat.string/join` surface.** That is 251's clojure flip. This stone moves the
  namespace in the CURRENT `::` spelling; the flip re-spells everything later, once.
