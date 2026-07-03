# NOTE (arc 109 vocabulary) — rational number support (EDN/clj parity)

**Filed 2026-07-03. A POINTER, not a decision.** Queue marker for a clj-parity gap surfaced by the
reader-parity work (300 `reader-unicode-parity`, commit `dd5ae864`). Records the gap, the grounded
current state, the direction, and the one nuance the deciding strike must weigh. To be resolved before
arc 109 (the clojure-ination) closes.

## The gap

`clojure.edn` reads `1/2` as a **Ratio**. wat has **no rational value type** yet, so wat's readers
refuse it. Builder direction (2026-07-03): *"we do not have rational support in wat yet — something
we'll work on later."* This note keeps it from being lost.

## Grounded current state (2026-07-03)

Standing up the **clj-oracle differential** for the reader (clj is the oracle; non-parity is an illegal
state), a 24-input corpus through both `clojure.edn` and `wat-edn` found two divergence classes:

- **Unicode tokens** (`😀`/`é`/`λ`/`foo→bar` symbols, `:a😀`/`:λ` keywords) — clj accepts, wat refused.
  **Fixed** this session (`dd5ae864`): `wat-edn` now accepts them (char-aware token lexing); `wat-reader`
  stops panicking. wat == clj on all of these.
- **Ratios** (`1/2`) — clj accepts (`Ratio`), wat refuses. **The sole remaining divergence** after the
  Unicode fix landed — deferred here, because it is not a lexer patch: it needs a whole value type.

So `1/2` is the one known, tracked exemption in the parity ward: everything else in the corpus matches
clj; ratios are owed.

## The nuance the deciding strike must weigh

Ratios are a **`clojure.edn` (the impl) feature, not an EDN-spec (the doc) feature.** The EDN spec
(`crates/wat-edn/docs/EDN-SPEC.md`) defines only **integers** and **floating-point numbers** — no
rationals. So the deciding call is explicitly: *do we chase full `clojure.edn` parity (add rationals) or
EDN-spec parity (rationals out of scope)?* The builder's stance is that **clj is the oracle** → add
rational support. Naming the tension here so the strike is honest about which reference it's serving.

## The direction (for the deciding strike, not locked here)

**The pivotal design fact (grounded 2026-07-03): wat has NO numeric tower.** `src/runtime.rs:4266` —
*"Integer arithmetic — strict i64. No promotion from f64."* Arithmetic is **type-locked** (`i64::+`,
`f64::+`, explicit `i64::to-f64` to cross) — the opposite of Clojure's auto-promoting int/ratio/float
tower. So in wat, **rationals are a self-contained strict type with their own ops**, not a tower
integration. That bounds the work; it also means we deliberately diverge from Clojure's *arithmetic*
semantics (parity is on the *reader*, not the tower).

The work splits into two separable layers:

**Layer 1 — EDN data (`crates/wat-edn`), the small half — this ALONE closes the ward's `1/2` exemption.**
`wat_edn::Value` already carries `Integer(i64)` / `BigInt(Box<BigInt>)` / `Float(f64)` / `BigDecimal`, and
the crate already deps `num_bigint`.
- A `Value::Rational` variant — cleanest as `num_rational::BigRational` (GCD-reduce/normalize for free;
  matches Clojure's BigInteger-backed Ratios).
- Lexer: recognize `<int>/<int>` as ONE token (today `1` lexes as a number and `/` starts a new token) →
  build + normalize (reduce to lowest terms, sign on numerator, denominator > 0, `d/1` → Integer,
  `0/n` → 0 — Clojure's normal form).
- Writer + equality/hash. This is `wat-edn` *reading* rationals as EDN data — moderate, self-contained,
  no runtime/type-system changes.

**Layer 2 — the language (`src/runtime.rs` + type system + `wat/core.wat` + `wat-reader`), the bigger half
— needed for wat PROGRAMS to compute with rationals.** The runtime's numeric values are `i64`/`f64`/`u8`
(note: **no `BigInt` in the runtime** — only in the EDN data layer). So:
- A runtime `Value::Rational` + a `:wat::core::Rational` type in the type system.
- Type-locked ops (`Rational::+ - * /`, comparison, `Rational::to-f64`, `i64::to-rational`) — strict/
  explicit, matching wat's i64/f64 discipline; **not** auto-promotion.
- `wat-reader` (source) lexing `1/2` → a rational literal in `WatAST`; Display/printing.

**The two gating calls:**
1. **Backing integer type.** Clojure Ratios are arbitrary-precision (BigInteger). Layer 1 can be
   `BigRational` trivially (dep exists). The *language* layer has no BigInt today — so a language Rational
   is either `i64`-bounded (simple, can overflow) or arrives with BigInt runtime support (bigger, the
   honest Clojure match). **This is the real fork.**
2. **Scope.** Ward-green / EDN-data parity = **Layer 1 alone** (no runtime/type changes). Rationals
   *usable in wat code* = Layer 1 + Layer 2 (where the BigInt question + type-system work live).

## Why deferred (not scaffolding-about-to-be-deleted)

Unlike some 109 notes, this is not deletable scaffolding — it is *missing* capability. It waits only on
appetite/sequencing, not on a representation decision that would rework it. The clj-oracle differential
ward should carry `1/2` as a **marked exemption** until this lands, then flip it to a required-pass row.

## Refs

- `docs/arc/2026/07/300-wat-source-is-edn/BRIEF-STONE-reader-unicode-parity.md` — the parity strike that
  surfaced this; `dd5ae864` (Unicode-token parity landed, ratio deferred).
- `crates/wat-edn/docs/EDN-SPEC.md` — the spec (integers + floats only; no rationals) — the doc/impl gap.
- `crates/wat-edn/src/lexer.rs` (number lexing), `crates/wat-edn/src/value.rs` (the value type to extend).
- The clj-oracle differential ward (to be built as the parity net) — `1/2` is its one standing exemption.
