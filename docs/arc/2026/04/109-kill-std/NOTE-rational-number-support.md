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

- A **rational value type** in the value system (a `Ratio { num, den }`, reduced to lowest terms, sign on
  the numerator — clojure's normal form).
- **Reader:** parse `<int>/<int>` as a Ratio in both `wat-edn` and (via the 300 convergence) `wat-reader`;
  match `clojure.edn`'s grammar (`1/2`, `-3/4`, denominator ≠ 0, no `0` leading — reuse the integer
  rules per part).
- **Writer / printing:** round-trip `Ratio` back to `n/d` so `wat-edn` write == clj read.
- **Arithmetic / typing:** the language-level surface (a `wat.core` rational type + ops) is the larger
  half and can follow the reader/value work; the reader-parity piece is what closes the `1/2` differential.

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
