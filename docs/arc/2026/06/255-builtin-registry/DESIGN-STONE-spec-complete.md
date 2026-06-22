# DESIGN+BRIEF — Stone: spec-complete (variadic + @yields + @category)

**Status: STRIKE-READY (locked with the builder 2026-06-22).** Freeze the doc-contract so the
520-migration declares to a stable target (no mass re-touch / "thrash"). Builds on HEAD `9d30dbf3`
(bytes perfect + @pure/@deterministic). Three parts; probe-first each; STOP-gate the variadic cascade.

## The locked decisions (from the riff)
- **@yields is SINGLETON** (like @ret): `@yields <type> <desc>`, exactly one. Multi-yield → a
  Tuple/Record in that one type. The fn-arg therefore takes ONE param (the yielded value).
- **Variadic**: `@arg <name>… <elem-type> <desc>` — the `…` marks the rest-param; `elem-type` is the
  ELEMENT type (the `…` implies `Vector<elem>`, mirroring the checker's `rest_param_type`).
- **@category is a CLOSED ENUM** (contract §5, like Kind/DefinedIn/Layer) — NOT free text, NOT a
  const string list. A `Category` enum; `@category <Variant>` checked against it → build-fail on
  unknown. Seed: `Encoding`, `Reflection`. Grows by variant (append-only; never touches existing docs).
- **expand-time-legal: DERIVED, no directive** (pure ∧ total; not declared) — out of scope here.

## Part A — variadic (the capability; STOP-gate the cascade)
1. `crates/wat-macros/src/wat_intrinsic.rs:~89` currently REJECTS `&[WatAST]` handlers. Teach it: a
   handler whose wat-arg is a single `&[WatAST]` is VARIADIC → emit a variadic shim (pass the whole
   slice) + arity = variadic.
2. **`Arity` on the registry**: today `IntrinsicSubmission`/`IntrinsicEntry.arity: usize` is
   Exact-only. Add an `Arity { Exact(usize), Variadic }` (minimal — Range/AtLeast NOT needed yet;
   don't build the forcing function). **STOP-A: if changing `arity: usize` → `Arity` cascades beyond
   the registry + metadata-of's arity reader, STOP and list the sites before proceeding** (this is
   the wide-cascade risk).
3. Grammar (`crates/wat-doc`): `@arg <name>… <elem-type> <desc>` — the `…` suffix on the name marks
   the rest-param; carry an `is_rest: bool` on `DocArg`.
4. The type cross-check (`doc_arg_ret_types_match_checker_scheme`): a `…` `@arg` compares its
   `elem-type` against the scheme's `rest_param_type` element (not `params[i]`).
5. **Witness** `:wat::intrinsic::variadic-args-measurement` in `src/intrinsic/reflect.rs` (or a new
   `src/intrinsic/witness.rs`): variadic, returns the **count of args** (`:wat::core::i64`),
   pure∧det, fully doc-compliant (`@arg xs… :wat::core::Value the args to count`, `@ret
   :wat::core::i64 …`, runnable `@example (:wat::intrinsic::variadic-args-measurement 1 2 3) #=> 3`,
   `@category Reflection`). Register its scheme in `check.rs` with `rest_param_type = Some(Vector<Value>)`.

## Part B — @yields (singleton directive + HOF witness)
1. Grammar (`wat-doc`): `@yields <type> <desc>` — OPTIONAL, singleton; type token starts with `:`.
   Carry `yields: Option<(type, desc)>` on `DocComment`.
2. Macro carries it onto the registry entry (`yields_type: Option<&'static str>`).
3. Cross-check (consumer test `yields_type_matches_fn_arg_param`): for an entry with `@yields`, find
   its fn-arg (the `@arg` whose scheme type is `Fn(P)->R`), assert `@yields` type == `P` (the fn's
   single param). **STOP-B: if the scheme's Fn param type isn't cleanly extractable, STOP + report.**
4. **Witness** `:wat::intrinsic::yields-witness` (a minimal HOF): `@arg f :wat::core::Fn(:wat::core::i64)->:wat::core::i64
   the fn applied`, `@yields :wat::core::i64 the value handed to f`, returns the fn's result;
   pure∧det if f is (declare honestly), `@category Reflection`. Register its scheme (param: the Fn type).
   render-doc renders a `Yields:` line.

## Part C — @category (closed enum + apply)
1. A `Category` enum in `src/intrinsic/mod.rs` (mirror `Kind`/`DefinedIn`/`Layer`): variants
   `Encoding`, `Reflection` (+ any the witnesses need). `to_enum_value()` for metadata-of.
2. Grammar (`wat-doc`): `@category <Variant>` REQUIRED (every intrinsic categorized); carry on `DocComment`.
3. Cross-check (in the macro OR a consumer test): `@category` must be a known `Category` variant →
   build-fail on unknown. (Macro-side if the variant set is shareable; else consumer test.)
4. Apply: bytes (`@category Encoding`), the trio + the two witnesses (`@category Reflection`).
5. metadata-of returns `:category`; render-doc renders a `Category:` line.

## RED probes (verify each bites)
- variadic: `(:wat::intrinsic::variadic-args-measurement 1 2 3) #=> 3` (a wat-tests deftest' or the
  reflection-surface file) — RED at HEAD (verb absent), GREEN after A.
- @yields: the cross-check fails if `@yields`'s type ≠ the fn-arg param type (prove it bites).
- @category: a doc with `@category Bogus` fails the build (unknown variant).

## Gate — spec frozen
- variadic witness GREEN; @yields cross-check GREEN + bites; @category unknown-variant rejected.
- the existing 4 cross-checks + reflect + floors hold (lib 961+/36/1; wat-tests 268+/1; wat-doc green).
- clippy clean; no new dead-code; no hand-list (Category is the canonical closed-domain enum, not a
  parallel list — it's authored taxonomy, not derived).

## Out of scope (named)
expand-time-legal/@total (derived; or a bounded future add for the macro-combinator subset); the
520-migration; the wiki generator (§7); fuzzy-docs MCP (HORIZON).
