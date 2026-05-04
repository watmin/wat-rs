# Arc 148 Slice 3 — Sonnet Brief — `values_compare` ord buildout

**Drafted 2026-05-03.** Foundation slice. Substrate-informed:
orchestrator's slice 1 audit (`AUDIT-SLICE-1.md`) found
`eval_compare` accepts a NARROWER set than `values_equal`; this
slice extends ord coverage to mirror `values_equal`'s recursive
shape so slice 5 can retire `infer_polymorphic_compare`'s
non-numeric branch without regressing ord on those types.

**Audit clarification:** the orchestrator's brief refers to
`values_compare` as the helper that mirrors `values_equal`. That
helper does NOT yet exist — `eval_compare` has the comparison
match INLINE at `src/runtime.rs:4622-4644`. Slice 3 either
extends the inline match OR extracts it into a `values_compare`
helper (sonnet's choice; both are valid; extracted-helper mirrors
arc 146's per-Type impl factoring style).

FM 9 baseline confirmed pre-spawn (post-slice-2):
- `wat_arc146_dispatch_mechanism` 7/7
- `wat_arc144_lookup_form` 9/9
- `wat_arc144_special_forms` 9/9
- `wat_arc144_hardcoded_primitives` 17/17
- `wat_arc143_define_alias` 3/3

**Goal:** extend the substrate's ord-comparison coverage so
`eval_compare` accepts the same types `values_equal` accepts
(minus the unordered ones). NO new substrate primitives
registered; NO architectural change beyond the comparison
match's reach.

**Working directory:** `/home/watmin/work/holon/wat-rs/`

## Required pre-reads (in order)

1. **`docs/arc/2026/05/148-arithmetic-comparison-correction/DESIGN.md`**
   — § "Architecture differs between arithmetic and comparison"
   + § "What 'same-type universal delegation' actually serves"
   + § "Slice 3" entry. Architecture context.
2. **`docs/arc/2026/05/148-arithmetic-comparison-correction/AUDIT-SLICE-1.md`**
   — Open Question 1 (`eval_compare` allowlist narrower than DESIGN).
3. **`docs/COMPACTION-AMNESIA-RECOVERY.md`** — discipline. § FM 9
   (re-run baselines pre-spawn — already done); § 12 (foundation
   work; eliminate failure domains).
4. **`src/runtime.rs:4491-4601`** — `values_equal` (the function
   to mirror). Pay attention to the recursive arms for `Vec`
   (`:4507-4519`), `Tuple` (`:4520-4532`), `Option` (`:4533-4537`),
   `Result` (`:4538-4548`). Slice 3 mirrors these recursion
   patterns for ord.
5. **`src/runtime.rs:4603-4645`** — `eval_compare`. The site to
   extend. Note the inline match at `:4622-4634`.
6. **`src/runtime.rs`** — grep for `Value::Bytes`, `Value::wat__time__Instant`,
   `Value::wat__time__Duration` to locate the Value enum variants
   (audit didn't enumerate exact variant names — sonnet finds them).
7. **`tests/wat_polymorphic_arithmetic.rs`** — canonical comparison
   test pattern. Slice 3 follows the same shape for new ord tests.

## What ships

### Substrate extension — extend ord coverage

Add ord arms to `eval_compare` (or a new `values_compare` helper)
mirroring `values_equal`'s acceptance for these types:

| Type | Ord rule | Recursion? |
|---|---|---|
| `:wat::time::Instant` | chronological (compare timestamp values) | no |
| `:wat::time::Duration` | chronological (compare duration values) | no |
| `:wat::core::Bytes` | byte-wise lexicographic | no |
| `:wat::core::Vector<T>` (the algebra Vector — bit-exact) | element-wise lexicographic | yes (recurse on element comparison) |
| `:wat::core::Vec<T>` (the parametric Vec) | element-wise lexicographic | yes (recurse) |
| `:wat::core::Tuple<T...>` | element-wise lexicographic | yes (recurse) |
| `:wat::core::Option<T>` | `None < Some(_)`; `Some(x) cmp Some(y) = x cmp y` | yes (recurse on Some payload) |
| `:wat::core::Result<T,E>` | `Err < Ok`; `Err(x) cmp Err(y) = x cmp y`; `Ok(x) cmp Ok(y) = x cmp y` | yes (recurse on payload) |

KEEP existing ord arms for `i64`, `u8`, `f64`, mixed-numeric,
`String`, `bool`, `wat__core__keyword`. Do NOT remove these.

### What gets REJECTED (compile-time TypeMismatch via existing fall-through)

These types are NOT ord-comparable; the existing fall-through
arm raises `TypeMismatch` for them — slice 3 keeps that behavior:

- `:wat::core::HashMap` — no canonical order
- `:wat::core::HashSet` — no canonical order
- `:wat::core::Enum` — variants have no inherent order; rejecting
  is honest (per arc 148 DESIGN — user-defined enums get ord only
  if they opt in, future feature)
- `:wat::core::Struct` — no field-ordering decision
- `:wat::core::unit` — only one value; ord meaningless
- `:wat::holon::HolonAST` — algebraic surface; no canonical order

### Tests

Add a new test file: `tests/wat_arc148_ord_buildout.rs` (mirror
the shape of `tests/wat_polymorphic_arithmetic.rs`).

Test coverage:
- For each NEW ord-comparable type: 4 test cases covering `<`, `>`,
  `<=`, `>=` semantics. (`=` and `not=` are already covered by
  `values_equal`.)
- For each REJECTED type: 1 test case asserting `TypeMismatch` is
  raised when ord is attempted.
- Recursive types (Vec, Tuple, Option, Result): 2 test cases each
  — one shallow (element-of-element fails fast) and one deep
  (recursion to leaf).

Pattern from `tests/wat_polymorphic_arithmetic.rs`: `run(src)` wat
program with assertions; verify expected `:bool` output.

## What this slice does NOT do

- NO renames; NO new substrate primitives ADDED to the registration
  table.
- NO changes to `eval_eq` / `values_equal` (equality already universal).
- NO touching `infer_polymorphic_compare` — slice 5 retires it
  using slice 3's expanded ord coverage.
- NO Dispatch entity creation (slice 5 work).
- NO wat-side files added.
- NO changes to the `:wat::core::<` / `<=` / `>` / `>=` / `=` /
  `not=` user-facing op routing — same evaluator, just expanded
  acceptance.

## STOP at first red

If you discover that one of the recursive cases (Vec/Tuple/Option/
Result) requires a primitive substrate concept you don't have
(e.g., a generic comparison trait in the Value enum that doesn't
exist), STOP and report. The recursion pattern in `values_equal`
is the model — if Rust's `Ord` trait isn't available on
`Value::Vec`'s element type for any reason, surface it.

If a baseline test fails post-extension (e.g., the recursive
case introduces a different ordering than expected for some
existing test), STOP and report which test + what failed +
what your investigation found.

## Source-of-truth files

- `src/runtime.rs:4491-4601` — `values_equal` (the recursion
  pattern to mirror)
- `src/runtime.rs:4603-4645` — `eval_compare` (the site to extend)
- `src/runtime.rs` (Value enum) — locate `Value::Bytes`,
  `Value::wat__time__Instant`, `Value::wat__time__Duration` exact
  names via grep
- `tests/wat_polymorphic_arithmetic.rs` — test pattern to follow

## Honest deltas

If you find a Value variant that should be ord-comparable but
isn't on the BRIEF's "What ships" list (e.g., a `Value::char` or
a numeric variant the audit missed), surface as honest delta.
If you find that one of the rejected types ACTUALLY has a
sensible ord under some interpretation (e.g., Enum variants in
declaration order), surface as open-question — DO NOT silently
add the arm.

## Report format

After shipping:

1. Total new arms added to `eval_compare` (should be 8 — Instant,
   Duration, Bytes, Vec, Tuple, Option, Result, Vector)
2. Whether you extracted to `values_compare` helper or extended
   inline (and why)
3. Total tests added (new test file line count + count of test
   functions)
4. Test results: list which tests pass; confirm baselines green
5. Workspace failure profile (per FM 9: should be unchanged from
   pre-slice baseline)
6. Any honest deltas surfaced

Time-box: 60 min wall-clock. Predicted Mode A 30-45 min — substrate
impl + tests; one-file scope (mostly runtime.rs).

## What this unlocks

Slice 5 (numeric comparison migration) can retire
`infer_polymorphic_compare`'s non-numeric branch without
regressing ord on time/Bytes/Vector/Tuple/Option/Result —
universal delegation now actually works.
