;; wat-tests/rete/differential-fuzz.wat — the rete differential fuzzer, in wat.
;;
;; THE PROPERTY: for every generated rule/query shape, `fire-rules` (native) and
;; `fire-rules$oracle` (the wat reference) must return the SAME number of rows.
;; Nothing here hardcodes a right answer — the oracle supplies every expected
;; value, which is what lets the space grow without hand-authoring per case.
;;
;; WHY QUERY ROWS AND NOT DERIVED FACTS. `production_delta` dedups derived facts
;; by value, so a rule whose `:then` binds one variable derives the SAME fact
;; however many tokens reach it. A differential over derived-fact counts reads
;; identically on a correct engine and on one that multiplies tokens — which is
;; exactly how a leading-`:not`/`:exists` defect survived the whole arc, and why
;; 37 of the 57 queries in the where-family corpus cannot see it. Every query
;; here carries the RULE'S OWN LHS, so `query` reads beta, below the dedup.
;;
;; SHAPE-SPACE, NOT FACT-VOLUME. The oracle is superlinear — measured, ~O(n²):
;; 31 facts 11.5ms, 136 facts 81ms, 556 facts 0.96s, 2236 facts 15.3s. So cases
;; stay at a handful of facts and the yield comes from shape diversity. A join or
;; negation defect shows at 3 facts exactly as at 3000. The floor is not 1 fact
;; though: telling 1 from N needs >= 2 of a class, and rounds need a chain.
;;
;; Rules and queries are assembled as `Rule` / `Query` VALUES from a pool of
;; quasiquoted forms — no source-text generation, no external tool. A case is a
;; COORDINATE, so a finding reproduces forever from its tuple.

;; `:wat::gen::` is STDLIB as of 2026-08-25 — no load-file! needed.

(:wat::core::defrecord :wat-tests::rete::fuzz::W  [k <- :wat::core::i64])
(:wat::core::defrecord :wat-tests::rete::fuzz::G  [k <- :wat::core::i64])
(:wat::core::defrecord :wat-tests::rete::fuzz::P1 [k <- :wat::core::i64])
(:wat::core::defrecord :wat-tests::rete::fuzz::P2 [k <- :wat::core::i64])
(:wat::core::defrecord :wat-tests::rete::fuzz::S1 [k <- :wat::core::i64])
(:wat::core::defrecord :wat-tests::rete::fuzz::S2 [k <- :wat::core::i64])
(:wat::core::defrecord :wat-tests::rete::fuzz::S3 [k <- :wat::core::i64])
(:wat::core::defrecord :wat-tests::rete::fuzz::S4 [k <- :wat::core::i64])

(:wat::core::defn :wat-tests::rete::fuzz::no-conds [] -> (:wat::core::PersistentVector :- [:wat::WatAST])
  (:wat::core::PersistentVector))

;; ── the chain pool: depth D = first D links, driving D+1 fixpoint rounds ──────
;; Round depth is the highest-yield dimension we have: every pre-existing leading
;; filter test fired ONE round, where "once per fire" and "once per round" are
;; the same number, which is precisely why the defect hid.
(:wat::core::defn :wat-tests::rete::fuzz::chain [d <- :wat::core::i64]
  -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::into (:wat::core::PersistentVector)
    (:wat::core::take
      (:wat::core::PersistentVector
        (:wat::rete::Rule :name "r1"
          :lhs (:wat::core::PersistentVector (:wat::core::quasiquote (:wat-tests::rete::fuzz::S1 (?k <- :k))))
          :rhs (:wat::core::PersistentVector (:wat::core::quasiquote (:wat-tests::rete::fuzz::S2 ?k))))
        (:wat::rete::Rule :name "r2"
          :lhs (:wat::core::PersistentVector (:wat::core::quasiquote (:wat-tests::rete::fuzz::S2 (?k <- :k))))
          :rhs (:wat::core::PersistentVector (:wat::core::quasiquote (:wat-tests::rete::fuzz::S3 ?k))))
        (:wat::rete::Rule :name "r3"
          :lhs (:wat::core::PersistentVector (:wat::core::quasiquote (:wat-tests::rete::fuzz::S3 (?k <- :k))))
          :rhs (:wat::core::PersistentVector (:wat::core::quasiquote (:wat-tests::rete::fuzz::S4 ?k)))))
      d)))

;; ── condition pool ───────────────────────────────────────────────────────────
(:wat::core::defn :wat-tests::rete::fuzz::prefix-conds [n <- :wat::core::i64]
  -> (:wat::core::PersistentVector :- [:wat::WatAST])
  (:wat::core::if (:wat::core::= n 0)
    (:wat-tests::rete::fuzz::no-conds)
    (:wat::core::if (:wat::core::= n 1)
      (:wat::core::PersistentVector (:wat::core::quasiquote (:wat-tests::rete::fuzz::P1 (?a <- :k))))
      (:wat::core::PersistentVector
        (:wat::core::quasiquote (:wat-tests::rete::fuzz::P1 (?a <- :k)))
        (:wat::core::quasiquote (:wat-tests::rete::fuzz::P2 (?b <- :k)))))))

;; THE FILTER POOL — each entry is a VECTOR of conditions, because some shapes are
;; more than one LHS element (accumulate carries its own threshold `where`).
;;
;;   0 none · 1 exists · 2 not · 3 accumulate+threshold · 4 intra-condition :or
;;   5 intra-condition :not-of-a-constraint · 6 top-level :or ACROSS conditions
;;   7 :not over a DERIVED class (stratified negation)
;;
;; 3 and 4 are the widening. The accumulate threshold is `>= 2` deliberately, so
;; the answer VARIES with the `dups` dimension instead of being vacuously true —
;; a gate that cannot change its mind measures nothing. And 4 is the THIRD `or`
;; engine in rete (top-level-across-conditions, where-expression, and
;; intra-condition); the intra-condition one had no corpus at all until
;; `where-or-inline`, and reaching `Op::Or`/`Op::Not` is exactly what that found.
(:wat::core::defn :wat-tests::rete::fuzz::filt-cond [f <- :wat::core::i64  fp <- :wat::core::i64]
  -> (:wat::core::PersistentVector :- [:wat::WatAST])
  (:wat::core::if (:wat::core::= f 0)
    (:wat-tests::rete::fuzz::no-conds)
    (:wat::core::if (:wat::core::= f 1)
      (:wat::core::PersistentVector (:wat::core::quasiquote (:wat::rete::exists (:wat-tests::rete::fuzz::W (?w <- :k)))))
      (:wat::core::if (:wat::core::= f 2)
        (:wat::core::PersistentVector (:wat::core::quasiquote (:wat::rete::not (:wat-tests::rete::fuzz::G (?g <- :k)))))
        (:wat::core::if (:wat::core::= f 3)
          (:wat::core::PersistentVector
            (:wat::core::quasiquote (?n <- (:wat::rete::acc::count) :from (:wat-tests::rete::fuzz::W)))
            ;; PARAMETERIZED: the threshold is generated, not hardcoded, so the
            ;; gate genuinely changes its mind across the space.
            (:wat::core::quasiquote
              (:wat::rete::where (:wat::rete::core::i64::>= ?n (:wat::core::unquote fp)))))
          (:wat::core::if (:wat::core::= f 4)
            (:wat::core::PersistentVector
              (:wat::core::quasiquote
                (:wat-tests::rete::fuzz::W (?w <- :k)
                  (:wat::rete::or (:wat::rete::core::i64::> ?w (:wat::core::unquote fp))
                                  (:wat::rete::core::i64::< ?w 3)))))
            (:wat::core::if (:wat::core::= f 5)
              ;; 5 — intra-condition :not of a CONSTRAINT (not of a condition).
              (:wat::core::PersistentVector
                (:wat::core::quasiquote
                  (:wat-tests::rete::fuzz::W (?w <- :k)
                    (:wat::rete::not (:wat::rete::core::i64::> ?w 100)))))
              (:wat::core::if (:wat::core::= f 6)
                ;; 6 — top-level :or ACROSS conditions: network branches, the
                ;; first of rete's three `or` engines, and the only one that
                ;; binds a DIFFERENT variable per branch.
                (:wat::core::PersistentVector
                  (:wat::core::quasiquote
                    (:wat::rete::or (:wat-tests::rete::fuzz::P1 (?a <- :k))
                                    (:wat-tests::rete::fuzz::W (?w <- :k)))))
                ;; 7 — :not over a DERIVED class. STRATIFICATION: S2 exists only
                ;; because the chain derives it, so the answer must depend on the
                ;; depth dimension. This is where family C lives.
                (:wat::core::PersistentVector
                  (:wat::core::quasiquote
                    (:wat::rete::not (:wat-tests::rete::fuzz::S2 (?s <- :k)))))))))))))

;; A CONSTANT predicate: POSITION is the variable under test here. A `where`
;; naming a variable bound LATER is a compile-time question, not a differential
;; one, and is deliberately NOT generated — a case that fails to compile would
;; take the whole batch with it. That gap is real and stated, not hidden.
(:wat::core::defn :wat-tests::rete::fuzz::where-cond [] -> (:wat::core::PersistentVector :- [:wat::WatAST])
  (:wat::core::PersistentVector
    (:wat::core::quasiquote (:wat::rete::where (:wat::rete::core::i64::> 1 0)))))

;; 0 none · 1 first · 2 last
(:wat::core::defn :wat-tests::rete::fuzz::build-lhs
  [prefix <- :wat::core::i64  f <- :wat::core::i64  fp <- :wat::core::i64  wpos <- :wat::core::i64]
  -> (:wat::core::PersistentVector :- [:wat::WatAST])
  (:wat::core::let [head (:wat::core::if (:wat::core::= wpos 1) (:wat-tests::rete::fuzz::where-cond) (:wat-tests::rete::fuzz::no-conds))
                    tail (:wat::core::if (:wat::core::= wpos 2) (:wat-tests::rete::fuzz::where-cond) (:wat-tests::rete::fuzz::no-conds))
                    body (:wat::core::PersistentVector/concat
                           (:wat-tests::rete::fuzz::prefix-conds prefix) (:wat-tests::rete::fuzz::filt-cond f fp))]
    (:wat::core::PersistentVector/concat
      (:wat::core::PersistentVector/concat head body) tail)))

;; THE CASE IS A RECORD, not a bare coordinate — built by `gen-record` over five
;; `gen-ints`. The property then reads `(:wat-tests::rete::fuzz::Case/depth c)` instead of
;; `(i64s c 4)`, so a dimension cannot be silently transposed by a reader, and
;; adding a dimension is a field rather than an index everyone must re-count.
(:wat::core::defrecord :wat-tests::rete::fuzz::Case
  [dups   <- :wat::core::i64
   wpos   <- :wat::core::i64
   prefix <- :wat::core::i64
   filt   <- :wat::core::i64
   fparam <- :wat::core::i64
   depth  <- :wat::core::i64])

;; ── the property ─────────────────────────────────────────────────────────────
(:wat::core::defn :wat-tests::rete::fuzz::prop [c <- :wat-tests::rete::fuzz::Case] -> :wat::core::bool
  (:wat::core::let [dups   (:wat::core::i64::+ (:wat-tests::rete::fuzz::Case/dups c) 1)
                    wpos   (:wat-tests::rete::fuzz::Case/wpos c)
                    prefix (:wat-tests::rete::fuzz::Case/prefix c)
                    f      (:wat-tests::rete::fuzz::Case/filt c)
                    fp     (:wat-tests::rete::fuzz::Case/fparam c)
                    d      (:wat-tests::rete::fuzz::Case/depth c)
                    q  (:wat::rete::Query :name "q" :params (:wat::core::PersistentVector)
                         :lhs (:wat-tests::rete::fuzz::build-lhs prefix f fp wpos))
                    s0 (:wat::rete::compile-all (:wat-tests::rete::fuzz::chain d) (:wat::core::PersistentVector q))
                    ws (:wat::core::into (:wat::core::PersistentVector)
                         (:wat::core::mapv
                           (:wat::core::fn [i <- :wat::core::i64] -> :wat-tests::rete::fuzz::W (:wat-tests::rete::fuzz::W 7))
                           (:wat::core::range 0 dups)))
                    s1 (:wat::rete::insert-all s0 ws)
                    s2 (:wat::rete::insert-all s1 (:wat::core::PersistentVector (:wat-tests::rete::fuzz::P1 1)))
                    s3 (:wat::rete::insert-all s2 (:wat::core::PersistentVector (:wat-tests::rete::fuzz::P2 1)))
                    st (:wat::rete::insert-all s3 (:wat::core::PersistentVector (:wat-tests::rete::fuzz::S1 1)))
                    nf (:wat::rete::fire-rules st)
                    of (:wat::rete::fire-rules$oracle st)
                    n  (:wat::core::length (:wat::rete::query nf q))
                    o  (:wat::core::length (:wat::rete::query of q))]
    ;; NO println here, deliberately: a deftest body runs before stdio services
    ;; are up, and more importantly a test should report through its ASSERTION, not
    ;; a side channel. The count is what the gate pins; WHICH coordinates diverge is
    ;; recoverable at any time because the space is finite, ordered and
    ;; deterministic — re-running with a printing `prop` is a one-line change, and
    ;; a coordinate is a permanent case name rather than a seed.
    (:wat::core::= n o)))


;; ── the SPACE excludes unmatchable shapes; the property no longer has to ─────
;; A case with no filter AND no fact condition leaves nothing to match on. That
;; used to be an `if` guarding the property body, plus a second pure enumeration
;; counting how many coordinates were real. Both are gone: `gen-such-that` makes
;; the exclusion part of the GENERATOR, so `Gen/card` IS the case count.
;;
;; This is the shape `test.check`'s `such-that` cannot have. There, filtering an
;; opaque random source means retry-and-discard — it can give up after N tries and
;; it skews what survives. Here the survivors are computed once and exactly.
(:wat::core::defn :wat-tests::rete::fuzz::shape-is-matchable [c <- :wat-tests::rete::fuzz::Case] -> :wat::core::bool
  (:wat::core::not (:wat::core::and (:wat::core::= (:wat-tests::rete::fuzz::Case/filt c) 0)
                                    (:wat::core::= (:wat-tests::rete::fuzz::Case/prefix c) 0))))

;; ── THE SPACE IS DEPENDENT — `bind`, not a fixed product ────────────────────
;;
;; Before `bind` existed this was a flat `record` over five fixed `ints`, so every
;; shape's parameters had to be HARDCODED — the accumulate threshold always 2, the
;; inline `:or` bound always 5. That was not a modelling choice; it is what a
;; fixed-shape coordinate space can express.
;;
;; `bind` generates the SHAPE first, then a parameter space that DEPENDS on it, so
;; branch cardinalities differ per shape — the thing a product cannot do.
;;
;; ⚠ AND THIS IS WHY `frequency` IS ABSENT AND STAYS ABSENT. In `test.check` it
;; biases a random DRAW. Here every point is visited exactly once, so a weight
;; cannot change what enumeration sees at all — and where it WOULD matter, in a
;; prefix of a sampled order, the bias is already expressible because CARDINALITY
;; IS THE WEIGHT: a shape with a 3-point parameter space occupies three times the
;; indices of a parameterless one, and does so here without anyone asking.
;; `one-of [a a b]` is a 2:1 mix. The combinator would add no expressive power,
;; only a second way to say the same thing.
(:wat::core::defn :wat-tests::rete::fuzz::param-space [f <- :wat::core::i64] -> (:wat::gen::Gen :- [:wat::core::i64])
  (:wat::core::if (:wat::core::= f 3)
    (:wat::gen::ints 1 4)
    (:wat::core::if (:wat::core::= f 4)
      (:wat::gen::ints 0 3)
      (:wat::gen::ints 0 1))))

(:wat::core::defn :wat-tests::rete::fuzz::for-shape [f <- :wat::core::i64] -> (:wat::gen::Gen :- [:wat-tests::rete::fuzz::Case])
  (:wat::gen::record :wat-tests::rete::fuzz::Case
    (:wat::gen::ints 0 3)
    (:wat::gen::ints 0 3)
    (:wat::gen::ints 0 3)
    (:wat::gen::ints f (:wat::core::i64::+ f 1))
    (:wat-tests::rete::fuzz::param-space f)
    (:wat::gen::ints 0 4)))

(:wat::core::defn :wat-tests::rete::fuzz::space [] -> (:wat::gen::Gen :- [:wat-tests::rete::fuzz::Case])
  (:wat::gen::such-that :wat-tests::rete::fuzz::shape-is-matchable
    (:wat::gen::bind (:wat::gen::ints 0 8) :wat-tests::rete::fuzz::for-shape)))

;; ── THE GATE — a RATCHET, not a zero ────────────────────────────────────────
;;
;; 120 real divergences of 1260 shapes, three families, all reproduced in
;; tests/rete/probe_arc278_fuzzer_found_divergences.{rs,wat} and listed in
;; docs/arc/2026/06/278-rules-engine/RETE-FIX-LIST.md:
;;
;;    66  f=3  accumulate  A leading accum: native = depth+1, oracle = 1
;;                         B second `where`: native = 0,      oracle = 1
;;    54  f=7  :not over a DERIVED class
;;                         C native = 1, oracle = 0 — ALL at depth >= 1, never
;;                           depth 0: exactly the dependence stratified negation
;;                           should have.
;;
;; Asserting 0 would redden the floor and block unrelated work; deleting the
;; accumulate shape to keep a gate green is the trade this codebase refuses. So
;; the count is PINNED, and movement either way demands an explanation. FEWER
;; means a fix landed — lower this and un-ignore the matching probe. MORE means a
;; new divergence, and each MISMATCH line names its coordinate: a permanent case
;; name, not a seed.
;;
;; An EMPTY space fails outright — a shape-space filtered to nothing means the run
;; tested NOTHING, which must never read as "no divergences found".
;; THE BUDGET, AND THE RUNNER THAT HAS TO GRANT IT. The default deftest budget is
;; 5000ms. This run takes ~9.2s alone and 17.175s on a loaded floor
;; (.floor/2026-08-26T06-16-14Z) — 1260 shapes, each firing BOTH engines. The
;; budget is raised rather than the space shrunk: cutting shapes to fit a timer
;; trades coverage for a green clock, and every shape here has either found a
;; defect or proved one absent.
;;
;; ⚠ RAISING IT HERE IS ONLY HALF. Until 2026-08-26 this annotation was
;; UNREACHABLE and nothing said so. `scripts/floor.sh` passes no `--profile`, so
;; nextest's `[profile.default]` SIGTERMed at 30s — half the budget argued for
;; above — and the rete cohort's 60s/120s override missed this test because it
;; filters `binary_id(wat::rete)` while a wat deftest compiles into `wat::kernel`.
;; `.config/nextest.toml` now names `test(deftest_wat_tests_rete_fuzz)` in all
;; three profile mirrors. If this test is ever renamed, THAT FILTER MUST MOVE WITH
;; IT or the 30s kill returns silently.
;;
;; ⛔ NO ORACLE RATIO IS QUOTED, DELIBERATELY. This comment used to say the
;; `$oracle` is "~300x the cost of everything else here (measured: ~7ms/case
;; against the library's ~23us/point)". The library figure was a `coords`
;; measurement; this file's space is `such-that o bind o record`, measured
;; 2026-08-26 at ~410us/point — ~17x dearer. What is true and stays true: the
;; generator is a SMALL fraction of a case (~0.41ms of the floor's 13.6ms/case,
;; ~3%) and the engines-plus-oracle are the rest. The RATIO itself is not worth
;; pinning: the `$oracle` is slow-but-correct by design, carries no perf
;; requirement, and gets passively faster as wat stops being interpreted — so any
;; multiple against it shrinks on its own, and the generator's SHARE grows.
(:wat::test::time-limit "60s")
(:wat::test::deftest :wat-tests::rete::fuzz::test-native-matches-oracle
  (:wat::core::match
    (:wat::gen::check (:wat-tests::rete::fuzz::space) :wat-tests::rete::fuzz::prop)
    ((:wat::gen::CheckOutcome::Checked cases bad _first)
      (:wat::core::let [_ (:wat::test::assert-true (:wat::core::> cases 0))]
        (:wat::test::assert-eq bad 120)))
    (:wat::gen::CheckOutcome::EmptySpace (:wat::test::assert-true false))))
