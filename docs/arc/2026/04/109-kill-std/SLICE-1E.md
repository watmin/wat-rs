# Arc 109 Slice 1e — FQDN four-of-five parametric type heads

**Compaction-amnesia anchor.** Read this first if you're picking up
slice 1e mid-flight.

## What this slice does

Promote four substrate-named parametric type heads under
`:wat::core::*`. The bare-source spellings retire in user code;
the FQDN form is canonical.

| Today | After arc 109 slice 1e |
|---|---|
| `:Option<T>` | `:wat::core::Option<T>` |
| `:Result<T,E>` | `:wat::core::Result<T,E>` |
| `:HashMap<K,V>` | `:wat::core::HashMap<K,V>` |
| `:HashSet<T>` | `:wat::core::HashSet<T>` |

**Vec<T> is NOT in this slice.** Vec ships in slice 1f because
the rename to `Vector` couples with § D's verb companion (`vec` →
`Vector` constructor). Keeping the rename out of slice 1e means
this slice is pure-FQDN-move, no name changes — the simpler
mechanism.

**§ C variant constructors NOT in this slice.** `Some`/`:None`/
`Ok`/`Err` → FQDN ride in slice 1g (or whenever queued). Slice
1e flips type names; constructors stay in their current form
(both `Some`/`:None` and `Ok`/`Err` continue to type-check
against the new typealiases via the same alias resolution).

**§ D' (Option/Result method forms — `try`/`expect`) NOT in this
slice either.** Per INVENTORY they couple with § B at the call
site. Future slice once the type names are FQDN'd.

## The protocol

Pattern 3 from `docs/SUBSTRATE-AS-TEACHER.md` — same as slices
1c and 1d. Dedicated `CheckError` variant + walker arm;
substrate's diagnostic stream IS the migration brief; sonnet
sweeps from it.

**Generalization claim**: slice 1c proved the walker template
against `Path` nodes; slice 1d proved it against `Tuple` nodes;
slice 1e proves it against `Parametric.head`. Three TypeExpr
shapes covered means the mechanism is durable for any future
TypeExpr-shape retirement.

## What to ship

### Substrate (Rust)

1. **Mint four typealiases** in `src/types.rs::register_builtin_types`:

   ```rust
   env.register_builtin(TypeDef::Alias(AliasDef {
       name: ":wat::core::Option".into(),
       type_params: vec!["T".into()],
       expr: TypeExpr::Parametric {
           head: "Option".into(),
           args: vec![TypeExpr::Path(":T".into())],
       },
   }));
   // ... and similarly for Result, HashMap, HashSet
   ```

   Alias resolution unifies FQDN with bare at the type level.
   Both forms type-check during the deprecation window.

2. **Add `CheckError::BareLegacyContainerHead`** variant in
   `src/check.rs`:

   ```rust
   BareLegacyContainerHead {
       head: String,    // "Option"
       fqdn: String,    // "wat::core::Option"
       span: Span,
   }
   ```

   Display IS the migration brief. `diagnostic()` arm produces
   structured `BareLegacyContainerHead` records.

3. **Extend `walk_type_for_bare` Parametric arm.** Currently
   recurses into args. Add a guard at the head: if `head` matches
   one of the four bare names AND not the FQDN form, emit
   `BareLegacyContainerHead`. Recurse into args regardless.

   ```rust
   const BARE_CONTAINER_HEADS: &[(&str, &str)] = &[
       ("Option",  "wat::core::Option"),
       ("Result",  "wat::core::Result"),
       ("HashMap", "wat::core::HashMap"),
       ("HashSet", "wat::core::HashSet"),
   ];
   ```

4. **Canonicalize FQDN→bare in `parse_type_inner`** when
   `canonicalize=true`. Today the substrate's internal form for
   Parametric heads is bare ("Option", "Result", etc.) — every
   special-case dispatch reads against those names. The
   canonicalization rewrites `wat::core::Option` → `Option` etc.
   at the parametric-head position when canonicalize=true. The
   audit walker (`canonicalize=false`) preserves source spelling
   so the diagnostic still distinguishes bare from FQDN.

   This is the same audit/canonical split slice 1c established
   for Path nodes and slice 1d extended for Tuple-from-typealias.

### Verification

Probe coverage:
- `(name :Option<i64>)` → fires (outer bare Option, plus inner
  bare i64 from slice 1c — composite case)
- `(name :wat::core::Option<wat::core::i64>)` → silent (FQDN throughout)
- `(name :Result<i64,String>)` → fires (bare Result + bare
  i64/String from 1c)
- `:HashMap<String,i64>` → fires (bare HashMap + bare String/i64)
- `:HashSet<MyType>` → fires (bare HashSet, MyType is user
  struct so no inner flag)
- `:my::pkg::Result<T,E>` → silent (user type happens to be
  named Result; head is "my::pkg::Result", doesn't match bare)
- Nested: `:Option<Result<i64,String>>` → fires (bare Option,
  bare Result, bare i64, bare String — four flags one site)

False-positive resistance: user types with the same head name
under their own namespace stay silent because the walker
compares full Parametric.head string identity, not substring.

### Sweep order

Same four tiers as slice 1c/1d. Substrate stdlib first.

1. **Substrate stdlib** — `wat/`, `crates/*/wat/`.
2. **Lib + early integration tests** — embedded wat strings in
   `src/check.rs::tests`, `src/runtime.rs::tests`, `tests/wat_*.rs`.
3. **`wat-tests/`** + **`crates/*/wat-tests/`**.
4. **`examples/`** + **`crates/*/examples/`**.

Verification gate after each tier: `cargo test --release
--workspace` → zero `BareLegacyContainerHead` errors before next
tier.

## Estimated scope

- `Option<T>` references: ~thousands across the codebase (used
  for every fallible-or-absent return).
- `Result<T,E>` references: ~thousands (every fallible op).
- `HashMap<K,V>` references: ~hundreds.
- `HashSet<T>` references: ~tens.

Substantial — likely the largest sweep so far, but the mechanism
is rehearsed (slice 1c's ~1000 sites + slice 1d's ~530 + this
slice's projected several thousand). Sonnet's diagnostic-driven
sweep keeps pace because the brief stays the same: read the
substrate's BareLegacyContainerHead errors per site, apply the
rename.

## What does NOT change

- **Internal Rust string literals** like `Parametric { head:
  "Option".into() }` — these are the canonical-form internal
  representations. Do not touch.
- **Vec<T>** — slice 1f territory. Not flagged by this slice's
  walker.
- **Constructor sites** like `(Some 5)`, `(:None)`, `(Ok 1)`,
  `(Err :failed)` — those are values, not types. Walker doesn't
  touch them. Slice 1g territory.
- **Method forms `:wat::core::option::expect` / `try`** — slice
  for § D' once type names are FQDN'd.

## Closure (slice 1e step N)

When sweep is structurally complete:

1. Update `INVENTORY.md` § B — strike four rows; mark `Option<T>`,
   `Result<T,E>`, `HashMap<K,V>`, `HashSet<T>` as ✓ shipped slice
   1e. `Vec<T>` row stays pending (slice 1f).
2. Update `J-PIPELINE.md` — slice 1e done.
3. Update `SLICE-1E.md` — flip from anchor to durable
   shipped-record.
4. Add 058 changelog row.
5. Optional: retire bare heads at parser level once consumer
   sweep is clean.

## Cross-references

- `docs/SUBSTRATE-AS-TEACHER.md` § "Three migration patterns" — Pattern 3.
- `docs/arc/2026/04/109-kill-std/INVENTORY.md` § B — the four
  rows this slice strikes.
- `docs/arc/2026/04/109-kill-std/SLICE-1C.md` — the precedent
  (Path-arm walker shape).
- `docs/arc/2026/04/109-kill-std/SLICE-1D.md` — the precedent
  (Tuple-arm walker shape; substrate-gap pattern with
  audit/canonical split).
- `src/check.rs::validate_bare_legacy_primitives` — the existing
  walker; slice 1e extends it with a Parametric-head-aware arm.
- `src/types.rs::parse_type_expr_audit` — the data-driven parse.
