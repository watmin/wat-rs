;; wat/gen.wat — FINITE GENERATORS: the core of generative testing, in wat.
;;
;; ⛔ UNDER AUDIT. An 18-ward vigilia (2026-08-25) plus `circumspicere` (2026-08-26) found
;; defects in this file that can compute a WRONG ANSWER, and several claims in the comments
;; below are known FALSE. Read
;; `docs/arc/2026/06/278-rules-engine/GEN-VIGILIA-2026-08-25.md` before trusting anything here
;; or changing anything.
;;
;; ⚠ ENTRIES NAME THE VERB, NEVER A LINE NUMBER. The first version of this banner cited
;; `:53`, `:150`, `:311` and six error-string lines — and EVERY ONE of those numbers was wrong
;; within a day, because fixing the entries above them moved the lines below. A retraction
;; notice that rots on each edit is the same hand-maintained-number defect this file's own test
;; suite was restructured to eliminate (wat-tests/gen.wat's header). Cite the verb; grep finds it.
;;
;; FIXED 2026-08-26
;;   ✓ negative `card` (findings A + B) — `:wat::gen::gen` is now the only constructor any verb
;;     uses and it floors card at 0, so `check` can no longer report a pass over a negative
;;     denominator and `one-of`/`bind` dispatch can no longer run its offset backwards. Gated by
;;     `law-no-negative-card` (L24), which drives the PRODUCERS, not the constructor.
;;   ✓ the "~23us/point / ~300x cheaper / never the bottleneck" claim — deleted, not corrected.
;;     See SHIPPED NUMBERS below for why a ratio against the `$oracle` cannot be repaired.
;;   ✓ the two gates that could not go red (finding D). `test-shrink-index` was passed by an
;;     IDENTITY `shrink-index` — it now pins the SEARCH, and the identity fails it. And `check`
;;     had NO gate on its own enumeration: mutating it to `(range 1 card)`, skipping the first
;;     point of every space in the library, left all 23 laws green. `law-check-not-vacuous`
;;     (L25) pins points/witness against literals the TEST states, and catches it.
;;
;;   ✓ `record` re-evaluated each generator ARGUMENT once per generated point (finding F), not
;;     twice as its comment claimed. The expansion now binds every argument once, via the
;;     `fresh-symbol` the macro already carried. Measured ~16x on an 800-point space; the
;;     multiple scales with the space, so it is not quoted as a constant.
;;   ✓ `lift2`/`lift3` hand-encoded mixed radix a SECOND time and disagreed with the
;;     `record`/`coords` path past the card (finding C). Both are now expressed over `coords`;
;;     there is exactly one mixed-radix encoding in this file. L10 — written as the tripwire for
;;     this exact drift, and which missed it by stopping one index short — is widened past the
;;     card boundary and now kills a re-introduced hand encoding.
;;   ✓ `digit` was a hand-rolled `i - (i/base)*base` justified by "No native i64 mod/rem", false
;;     since 2026-07-05 (finding G). `digit` IS `:wat::core::i64::rem`, verified equal on
;;     (0,3) (7,3) (8,4) (1234,10) before the swap.
;;   ✓ five `raise` strings naming retired `gen-` verbs, renamed to the shipping names
;;     (finding K). ⚠ THE ROOT IS STILL OPEN — see below.
;;   ✓ the containment claim (finding E) — the false sentence is gone and the real behaviour is
;;     stated at the `Gen` defstruct. The SUBSTRATE half is diagnosed with a proven patch and
;;     handed to arc 293; until it lands, nothing stops a Gen crossing a boundary.
;;   ✓ every aggregate here that holds only pure data is now a `defrecord` (GenAcc, CheckAcc,
;;     GenRev, Pick). A record is FORCED pure; a struct merely may be. `Gen` is the one
;;     struct, and it earns it by carrying a function. (`BindPick` was a sixth until `bind`
;;     collapsed onto `one-of` and it became unreachable — it is deleted, not converted.)
;;
;; STILL FALSE / STILL OPEN
;;   ✗ NOTHING GATES THE NAMES ABOVE (circumspicere finding 4). `tests/lint/retired_name_justified.rs`
;;     exists to stop exactly this — "a wat name in a Rust string must be a name a user can type" —
;;     but it scans `src/**/*.rs` ONLY and matches only the prime-suffix shape, so it is
;;     structurally blind to `.wat`. The five renames above are STEMS; the root is that the stdlib
;;     is now a first-class diagnostic surface and no gate reads its user-facing strings. They can
;;     rot again tomorrow and nothing will say so. HANDED TO ARC 255 (it owns what a name IS):
;;     `docs/arc/2026/06/255-builtin-registry/NOTE-two-registry-adjacent-findings-from-arc-278.md`.
;;
;;
;; PROMOTED from `wat-scripts/lib/gen.wat` 2026-08-25, on the `wat/grep.wat`
;; precedent: a MOVE of proven code, with the numbers that earned it.
;;
;;   SHIPPED NUMBERS  24 laws, every one mutation-proven — the library proves its
;;                    own laws THROUGH its own driver. They live in
;;                    `wat-tests/gen.wat` and are discovered by `wat::test! {}`
;;                    (`tests/kernel/test.rs`), so there is no hand-maintained
;;                    total to drift. (This block used to cite
;;                    `wat-scripts/fuzz/gen-selftest.wat` + `tests/lint/gen_lib_laws.rs`
;;                    as the gate; BOTH were deleted the same day it was written,
;;                    by the commit that moved the laws — so the file's
;;                    load-bearing honesty claim pointed at nothing.)
;;                    THREE live rete defects found by the first consumer
;;                    (`docs/arc/2026/06/278-rules-engine/RETE-FIX-LIST.md`).
;;                    COST IS PER SHAPE, and there is no single number. Measured
;;                    2026-08-26, release, wall-clock minus the ~337ms stdlib
;;                    bootstrap, this box:
;;                       ints                            ~2.4 us/point  (500k)
;;                       coords, bases [50 100 100]      ~33  us/point  (500k)
;;                       such-that o bind o record       ~287 us/point  (1260)
;;                    The last row is the ONLY shape that ships — it is the rete
;;                    differential fuzzer's own space, replicated to its exact
;;                    card of 1260 (wat-tests/rete/differential-fuzz.wat:236).
;;                    Budget from the row you are actually building.
;;
;;                    ⛔ NO RATIO AGAINST THE `$oracle` IS QUOTED HERE, AND THAT
;;                    IS DELIBERATE. This header used to claim "~23us/point,
;;                    roughly 300x cheaper than the oracle it drives, so the
;;                    library is never the bottleneck". Every clause was wrong or
;;                    rotting: 23us/point was a `coords` measurement generalised
;;                    to a claim with no shape qualifier, while the shape that
;;                    ships is ~20x dearer. And the RATIO cannot be repaired by
;;                    re-measuring, because the `$oracle` is slow-but-correct BY
;;                    DESIGN and carries no perf requirement — it gets passively
;;                    faster as wat stops being interpreted. A ratio whose
;;                    denominator is expected to shrink is a claim that decays
;;                    toward false with nobody touching it, and "never the
;;                    bottleneck" decays fastest of all: as the oracle speeds up,
;;                    this library's SHARE grows. Absolute per-shape cost is the
;;                    only figure that stays true.
;;
;; The `gen-` name prefix dissolved into the namespace on promotion, the way
;; `:user::wat-grep` became `:wat::grep::`. `(:wat::gen::ints 0 3)`, not
;; `gen-ints`.
;;
;; Design record + what wat needs LESS of than Clojure: docs/arc/2026/06/278-rules-engine/GENERATIVE-TESTING.md
;;
;; A generator is an INDEXED SET, not a seeded random source:
;;
;;     Gen<T> = { card : i64,  at : i64 -> T }
;;
;; That one choice is what the whole design turns on, and it differs deliberately from the
;; QuickCheck / `clojure.test.check` lineage this borrows from (`gen/elements`, `gen/fmap`,
;; `gen/tuple` map onto the verbs below). Because `at` is a total function of an index, three
;; operations that are separate machinery there collapse into one here:
;;
;;   ENUMERATE  iterate 0..card                 — exhaustive whenever the space fits
;;   SAMPLE     pick any i < card               — uniform, and reproducible by construction
;;   SHRINK     walk a coordinate's digits down — index arithmetic, not tree surgery
;;
;; And a failing case gets a PERMANENT name. A `test.check` seed is meaningless the moment the
;; generator changes; a coordinate like `[3 1 0 2]` still dials in the same case.
;;
;; The cost, stated plainly: every dimension must be BOUNDED. You cannot generate an unbounded
;; structure. For differential testing against a slow reference that is a feature, not a limit —
;; it is what keeps the oracle affordable.
;;
;; `defstruct`, not `defrecord`: a Gen carries a FUNCTION, and the containment rule (arc 293.W)
;; holds that a pure aggregate must survive an EDN round-trip across a comms boundary. A
;; generator is local computation and never crosses one. This is also the general rule for
;; everything in this file — a `defrecord` is FORCED pure, a `defstruct` merely may be, so
;; anything that only ever holds pure data is a record here and only `Gen` is a struct.
;;
;; ⚠ AND THE CHECKER DOES NOT ENFORCE IT FOR THIS TYPE. This comment used to end "The checker
;; names this itself if you try — it is a good error." That is TRUE of a bare function field
;; and FALSE of a `Gen`, which is the shape it was actually about. Both measured 2026-08-26:
;;
;;   (defrecord Holder [at <- [i64 :-> i64]])          -> REFUSED, ImpureFieldInPureAggregate
;;   (defrecord Wrap   [g  <- (Gen :- [i64])])         -> LOADS CLEAN
;;
;; `is_pure_type`'s Parametric arm (src/check.rs:13782) resolves a head against a hardcoded
;; list and otherwise falls through to "pure iff its type args are pure" — it never asks the
;; TypeEnv what the head IS. So `(Gen :- [i64])` reads pure because `i64` is, and a Gen enters
;; a record and crosses the wire arriving `:at #wat.core/fn nil` — `card` honest, `at` dead,
;; and nothing in the value saying so.
;;
;; A SUBSTRATE GAP, NOT A GEN GAP, and not this arc's to close — but gen.wat is the file that
;; certifies the gate, so the claim had to go. Diagnosed, with a proven patch and a floor
;; result, in
;; `docs/arc/2026/06/293-struct-record-symmetry/NOTE-a-parametric-struct-passes-the-purity-gate.md`.
;; Until it lands, DO NOT rely on the checker to stop a Gen crossing a boundary — nothing does.
;;

(:wat::core::defstruct :wat::gen::Gen :- [T]
  [card <- :wat::core::i64
   at   <- [:wat::core::i64 :-> T]])

;; THE ONLY CONSTRUCTOR ANY VERB IN THIS FILE USES. A `card` is a COUNT, and a
;; count below zero is not a small space -- it is not a space. `(ints 5 2)` is an
;; EMPTY range, and 0 is its true cardinality, so flooring here is the right
;; answer rather than a suppressed error. It is also what this file already
;; decided: `EmptySpace` is documented above as "the honest name for card 0, not
;; an error condition", whose ruling belongs to the caller's own match arm.
;;
;; WHY A CONSTRUCTOR AND NOT A TEST AT EACH CONSUMER. Before this existed, twelve
;; sites each computed `card` freely and four could go negative -- `ints` on
;; `hi < lo`, `card-of` on a negative base, `take` on a negative `n`, and anything
;; downstream. Two defects followed, and neither announced itself
;; (GEN-VIGILIA-2026-08-25, findings A and B):
;;   A  `check`'s emptiness guard is `(= card 0)`, and -3 is not 0, so the fold ran
;;      zero iterations and reported `Checked(points -3, violations 0)` -- a clean
;;      pass over a NEGATIVE denominator, for a property that always fails.
;;   B  `one-of`/`bind` dispatch by SUBTRACTING each branch's card from the offset,
;;      so a negative branch ADVANCES it. Measured: a card -2 branch beside a card 3
;;      branch yielded card 1, and two of the three real points were unreachable.
;;      No raise, no `EmptySpace`, no signal of any kind.
;; Both dissolve here, and NEITHER dispatch needed changing: with no negative card
;; in existence the subtraction cannot run backwards.
;;
;; HOW HIGH THIS RUNG IS, stated honestly (`extirpare`'s ladder). This is a CHECK
;; AT CONSTRUCTION, not an unrepresentable shape, and it cannot be raised further
;; with the material wat has today:
;;   - a `card` that cannot HOLD a negative needs an unsigned type; BARE_PRIMITIVES
;;     (src/check.rs:993) is i64/f64/bool/String/u8, and u8 is nowhere near wide
;;     enough for a cardinality.
;;   - a constructor that cannot be BYPASSED needs privacy; wat has none.
;;     `:wat::core::Fault/of` (wat/core.wat:2114) is the precedent -- the raw
;;     `Fault` constructor stays reachable beside its smart one.
;; So the guarantee is exactly this and no more: NO VERB IN THIS LIBRARY PRODUCES
;; A NEGATIVE CARD. A caller hand-building the raw struct with a negative card
;; still can. `law-no-producer-yields-a-negative-card` (wat-tests/gen.wat) is the
;; gate, and it drives the PRODUCERS, never this function -- a law aimed here
;; would prove the floor and say nothing about the twelve callers, which is
;; exactly the seam-blindness (FM 24) that let A and B ship.
(:wat::core::defn :wat::gen::gen :- [T]
  [card <- :wat::core::i64  at <- [:wat::core::i64 :-> T]] -> (:wat::gen::Gen :- [T])
  (:wat::gen::Gen
    :card (:wat::core::if (:wat::core::< card 0) 0 card)
    :at   at))

;; ── index arithmetic ─────────────────────────────────────────────────────────
;; `digit` IS `rem`, and `shift` IS truncating division. Both args are non-negative
;; at every call here, so the truncating and flooring readings coincide.
;;
;; ⚠ THIS USED TO SAY "No native i64 mod/rem (only + - * /)", which justified a
;; hand-rolled `i - (i/base)*base`. That was FALSE for seven weeks before this file
;; was promoted: `:wat::core::i64::{mod,rem,quot}` all ship (wat/core.wat:493-501,
;; src/runtime.rs:5928,5941), landed 2026-07-05 in `720303f46`. The claim was TRUE
;; when it was first written in a grid axis two days earlier, was copied forward
;; into a scratch probe, and by 2026-08-01 a sibling in the SAME directory had
;; already corrected it — I inherited the retired version into the stdlib while
;; citing the very files where the correction lived. Verified equal on
;; (0,3) (7,3) (8,4) (1234,10) before the swap.
(:wat::core::defn :wat::gen::digit [i <- :wat::core::i64  base <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::i64::rem i base))

(:wat::core::defn :wat::gen::shift [i <- :wat::core::i64  base <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::i64::/ i base))

;; ── the primitive generator ──────────────────────────────────────────────────
(:wat::core::defn :wat::gen::ints [lo <- :wat::core::i64  hi <- :wat::core::i64]
  -> (:wat::gen::Gen :- [:wat::core::i64])
  (:wat::gen::gen (:wat::core::i64::- hi lo)
              (:wat::core::fn [i <- :wat::core::i64] -> :wat::core::i64
                      (:wat::core::i64::+ lo i))))

;; ── bools — the TOTAL generator, and why it is the only one ─────────────────
;;
;; `ints` and `elements` both make the CALLER choose: a range, a pool. That is
;; RIGHT for an unbounded domain. A `Gen` is `{card, at}` — finite by construction
;; — so "all i64" is a 2^64-point space, which is not a test. WHICH SUBSET is
;; interesting is the test design itself, and no type and no reflection knows it.
;; That is also why this library has no `Arbitrary`-style derivation: a type
;; carries no bounds, and a finite generator is nothing but bounds.
;;
;; `bool` is the other case, and it earns a verb of its own. Its domain is finite,
;; total, and two points wide: there is no range to pick, no pool to write, and
;; nothing the author knows that this library does not. A caller hand-writing
;; `(elements [false true])` is transcribing the one generator that CAN be derived
;; with certainty — so it ships instead.
;;
;; ⚠ EXHAUSTIVE, NOT A SAMPLE — the enumerative advantage, in its smallest case.
;; `test.check`'s `gen/boolean` DRAWS; a property checked against it has seen some
;; booleans. `check` over this has seen BOTH, always, and the `Checked(2, ...)` it
;; returns says so in the same match arm.
;;
;; The same reasoning extends to any all-unit enum (card = variant count) and to
;; `u8` (card 256 — small enough to enumerate). NEITHER IS BUILT: they have no
;; caller, and a verb with no caller is a claim, not a capability. When one
;; appears, this comment is the argument for building it.
(:wat::core::defn :wat::gen::bools [] -> (:wat::gen::Gen :- [:wat::core::bool])
  (:wat::gen::gen 2
    (:wat::core::fn [i <- :wat::core::i64] -> :wat::core::bool
      (:wat::core::= i 1))))

;; ── fmap: reshape what a generator yields, keeping its cardinality ────────────
(:wat::core::defn :wat::gen::fmap :- [A B]
  [f <- [A :-> B]  g <- (:wat::gen::Gen :- [A])] -> (:wat::gen::Gen :- [B])
  (:wat::core::let [inner (:wat::gen::Gen/at g)]
    (:wat::gen::gen (:wat::gen::Gen/card g)
                (:wat::core::fn [i <- :wat::core::i64] -> B (f (inner i))))))

;; ── the workhorse: a COORDINATE generator over mixed bases ───────────────────
;; `gen-coords [b0 b1 b2]` has card b0*b1*b2 and yields [d0 d1 d2] with di < bi —
;; positional notation in mixed radix. This is `gen/tuple` for the enumerable
;; case, and it is what a target actually wants: one index in, its own tuple of
;; dimension choices out, with no heterogeneous tuple type needed.
;; ── the two nouns the depth was hiding ───────────────────────────────────────
;;
;; `(:wat::core::PersistentVector :- [:wat::core::i64])` appeared TWENTY-FIVE times in
;; this file and meant TWO different things: a COORDINATE (one digit per dimension,
;; digit i < base i) and a BASES vector (the radix of each dimension). Naming them
;; makes every signature say which it wants — `reverse-index [bases k]` reads as a
;; radix vector and an index, where before it read as two vectors of numbers.
;;
;; ⚠ THESE ARE TRANSPARENT ALIASES, NOT DISTINCT TYPES, AND THAT MATTERS HERE.
;; `is_pure_type` opens by canonicalizing — "expand aliases" — so `Coord` and `Bases`
;; are the SAME type to the checker, and handing a coordinate where bases belong
;; still type-checks clean. `perspicere` flagged exactly that swap. Naming does not
;; close it; it makes the intent legible at every call site, which is the whole of
;; what an alias can buy.
;;
;; MAKING THE SWAP UNREPRESENTABLE would need a wrapper record per noun, and every
;; `digit`/`nth`/fold in the hot enumeration path would then unwrap on each access —
;; a real cost on the one path this library exists to make cheap, to close a confusion
;; with no recorded instance. Not taken. If one ever bites, this is the note to
;; overturn, and the wrapper is the answer.
;;
;; The name is `Coord`, singular, deliberately: `wat/core.wat:1096` already generates
;; a `<fqdn>::Coords` record for the service-pool carrier, and two different `Coords`
;; in one substrate is the collision this comment exists to avoid.
(:wat::core::typealias :wat::gen::Coord
  (:wat::core::PersistentVector :- [:wat::core::i64]))

(:wat::core::typealias :wat::gen::Bases
  (:wat::core::PersistentVector :- [:wat::core::i64]))


(:wat::core::defrecord :wat::gen::GenAcc
  [rem <- :wat::core::i64
   out <- :wat::gen::Coord])

;; CARDINALITY OVERFLOW NEEDS NO GUARD HERE, and that is a substrate fact worth
;; recording rather than a shortcut. A wrapped `card` would be the worst kind of
;; defect — a SILENT under-count that every law in the self-test still passes,
;; since each individual coordinate decodes correctly. It cannot happen: wat's
;; `i64::*` is CHECKED and raises `IntegerOverflow` naming both operands. Verified
;; 2026-08-25 with bases [4000000000 4000000000].
;;
;; This is a place where wat needs LESS than the Clojure lineage, for a reason that
;; has nothing to do with generators: Clojure would promote to BigInt (changing the
;; type under you) and C-family arithmetic would wrap in silence. A hand-rolled
;; checked multiply here was written and then DELETED — it was unreachable, because
;; the multiply inside it raised first.
(:wat::core::defn :wat::gen::card-of [bases <- :wat::gen::Bases]
  -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [a <- :wat::core::i64  b <- :wat::core::i64] -> :wat::core::i64
      (:wat::core::i64::* a b))
    1 bases))

(:wat::core::defn :wat::gen::coords [bases <- :wat::gen::Bases]
  -> (:wat::gen::Gen :- [:wat::gen::Coord])
  (:wat::gen::gen
    (:wat::gen::card-of bases)
    (:wat::core::fn [i <- :wat::core::i64] -> :wat::gen::Coord
          (:wat::gen::GenAcc/out
            (:wat::core::foldl
              (:wat::core::fn [acc <- :wat::gen::GenAcc  b <- :wat::core::i64] -> :wat::gen::GenAcc
                (:wat::gen::GenAcc
                  :rem (:wat::gen::shift (:wat::gen::GenAcc/rem acc) b)
                  :out (:wat::core::PersistentVector/conj (:wat::gen::GenAcc/out acc)
                         (:wat::gen::digit (:wat::gen::GenAcc/rem acc) b))))
              (:wat::gen::GenAcc :rem i :out (:wat::core::PersistentVector))
              bases)))))

;; ── the driver ───────────────────────────────────────────────────────────────
;;
;; RETURNS A MATCHABLE VALUE, NEVER RAISES — the no-hidden-failures LAW (builder,
;; 2026-07-17: "i want wat to never hide failures ever again"), and the shape is
;; the point rather than the politeness.
;;
;; An earlier version RAISED on an empty generator, which was the same defect the
;; LAW forbids, written hours after reading it. But a nicer raise was never the
;; fix. The real hazard is that `violations = 0` reads as success whether the
;; property held at ten thousand points or was never applied at all — and a guard
;; that raises still hands back a bare count everywhere else, leaving every caller
;; free to report a denominator it never looked at.
;;
;; `Checked` therefore carries BOTH numbers. A caller cannot extract a violation
;; count without the point count arriving in the same match arm, so "0 violations"
;; can no longer be reported without knowing what it was 0 out of. The wrong
;; reading has no form.
;;
;; AND THE TWO NUMBERS ARE NOW ORDERED BY CONSTRUCTION: 0 <= violations <= points.
;; `prop` returns `bool`, not `i64`. It used to be `[T :-> :wat::core::i64]` and the
;; driver SUMMED what it returned, which permitted two states that cannot be true of
;; any real check (GEN-VIGILIA L2, conformare + struere + sequi):
;;   - `violations > points` — a prop returning 5 counted five failures at one point;
;;   - a WITNESS beside a ZERO count — negative returns cancelling positive ones, so
;;     `first-failure` was `Some i` while `violations` summed to 0.
;; Every `prop` in the tree already returned only 0 or 1, so the extra width bought
;; nothing and cost exactly those two readings. `bool` removes them at the type: a
;; weight has no form, so neither does either state. The wrong thing is not caught,
;; it is unrepresentable — the rung above the guard that `:wat::gen::gen` holds. `EmptySpace` is then simply the honest name for card 0,
;; not an error condition — a `such-that` whose predicate excludes everything is
;; usually a caller bug, but that is the CALLER's ruling to make, in its own arm.
;; `EmptySpace` IS NULLARY, AND THAT IS A DECISION — recorded because `conformare`
;; flagged it and the flag is fair on its face: eight distinct producers can yield a
;; card-0 space (`ints lo lo`, a `such-that` that excludes everything, `take 0`,
;; `elements []`, a `coords` base of 0, …) and they all collapse to one dataless
;; token, so a caller told to "make a ruling" is handed nothing to rule on.
;;
;; It stays nullary because `check` CANNOT HONESTLY KNOW. A `Gen` is `{card, at}` and
;; carries no provenance — that absence is the library's whole thesis, the reason
;; every combinator returns the same type and composes without a hierarchy. Giving
;; this arm a reason means threading a reason through all twelve construction sites
;; and through `fmap`/`take`/`one-of`/`bind`, so that every generator carries an
;; explanation of an emptiness that most of them never have. That is a large, viral
;; change to buy a field whose only consumer today discards it: the ONE arm in the
;; tree is `(assert-true false)`, because for a LAW an empty space is always a
;; failure — it was not tested.
;;
;; So the bound is: this arm says THAT the space is empty and never WHY, and the
;; caller that needs why must look at the generator it built rather than the outcome
;; it got back. If a consumer ever appears that must branch on the reason, this is
;; the note to overturn — the cost above is the price, and it was not paid blind.
(:wat::core::defenum :wat::gen::CheckOutcome :wat::enum::Pure
  :Checked    [points <- :wat::core::i64
               violations <- :wat::core::i64
               first-failure <- (:wat::core::Option :- [:wat::core::i64])]
  :EmptySpace)

(:wat::core::defrecord :wat::gen::CheckAcc
  [bad <- :wat::core::i64  first <- (:wat::core::Option :- [:wat::core::i64])])

(:wat::core::defn :wat::gen::check :- [T]
  [g <- (:wat::gen::Gen :- [T])  prop <- [T :-> :wat::core::bool]] -> :wat::gen::CheckOutcome
  (:wat::core::let [card (:wat::gen::Gen/card g)
                    at   (:wat::gen::Gen/at g)]
    (:wat::core::if (:wat::core::= card 0)
      :wat::gen::CheckOutcome::EmptySpace
      (:wat::core::let
        [acc (:wat::core::foldl
               (:wat::core::fn [a <- :wat::gen::CheckAcc  i <- :wat::core::i64] -> :wat::gen::CheckAcc
                 ;; `true` = the property HELD at this point (the QuickCheck reading).
                 ;; A failing point contributes EXACTLY 1 — see the type note above
                 ;; `prop`: a weight is not expressible, so `violations` cannot exceed
                 ;; `points` and cannot go negative.
                 (:wat::core::if (prop (at i))
                     a
                     (:wat::gen::CheckAcc
                       :bad (:wat::core::i64::+ (:wat::gen::CheckAcc/bad a) 1)
                       :first (:wat::core::match (:wat::gen::CheckAcc/first a)
                                ((:wat::core::Some f) (:wat::core::Some f))
                                (:wat::core::None (:wat::core::Some i))))))
               (:wat::gen::CheckAcc :bad 0 :first :wat::core::None)
               (:wat::core::range 0 card))]
        (:wat::gen::CheckOutcome::Checked card
          (:wat::gen::CheckAcc/bad acc)
          (:wat::gen::CheckAcc/first acc))))))

;; ── shrink-index — GENERATOR-INDEPENDENT, unlike the coordinate shrink ──────
;;
;; ⚠ A CLAIM CORRECTED. This library's design record said "shrinking is
;; generator-independent" from the day `shrink` was written. It was not true.
;; `shrink` takes a COORDINATE (`PV<i64>`) and descends its digits, so it works on
;; a raw `coords` space and composes with NONE of `bind`, `such-that`, `one-of` or
;; `record` — the combinators that make the library worth having. The claim was
;; about the design's potential; the code only ever delivered it for one shape.
;;
;; This is the general form: it shrinks an INDEX into any `Gen`, by walking down
;; for the smallest index that still fails. That is meaningful precisely because
;; enumeration order is a simplicity order here — `coords` yields all-zero first,
;; `one-of`/`bind` place earlier branches first, `vector-upto` puts short vectors
;; before long ones. "Earlier" IS "simpler" by construction.
;;
;; It is O(k) in the index, where the coordinate shrink is O(sum of bases). For a
;; coords-shaped space the coordinate version is sharper and should be preferred;
;; this one is what you reach for when the space has any other shape.
;; ── descend — THE search both shrinkers run ─────────────────────────────────
;; Walk 0..start and keep the FIRST (smallest) candidate that still fails; once
;; lowered, `best` differs from `start` and every later candidate is skipped, so this
;; is one linear pass and not a scan.
;;
;; `shrink-index` and `shrink-dim` were this fold written out twice — "comment and
;; all" (`solvere`), and the duplicated comment is the tell. They differ in exactly
;; one thing: what a candidate index MEANS. For `shrink-index` it is a point of a
;; generator (`Gen/at`); for `shrink-dim` it is a value of one coordinate dimension
;; (`with c j v`). That difference is a function, so it becomes a parameter.
(:wat::core::defn :wat::gen::descend :- [T]
  [start <- :wat::core::i64
   probe <- [:wat::core::i64 :-> T]
   still-fails? <- [T :-> :wat::core::bool]]
  -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [best <- :wat::core::i64  i <- :wat::core::i64] -> :wat::core::i64
      (:wat::core::if (:wat::core::and (:wat::core::= best start) (still-fails? (probe i)))
        i best))
    start
    (:wat::core::range 0 start)))

(:wat::core::defn :wat::gen::shrink-index :- [T]
  [g <- (:wat::gen::Gen :- [T])  k <- :wat::core::i64  still-fails? <- [T :-> :wat::core::bool]]
  -> :wat::core::i64
  (:wat::gen::descend k (:wat::gen::Gen/at g) still-fails?))

;; ── gen-elements: pick from a value vector ───────────────────────────────────
;; The most-used combinator in the QuickCheck tradition (`gen/elements`), and the
;; one every non-numeric dimension reaches for first.
(:wat::core::defn :wat::gen::elements :- [T]
  [vs <- (:wat::core::PersistentVector :- [T])] -> (:wat::gen::Gen :- [T])
  (:wat::gen::gen (:wat::core::length vs)
              (:wat::core::fn [i <- :wat::core::i64] -> T
                      (:wat::core::Option/expect (:wat::core::get vs i)
                        "elements: index outside the vector it was built from"))))

;; ── gen-such-that: an EXACT filter, with no retries ──────────────────────────
;; `test.check`'s `such-that` filters an opaque random source by retry-and-discard:
;; it can fail outright after N tries, and it biases the distribution of whatever
;; survives. A finite indexed generator has neither problem — walk the space ONCE,
;; keep the indices that pass, and the survivors ARE the new generator with an
;; exact new cardinality. No retry budget, no failure mode, no bias.
;;
;; The cost is honest and bounded: it materializes one i64 per surviving index, and
;; it evaluates `at` over the whole source space once at construction.
(:wat::core::defn :wat::gen::such-that :- [T]
  [pred <- [T :-> :wat::core::bool]  g <- (:wat::gen::Gen :- [T])] -> (:wat::gen::Gen :- [T])
  (:wat::core::let [at   (:wat::gen::Gen/at g)
                    keep (:wat::core::into (:wat::core::PersistentVector)
                           (:wat::core::filter
                             (:wat::core::fn [i <- :wat::core::i64] -> :wat::core::bool (pred (at i)))
                             (:wat::core::range 0 (:wat::gen::Gen/card g))))]
    (:wat::gen::gen (:wat::core::length keep)
                (:wat::core::fn [j <- :wat::core::i64] -> T
                        (at (:wat::core::Option/expect (:wat::core::get keep j)
                              "such-that: index outside the surviving set"))))))

;; ── gen-one-of: the SUM, where gen-coords is the PRODUCT ─────────────────────
;; `card` is the sum of the branches' cardinalities and `at` dispatches by range,
;; so branch k occupies a contiguous block of indices. Enumeration therefore walks
;; branch 0 exhaustively, then branch 1, and so on — which means a failure's
;; coordinate still localizes it, exactly as with a product space.
(:wat::core::defrecord :wat::gen::Pick :- [T]
  [rest <- :wat::core::i64
   got  <- (:wat::core::Option :- [T])])

(:wat::core::defn :wat::gen::one-of :- [T]
  [gs <- (:wat::core::PersistentVector :- [(:wat::gen::Gen :- [T])])] -> (:wat::gen::Gen :- [T])
  ;; THE ONE DISPATCH IN THIS FILE. `bind` is expressed over it: once a bind has
  ;; materialised its branch generators, choosing among them by subtracting
  ;; cardinalities IS this function, and it used to be written out a second time
  ;; there with its own `BindPick` twin of `Pick` (`solvere`).
  ;;
  ;; CARDS ARE HOISTED, and that is what made the collapse free rather than a
  ;; regression. This body used to fold over `gs` itself and read
  ;; `(:wat::gen::Gen/card g)` TWICE per branch per lookup — once for the `<` test
  ;; and once for the subtraction. `bind`'s copy folded over a precomputed
  ;; `PV<i64>`, so it was the faster of the two, and collapsing onto the slower one
  ;; measured +9% on the rete fuzzer's space (~265 -> ~288 us/point). Hoisting
  ;; `cards` here — one pass at construction, one read per branch — recovers it and
  ;; speeds up every `one-of` caller besides.
  (:wat::core::let
    [n     (:wat::core::length gs)
     cards (:wat::core::into (:wat::core::PersistentVector)
             (:wat::core::mapv
               (:wat::core::fn [g <- (:wat::gen::Gen :- [T])] -> :wat::core::i64
                 (:wat::gen::Gen/card g))
               gs))]
    (:wat::gen::gen
      (:wat::core::foldl
        (:wat::core::fn [a <- :wat::core::i64  c <- :wat::core::i64] -> :wat::core::i64
          (:wat::core::i64::+ a c))
        0 cards)
      (:wat::core::fn [i <- :wat::core::i64] -> T
        (:wat::core::Option/expect
          (:wat::gen::Pick/got
            (:wat::core::foldl
              (:wat::core::fn [acc <- (:wat::gen::Pick :- [T])  j <- :wat::core::i64]
                              -> (:wat::gen::Pick :- [T])
                (:wat::core::match (:wat::gen::Pick/got acc)
                  ((:wat::core::Some _v) acc)
                  (:wat::core::None
                    (:wat::core::let [c (:wat::gen::nth cards j)
                                      r (:wat::gen::Pick/rest acc)]
                      (:wat::core::if (:wat::core::< r c)
                        (:wat::gen::Pick :rest r
                          :got (:wat::core::Some
                                 ((:wat::gen::Gen/at
                                    (:wat::core::Option/expect (:wat::core::get gs j)
                                      "one-of: branch index outside the generator vector")) r)))
                        (:wat::gen::Pick :rest (:wat::core::i64::- r c)
                          :got :wat::core::None))))))
              (:wat::gen::Pick :rest i :got :wat::core::None)
              (:wat::core::range 0 n)))
          "one-of: index outside the summed cardinality")))))

;; ── nth: total indexed read, at ONE type ────────────────────────────────────
;; This was two functions — `nth` at `Coord -> i64` and `nth-str` at
;; `PV<String> -> String`, "the String twin" — which is one generic written twice
;; (`solvere`). The `Coord` typing was documentation rather than a constraint: aliases
;; are transparent, and this was already being called on `cards`, a plain `PV<i64>`
;; that is not a coordinate at all. So the narrow type was not buying the safety its
;; message implied, and the message ("coordinate digit out of range") was wrong at
;; those call sites.
(:wat::core::defn :wat::gen::nth :- [T]
  [v <- (:wat::core::PersistentVector :- [T])  i <- :wat::core::i64] -> T
  (:wat::core::Option/expect (:wat::core::get v i) "nth: index out of range"))

;; ── gen-record: a generator for a RECORD, from one generator per field ───────
;;
;; `(gen-record :user::Point (gen-ints 0 3) (gen-elements names))`
;;   -> Gen<:user::Point> with card 3 * len(names)
;;
;; WHY A MACRO — and it is ONLY for arity. `gen-lift2` / `gen-lift3` below do the
;; same job as plain functions, and are the better tool at those arities. A
;; function cannot be N-ary over generators, so this macro exists to cover 4+
;; fields, nothing more.
;;
;; ⚠ CORRECTION (2026-08-25). This comment previously justified the macro by
;; claiming wat "cannot construct from a type value" — that reflection reads
;; shapes but cannot build them, so explicit generators were forced. **The premise
;; was wrong, and the builder caught it: a type's constructor IS a first-class
;; function value.**
;;
;;     (:user::apply2 :user::Point' 3 4)   ->   #user/Point {:x 3 :y 4}
;;
;; What is genuinely unavailable is construction from a type KEYWORD — and that is
;; not a gap to fill: the result type could not be known statically, which is a
;; hole in the checker, not a missing intrinsic. The constructor value is the
;; language's answer and it is strictly better — fully typed, and general past
;; records to variants and smart constructors alike.
;;
;; NOR is macro-time reflection a gap: `wat/telemetry.wat:286` already records that
;; "compile-time/macro-expand reflection of a baked record is DEAD, proven; runtime
;; resolves for both stdlib and user records" — documented, with the resolution
;; already chosen. This macro never needed it anyway: its arity comes from the
;; caller's argument count, not from a type.
;;
;; What survives from the original reasoning is the half about VALUES, and it is
;; the half worth keeping: `field-types-of` yields `wat.type/i64` — a type,
;; carrying NO BOUNDS. A finite generator is nothing BUT bounds, so deriving one
;; from a type would have to invent a range, which is exactly the decision the
;; author must make. `spec`'s auto-gen earns its keep in a language with no types
;; to lean on; here what is missing is the interesting SUBSET, which no reflection
;; knows.
;;
;; The emitted constructor call is ORDINARY CHECKED WAT: too few, too many, or
;; wrongly-typed generators and the type-checker rejects the expansion — proven,
;; `ArityMismatch` and `expects :wat::core::i64; got :wat::core::String`.
;;
;; ⚠ EACH GENERATOR ARGUMENT IS EVALUATED EXACTLY ONCE — and until 2026-08-26 it
;; was not. This comment used to read "each generator expression is emitted TWICE
;; (once for its `card`, once for its `at`)... let-bind that at the call site
;; rather than inlining it". Both halves were wrong:
;;   - the `card` copy is evaluated once, but the `at` copy was spliced into the
;;     fmap lambda's BODY, so it re-ran ONCE PER GENERATED POINT. The cost model
;;     was wrong by a factor of the point count, not by 2.
;;   - MEASURED 2026-08-26, an 800-point `record` whose first argument is an
;;     inlined `such-that` over a 200-point source, release, minus the ~324ms
;;     stdlib bootstrap: pre-fix 395ms of generator, post-fix 25ms — ~16x here.
;;     THE MULTIPLE IS NOT A CONSTANT and must not be quoted as one: the waste is
;;     (cost of the inlined expression) x (point count), so it grows with the
;;     space. An earlier note here said 52x from a different space; that is the
;;     same defect measured on a dearer argument, not a disagreement.
;;   - and it documented the footgun instead of removing it, while the macro
;;     already carried `fresh-symbol`, which is exactly what binding each
;;     generator once requires.
;; The expansion now opens with a `let` that binds every generator argument to its
;; own hygienic `fresh-symbol`, and both the `card` and the `at` reference that
;; binding. A caller may inline `such-that` (or any enumerating combinator) freely.
(:wat::core::defmacro :wat::gen::record
  [T <- :wat::WatAST  & gens <- (:wat::core::Vector :- [:wat::WatAST])]
  -> :wat::WatAST
  (:wat::core::let
    [;; HYGIENIC binder. A literal `c` in binder position is REFUSED by the macro
     ;; system (hygiene bound gate E, arc 249 stone 249.2b-ii) because it could
     ;; capture a caller-site name. `fresh-symbol` stamps a fresh unique scope, and
     ;; the name is spliced with `~` at both its binding and its uses.
     cv    (:wat::core::fresh-symbol "coord")
     ;; The POSITIONAL constructor is the PRIME name: bare-positional construction
     ;; is retired (the bare name is the kwargs macro), so `:user::Point` becomes
     ;; `:user::Point'`. Same node-building idiom `:wat::core::kwargs-lower` uses.
     ctor  (:wat::core::keyword-node (:wat::core::string::concat (:wat::core::ast-name T) "'"))
     n     (:wat::core::length gens)
     ;; ONE hygienic binder per generator argument — this is what stops the `at`
     ;; copy re-evaluating its whole expression on every generated point.
     syms  (:wat::core::foldl
             (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST])  _i <- :wat::core::i64]
                             -> (:wat::core::Vector :- [:wat::WatAST])
               (:wat::core::conj acc (:wat::core::fresh-symbol "gen")))
             (:wat::core::Vector :wat::WatAST)
             (:wat::core::range 0 n))
     ;; the `let` binder vector, flat: sym expr sym expr ...
     binds (:wat::core::foldl
             (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST])  i <- :wat::core::i64]
                             -> (:wat::core::Vector :- [:wat::WatAST])
               (:wat::core::conj
                 (:wat::core::conj acc
                   (:wat::core::Option/expect (:wat::core::get syms i) "record: sym index"))
                 (:wat::core::Option/expect (:wat::core::get gens i) "record: gen index")))
             (:wat::core::Vector :wat::WatAST)
             (:wat::core::range 0 n))
     ;; both `card` and `at` now reference the BINDING, never the expression
     cards (:wat::core::foldl
             (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST])  g <- :wat::WatAST]
                             -> (:wat::core::Vector :- [:wat::WatAST])
               (:wat::core::conj acc `(:wat::gen::Gen/card ~g)))
             (:wat::core::Vector :wat::WatAST)
             syms)
     args  (:wat::core::foldl
             (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST])  i <- :wat::core::i64]
                             -> (:wat::core::Vector :- [:wat::WatAST])
               (:wat::core::conj acc
                 `((:wat::gen::Gen/at ~(:wat::core::Option/expect (:wat::core::get syms i) "record: sym index"))
                   (:wat::gen::nth ~cv ~i))))
             (:wat::core::Vector :wat::WatAST)
             (:wat::core::range 0 n))]
    `(:wat::core::let [~@binds]
       (:wat::gen::fmap
         (:wat::core::fn [~cv <- :wat::gen::Coord] -> ~T
           (~ctor ~@args))
         (:wat::gen::coords (:wat::core::PersistentVector ~@cards))))))

;; ── gen-lift2 / gen-lift3: apply an N-ary FUNCTION across N generators ───────
;;
;; The applicative lift (`liftA2`), and the honest primitive `gen-record` should
;; have been built on. It takes a FUNCTION, not a type — which matters because in
;; wat a constructor IS a first-class function value:
;;
;;     (:user::apply2 :user::Point' 3 4)   ->   #user/Point {:x 3 :y 4}
;;
;; so `(gen-lift2 :user::Point' gx gy)` generates records with no macro, no
;; hygiene ceremony, and no reflection. It also generalizes past records for free:
;; the function can be an enum variant, a smart constructor, or any computation —
;; anything `[A B :-> R]`.
;;
;; This corrects a claim made earlier in this file's history: that wat "cannot
;; construct from a type value". The premise was wrong. Construction from a type
;; KEYWORD is indeed unavailable (and would punch a hole in the checker, since the
;; result type could not be known statically) — but that was never the language's
;; answer. The constructor value is, and it is strictly better: fully typed, and
;; not limited to records.
(:wat::core::defn :wat::gen::lift2 :- [A B R]
  [f <- [A B :-> R]  ga <- (:wat::gen::Gen :- [A])  gb <- (:wat::gen::Gen :- [B])]
  -> (:wat::gen::Gen :- [R])
  ;; EXPRESSED OVER `coords`, not over a second hand-written mixed radix. Until
  ;; 2026-08-26 this encoded the radix itself with `digit`/`shift`, which made it a
  ;; SECOND implementation of what `coords` already does — and the two disagreed
  ;; (GEN-VIGILIA finding C: same card-6 space, index 6, `lift2` said b=12 and the
  ;; `record`/`coords` path said b=10). Out of contract, since 6 is past a card of
  ;; 6 — but `ints`' `at` has no bounds check, so neither refused, and a disagreement
  ;; between two encodings of one idea is a defect whichever side is "right".
  ;; `coords` survives because `record`, `vector-of` and `coords-scattered` all
  ;; already go through it; there is now exactly one mixed-radix encoding in the file.
  (:wat::core::let [fa (:wat::gen::Gen/at ga)
                    fb (:wat::gen::Gen/at gb)]
    (:wat::gen::fmap
      (:wat::core::fn [c <- :wat::gen::Coord] -> R
        (f (fa (:wat::gen::nth c 0)) (fb (:wat::gen::nth c 1))))
      (:wat::gen::coords (:wat::core::PersistentVector
                           (:wat::gen::Gen/card ga)
                           (:wat::gen::Gen/card gb))))))

(:wat::core::defn :wat::gen::lift3 :- [A B C R]
  [f <- [A B C :-> R]  ga <- (:wat::gen::Gen :- [A])  gb <- (:wat::gen::Gen :- [B])  gc <- (:wat::gen::Gen :- [C])]
  -> (:wat::gen::Gen :- [R])
  ;; over `coords`, for the same reason as `lift2` above.
  (:wat::core::let [fa (:wat::gen::Gen/at ga)
                    fb (:wat::gen::Gen/at gb)
                    fc (:wat::gen::Gen/at gc)]
    (:wat::gen::fmap
      (:wat::core::fn [c <- :wat::gen::Coord] -> R
        (f (fa (:wat::gen::nth c 0))
           (fb (:wat::gen::nth c 1))
           (fc (:wat::gen::nth c 2))))
      (:wat::gen::coords (:wat::core::PersistentVector
                           (:wat::gen::Gen/card ga)
                           (:wat::gen::Gen/card gb)
                           (:wat::gen::Gen/card gc))))))

;; ── SAMPLING, as composition rather than a second driver ─────────────────────
;;
;; The obvious design was `gen-check-sampled g order n prop` — a parallel driver
;; beside `gen-check`. That would have been a second thing to keep correct, and a
;; second place for a vacuity bug to live. Sampling is two ordinary generator
;; TRANSFORMERS instead, and `gen-check` never changes:
;;
;;     (gen-check (gen-take 500 (gen-coords-scattered bases)) prop)
;;
;; `gen-take` is a prefix. `gen-coords-scattered` is the SAME space in a different
;; ORDER. Sampling is therefore "enumerate a reordering, stop early" — which is
;; why it keeps every property enumeration has: no seed, resumable, and a case is
;; still a coordinate.

;; A PREFIX of a generator. Refuses to invent points it does not have.
(:wat::core::defn :wat::gen::take :- [T]
  [n <- :wat::core::i64  g <- (:wat::gen::Gen :- [T])] -> (:wat::gen::Gen :- [T])
  (:wat::gen::gen
    (:wat::core::if (:wat::core::< n (:wat::gen::Gen/card g)) n (:wat::gen::Gen/card g))
    (:wat::gen::Gen/at g)))

;; MIXED-RADIX DIGIT REVERSAL — van der Corput / Halton, adapted to mixed bases.
;; Digit j of k sits at position (n-1-j) of the reversed sequence, whose place
;; value is the product of the bases AFTER j — that is card / (b0*..*bj). A
;; running prefix product gives each digit its reversed place in ONE fold, with no
;; vector reversal.
(:wat::core::defrecord :wat::gen::GenRev
  [rem <- :wat::core::i64  idx <- :wat::core::i64  pref <- :wat::core::i64])

(:wat::core::defn :wat::gen::reverse-index
  [bases <- :wat::gen::Bases  k <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::let [card (:wat::gen::card-of bases)]
    (:wat::gen::GenRev/idx
      (:wat::core::foldl
        (:wat::core::fn [a <- :wat::gen::GenRev  b <- :wat::core::i64] -> :wat::gen::GenRev
          (:wat::core::let [d  (:wat::gen::digit (:wat::gen::GenRev/rem a) b)
                            pf (:wat::core::i64::* (:wat::gen::GenRev/pref a) b)]
            (:wat::gen::GenRev
              :rem  (:wat::gen::shift (:wat::gen::GenRev/rem a) b)
              :idx  (:wat::core::i64::+ (:wat::gen::GenRev/idx a)
                      (:wat::core::i64::* d (:wat::core::i64::/ card pf)))
              :pref pf)))
        (:wat::gen::GenRev :rem k :idx 0 :pref 1)
        bases))))

;; The same coordinate space, visited so the SLOWEST-varying dimensions move
;; first. Measured over `[3 3 3 3 4]` (`wat-tests/gen.wat`'s L26, which RUNS on every floor —
;; it replaced a wat-scripts probe that never ran and cloned `reverse-index` rather than calling it):
;; in the first 16 of 324, sequential order covers the last dimension 1/4 and this
;; order covers it 4/4; at 64 sequential is STILL 1/4. The two orders are mirror
;; images — this one under-covers the fastest-varying dimensions at very small K —
;; so it is the right default only because a prefix is what sampling takes, and a
;; prefix that never varies a dimension has not sampled that dimension at all.
(:wat::core::defn :wat::gen::coords-scattered
  [bases <- :wat::gen::Bases]
  -> (:wat::gen::Gen :- [:wat::gen::Coord])
  (:wat::core::let [g (:wat::gen::coords bases)
                    at (:wat::gen::Gen/at g)]
    (:wat::gen::gen (:wat::gen::Gen/card g)
                (:wat::core::fn [k <- :wat::core::i64]
                                    -> :wat::gen::Coord
                      (at (:wat::gen::reverse-index bases k))))))

;; ── SHRINKING — coordinate descent, and GENERATOR-INDEPENDENT ────────────────
;;
;; In the QuickCheck lineage every generator carries its own shrink tree, because
;; a generated value is opaque and only its producer knows how to make it smaller.
;; Here the structure lives in the INDEX, so shrinking is arithmetic on digits and
;; ONE implementation shrinks everything built from `gen-coords`. There is nothing
;; per-generator to write, and so nothing per-generator to get wrong.
;;
;; Greedy per-dimension descent: for each dimension in turn, take the SMALLEST
;; value that still fails. O(sum of bases) property evaluations — cheap enough
;; that it is never worth being clever, and predictable enough to reason about.
;;
;; `still-fails?` is the caller's, and it must be the SAME predicate the search
;; used. Handing this a different predicate produces a confident, minimal, wrong
;; answer — the failure mode to watch for, since nothing here can detect it.

;; Replace digit `j` of a coordinate. Written as a rebuild because index-`assoc`
;; on a PersistentVector is not shipped (the collections campaign's open
;; index-assoc item); `assoc` reaches maps, not vector positions.
(:wat::core::defn :wat::gen::with
  [c <- :wat::gen::Coord
   j <- :wat::core::i64  v <- :wat::core::i64]
  -> :wat::gen::Coord
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::gen::Coord  i <- :wat::core::i64]
                    -> :wat::gen::Coord
      (:wat::core::PersistentVector/conj acc
        (:wat::core::if (:wat::core::= i j) v (:wat::gen::nth c i))))
    (:wat::core::PersistentVector)
    (:wat::core::range 0 (:wat::core::length c))))

;; Lower ONE dimension as far as it will go while still failing.
(:wat::core::defn :wat::gen::shrink-dim
  [c <- :wat::gen::Coord
   j <- :wat::core::i64
   still-fails? <- [:wat::gen::Coord :-> :wat::core::bool]]
  -> :wat::gen::Coord
  ;; the SAME descent as `shrink-index`, differing only in what a candidate means:
  ;; here it is a value of dimension `j`, so the probe rebuilds the coordinate.
  (:wat::core::let [cur  (:wat::gen::nth c j)
                    best (:wat::gen::descend cur
                           (:wat::core::fn [v <- :wat::core::i64] -> :wat::gen::Coord
                             (:wat::gen::with c j v))
                           still-fails?)]
    (:wat::gen::with c j best)))

;; Descend every dimension, left to right.
(:wat::core::defn :wat::gen::shrink
  [c <- :wat::gen::Coord
   still-fails? <- [:wat::gen::Coord :-> :wat::core::bool]]
  -> :wat::gen::Coord
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::gen::Coord  j <- :wat::core::i64]
                    -> :wat::gen::Coord
      (:wat::gen::shrink-dim acc j still-fails?))
    c
    (:wat::core::range 0 (:wat::core::length c))))

;; ── bind: DEPENDENT generation — the shape of B depends on the VALUE of A ────
;;
;; `(bind (ints 1 4) (fn [n] (ints 0 n)))` — generate n, THEN generate something
;; whose space is determined by n. This is the one combinator the finite model
;; does not get for free from `coords`, because a coordinate space has a FIXED
;; shape and this one does not.
;;
;; It is `one-of` over a COMPUTED branch list rather than a literal one:
;;
;;     card = SUM over a in ga of card(f(a))
;;     at k = walk the branches, subtracting each card, until k lands in one
;;
;; so the same contiguous-block property holds: branch i occupies a run of
;; consecutive indices, enumeration walks branch 0 exhaustively then branch 1,
;; and a failing index still localizes.
;;
;; COST, stated because it is real and unlike every other combinator here. `f` is
;; applied once per source point AT CONSTRUCTION to learn the branch cardinalities
;; (cached in `cards`), and once more per `at` call to reach the chosen branch.
;; So `bind` is O(card(ga)) per lookup where everything else is O(1)-ish. That is
;; affordable when the source is small — which is the shape dependent generation
;; actually takes — and it is why `cards` is precomputed rather than recomputed:
;; without the cache, `card` alone would rebuild every branch generator on every
;; call.
(:wat::core::defn :wat::gen::bind :- [A B]
  [ga <- (:wat::gen::Gen :- [A])  f <- [A :-> (:wat::gen::Gen :- [B])]]
  -> (:wat::gen::Gen :- [B])
  ;; `f` RUNS EXACTLY ONCE PER BRANCH, HERE, and the Gen it returns is KEPT.
  ;; It used to compute a `cards` vector by calling `f` and THROWING THE GEN AWAY,
  ;; so the dispatch had to call `f` again on every lookup — measured at 29-35% of
  ;; the rete fuzzer's generator time (GEN-VIGILIA L2). `f` may be arbitrarily
  ;; expensive: in that fuzzer it builds a whole `record` of six generators.
  ;;
  ;; MEASURED 2026-08-26, release, MEAN OF SIX, minus the ~325ms stdlib bootstrap,
  ;; with a REBUILD on each side (the stdlib is `include_str!`'d — editing this file
  ;; without rebuilding measures the OLD binary and reads as "no difference"):
  ;;   the rete fuzzer's own space, card 1260:  ~437 -> ~287 us/point  (-34%)
  ;;   depth-4 nested bind, 10,000 points:      ~14.7s -> ~1.98s       (~8.7x)
  ;; The nested case is where it compounded: every level re-ran every level below it.
  ;; Against `coords` on the identical 10,000-point space (~38us/point), nested `bind`
  ;; was ~38x and is now ~4.3x. `coords` is still the right shape for a FIXED product;
  ;; `bind` is for spaces a product cannot express.
  ;;
  ;; ⚠ MEAN OF SIX, AND THAT CORRECTION IS PART OF THE RECORD. These figures were
  ;; first published from 3-sample medians as `~406 -> ~265` and `~10.7x`. On a
  ;; ~700ms measurement whose run-to-run spread is ~5%, three samples cannot resolve
  ;; a 10% difference — two successive 3-sample reads of the SAME binary gave 265 and
  ;; 288, and a "regression" chased on that basis did not exist. The direction and
  ;; the -34% were right; the absolutes and the nested multiple were not.
  ;;
  ;; AND ONCE THE BRANCHES ARE MATERIALISED, `bind` IS `one-of`. It dispatches an
  ;; index across a vector of generators by subtracting cardinalities — which is
  ;; exactly, and only, what `one-of` does. That fold used to be written out twice
  ;; here, with its own `BindPick` twin of `Pick` (`solvere`).
  ;;
  ;; ⚠ THIS DEDUP WAS A REGRESSION UNTIL THE LINE ABOVE MADE IT FREE, and the order
  ;; matters. `temperare` warned specifically against collapsing `bind` onto `one-of`
  ;; BECAUSE it would materialise every branch when `f` might be cheap. That warning
  ;; was correct against the old body. The `f`-runs-once fix materialises every branch
  ;; anyway — so the cost temperare priced is now already paid, and what is left is
  ;; one dispatch instead of two. Re-measured after the collapse to confirm it is free,
  ;; not assumed.
  (:wat::core::let [ga-at (:wat::gen::Gen/at ga)]
    (:wat::gen::one-of
      (:wat::core::into (:wat::core::PersistentVector)
        (:wat::core::mapv
          (:wat::core::fn [i <- :wat::core::i64] -> (:wat::gen::Gen :- [B])
            (f (ga-at i)))
          (:wat::core::range 0 (:wat::gen::Gen/card ga)))))))

;; ── bounded collections ──────────────────────────────────────────────────────
;;
;; `vector-of g n` is a FIXED-length vector: card(g)^n, which is `coords` over n
;; uniform bases with each digit read through `g`. Nothing new is needed for it —
;; it is the coordinate model applied to itself.
;;
;; `vector-upto g lo hi` is VARIABLE length, and that one genuinely needs `bind`:
;; the element space depends on a generated length. Its card is the SUM over
;; lengths, so short vectors are enumerated before long ones and a failing index
;; still names a length.
(:wat::core::defn :wat::gen::vector-of :- [T]
  [g <- (:wat::gen::Gen :- [T])  n <- :wat::core::i64]
  -> (:wat::gen::Gen :- [(:wat::core::PersistentVector :- [T])])
  (:wat::core::let
    [c     (:wat::gen::Gen/card g)
     at    (:wat::gen::Gen/at g)
     bases (:wat::core::into (:wat::core::PersistentVector)
             (:wat::core::mapv
               (:wat::core::fn [i <- :wat::core::i64] -> :wat::core::i64 c)
               (:wat::core::range 0 n)))
     coords (:wat::gen::coords bases)
     cat    (:wat::gen::Gen/at coords)]
    (:wat::gen::gen
      (:wat::gen::Gen/card coords)
      (:wat::core::fn [k <- :wat::core::i64] -> (:wat::core::PersistentVector :- [T])
            (:wat::core::into (:wat::core::PersistentVector)
              (:wat::core::mapv at (cat k)))))))

(:wat::core::defn :wat::gen::vector-upto :- [T]
  [g <- (:wat::gen::Gen :- [T])  lo <- :wat::core::i64  hi <- :wat::core::i64]
  -> (:wat::gen::Gen :- [(:wat::core::PersistentVector :- [T])])
  (:wat::gen::bind (:wat::gen::ints lo (:wat::core::i64::+ hi 1))
    (:wat::core::fn [n <- :wat::core::i64]
                    -> (:wat::gen::Gen :- [(:wat::core::PersistentVector :- [T])])
      (:wat::gen::vector-of g n))))
