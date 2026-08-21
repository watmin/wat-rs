# DESIGN — arc 109 Stone ②-i-b: the Tuple arm (finishing what ②-i scoped out)

**Status: DRAWN 2026-08-20.** Blocks ②-iii. Written against `e89821450`.

## Why

②-i (`0422b67ff`) gave `type_expr_to_clojure_form` a head-spelling mode and bracketed the
`Parametric` arm. Its rider scoped ONE arm out and said so plainly:

> *"`TypeExpr::Tuple`'s head (`wat.type/Tuple`) I left OUT OF SCOPE for `mode` — it's not part of
> the 4-way ladder Room 2 scopes to, and nothing in the acceptance criteria or the 8-fixture
> contract suite exercises a COLON-mode Tuple."*

Accurate and correctly reported. ②-ii then walked into it, and the codemod had to grow a
rendered-output guard that SKIPS rather than corrupts. `NOTE-the-Tuple-arm-is-mode-blind.md` files
the measurement. This stone closes it.

## Measured at HEAD, by the orchestrator's own hand

`wat-scripts/scratch-pad/arc109-tuple-arm-faults.wat`:

```
1 nil bare      : (wat.type/Tuple)
2 nil nested    : (:wat::core::Result [(wat.type/Tuple) :wat::core::String])
3 tuple 3-ary   : (wat.type/Tuple :wat::core::i64 :wat::core::i64 :wat::core::String)
4 tuple empty   : (wat.type/Tuple)
5 control parm  : (:wat::core::Vector [:wat::core::i64])
```

Three faults: wrong head spelling in COLON mode · mixed spelling inside one otherwise-correct form ·
args spliced FLAT instead of bracketed. Row 5 is the control — `Parametric` is already right.

**Rows 1 and 4 are the finding: `nil` and `:()` render IDENTICALLY.**

## The correction this stone rests on

The seam recorded `nil` and `()` as *"verified distinct at the surface"* because `-> :()` with a nil
body exits 1. The exit code was right; the inference was not. The error text says the opposite:

> `BareLegacyUnitType`: *"bare unit type '()' is retired (arc 109 slice 1d); canonical FQDN form is
> ':wat::core::nil' (arc 153 renamed unit -> nil)"*

`:()` is rejected as a **retired spelling of the same type**, not as a different type. Internally
`nil ≡ TypeExpr::Tuple(vec![])`, and ~30 sites in `check.rs`/`runtime.rs`/`freeze.rs` use
`Tuple(vec![])` *as* the unit type.

Two consequences:

- **It shrinks the strike.** `:()` appears **0 times** as a type annotation in the corpus (the only
  three `:()` hits are a string fed to the verb in a fixture, a comment, and this stone's own probe).
  An empty Tuple is unreachable from legal source; the empty case is defensive. The real corpus work
  is the non-empty tuples: **243 occurrences** — `wat/` 52 · `wat-scripts/` 165 · `tests/` 26.
  ⚠ The NOTE's "30 standalone tuples" is a DIFFERENT measurement — what the codemod's guard skipped
  on the paths it ran, not a corpus census. Neither number is wrong; they answer different questions.
- **It enlarges the builder's ruling.** `nil != ()` is not what the substrate says today — slice 1d
  retired the empty-tuple spelling BY ALIASING IT TO UNIT. Making `(Tuple [])` writable as a thing
  distinct from `nil` means un-retiring it against those ~30 checker sites. **That is a separate,
  larger question and it is the builder's.** It does not block this stone: keeping `nil` as
  `Path(":wat::core::nil")` at parse time is correct under either future.

## The change

**(a) The verb stops canonicalizing.** `eval_keyword_to_type_form_impl` (`src/edn_shim.rs:1364`)
calls `parse_type_expr`, which hardcodes `canonicalize=true`; `src/types.rs:4728` then collapses
`:wat::core::nil` → `Tuple(vec![])`, which is why the renderer cannot say `nil`. A `canonicalize:
bool` already exists (`src/types.rs:4625`). The verb gets a non-canonicalizing sibling entry point.

The one other thing that flag governs is the `:wat::type::` → `:wat::core::` alias. Measured: every
`:wat::type::` keyword in the corpus is `:wat::type::Infer`, **all 39 of them**, and the codemod's
`type-shaped-keyword?` never selects it (no matching `<…>`). Preserving its spelling is *more*
faithful, not less. The flip is clean.

**(b) The Tuple arm brackets and honours the mode** — exactly what `Parametric` got in ②-i.
`(:wat::core::Tuple [:wat::core::i64 …])` in COLON, `(wat.type/Tuple [wat.type/i64 …])` in Clojure,
and the empty tuple is `(:wat::core::Tuple [])` — head always takes a bracket, even empty. This is
the builder's ruling, 2026-08-20:

> *"nil is rust's unit… but `nil != ()` in wat. nil is not an empty list. `(wat.type/Tuple)` is
> illegal, it'd be `(wat.type/Tuple [])` to be an empty tuple."*

## The contract decision, pinned

`pub fn parse_type_expr_preserving_with_span(kw: &str, span: &Span) -> Result<TypeExpr, TypeError>`
— byte-identical to `parse_type_expr_with_span` (`src/types.rs:4334`) except `canonicalize=false`.
It **still calls `reject_any`**. It returns `Result`, NEVER `Option` — the verb surfaces parse
errors and `parse_type_expr_audit` (the existing `canonicalize=false` path) swallows them, which is
why that one cannot be reused.

## The reader already accepts the bracketed form — proven, not read

`wat-scripts/scratch-pad/arc109-tuple-bracket-reader.wat`, `--check` EXIT=0. `parse_type_node`'s
bracket unwrap (`src/types.rs:4528`) is head-agnostic and the `Tuple` branch (`src/types.rs:4540`)
reads `args` AFTER it, so the bracket rule reached Tuple for free at step ①. The probe pins:
bracketed 2-ary as a param type · bracketed with a nested parametric as a return type · the EMPTY
`(wat.type/Tuple [])` unifying with a nil-returning body · and the FLAT form still reading.

**Non-vacuity control:** perturbing one inner member to `wat.type/Bogus` goes RED —
`":wat::core::Tuple: parameter #2 expects :wat::core::Bogus; got :wat::core::String"`. The bracketed
inner types are genuinely resolved and unified; the green is not free.

So the round-trip is safe and **the writer is the only side that changes.**

## Out of scope — affirmatively cut, with the reason

- **The reflection path still shows a nil return as an empty tuple.** `runtime.rs:13034` and
  `runtime.rs:14649` (`signature-of-defn`) share this renderer but receive an ALREADY-canonicalized
  `TypeExpr` from the stored scheme — there is no source keyword left to preserve, so half (a)
  cannot reach them. After this stone they render `(wat.type/Tuple [])` where they render
  `(wat.type/Tuple)` today. That is the pre-existing `nil ≡ unit` identity, not something this stone
  introduces, and closing it requires the substrate split named above — the builder's call.
  Measured: **no golden currently renders a nil return through that path** (`(wat.type/Tuple)`
  appears in exactly one golden, contract-07, which feeds `:()`).
- **The corpus migration of the 243 tuple sites** is ②-iii's job, not this stone's. This stone
  unblocks it by making the codemod's guard stop firing.
