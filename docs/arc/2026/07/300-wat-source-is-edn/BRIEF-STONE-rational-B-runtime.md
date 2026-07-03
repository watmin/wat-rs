# BRIEF — Stone B: rationals in the wat runtime (REPRESENTATION; un-breaks the workspace)

**The work (one paragraph).** Stone A added `Value::Rational(BigRational)` to the `wat-edn` crate; that
broke 3 exhaustive matches in the root `wat` crate (they match `wat_edn::Value` with no `Rational` arm →
3× E0004, the workspace does not build). Add the **runtime representation** of a rational — a
`Value::wat__core__Rational(Box<num_rational::BigRational>)` variant, a `WatAST::RationalLit(BigRational,
Span)` literal, `wat-reader` lexing of `<int>/<int>` source tokens, evaluation, rendering, and the
`:wat::core::Rational` type — so a rational is representable in wat source **and the workspace builds green
again**. REPRESENTATION only: **no** arithmetic operators, **no** `Rational/of` constructor. Turn the RED
spec `tests/value/probe_rational_B_runtime_representation.rs` green. Normalization mirrors clj EXACTLY
(grounded this session): a literal reducing to a whole number is an Integer (`4/2 → 2`), a genuine ratio is
a Rational (`1/2`, `-6/4 → -3/2`), `1/0` is a clean reader error.

## Read in order (the rooms)

1. `docs/arc/2026/07/300-wat-source-is-edn/DESIGN-STONE-rational-B-runtime.md` — the design, the pinned
   contract (RationalLit lane, NOT desugar), the oracle table. **Read first.**
2. `tests/value/probe_rational_B_runtime_representation.rs` — the RED spec you turn green.
3. `crates/wat-edn/src/lexer.rs` `lex_number` (~645) — Stone A's DONE rational lexing at the DATA layer.
   **Copy its normalization** (reduce, sign on numerator, den==1→Integer, `/0`→error) for the source lexer.
4. `crates/wat-reader/src/lexer.rs:835` `lex_numeric_or_symbol` — add the rational branch **between** the
   f64 parse (849) and the `InvalidNumber` error (851): `raw` is already `"1/2"`; split on `/`, parse both
   sides, normalize via `BigRational`, `den==1 → Token::Int`, `/0 → InvalidNumber("divide by zero")`, else
   `Token::Rational(r)`. Add `Token::Rational(BigRational)` to the `Token` enum in this file.
5. `crates/wat-reader/src/ast.rs:58` — add `WatAST::RationalLit(BigRational, Span)` (follow `IntLit`/
   `FloatLit`). Thread the fan-out arms in this file: `span()` ~147, constructors ~165, type-string ~413,
   `Hash` ~460.
6. `crates/wat-reader/src/parser.rs:335` — `Token::Rational(r) => Ok(Some(WatAST::RationalLit(r, span)))`,
   right after the `Float` arm.
7. `src/value/value.rs:309` — add `wat__core__Rational(Box<num_rational::BigRational>)` beside
   `wat__core__Char`. Arms: `PartialEq` :591, `Hash` :740, `type_name` :1134 → `"wat::core::Rational"`, the
   type-string path :1225.
8. `src/runtime.rs:3616` — the eval arm, right after `FloatLit`:
   `WatAST::RationalLit(r, span) => Ok(TrackedValue::new(Value::wat__core__Rational(Box::new(r.clone())),
   Provenance::Literal { span: span.clone() }))`. Also the reverse `Value → WatAST` at :9397-9398, and the
   `FloatLit` neighbors at `src/lower.rs:216` + `src/check.rs:3268,7480` — add `RationalLit` arms where the
   compiler demands (the cascade names them).
9. `src/value/observe.rs:390` — `render_value`: `Value::wat__core__Rational(r) => format!("{}/{}",
   r.numer(), r.denom())` (a genuine ratio always has den≥2 — no `"/1"` case).
10. `src/edn_shim.rs:1240` — `Edn::Rational(r) => Ok(Value::wat__core__Rational(Box::new((**r).clone())))`;
    shape_name :1334 → `"Rational"`; the coercion table :1476/1484 → add `":wat::core::Rational" => match
    edn { Edn::Rational(r) => Ok(...), _ => mismatch }`. **These are the 3 E0004 sites** — adding the arms
    un-breaks the build.
11. `src/wat_edn_bridge.rs:134` — `Edn::Rational(r) => Ok(WatAST::RationalLit((**r).clone(),
    crate::rust_caller_span!()))`.
12. `src/types.rs` ~2934 + `src/check.rs` — ensure `:wat::core::Rational` is accepted as a valid scalar
    `Path` type (scalars flow as `TypeExpr::Path`; no `register_builtin`). Add to `BARE_PRIMITIVES`
    (check.rs:1599) ONLY if a bare `:Rational` spelling is wanted — else FQDN-only like `Uuid`/`char`.
13. `Cargo.toml:90` — add `num-rational.workspace = true` and `num-bigint.workspace = true` to the root
    `[dependencies]`, beside `uuid.workspace = true`.

## How to work

The workspace is **currently broken by 3 E0004** — Stone A landed the `wat-edn` variant; the root crate's
3 arms (edn_shim ×2, wat_edn_bridge ×1) don't cover it yet. **This is EXPECTED** — your Stone B un-breaks
it. Build with `cargo build -p wat` and follow the compile cascade toward zero — each error names the next
arm to add (the progress meter, not a crisis). When it builds:
`cargo test -p wat --test probe_rational_B_runtime_representation` (green) → `cargo test -p wat-edn`
(Stone A still green) → a broad `cargo nextest run`, **read the Summary line, not a grep**. Capture the
full run ONCE to a temp file and grep the FILE if you need to inspect failures; use targeted `-p wat
--test X` for iteration. Do NOT re-run the whole suite to re-grep.

## STOP triggers (halt + report; never improvise a workaround)

- STOP if `4/2` in wat source does NOT become a runtime `Value::i64(2)` — it must mirror clj's Long.
- STOP if `1/0` panics rather than a clean parse `Err`.
- STOP if turning the probe green requires an **arithmetic or comparison** operator — that is Stone C.
- STOP if a reduced literal numerator/denominator exceeds `i64` (a big-literal den==1 that can't be an
  `i64` Integer) — report it; runtime BigInt is out of scope, not a thing to invent here.
- STOP if `RationalLit` cascades into far more than the ~12 mapped sites — report the surprise.

## Done = green

- `tests/value/probe_rational_B_runtime_representation.rs` → green.
- `cargo build -p wat` → clean (0 of the 3 E0004).
- `cargo test -p wat-edn` → still green (Stone A untouched).
- Broad `cargo nextest run` weighed (Summary line — no regression).

Report: files changed; the exact fan-out sites the cascade surfaced (vs the ~12 mapped); the normalization
approach (how you reused Stone A's); any STOP hits.

**Prior reference to copy for shape:** `BRIEF-STONE-rational-A-wat-edn.md` + its DONE lexer
`crates/wat-edn/src/lexer.rs` (the normalization to mirror).
