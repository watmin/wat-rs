# SCORE — the fifth false claim was this strike's own headline, twice over

> **Written after the orchestrator's own re-run.** Rows marked *re-driven* were run on this machine
> at HEAD `b3dee4619` + the strike.

## The scorecard, graded

| # | required | result |
|---|---|---|
| 1 ★ | a live accessor resolves | ✅ **258/258, full `wat::lint` binary, probe present** — re-driven by the orchestrator |
| 2 ★ | a typo'd accessor still fails | ✅ **re-driven**: `zz-c15-probe.wat:3 :wat::rete::DerivationNode/vai` REDs |
| 3 ★ | the new source cannot silently empty | ✅ floor added; mutation 3 REDs **with the probe deleted**, so nothing in the corpus could red it |
| 4 | the population, with an anchor | ✅ **25 records / 80 accessors** — and the anchors are now **assertions** pinning field names *in order*, not a count |
| 5 | zero runes minted for live names | ✅ **zero runes of any kind** |
| 6 | no allowlist | ✅ the declaration is parsed |
| 7 | corpus unchanged otherwise | ✅ exactly one name newly resolves — the probe's. None newly fails |
| 8 | blast radius | ✅ one file; `git diff --stat -- src/ wat/` = **0 lines** |
| 9 | floor / lints / clippy | ✅ **`5407 tests run: 5407 passed, 21 skipped`** (439.6 s), **0 FAIL rows**, lints **258** (+4), clippy rc=0 |
| 10 | the probe does not survive | ✅ `find . -name 'zz-c15-probe*'` → 0 |

## ⛔⛔ THE ★ WAS THIS DESIGN'S HEADLINE, WRONG IN TWO INDEPENDENT WAYS

> *"19 `defrecord`s in `wat/rete.wat`, 66 synthesized accessors — every one of them unwritable in
> `wat-scripts/` today."*

**1. The scope was too narrow, and the pinned contract encoded the error.** I pinned *"`<Type>` is an
aggregate declared in **`wat/rete.wat`**"*. Six more `:wat::rete::` records live elsewhere —
`CompileState`, `MintResult`, `CondFoldAcc`, `AxisViolation` (`wat/rete/compile.wat`), `FireStratAcc`
(`wat/rete/oracle/fire.wat`), `StratifyAcc` (`wat/rete/oracle/stratify.wat`). Verified by the
orchestrator. **Implementing my pin literally would have left 14 accessors blocked by the exact
defect this strike was drawn to cure**, and the next hand would have rediscovered C15 in a different
file. The rider scanned all of `wat/` and pinned `CompileState` in an anchor test so the multi-file
scope cannot silently narrow.

**2. "Every one of them unwritable" is FALSE.** Measured on the final code: **30 of 80** are attested
nowhere; the other **50 already resolved by accident**, because the stdlib happens to call them. Of
the 66 in `wat/rete.wat`, **28 were blocked and 38 were already writable.** Spot-checked by the
orchestrator both ways: `:wat::rete::Rule/rhs` has three attesting sites
(`oracle/explain.wat:25`, `compile.wat:1084`, `oracle/stratify.wat:54`); `DerivationNode`'s three
accessors have **zero**.

C15's real shape is **"some accessors resolve and some do not, depending on whether the stdlib
happens to call them"** — harder to reason about than "all blocked", and now recorded in the gate's
header with the measurement.

### And that correction is load-bearing for the EVIDENCE, not just the prose

Because `accessors ⊄ attested` — 30 names sit outside — the new source **can fail loudly on its own**.
Had the overlap been total, **mutation 3 would have been unfalsifiable through the resolution arm**:
emptying the record source would have changed no verdict, because attestation already covered every
name. That is `[[a-resolver-whose-halves-overlap-proves-nothing]]`, which the file's own header
applies to its first two halves — and the rider applied it to its own cure.

## ⛔ AND MY BLAST RADIUS WAS WRONG IN A WAY MY OWN PROBE WOULD HAVE HIDDEN

The BRIEF says *"`tests/lint/rete_names_in_wat_scripts_resolve.rs` only."* True of the file — but it
lands in a gated tree, and the natural implementation reddens **three other lint gates neither
artifact mentions**: `no_inlined_wat_in_tests` (a `defrecord` string literal reads as an inlined
world), `no_loose_string_assert` (5 sites), `no_inlined_edn` (a `[a <- …]` literal opens with `[`,
5 sites).

**A rider running only the scoped filter `probe-c15.wat.txt`'s own header prescribes would have
handed back a red floor.** That is why the orchestrator's row-1 re-drive used the **full lint
binary**, not the scoped test.

All three were cured **without a rune**: the parser split so `declared_field_names` takes the
vector's *inside*; the accessor claim became an exact `BTreeSet` equality over `DerivationNode`'s
three accessors — which asserts `via` is in **and** that `vai` and the bare type name are not, a
thing a `contains` pair cannot say; and the generic-binder rule anchored on a **real** declaration
(`wat/gen.wat`'s `:wat::gen::Pick :- [T]`) instead of a synthetic form.

## Mutations — two re-driven by the orchestrator

- **1 — cure removed** (file hash `8c0165da…` → `2b7b2aed…`), probe present → RED at
  `zz-c15-probe.wat:4 :wat::rete::DerivationNode/via`.
- **2 ★ — typo'd accessor** → **re-driven**: RED, *"not attested … and not a field accessor minted by
  any `:wat::rete::` record declared there"*. **The field half refuses**, which is what separates a
  cure from a slash-shaped hole.
- **3 — source silently emptied** (`RECORD_DECL_FORMS` repointed at the retired
  `:wat::core::record-def` spelling), **probe deleted first** → 2 tests RED: the floor fires with
  *"the record parse went blind: 0 `:wat::rete::` record(s) read from `wat/` minting 0 accessor(s)"*,
  and the anchor test fires on `DerivationNode`. **The invisible re-blockage is caught with no
  corpus consumer** — which is the whole point of row 3.

## What the rider did that is worth copying

- **It caught its own false claim before hand-back.** It wrote, twice, that `:wat::gen::Pick` is the
  tree's only aggregate with both a type binder and a nested last field. `wat/cache.wat:192` is
  another. It grepped for its own phrase, found the second copy *at a line it did not remember
  writing*, and struck both. Same shape as the breadcrumb stamp that went stale for four commits:
  **a claim you wrote is the one you will not re-read.**
- **Its first expected-error string was wrong** — `[a :wat::core::i64]` is 2 cells, so it hits the
  cardinality arm, not the arrow arm. The test reddened and told it; both arms plus the empty case
  are now asserted separately.
- **It stopped an in-flight floor** (green so far, **not** a red) on finding the false doc claim, and
  re-ran from scratch on the final file. No red dismissed, no re-run after a failure.

## Boundaries, stated rather than assumed

Only `defrecord` — enum variants are `Type::Variant`, no slash, already attested. No `:wat::rete::`
aggregate is declared in Rust, via bare `recordtype`, or with a `~@:Surface` splice; all three were
checked before trusting a textual parse. Only `:wat::rete::` — `wat/query.wat` mints its accessors
identically, and the anchor test pins `:wat::query::Row` **out** of the set, so the namespace cut is
deliberate rather than incidental.

## A dated header in my own artifact

`probe-c15.wat.txt` is stamped `b5c068ebd`, which is not this strike's HEAD (`b3dee4619`). It
reproduced, so no harm — but it is the self-certifying-date shape `wat-rs/CLAUDE.md`'s preamble
warns about, in the orchestrator's own probe. Corrected.
