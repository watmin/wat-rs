# SCORE — the oracle's `rule-negates` now recurses through `and`/`or`

Executing strike per `DESIGN.md`. Written as-I-go per house rule; the probe landed before any
edit to the oracle.

## ⛔ THE PROBE, FIRST — before any edit

DESIGN's open question: `where-nested-combinators.wat` q6/q7 (`(or A (and B (not C)))`,
`(and (not A) (not B))`) are `defquery`, not `defrule`, and stratification orders *producers* —
a query has no `:then`. Whether a query's own stratum is computed or read at all was **not**
established, so whether the oracle/engine divergence is reachable **for a rule** was unproven.

**Built a `defrule`, not a query, of exactly DESIGN's shape**, with `C` DERIVED (produced by
another rule, `mkc`), and added it to `wat-scripts/scratch-pad/arc278-l2-3-stratify-numbers.wat`
(the `:l24::*` section) plus a new Rust test,
`native_stratify_numbers_nested_or_and_not_against_the_oracle`
(`src/rete/kernel/tests/stratify_numbers.rs`), driving both engines in one process the same way
the pre-existing `native_stratify_numbers_against_the_oracle_scratch` does:

```wat
(:wat::core::defrecord :l24::A    [k <- :wat::core::i64])
(:wat::core::defrecord :l24::B    [k <- :wat::core::i64])
(:wat::core::defrecord :l24::C    [k <- :wat::core::i64])
(:wat::core::defrecord :l24::Seed [k <- :wat::core::i64])
(:wat::core::defrecord :l24::Out  [k <- :wat::core::i64])

;; C is DERIVED
(:wat::rete::defrule :l24::mkc
  :when [(:l24::Seed (?k <- :k))]
  :then [(:l24::C :k ?k)])

;; (or A (and B (not C))) — DESIGN's shape, verbatim, as a RULE
(:wat::rete::defrule :l24::nested
  :when [(:wat::rete::or (:l24::A) (:wat::rete::and (:l24::B) (:wat::rete::not (:l24::C))))]
  :then [(:l24::Out :k 1)])
```

Ran `cargo test --release native_stratify_numbers_nested_or_and_not_against_the_oracle --
--nocapture` — **before touching the oracle at all.** Verbatim output:

```
running 1 test
NATIVE NESTED keys (raw): ["l24::Out" 1]
ORACLE NESTED keys (raw): []

thread 'rete::kernel::tests::stratify_numbers::native_stratify_numbers_nested_or_and_not_against_the_oracle' (2476764) panicked at src/rete/kernel/tests/stratify_numbers.rs:241:5:
assertion `left == right` failed: NESTED: oracle must ALSO raise Out to 1 once rule-negates recurses through Or/And — if this is None the oracle still only recognises a top-level `:not` head
  oracle NESTED = {}
  left: None
 right: Some(1)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test rete::kernel::tests::stratify_numbers::native_stratify_numbers_nested_or_and_not_against_the_oracle ... FAILED

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1235 filtered out; finished in 0.35s
```

**Reading: the numbers DIFFER.** Native raises `l24::Out` to stratum 1 — it recurses through
`Or`/`And` unconditionally via `negate_types`, sees the `not` on the derived `C`, and requires
`Out`'s stratum to exceed `C`'s producer `mkc`'s stratum. The oracle's map is **empty**:
`l24::Out` is never raised, i.e. `rule-negates` never saw the nested `not` at all.

**⛔ Divergence confirmed live and reachable for a RULE. Proceeding to the cure**, per DESIGN's
pinned instruction.

### A correction to DESIGN's own framing, found while re-deriving it

DESIGN describes the oracle as folding over LHS forms and checking `hd == ":wat::rete::not"`
with no recursion machinery at all. Reading `wat/rete/oracle/stratify.wat:108-160` directly
(pre-cure), the recursion helper — `negated-types-under`, which already walks `:and`/`:or`
(collecting from every branch) and `:not` (recursing into its child) correctly — **already
existed in the file**. It was just never reached for a top-level `:or`/`:and`: the OUTER driver,
`rule-negates`'s own `foldl` over top-level LHS forms, called into it only when a top-level
form's own head was exactly `:wat::rete::not`. So the defect is narrower than "no recursion
exists" — it's "the existing recursion is gated behind a check that only fires at the top level."
This changed the shape of the fix (see below): reuse `negated-types-under`, don't reimplement it.

## The cure

**Fixed the ORACLE, not the engine**, per DESIGN's pin — `src/rete/kernel/stratify.rs`'s
`negate_types` is untouched, byte-for-byte.

### First attempt (naive delegate) — built, driven, and REJECTED before shipping

My first idea was the smallest possible patch: make `rule-negates`'s outer fold call
`negated-types-under` directly on every top-level form (dropping the `hd == :not` gate
entirely), since that function already walks `:and`/`:or`/`:not`. Implemented it, rebuilt
(`wat/` is `include_str!`'d — rebuilt every time before testing), and ran BOTH existing
stratify-numbers tests before deciding anything:

```
running 2 tests
NATIVE NESTED keys (raw): ["l24::Out" 1]
ORACLE NESTED keys (raw): ["l24::C" 1, "l24::Out" 2]

thread 'rete::kernel::tests::stratify_numbers::native_stratify_numbers_nested_or_and_not_against_the_oracle' (2530774) panicked at src/rete/kernel/tests/stratify_numbers.rs:241:5:
assertion `left == right` failed: NESTED: oracle must ALSO raise Out to 1 once rule-negates recurses through Or/And — if this is None the oracle still only recognises a top-level `:not` head
  oracle NESTED = {"l24::C" 1, "l24::Out" 2}
  left: Some(2)
 right: Some(1)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
NATIVE ANCHOR keys (raw): ["l23::Ok2" 1]
ORACLE ANCHOR keys (raw): ["l23::Ok" 1, "l23::Ok2" 2]
NATIVE MEASURE keys (raw): ["l23::Tally" 1]
ORACLE MEASURE keys (raw): ["l23::Ok" 1, "l23::Tally" 1]
bag exists_and_from_types: [[], ["l23::Ok"]]

thread 'rete::kernel::tests::stratify_numbers::native_stratify_numbers_against_the_oracle_scratch' (2530773) panicked at src/rete/kernel/tests/stratify_numbers.rs:167:5:
assertion `left == right` failed: ANCHOR: both engines must raise Ok2 to 1 — the term they share
  native ANCHOR = {"l23::Ok2" 1}
  oracle ANCHOR = {"l23::Ok" 1, "l23::Ok2" 2}
  left: (Some(1), Some(2))
 right: (Some(1), Some(1))

test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 1234 filtered out; finished in 0.36s
```

**Both tests went red, and the ANCHOR failure shows the defect precisely**: `l23::ok`'s LHS is a
bare positive `(:l23::A (?k <- :k))` with no `:not` anywhere. `negated-types-under`, called
directly on it, falls through its `:and`/`:or`/`:not` checks to `type-name-of`, which returns
`Some("l23::A")` for ANY fact-shaped form regardless of whether it's under a negation —
`negated-types-under` was written to be called ONLY from inside a `:not`'s arm, where every leaf
it reaches genuinely is negated by construction. Called on a plain positive top-level form, it
falsely reports that form's type as negated. Concretely: `l23::ok`'s own `A` read got reported as
"negates A," raising `Ok`'s own stratum to 1 (a rule with no negation in it at all), which then
cascaded into `l23::neg`'s `Ok2` reaching stratum 2 instead of 1. **This naive delegate makes
every plain positive top-level read look like a negation** — strictly worse than the bug it was
meant to fix. Discarded before shipping; not committed as an intermediate state.

### Shipped fix — thread an explicit under-not flag, mirroring `negate_types`

Added a new function, `rule-negates-in`, that walks a form the way native's
`negate_types(form, out, under_not)` does: `:and`/`:or` recurse **unconditionally** (looking for
a nested `:not`, contributing nothing themselves), and only a `:not` arm calls into
`negated-types-under` — which is exactly its existing, correct behaviour, since everything it
reaches from there really is under a negation. `rule-negates`'s outer fold now delegates to
`rule-negates-in` for every top-level LHS form instead of its old inline `hd == :not` check:

```wat
;; rule-negates-in — walk one LHS form looking for a :not, RECURSING through :and/:or at
;; every depth (not only once already inside a :not). Mirrors native `negate_types(form, out,
;; under_not)`: :and/:or recurse unconditionally; a bare positive leaf (not :and/:or/:not)
;; contributes NOTHING here — only `negated-types-under`, reached once a :not is found, may
;; turn a leaf into a negated type.
(:wat::core::defn :wat::rete::rule-negates-in
  [form <- :wat::WatAST] -> (:wat::core::PersistentVector :- [:wat::core::String])
  (:wat::core::let [ch (:wat::core::ast->children form)
                    hd (:wat::core::if (:wat::core::empty? ch)
                         ""
                         (:wat::core::ast-name (:wat::core::first ch)))]
    (:wat::core::if (:wat::core::if (:wat::core::= hd ":wat::rete::and")
                      true
                      (:wat::core::= hd ":wat::rete::or"))
      (:wat::core::foldl
        (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::String])
                         kid <- :wat::WatAST]
          -> (:wat::core::PersistentVector :- [:wat::core::String])
          (:wat::core::foldl
            (:wat::core::fn [a <- (:wat::core::PersistentVector :- [:wat::core::String])
                             t <- :wat::core::String]
              -> (:wat::core::PersistentVector :- [:wat::core::String])
              (:wat::core::PersistentVector/conj a t))
            acc
            (:wat::rete::rule-negates-in kid)))
        (:wat::core::PersistentVector)
        (:wat::core::rest ch))
      (:wat::core::if (:wat::core::= hd ":wat::rete::not")
        (:wat::rete::negated-types-under (:wat::core::second ch))
        (:wat::core::PersistentVector)))))

;; rule-negates — :not of a fact AND :not of :and/:or, reachable from ANY position in the LHS
;; list — not only a top-level form whose OWN head is :not. Delegates to `rule-negates-in`.
(:wat::core::defn :wat::rete::rule-negates
  [rule <- :wat::rete::Rule]
  -> (:wat::core::PersistentVector :- [:wat::core::String])
  (:wat::core::let [lhs (:wat::rete::Rule/lhs rule)]
    (:wat::core::foldl
      (:wat::core::fn [acc  <- (:wat::core::PersistentVector :- [:wat::core::String])
                       form <- :wat::WatAST]
        -> (:wat::core::PersistentVector :- [:wat::core::String])
        (:wat::core::foldl
          (:wat::core::fn [a <- (:wat::core::PersistentVector :- [:wat::core::String])
                           t <- :wat::core::String]
            -> (:wat::core::PersistentVector :- [:wat::core::String])
            (:wat::core::PersistentVector/conj a t))
          acc
          (:wat::rete::rule-negates-in form)))
      (:wat::core::PersistentVector)
      lhs)))
```

`negated-types-under` itself is **untouched** — it is now called from exactly the same kind of
position it always was (inside a confirmed `:not`), just reached via one more level of `:and`/
`:or` recursion above it.

This is a **hand-edit of two functions (one existing, one new) in one file**
(`wat/rete/oracle/stratify.wat`). `wat-rs/CLAUDE.md`'s codemod doctrine covers a *structural
rewrite across many `.wat` files* (a rename, a record→enum migration, a form flip); this is a
single-site logic change to one function plus a new sibling helper in one file, not a corpus
rewrite, so `wat-fix` does not apply — checked against `wat-rs/CLAUDE.md` before deciding this,
not assumed.

**Rebuilt before testing** (`cargo build --release`; `wat/` is `include_str!`'d — a stale binary
answers from the old stdlib with no error) then reran both stratify-numbers tests:

```
running 2 tests
NATIVE ANCHOR keys (raw): ["l23::Ok2" 1]
ORACLE ANCHOR keys (raw): ["l23::Ok2" 1]
NATIVE MEASURE keys (raw): ["l23::Tally" 1]
ORACLE MEASURE keys (raw): []
bag exists_and_from_types: [[], ["l23::Ok"]]
NATIVE NESTED keys (raw): ["l24::Out" 1]
ORACLE NESTED keys (raw): ["l24::Out" 1]
test rete::kernel::tests::stratify_numbers::native_stratify_numbers_against_the_oracle_scratch ... ok
test rete::kernel::tests::stratify_numbers::native_stratify_numbers_nested_or_and_not_against_the_oracle ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1234 filtered out; finished in 0.35s
```

Both engines now raise `l24::Out` to stratum 1 — the NESTED divergence is closed. The
pre-existing ANCHOR/MEASURE pair is unchanged (`l23::Ok2` agrees at 1 on both sides, same as
before the cure) — the fix did not regress the plain-positive-read case the naive attempt broke.
`l23::Tally` (MEASURE) still diverges: that is the SEPARATE, already-known, already-gated
`exists_and_from_types` divergence documented in this same file's header and
`stratify.rs`'s header — untouched, out of scope for this strike.

## Mutation proof

Mutated the shipped `rule-negates-in`'s `:and`/`:or` detection to a hardcoded `false` (i.e.
"never recurse into a combinator" — the pre-cure behaviour, reached through the new function
instead of the old one):

```wat
    (:wat::core::if (:wat::core::if false
                      true
                      false)
```

Rebuilt, reran both tests:

```
running 2 tests
NATIVE NESTED keys (raw): ["l24::Out" 1]
ORACLE NESTED keys (raw): []

thread 'rete::kernel::tests::stratify_numbers::native_stratify_numbers_nested_or_and_not_against_the_oracle' (2610813) panicked at src/rete/kernel/tests/stratify_numbers.rs:241:5:
assertion `left == right` failed: NESTED: oracle must ALSO raise Out to 1 once rule-negates recurses through Or/And — if this is None the oracle still only recognises a top-level `:not` head
  oracle NESTED = {}
  left: None
 right: Some(1)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test rete::kernel::tests::stratify_numbers::native_stratify_numbers_nested_or_and_not_against_the_oracle ... FAILED
NATIVE ANCHOR keys (raw): ["l23::Ok2" 1]
ORACLE ANCHOR keys (raw): ["l23::Ok2" 1]
NATIVE MEASURE keys (raw): ["l23::Tally" 1]
ORACLE MEASURE keys (raw): []
bag exists_and_from_types: [[], ["l23::Ok"]]
test rete::kernel::tests::stratify_numbers::native_stratify_numbers_against_the_oracle_scratch ... ok

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 1234 filtered out; finished in 0.35s
```

**RED**, at exactly the new NESTED test's assertion (`oracle_nested.get("l24::Out")` expected
`Some(&1)`, got `None`) — the ANCHOR test stays green under this mutation, confirming the
mutation isolates the `:and`/`:or`-recursion arm specifically rather than breaking something
else. Restored the condition (`(:wat::core::= hd ":wat::rete::and") … (:wat::core::= hd
":wat::rete::or")`), rebuilt, reran — both tests green again (verbatim output above, "Rebuilt
before testing").

## What the stratum differential test gained

Per DESIGN's item 2: added `native_stratify_numbers_nested_or_and_not_against_the_oracle` to
`src/rete/kernel/tests/stratify_numbers.rs`, driving a THIRD rule pair (`:l24::mkc`,
`:l24::nested`) sourced from the same scratch `.wat` file
(`wat-scripts/scratch-pad/arc278-l2-3-stratify-numbers.wat`, `:l24::*` addition) rather than a
Rust-only literal fixture — growing the one shared scratch file rather than hand-writing a third
AST inline in Rust, per DESIGN's "better if it reads real fixtures." It is now a standing
regression gate on the FIXED, post-cure behaviour (both assertions require `Some(&1)`) rather
than the disconfirming-or-confirming probe it started as; the mutation proof above shows it
reddens if `rule-negates-in`'s combinator recursion regresses.

## Floor

`pgrep -af 'cargo|nextest'` — nothing running before the floor started. Ran `./scripts/floor.sh`
in the foreground (no backgrounding), one full run, no `head`/`tail` piping on the run itself.
Completed on the first run — **no red, so nothing to capture-and-not-rerun.** Read the `Summary`
line directly from `.floor/latest/clean.log`:

```
Summary [ 454.857s] 5489 tests run: 5489 passed (2 slow), 19 skipped
```

**5489** = baseline **5488** + the **1** new test
(`native_stratify_numbers_nested_or_and_not_against_the_oracle`). Zero failures, 19 skipped
(unchanged from baseline).

## What I did NOT do, and why

- **Did not touch `negate_types` / the engine** (`src/rete/kernel/stratify.rs`) — DESIGN pins the
  engine as correct and the oracle as the side to fix; both the probe and the cure leave
  `negate_types` byte-for-byte unchanged.
- **Did not make the grid compare stratum numbers.** DESIGN explicitly rejects this as a separate
  strike — the grid's contract stays "facts," per `stratify.rs:29-30`'s own words.
- **Did not touch `3P1`'s axis-pair coverage** — DESIGN names this as the same class, its own
  row, out of scope here.
- **Did not resolve whether a `defquery`'s own stratum is computed or read anywhere.** The probe
  answers the narrower, DESIGN-posed question ("is the divergence reachable for a RULE") by
  direct construction; it does not drive `q6`/`q7` themselves through `compile`/`fire-rules`, and
  DESIGN did not ask for that either. If a query's own stratification path exists and is
  separately wrong, that is untouched and unexamined here.
- **Did not fix the pre-existing, already-known `exists_and_from_types` (`l23::Tally`) MEASURE
  divergence** documented in this same scratch file and in `stratify.rs`'s own header —
  reconfirmed it is still present post-cure (`ORACLE MEASURE keys (raw): []` vs. native's
  `["l23::Tally" 1]`, shown above) and left it exactly as it was; it is a different,
  already-tracked divergence, not this strike's target.
- **Did not use the `wat-fix` codemod** for this change — it is a hand-edit of one existing
  function plus one new sibling helper in a single `.wat` file, not a structural rewrite across
  the corpus, so the codemod doctrine in `wat-rs/CLAUDE.md` does not apply (checked, not
  assumed).
- **Did not commit the rejected naive-delegate attempt** as an intermediate commit — it lived
  only in the working tree during the self-check described above and was discarded before the
  shipped fix was written; its output is preserved in this SCORE for the record, per house rule
  that a caught false start is worth recording, not the diff itself.
