# BRIEF — STONE Q: one authority that can answer for every type

> Read `DESIGN-STONE-Q-one-authority-for-types.md` first. **No probe is committed and that is
> deliberate** — see below; the first act is a measurement whose result decides the shape.

## ⛔ WHY THERE IS NO PROBE YET

FM 2-bis says the orchestrator writes the disconfirming probe before the brief. Here the probe
cannot be written honestly yet: **its assertion depends on the answer to step 1.** If primitives
must be reachable through a different mechanism than the TypeEnv, the verb's signature is different,
and a probe written now would pin a shape nobody has measured.

★ So step 1 IS the probe, and it is a measurement, not a guess. **Write it, commit it, and report
the shape BEFORE building anything.** If step 1 says the mechanisms cannot be unified behind one
askable predicate, STOP and say so — that is a finding, and the stone is redrawn.

## THE WORK

**1. MEASURE THE MECHANISMS.** For every type keyword in the census (DESIGN), determine what backs
it: `TypeEnv` entry · `TypeExpr` primitive · builtin table · something else. **Commit the table.**
Resolve the five the DESIGN's instrument could not see — `i64 · f64 · bool · String · nil` —
by reading the source, not by asking `type-of` (Doctrine 1 refuses them as values).

**2. DECIDE THE SHAPE ON THAT TABLE, with the four questions, and say the verdict:**

```
(A) one query consulting several mechanisms       likely; nothing moves
(B) register the absent ones into TypeEnv         only for those that ARE aggregates/aliases
(C) primitives migrate into TypeEnv               ⛔ the trap — a shape invented for a query
```

**3. BUILD IT, ASKABLE FROM WAT.** 255 R9: *an authority that cannot be queried gets counted, and a
count is a frozen moment.* If `type-of` cannot answer for primitives, either it learns to or a
sibling verb does — but a wat program must be able to ask.

⚠ Doctrine 1 (arc 242) stands: a type keyword is not a value. The verb takes a type keyword in TYPE
position or as a quoted name — design around that constraint, do not delete it.

**4. THE PROBE, WRITTEN ONCE THE SHAPE IS KNOWN.** Rows: a primitive answers true · a builtin
answers true · a user type answers true · **a nonexistent name answers FALSE** — that last is the
row the whole stone exists for.

## READ IN ORDER

```
src/types.rs:931   register_builtin_types — what it inserts (Bytes, EvalError, Struct, nil,
                   :wat::eval::StepResult, three :wat::holon::*). NOT i64/f64/bool/String.
src/reflect/verbs.rs:1498   type-of's lookup: sym.types().and_then(|t| t.get(&type_kw)) — it queries
                   the TypeEnv and nothing else. This is why 14 names come back "unknown type".
src/types.rs:218-246   Nature, rank(), root_keyword() — the nature roots ARE registered; they are
                   the model for what a registered builtin looks like.
docs/arc/2026/06/255-builtin-registry/NOTE-the-registry-is-not-yet-the-largest-membership-set.md
                   ★ THE EPITAPH. 255 measured this exact disease in the VERB position: 121 names,
                   68 with a checker scheme but no registry row, 53 known by neither. Read what it
                   concluded about ORDER before proposing one here.
```

## STOP TRIGGERS

- **STOP-1 — a shape is built before step 1's table is committed.** The table decides the shape.
- **STOP-2 — primitives are migrated into the TypeEnv to make one lookup work.** They are
  `TypeExpr` primitives guarded by Doctrine 1; making them aggregates invents a shape for a query.
- **STOP-3 — the authority is Rust-only.** `QVOD NON ROGATVR, NVMERATVR`. It must be askable from wat.
- **STOP-4 — Doctrine 1 is weakened to let a type keyword be a value.** Design around it.
- **STOP-5 — the annotation wall (P-1) is built in this stone.** Q builds the authority; P-1 uses
  it, and P-1's blast radius is corpus-wide. Separate stones, separate floors.
