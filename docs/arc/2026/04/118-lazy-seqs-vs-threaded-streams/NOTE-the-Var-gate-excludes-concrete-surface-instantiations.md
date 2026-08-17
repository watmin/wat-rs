# NOTE — 118.3-B's `Var`-gate excludes CONCRETE surface instantiations. Known, deliberate, recorded.

**Found 2026-08-17 by the orchestrator's independent verification, not by the rider.** The stone
shipped its stated goal; this is the edge its safety gate leaves open, written down so the next
person who hits the error knows it is a boundary and not a bug.

## What works and what does not

```wat
(:wat::core::defsurface :sq::Seqable<T> :nature :wat::core::Struct
  :features [(as-vec [self <- :sq::Seqable<T>] -> :wat::core::Vector<T>)])
(:wat::core::extend-type :wat::core::Vector :sq::Seqable<T> …)
```

| the parameter type | result |
|---|---|
| `[s <- :sq::Seqable<T>]` — **generic** | ✅ works. Instantiates to a fresh `?N`, the new branch fires, `T` binds |
| `[s <- :sq::Seqable<wat::core::i64>]` — **concrete** | ⛔ still fails |

```
:cs::count-concrete: parameter #1 expects :cs::Seqable<wat::core::i64>;
                     got :wat::core::Vector<wat::core::i64>
```

## Why the gate exists — it is buying row 4, and row 4 is worth buying

118.3-B's new branch is gated on the **expected** side still carrying an unbound `TypeExpr::Var`.
That gate is what keeps the branch away from the arm's three existing tenants — `Dialable`,
`TypedCapability`, `Handle` (arcs 267 / 170 C2 / 293.W.2f) — whose type args are **fully concrete**
at the call site (confirmed live at `tests/services/probe_arc170_c2_d_bodiless_edge_ok.wat:32`).

With the gate, the new branch is **provably unreachable** for them and the old exact-string arm
decides those calls byte-identically. **All ten of their floor tests pass, including every
swap-gate negative control** — which is the strongest available evidence that the tenants did not
move.

Remove the gate to admit concrete instantiations and every one of those calls starts flowing
through the new binding path. That is exactly the blast radius the brief fenced as STOP-2 territory.

**So this is a real trade, made deliberately, not an oversight.** The rider documented the gate as
*"deliberate and structural, not incidental"* and argued its unreachability. It did not report the
concrete-instantiation consequence; verification found that.

## What it costs, honestly

For **chain-D** — nothing. `join`, `map`, `filter` are generic; their parameter types instantiate to
fresh vars and take the working path.

For a **user** writing a monomorphic consumer — `(defn f [s <- :Seqable<i64>] …)` — it fails, with an
error that reads like the type system is broken rather than like a boundary. That is the honest cost,
and it is the reason this note exists rather than a silence.

## The disposition

**Not a defect to fix now.** Fixing it means either admitting concrete instantiations through the
new path (re-opening the tenant blast radius) or discriminating more finely than `is there a Var`.
Both are real work with a real risk surface, and neither is needed by anything currently asking.

**It becomes the next stone the moment a consumer needs a monomorphic surface parameter** — and at
that point the question to answer first is the one the rider already flagged as untested territory:
for a surface with **more than one type parameter**, is the positional-order assumption (surface's
declared param *i* ↔ actual's own arg *i*) correct? Nothing in the corpus exercises it today; every
container here and every existing tenant is arity-1 or matched-arity.

## Kin

- `docs/arc/2026/04/118-lazy-seqs-vs-threaded-streams/MEASURED-118.3-B-is-a-string-compare-not-a-mechanism.md`
  — the diagnosis the stone acted on.
- `[[feedback_a_guard_drawn_too_tight_makes_the_honest_path_noncompliant]]` — the near-miss shape.
  This gate is *not* that: it is drawn exactly tight enough to protect three live tenants, and the
  path it closes has no consumer. But it is the same family, so it gets written down rather than
  remembered.
