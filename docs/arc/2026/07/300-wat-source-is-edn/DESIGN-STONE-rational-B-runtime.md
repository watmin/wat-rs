# DESIGN — Stone B: rationals in the runtime (REPRESENTATION; un-breaks the workspace)

**Thesis.** Stone A gave `wat-edn` (the EDN data layer) a `Value::Rational(BigRational)`. Adding that
variant broke 3 exhaustive matches in the ROOT crate (they match `wat_edn::Value` with no `Rational`
arm). Stone B adds the **runtime** representation those arms convert *into*, un-breaks the workspace, and
makes a rational **representable in wat source** — then **A + B commit atomically** (never a broken commit
in history). This is REPRESENTATION only. Arithmetic is Stone C.

## The oracle grounding (clj 1.12.4, run this session — AD ORACVLVM NON AD LIBRVM)

clj runs TWO different reductions, and the split is the B/C boundary:

| form | clj value | clj type | our layer |
|---|---|---|---|
| `4/2` `6/3` `1/1` `0/5` | `2 2 1 0` | **Long** | literal → B (i64 Integer) |
| `1/2` `-6/4` `10/4` | `1/2 -3/2 5/2` | **Ratio** | literal → B (Rational) |
| `(* 1/2 2)` `(+ 1/2 1/2)` | `1N` | **BigInt** | arithmetic → **C** |

**Consequence:** the literal/representation surface (B) reduces whole-number results to **Long**, so an
`i64` `Value::Integer` is the correct, faithful target — B needs NO runtime BigInt. The BigInt collapse is
arithmetic-only → Stone C's problem (and C's real fork: keep-as-Rational vs add runtime BigInt).

## THE ONE PINNED CONTRACT — a rational is a NUMERIC LITERAL, not a desugared constructor

Rational follows the **Int/Float precedent** (`WatAST::IntLit`/`FloatLit`, parser.rs:334-335) — a real
`*Lit` variant — **NOT** the Char/Uuid precedent (desugar → `(:wat::core::char/of "x")`, parser.rs:355).

```clojure
;; lexer normalizes at lex time — mirrors Stone A EXACTLY (clj-faithful):
1/2   -> Token::Rational(1/2)              ; genuine ratio (den >= 2)
4/2   -> Token::Int(2)                     ; den==1 reduces to Integer (clj: Long)
-6/4  -> Token::Rational(-3/2)             ; reduced, sign on numerator, den > 0
1/0   -> InvalidNumber("divide by zero")   ; clean lex error, NEVER panic

;; so a RationalLit ALWAYS holds a reduced ratio with den>=2 — never integer-valued:
Token::Rational(r) -> WatAST::RationalLit(r, span) -> Value::wat__core__Rational(box r)
```

**Why this lane (the four questions):**
- **Simple? YES** — the literal is fully normalized at lex; the den==1 case already became `Int`, so a
  `RationalLit` never holds an integer-valued ratio. No runtime constructor needed.
- **Honest / decision-free? YES** — with no `Rational/of` constructor, the question *"what does
  `Rational/of 4 2` return?"* (adjacent to C's `1/2+1/2` → BigInt) NEVER ARISES in B. The desugar lane
  would drag C's decision into B. A rational literal *is* a number like `1`/`1.5`, so the numeric lane is
  also the honest one.
- **Cost:** a new `WatAST::RationalLit` fans out to ~5 mechanical one-line match arms
  (span/hash/constructors/type-string) + one eval arm. Worth it to stay decision-free.

*(Deliberate divergence from the NEWEST scalar precedent (Char, Arc 220, which desugars). Rationals follow
the OLDER numeric-literal precedent instead — grounded in decision-freeness, ratified in this design.)*

## Rooms (verified file:line this session)

| room | file:line | what |
|---|---|---|
| root deps | `Cargo.toml:90` | add `num-rational.workspace = true` + `num-bigint.workspace = true` beside `uuid.workspace = true` |
| lexer | `crates/wat-reader/src/lexer.rs:849-851` | rational branch in `lex_numeric_or_symbol` (raw already = "1/2"); new `Token::Rational(BigRational)` |
| ast | `crates/wat-reader/src/ast.rs:58` | `WatAST::RationalLit(BigRational, Span)`; fan-out span()~147, constructors~165, type-string~413, Hash~460 |
| parser | `crates/wat-reader/src/parser.rs:335` | `Token::Rational(r) => WatAST::RationalLit(r, span)` |
| runtime value | `src/value/value.rs:309` | `wat__core__Rational(Box<BigRational>)`; eq:591, hash:740, type_name:1134 ("wat::core::Rational"), type-string:1225 |
| eval arm | (IntLit's eval neighbor — find in runtime) | `RationalLit => Value::wat__core__Rational(...)` |
| edn→value | `src/edn_shim.rs:1240` | `Edn::Rational(r) => Value::wat__core__Rational(...)` |
| shape name | `src/edn_shim.rs:1334` | `Edn::Rational(_) => "Rational"` |
| edn coercion | `src/edn_shim.rs:1476/1484` template | `":wat::core::Rational" => match edn { Edn::Rational(r) => ... }` |
| edn→watast | `src/wat_edn_bridge.rs:134` | `Edn::Rational(r) => WatAST::RationalLit(...)` |
| render | `src/value/observe.rs:390` | `wat__core__Rational(r) => format!("{}/{}", r.numer(), r.denom())` |
| type path | `src/types.rs:2934` / `src/check.rs` | recognize `:wat::core::Rational` as a valid scalar Path type (scalars are `TypeExpr::Path`, no `register_builtin`) |

## Escape (return to green — the fight, per 299's doctrine)

1. `cargo build -p wat` → 0 errors (the 3 E0004 gone).
2. `cargo test -p wat-edn` still green (Stone A untouched); the new Stone B probe green.
3. Whole workspace weighed (`cargo nextest run`, read the Summary line — not a grepped fraction).
4. **A + B commit atomically** — one commit, green workspace, never broken in history.

## Out of scope = REJECTED (→ Stone C)

- All arithmetic (`Rational::+ - * /`, comparison, `to-f64`).
- Any `Rational/of` runtime constructor.
- Runtime BigInt (the `(+ 1/2 1/2) → 1N` collapse target).

## STOP triggers (halt + report; never improvise)

- STOP if `4/2` in wat SOURCE does not become a runtime **Integer** (must mirror Stone A / clj Long).
- STOP if `1/0` in source panics rather than a clean lex error.
- STOP if turning a `RationalLit` green requires touching an **arithmetic or comparison** operator — that
  is Stone C; a rational literal that *evaluates and renders* is the whole of B.
- STOP if a literal numerator/denominator exceeds `i64` after reduction (a big-literal den==1 case that
  can't be an `i64` Integer) — report it; runtime BigInt is out of scope, so this edge is C's, not a
  workaround to invent here.
- STOP if `WatAST::RationalLit` cascades into far more match sites than the ~6 mapped — report; do not force.
