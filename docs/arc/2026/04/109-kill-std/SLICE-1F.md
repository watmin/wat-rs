# Arc 109 Slice 1f — Vec<T> renames to Vector + vec verb shares the type name

**Compaction-amnesia anchor.** Read this first if you're picking up
slice 1f mid-flight.

## What this slice does

Two coupled renames per INVENTORY § B + § D's slicing note ("verb
= type"):

1. **Type rename + move:** `Vec<T>` → `:wat::core::Vector<T>`. The
   type's bare form `Vec<T>` retires; the FQDN under
   `:wat::core::*` carries a NEW name (`Vector`, not `Vec`).
2. **Verb rename:** `:wat::core::vec` → `:wat::core::Vector`. The
   constructor verb shares the type name. `(:wat::core::Vector :T
   x y z)` reads as "construct a Vector of T from these elements."

The coupling matters: doing them apart leaves a half-migrated
state where `(:wat::core::vec :T)` returns one type and
annotations expect another. Bundle, atomic.

## What this slice does NOT do

- **`:wat::core::list` retire** — slice 1g territory.
  Independent rename. Currently a duplicate of `vec`.
- **`:wat::core::tuple` → `:wat::core::Tuple`** — slice 1g.
  Verb-matches-type companion to slice 1f, but no call-site
  coupling with Vec.
- **`:wat::core::range` → `:wat::list::range`** — § H territory.
  Composition-over-primitives move; bigger surgery.

## The protocol

Two mechanisms, both proven:

- **Pattern 3** (slice 1c/1d/1e shape) for the **type** rename:
  extend `BARE_CONTAINER_HEADS` with `("Vec", "wat::core::Vector")`.
  Walker fires on `Parametric { head: "Vec", ... }` in user code.
- **Pattern 2** (arc 114 shape) for the **verb** retirement:
  synthetic `CheckError::TypeMismatch` in the
  `:wat::core::vec` dispatcher with `expected:
  ":wat::core::Vector"`, `got: ":wat::core::vec"`. Add
  `arc_109_vec_verb_migration_hint` to `collect_hints` to detect
  the shape pair and emit the rename brief.

Same audit/canonical split: the FQDN form `:wat::core::Vector<T>`
parses to a Parametric with head `"wat::core::Vector"` (audit
walker silent); canonicalize=true rewrites it to `"Vec"` so the
substrate's existing `Vec`-keyed dispatch keeps working.

## What to ship

### Substrate (Rust)

1. **Mint typealias** in `src/types.rs::register_builtin_types`:
   ```rust
   env.register_builtin(TypeDef::Alias(AliasDef {
       name: ":wat::core::Vector".into(),
       type_params: vec!["T".into()],
       expr: TypeExpr::Parametric {
           head: "Vec".into(),
           args: vec![TypeExpr::Path(":T".into())],
       },
   }));
   ```

2. **Extend `BARE_CONTAINER_HEADS`** in `src/check.rs`:
   ```rust
   ("Vec", "wat::core::Vector"),
   ```
   The walker emits `BareLegacyContainerHead` with head="Vec",
   fqdn="wat::core::Vector". Display IS the migration brief; no
   new variant needed.

3. **Extend `parse_type_inner` canonicalization** for the
   parametric-head FQDN→bare map:
   ```rust
   "wat::core::Vector" => "Vec".to_string(),
   ```
   Same shape as the four heads slice 1e added.

4. **Add `:wat::core::Vector` constructor verb dispatch** in
   `src/check.rs` + `src/runtime.rs`. Same special-form arms the
   `:wat::core::vec` verb has today, just under the new name. The
   verb produces `Parametric { head: "Vec", ... }` internally
   (the canonical shape) — both spellings work; the
   `BareLegacyContainerHead` walker will steer call sites.

5. **Poison `:wat::core::vec`** at the dispatcher (Pattern 2 from
   SUBSTRATE-AS-TEACHER). Synthetic TypeMismatch with
   `callee: ":wat::core::vec"`, `expected:
   ":wat::core::Vector"`, `got: ":wat::core::vec"`. Pair with
   `arc_109_vec_verb_migration_hint` in `collect_hints`.

### Verification

Probe coverage:
- `(name :Vec<wat::core::i64>)` → fires (bare Vec head, Pattern 3)
- `(name :wat::core::Vector<wat::core::i64>)` → silent (FQDN)
- `(:wat::core::vec :wat::core::i64 1 2 3)` → fires (poisoned
  verb, Pattern 2)
- `(:wat::core::Vector :wat::core::i64 1 2 3)` → silent (canonical)
- `:Option<Vec<wat::core::i64>>` → fires (inner bare Vec inside
  FQDN Option from slice 1e)
- `:my::pkg::Vec<T>` → silent (user path; head "my::pkg::Vec"
  doesn't match)

## Sweep order

Same four tiers as slices 1c/1d/1e. Substrate stdlib first.

1. **Substrate stdlib** — `wat/`, `crates/*/wat/`. Substrate
   boots clean.
2. **Lib + early integration tests** — `src/check.rs::tests`,
   `src/runtime.rs::tests`, `tests/wat_*.rs`.
3. **`wat-tests/`** + **`crates/*/wat-tests/`**.
4. **`examples/`** + **`crates/*/examples/`**.

After each tier, run `cargo test --release --workspace` and
confirm zero `BareLegacyContainerHead` (head=Vec) AND zero
TypeMismatch-on-`:wat::core::vec` errors.

## Estimated scope

- `Vec<T>` references: ~hundreds-to-low-thousands. Vec is the
  most-used parametric in the codebase.
- `:wat::core::vec` callees: dozens to hundreds.

Together: probably 800–1500 rename sites across user code +
substrate stdlib. Larger than slice 1c (~1000), comparable to
slice 1e (~365 — but Vec is likely 2-3× more common than the
four heads combined).

Sonnet's diagnostic-driven sweep keeps pace because the brief
stays the same: read the substrate's hints, apply the rename per
site.

## What does NOT change

- **Internal Rust string literals** like `Parametric { head:
  "Vec".into() }` — canonical-form internal representations. Do
  not touch. The substrate dispatches against bare "Vec" head;
  that's the canonical internal form and it stays that way for
  this slice.
- **`:wat::core::list` / `:wat::core::tuple` / `:wat::core::range`**
  — slice 1g territory.
- **The walker logic itself** — `BareLegacyContainerHead` variant
  + `walk_type_for_bare`'s Parametric arm + `parse_type_expr_audit`
  shipped in slice 1e (commit `f8a82be`). Slice 1f extends the
  data tables, doesn't reshape the mechanism.
- **Variant constructors** — `Some`/`:None`/`Ok`/`Err` are slice
  § C territory. The walker doesn't touch them.

## Closure (slice 1f step N)

When sweep is structurally complete:

1. Update `INVENTORY.md` § B — strike `Vec<T>` row; mark ✓
   shipped slice 1f.
2. Update `INVENTORY.md` § D — strike the `vec` → `Vector` verb
   row; mark ✓ shipped slice 1f.
3. Update `J-PIPELINE.md` — slice 1f done.
4. Update `SLICE-1F.md` — flip from anchor to durable
   shipped-record.
5. Add 058 changelog row.

## Cross-references

- `docs/SUBSTRATE-AS-TEACHER.md` — Pattern 2 (verb retirement)
  and Pattern 3 (TypeExpr-shape detection). Slice 1f bundles
  both.
- `docs/arc/2026/04/109-kill-std/INVENTORY.md` § B + § D — the
  rows this slice strikes.
- `docs/arc/2026/04/109-kill-std/SLICE-1E.md` — the precedent for
  the type rename mechanism.
- `docs/arc/2026/04/114-spawn-as-thread/INSCRIPTION.md` — the
  precedent for the Pattern 2 verb retirement.
- `src/check.rs::BARE_CONTAINER_HEADS` — the table this slice
  extends.
- `src/check.rs::collect_hints` — where
  `arc_109_vec_verb_migration_hint` lands.
