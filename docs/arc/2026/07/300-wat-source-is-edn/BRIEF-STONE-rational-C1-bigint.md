# BRIEF — Stone C1: numeric-scalar naming cleanup + `bigint` as a first-class arithmetic type

**The work (one paragraph).** Two parts, one strike. **Part A (naming):** per Doctrine 2 (scalar types
lowercase), rename the SURFACE keyword `:wat::core::Rational` → `:wat::core::rational` (the Rust *variant*
`Value::wat__core__Rational` stays Capital — exactly the `Char → char` precedent), finish the half-done
`Char` rename (`type_name` still emits capital `"wat::core::Char"`), and register `rational`/`bigint` as
builtin scalars. **Part B (bigint):** add `Value::wat__core__BigInt(Box<num_bigint::BigInt>)` (surface
`:wat::core::bigint`) as a **full first-class arithmetic integer type** — arbitrary precision (never wraps),
contagious (`i64 ⊕ bigint → bigint`), never demotes, `/` collapses to `bigint`|`rational`, total-order
comparison, category-aware `=`, renders `"<n>N"`, reads the `1N` literal, `to-f64`. Turn the RED spec
`tests/value/probe_rational_C1_bigint.rs` green. Grounded against the clj oracle (rows in the design).

## Read in order (the rooms)

1. `docs/arc/2026/07/300-wat-source-is-edn/DESIGN-STONE-rational-C1-bigint.md` — the design, the oracle
   contract, the exact `file:line` room table, the naming rule. **Read first.**
2. `tests/value/probe_rational_C1_bigint.rs` — the RED spec (7 tests) you turn green.
3. `tests/value/probe_rational_B_runtime_representation.rs` — Stone B's probe. Its
   `rational_literal_reads_as_runtime_rational` asserts capital `"wat::core::Rational"`; **update that one
   assertion to lowercase `"wat::core::rational"`** as part of Part A.
4. `crates/wat-edn/src/lexer.rs` — wat-edn already lexes the `1N` literal → `Value::BigInt` (the `N`-suffix
   branch). **Copy its approach** for the source lexer. Its writer emits `"{}N"` (`writer.rs`).
5. `crates/wat-reader/src/lexer.rs` `lex_numeric_or_symbol` (~835) — Stone B added the rational `/` branch
   here; add a **`1N` branch** beside it: `raw` matches `<int>N` (all digits + trailing `N`, optional `-`)
   → strip `N`, parse `num_bigint::BigInt` → `Token::BigInt(BigInt)`. Add `Token::BigInt` to the enum;
   parser arm → `WatAST::BigIntLit` (mirror `RationalLit` at parser.rs:339 / ast.rs).

## Part A — naming (exact sites from the intueri cast)

- Rename SURFACE string `":wat::core::Rational"` → `":wat::core::rational"`: `src/value/value.rs:1151`
  (type_name) · `src/runtime.rs:6726` · `src/edn_shim.rs:1495` · `src/check.rs:3270` · `src/check.rs:7498`
  (+ doc comments `value.rs:311`, `check.rs:3269`). **Do NOT touch** the Rust variant `wat__core__Rational`.
- Fix `Char`: `src/value/value.rs:1149` + `src/runtime.rs:13438` emit capital `"wat::core::Char"` → lowercase
  `"wat::core::char"` (`check.rs:13587` is already lowercase — match it).
- Register scalars: `is_builtin_primitive` (`runtime.rs:13427-13454`) + the pure-scalar list
  (`check.rs:13579-13596`) → add `"wat::core::rational"` and `"wat::core::bigint"` beside `i64`/`f64`/`char`.

## Part B — bigint (rooms in the design's room table)

`Value::wat__core__BigInt(Box<num_bigint::BigInt>)` beside `wat__core__Rational` (value.rs:319). Fan-out:
PartialEq (+ **i64↔bigint category equality**), Hash, type_name → `"wat::core::bigint"`, the type-keyword
map (runtime.rs:6726), eval of `BigIntLit`, render_value → `format!("{}N", n)`. Arithmetic intrinsics
`:wat::core::bigint::+ - * /` at the eval dispatch (runtime.rs:4278-4293) + inner dispatch (8785-8790),
impl fns modeled on `eval_i64_arith`/`arith_i64_i64_inner` but **no overflow branch** (BigInt is arbitrary
precision); `/` → `bigint` if divisible else build a `Rational`. Extend the `wat/core.wat` `+ - * /`
defclauses with `:wat::core::bigint` arms (1/2/N-ary, mirroring i64) **plus the mixed `i64 ⊕ bigint →
bigint` contagion arms**. Comparison: `values_compare` (8360-8437) + `values_equal` (8156-8331) gain bigint
+ cross-type arms. `to-f64`: dispatch arm + fn (BigInt `to_f64` via num-traits `ToPrimitive`).

## How to work

The workspace is green at HEAD. Do **Part A first** (rename + register — the cascade is small and
mechanical), rebuild, then **Part B** (bigint) and follow the compile cascade toward zero (each error names
the next arm — the progress meter). Then:
`cargo test -p wat --test value probe_rational_C1_bigint` (green) →
`cargo test -p wat --test value probe_rational_B` (Stone B still green, with the updated lowercase
assertion) → `cargo test -p wat-edn` (green) → a broad `cargo nextest run`. **Read the Summary line.**
Capture the full run ONCE to a temp file and grep the FILE. The suite has TWO known reds that are NOT
yours: `no_inlined_wat_in_tests` (the intended expected-red campaign meter, baseline 351) and
`wat-cli sigterm_to_cli_cascades_via_polling_contract` (a rare arc-170 race that passes with
`--test-threads=1`). Anything else red is yours.

## STOP triggers (halt + report)

- STOP if `bigint` arithmetic wraps/overflows — it MUST be arbitrary precision.
- STOP if `(= 1N 1)` ≠ `true` or `(= 1N 1.0)` ≠ `false` (category-aware `=`).
- STOP if `/` divisible ≠ `bigint` or non-divisible ≠ `rational`.
- STOP if the rename touches the Rust *variant* identifiers `wat__core__Rational`/`wat__core__BigInt`
  (those stay Capital — only the surface `:wat::core::…` strings lowercase).
- STOP if closing C1 needs i64-overflow behavior changes (that is C3) or rational *arithmetic* (that is C2).

## Done = green

- `probe_rational_C1_bigint` → 7/7 green.
- `probe_rational_B` → green (with the lowercase assertion).
- `cargo build -p wat` clean; `cargo test -p wat-edn` green.
- Broad `cargo nextest run` weighed — only the two known reds above; no new failures.

Report: files changed; the fan-out sites the cascade surfaced; how you mirrored wat-edn's `1N` lexing;
any STOP hits.

**Prior reference:** `BRIEF-STONE-rational-B-runtime.md` + Stone B's committed lexer/value fan-out — C1's
`bigint` mirrors that shape exactly, one type over.
