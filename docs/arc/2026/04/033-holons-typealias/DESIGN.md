# Arc 033 — `:wat::holon::Holons` typealias

**Status:** opened 2026-04-23. Second arc cut under `/gaze`. Same
discipline arc 032 established: name what repeats, keep the name
Level-2-safe.

**Motivation.** `:Vec<wat::holon::HolonAST>` is Bundle's input
type and the ubiquitous "list of holons" shape. 12 occurrences
in wat-rs source / tests / wat / wat-tests; 35 in the trading
lab; likely more as wat crates proliferate. Verbose, and reads
as structure rather than concept.

`/gaze` applied. First candidate `:wat::holon::Facts` was
rejected: Level 1 lie — presumes truth. The type is content-
agnostic; predictions-not-yet-measured go in the same shape as
ground-truth measurements. Name must be epistemically neutral.

**`:wat::holon::Holons`** — plural of the element type. A list
of HolonAST instances. Structurally honest, epistemically
neutral, Level-1-safe across every content context. Time
measurements are holons. Predictions are holons. Learned
engrams are holons. Every call site fits.

---

## Semantics

```
typealias :wat::holon::Holons = :Vec<wat::holon::HolonAST>
```

Non-parametric. The list-of-HolonAST shape is Bundle's input;
Bundle is the one algebra primitive that takes a list. Every
`encode-*-facts` function in the current lab returns this shape
(they emit facts — time measurements, price readings, indicator
values — all ground-truth about the candle). Future prediction
code would also return this shape (claims about future direction,
learned patterns). The TYPE is content-agnostic; the alias IS
the type. No parametric
generalization needed.

---

## Why substrate

`:wat::holon::*` already hosts `HolonAST`, `CapacityExceeded`,
and (post-arc-032) `BundleResult`. `Holons` joins that family —
the "collection of facts about which Bundle can reason."

Lab-local alias would work for the lab alone but leave every
other wat consumer restating the long form. The substrate is
the honest location.

---

## Registration

`TypeEnv::register_builtin` accepts `TypeDef::Alias(AliasDef
{ name, type_params, expr })`. Ships next to arc 032's
`BundleResult` registration in `src/types.rs::register_builtin_types`:

```rust
env.register_builtin(TypeDef::Alias(AliasDef {
    name: ":wat::holon::Holons".into(),
    type_params: vec![],
    expr: TypeExpr::Parametric {
        head: "Vec".into(),
        args: vec![
            TypeExpr::Path(":wat::holon::HolonAST".into()),
        ],
    },
}));
```

Alias resolution via `expand_alias` — no new checker work.

---

## Sweep targets

wat-rs:
- `src/runtime.rs` — type annotations + doc comments
- `src/check.rs` — comments referencing the type
- `tests/wat_bundle_capacity.rs`, `tests/wat_run_sandboxed.rs`,
  `tests/wat_bundle_empty.rs`, other test files with Bundle-shaped
  inputs
- `wat/holon/*.wat` — stdlib algebra modules
- `wat-tests/holon/*.wat` — their tests

Lab (deferred to lab arc 004 — same session):
- `wat/encoding/rhythm.wat`, `wat-tests/encoding/*.wat`
- `wat/vocab/shared/time.wat`, `wat/vocab/exit/time.wat`
- `wat-tests/vocab/**/*.wat`

Docs:
- `docs/arc/2026/04/005-stdlib-naming-audit/INVENTORY.md` — new row
- `docs/README.md` — arc index row
- `holon-lab-trading/docs/proposals/.../FOUNDATION-CHANGELOG.md`
  — CHANGELOG row

---

## What does NOT change

- Bundle's actual signature. Bundle still takes a Vec<HolonAST>;
  callers can write either `:wat::holon::Holons` or
  `:Vec<wat::holon::HolonAST>` — alias resolution unifies both.
- Historical docs — INSCRIPTIONs from prior arcs keep their
  shipped-at-the-time language.
- BOOK prose.

---

## Liberally aliasing: the standing practice

Arcs 032 + 033 are the first two instances. The discipline
stands: when a concrete generic or tuple shape repeats 10+
times, it gets a name via `/gaze`. The name is Level-2-safe
(the first-read reader can guess the shape). The alias lives
next to the types it wraps — substrate alias at substrate,
lab-local alias in the lab.

Future arcs in this series: whatever else `/gaze` surfaces as
mumbly and frequent.
