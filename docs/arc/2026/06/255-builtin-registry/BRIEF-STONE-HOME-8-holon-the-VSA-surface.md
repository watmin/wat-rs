# STONE HOME-8 — holon, the VSA surface, gets a home (and `runtime.rs` gets its first real seam)

DRAWN 2026-08-27 against `732efa3b5`. **BLOCKED on Stone G** — strike G first; this home is the
largest producer surface in the tree and carving it through today's registry silently downgrades
every constructor it contains.

**PRIOR ART:** `git log -1 315bbf546` (Stone F, the three-phase shape) and
`BRIEF-STONE-E-iv-keyword-gets-its-home.md` (the closest carve). Directory-home precedent:
`src/intrinsic/io/` (4 files, 29 verbs) and `src/intrinsic/kernel/` (10 files, 49 verbs).

## The move — the biggest home yet, by 2.3×

```
:wat::holon::*   ->   src/holon/  +  src/intrinsic/holon/       95 verbs
```

For scale, the largest home ever carved is `time.rs` at 41 verbs. `src/intrinsic/` has **no holon at
all** — the VSA surface this project exists for is the one namespace with zero registry presence.

```
<bare ops>       56    Atom · Bind · Bundle · Blend · Permute · Map · List
OnlineSubspace   10    new · update · project · residual · eigenvalues · threshold · dim · k · n
Reckoner          8
Hologram          7    make · put · get · find · remove · len · capacity
EngramLibrary     6    new · add · match-vec · contains · names · len
Engram            4    n · name · residual · eigenvalue-signature
Bundle / Bind     4
```

## ⛔ THE ONE CONTRACT DECISION — TWO layers, and the code already drew the line

Measured in `runtime.rs`, 51 holon-named fns:

```
26 fns, 1208 lines   take env / sym  ->  src/intrinsic/holon/   the registered, queryable interface
25 fns, 1169 lines   take neither    ->  src/holon/             the VSA algebra
   +535 lines        src/hologram.rs (420) + src/sigma.rs (115) already outside — ABSORB them
```

**This is not a new doctrine.** 15 of 17 existing homes keep shim and algorithm together because
their algorithm is one stdlib call — measured at **~11 body lines per verb**. `string` and `time`,
the two with real implementation, ALREADY split it out (`src/string/` 184, `src/time.rs` 46). They
were right, not anomalous. Holon is the first home whose implementation is worth naming: 1,169 lines
of algebra that touch no `env`/`sym`.

Holon's binding half is **12 body lines per verb** — squarely in the existing pack. The 26 are
ordinary shims that merely happen to live in `runtime.rs`.

★ **`env`/`sym` in a signature is the seam `runtime.rs` has been missing** — a line the compiler
already enforces on all 941 of its fns. Every future home with real implementation gets these two
layers. Say so in the module docs so the next carve inherits it.

⚠ `src/hologram.rs` has only **3 importers** — relocating it is cheap. Verify before moving.

## Shape

```
src/holon/                the VSA algebra (~1704 lines)
    mod.rs                absorbs src/hologram.rs + src/sigma.rs
    <by cluster>          the 25 pure fns lifted out of runtime.rs
src/intrinsic/holon/      the interface (~1208 lines)
    mod.rs                module + shared arg helpers
    atom.rs               the 56 bare ops
    hologram.rs           Hologram/*        -> calls src/holon/
    engram.rs             Engram/* + EngramLibrary/*
    subspace.rs           OnlineSubspace/*
    reckoner.rs           Reckoner/*
```

## ⛔ PHASE ORDER — unchanged, and non-negotiable

```
PHASE 1   register.  BOTH SPELLINGS LIVE.  Nothing moves.
PHASE 2   corpus moves by wat-fix codemod.  Both still work.
PHASE 3   retire.  Delete the runtime.rs arms.
```

Check `wat/*.wat` for holon verbs inside `defmacro` bodies before Phase 3 — E-ii and Stone F both
tripped `is_pure_total`'s allow-list (`src/macros/eval.rs`), a SEVENTH surface the briefs keep
forgetting. **Measure it; do not assume.**

## Rooms

```
src/runtime.rs:5241–29927        the 95 arms, scattered across 24,686 lines
src/hologram.rs (420)            absorb          src/sigma.rs (115)  absorb
src/intrinsic/io/, kernel/       directory-home precedent
src/intrinsic/string.rs          the call-out-to-impl precedent (8 call-outs to src/string/)
src/macros/eval.rs               is_pure_total — the surface that bites at Phase 3
src/rete/vocabulary.rs           the rete naming invariant, IF any holon verb has a rete row
src/remedy/retirement.rs         the RetirementEntry rows Phase 3 owes
```

## STOP triggers — each REJECTS

1. **STOP-1 — Stone G has not landed.** This home routes producers; without G they downgrade silently.
2. **STOP-2 — you would move a pure-algebra fn into `src/intrinsic/holon/`, or a binding fn into
   `src/holon/`.** The `env`/`sym` test decides, mechanically. If one is ambiguous, report it.
3. **STOP-3 — you would move the bare TYPE `:wat::holon::Hologram`** (or any bare type). Only
   `/`-verbs and `::`-verbs move.
4. **STOP-4 — `is_pure_total` needs an entry you did not measure.**
5. **STOP-5 — a room's line does not hold.** Written against `732efa3b5`.

## Acceptance

```bash
# 1. all 95 verbs RUN under the home spelling — a scratch-pad probe asserting a result for each.
# 2. the old spelling is a CHECK error naming its replacement (RetirementEntry rows).
# 3. ★ provenance SURVIVES — a holon constructor still stamps RuntimeBuilt after the carve.
#    This is why Stone G comes first; prove it, do not assume it.
# 4. runtime.rs SHRANK — report the before/after line count and the delta.
# 5. BOTH renderings, ALL extensions, after: classify every survivor.
# 6. cargo build --release --all-targets
```

## Report back with

Each row's output. The runtime.rs line delta. Every fn you classified binding-vs-algebra and any that
were ambiguous. What `is_pure_total` needed. Whether the rete invariant fired. Anything this brief got
wrong; what you did NOT do, and why.
