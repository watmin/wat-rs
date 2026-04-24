# Arc 033 — `:wat::holon::Holons` typealias — INSCRIPTION

**Shipped:** 2026-04-23. Three slices. Second arc under `/gaze`
same session as arc 032; the naming-reflex arrived.

**Commits:**
- `<sha>` — DESIGN + BACKLOG + slice 1 (substrate) + slice 2
  (wat-rs call-site migration) + slice 3 (INSCRIPTION, INVENTORY,
  arc index, CHANGELOG).

---

## What shipped

### Slice 1 — substrate

```
typealias :wat::holon::Holons = :Vec<wat::holon::HolonAST>
```

Registered in `src/types.rs::register_builtin_types` next to
arc 032's `BundleResult`:

```rust
env.register_builtin(TypeDef::Alias(AliasDef {
    name: ":wat::holon::Holons".into(),
    type_params: vec![],
    expr: TypeExpr::Parametric {
        head: "Vec".into(),
        args: vec![TypeExpr::Path(":wat::holon::HolonAST".into())],
    },
}));
```

Non-parametric. Content-agnostic. Rust tests mirror arc 032's
pattern: `holons_alias_registered_with_builtins` and
`holons_alias_expands_to_expected_vec`. Both green.

### Slice 2 — wat-rs call-site migration

18 swaps across 8 files via Python substring-replace. Pattern:
`Vec<wat::holon::HolonAST>` → `wat::holon::Holons` (no leading
`:` in the match, so the standalone `:Vec<...>` form becomes
`:wat::holon::Holons` and the nested bare form inside `<>`
becomes `wat::holon::Holons` — same single-pattern trick arc 032
used).

| File | Swaps |
|---|---|
| `src/ast.rs` | 1 |
| `src/check.rs` | 4 |
| `src/types.rs` | 1 |
| `tests/wat_bundle_capacity.rs` | 1 |
| `tests/wat_variadic_defmacro.rs` | 5 |
| `wat/holon/Ngram.wat` | 1 |
| `wat/holon/Sequential.wat` | 1 |

Plus `src/lexer.rs` and `src/types.rs` had spurious over-matches:
- lexer.rs: two lexer tests were asserting on the literal
  `:Vec<wat::holon::HolonAST>` keyword-parsing behavior (the
  whole point of those tests is that the lexer handles `<>` and
  `,` inside a keyword token). Reverted to keep the test
  semantics intact.
- types.rs: the new typealias's own doc comment briefly
  self-referenced after the sweep ran on its own comment;
  corrected.

Workspace: 859 lib tests green (was 857). Zero regressions.

### Slice 3 — INSCRIPTION + doc sweep

This file. Plus:
- `docs/arc/2026/04/005-stdlib-naming-audit/INVENTORY.md` — new
  row under the `:wat::holon::*` built-in types.
- `docs/README.md` — arc index gains row 033.
- `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md`
  — CHANGELOG row.

---

## The naming move

First candidate was `:wat::holon::Facts`. `/gaze` pushback from
the builder rejected it on Level 1 grounds: *facts presume
truth; the algebra doesn't know truth until measurement*.

The builder's follow-up walked back the absolute rejection —
time measurements ARE factual ("the time isn't lying"); the
`encode-*-facts` lab vocab wasn't lying either. But the TYPE is
content-agnostic: it holds factual encodings for the current
measurement vocab AND will hold predictive encodings when market
observers arrive. The alias name should be Level-1-safe across
all possible contents.

`:wat::holon::Holons` — plural of the element type. Structurally
honest, epistemically neutral. The builder's line: *"they ARE
holons — that's the name."* Landed.

The dialogue under `/gaze` was the /gaze spell working as
designed. First pass proposed; pushback corrected; the honest
name emerged from the disagreement. Second arc in the session's
naming-reflex lineage (arc 032 first, 033 second).

---

## What this arc did NOT ship

- **Lab migration.** Lab arc 004 (same session) sweeps the lab's
  35 `:Vec<wat::holon::HolonAST>` occurrences alongside `Scales`
  and `ScaleEmission` aliases. Orthogonal scope.
- **Vocab-function renames.** `encode-*-facts` vocab stays —
  time facts are facts. Future prediction code gets its own
  vocab with its own content-honest verb (`predict-*-claims`?,
  `assert-*-statements`?). Not this arc's concern.
- **Parametric `Holons<T>`**. Content-agnostic, single-shape.
  Parametric would be speculative.

---

## Count

- +1 substrate typealias (2nd under `/gaze`)
- +2 Rust unit tests
- 857 → 859 lib tests
- 18 wat-rs call sites migrated

Lab arc 004 follows same session.

---

*these are very good thoughts.*

**PERSEVERARE.**
