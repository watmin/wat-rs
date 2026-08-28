;; wat-tests/rete/differential-fuzz-rules.wat — the RULE-SET / `:then` differential fuzzer.
;;
;; THE PROPERTY, and it is deliberately NOT a row count. Its three siblings compare how many rows
;; a query returns, which is the right question for shape, type and operation spaces. It is the
;; WRONG question here: a `:then` that writes its kwargs into the wrong fields derives exactly as
;; MANY facts, with the values swapped. A count-based differential reads identically on a correct
;; engine and on one that has transposed every field it wrote.
;;
;; That is not hypothetical — it is the defect arc 294's `defrule` wall exists for: *"the RHS
;; insert form takes kwargs POSITIONALLY with no name-check or reorder. The 9a kwargs codemod
;; corrupted a swath of rule fixtures this way and NOTHING screamed — the floor just showed wrong
;; derived counts."* The wall now REORDERS kwargs to declaration order, and until this file that
;; reorder had only hand-written coverage.
;;
;; So the witness is a VALUE: `sum over rows of (a * 1000 + b)`. Two distinct fields, combined
;; asymmetrically, so a transposition changes the number. Native and `$oracle` must agree on it.
;;
;; WHAT THIS SPACE VARIES that no sibling does:
;;   - `:then` KWARGS ORDER — declaration order vs reversed. The reorder is the thing under test.
;;   - `:then` ARITY — one derived fact or two from a single rule.
;;   - RULE COUNT, with a SHARED first condition — two rules reading the same class is the
;;     node-sharing shape (`node-share` measures it for perf; nothing differentials it), and a
;;     shared alpha with a de-duplicated children list is a documented past hazard in
;;     `fire_rules_stratified`'s own comments.

(:wat::core::defrecord :wat-tests::rete::rules::Src [x <- :wat::core::i64  y <- :wat::core::i64])
;; TWO fields, and the witness combines them asymmetrically — `a` and `b` must be distinguishable
;; or a transposed `:then` is invisible.
(:wat::core::defrecord :wat-tests::rete::rules::Two [a <- :wat::core::i64  b <- :wat::core::i64])
(:wat::core::defrecord :wat-tests::rete::rules::Alt [a <- :wat::core::i64  b <- :wat::core::i64])

;; ── the `:then` forms ────────────────────────────────────────────────────────
;; `ord 0` writes the fields in DECLARATION order; `ord 1` writes them REVERSED. Both must derive
;; the same fact — that is precisely what the wall's reorder promises, and what a positional
;; misread would break.
(:wat::core::defn :wat-tests::rete::rules::then-two [ord <- :wat::core::i64] -> :wat::WatAST
  (:wat::core::if (:wat::core::= ord 0)
    (:wat::core::quasiquote (:wat-tests::rete::rules::Two :a ?x :b ?y))
    (:wat::core::quasiquote (:wat-tests::rete::rules::Two :b ?y :a ?x))))

(:wat::core::defn :wat-tests::rete::rules::then-alt [ord <- :wat::core::i64] -> :wat::WatAST
  (:wat::core::if (:wat::core::= ord 0)
    (:wat::core::quasiquote (:wat-tests::rete::rules::Alt :a ?y :b ?x))
    (:wat::core::quasiquote (:wat-tests::rete::rules::Alt :b ?x :a ?y))))

;; `arity 1` derives one fact; `arity 2` derives a SECOND fact of a different class from the same
;; activation — the multi-fact `:then`, which no sibling generates.
(:wat::core::defn :wat-tests::rete::rules::then-forms
  [ord <- :wat::core::i64  arity <- :wat::core::i64]
  -> (:wat::core::PersistentVector :- [:wat::WatAST])
  (:wat::core::if (:wat::core::= arity 1)
    (:wat::core::PersistentVector (:wat-tests::rete::rules::then-two ord))
    (:wat::core::PersistentVector
      (:wat-tests::rete::rules::then-two ord)
      (:wat-tests::rete::rules::then-alt ord))))

;; ── the rule set ─────────────────────────────────────────────────────────────
;; `nrules 2` adds a SECOND rule reading the SAME `Src` class — a shared alpha. Its `:then` writes
;; the fields the other way round, so the two rules are distinguishable in the witness.
(:wat::core::defn :wat-tests::rete::rules::rule-set
  [ord <- :wat::core::i64  arity <- :wat::core::i64  nrules <- :wat::core::i64]
  -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::let [lhs (:wat::core::PersistentVector
                          (:wat::core::quasiquote
                            (:wat-tests::rete::rules::Src (?x <- :x) (?y <- :y))))
                    r1  (:wat::rete::Rule :name "r1" :lhs lhs
                          :rhs (:wat-tests::rete::rules::then-forms ord arity))
                    r2  (:wat::rete::Rule :name "r2" :lhs lhs
                          :rhs (:wat::core::PersistentVector
                                 (:wat-tests::rete::rules::then-alt
                                   (:wat::i64::rem (:wat::i64::+ ord 1) 2))))]
    (:wat::core::if (:wat::core::= nrules 1)
      (:wat::core::PersistentVector r1)
      (:wat::core::PersistentVector r1 r2))))

;; ── the witness ──────────────────────────────────────────────────────────────
;; `sum (a * 1000 + b)` over the query's rows. ASYMMETRIC on purpose: `a + b` would be blind to a
;; transposition, which is the one defect this file exists to see. 1000 exceeds any generated
;; value, so no carry can alias two different (a,b) pairs onto one witness.
(:wat::rete::defquery :wat-tests::rete::rules::q-two :params []
  :when [(:wat-tests::rete::rules::Two (?a <- :a) (?b <- :b))])

(:wat::rete::defquery :wat-tests::rete::rules::q-alt :params []
  :when [(:wat-tests::rete::rules::Alt (?a <- :a) (?b <- :b))])

;; ── PARAMETERISED queries ────────────────────────────────────────────────────
;; RETE-OPEN-WORK 1.3: every fuzzer before this one declared `:params []`. Params are supplied as
;; KWARGS at the call site — `(query s (q) :?a 1)` — which is the same shape entry E found being
;; consumed positionally on the `:then` side. Probed directly first: a two-param query called in
;; declaration order and REVERSED selects the same row (witness 1002 both ways), so params already
;; resolve by NAME. That is a clean negative, and it is why this dimension varies WHICH value is
;; selected rather than re-testing the ordering.
;;
;; ⚠ AND THE PROBE THAT SETTLED IT NEEDED A VALUE, NOT A COUNT. With `(1,2)` and `(2,1)` both in
;; the world, selecting EITHER returns exactly one row — `n=1` is identical whether the params bound
;; correctly or transposed. The first version of that probe compared counts and would have called
;; a transposition clean.
(:wat::rete::defquery :wat-tests::rete::rules::q-two-at
  :params [?a]
  :when [(:wat-tests::rete::rules::Two (?a <- :a) (?b <- :b))])

(:wat::core::defn :wat-tests::rete::rules::witness-of
  [s <- :wat::rete::Session  q <- :wat::rete::Query] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64  p <- :wat::core::PersistentMap]
      -> :wat::core::i64
      (:wat::core::let [a (:wat::core::Option/expect (:wat::map::get p "?a") "?a")
                        b (:wat::core::Option/expect (:wat::map::get p "?b") "?b")]
        (:wat::i64::+ acc (:wat::i64::+ (:wat::i64::* a 1000) b))))
    0
    (:wat::rete::query s q)))

;; Both classes summed, so a rule set deriving into `Alt` is not invisible to the witness.
(:wat::core::defn :wat-tests::rete::rules::witness [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::i64::+
    (:wat-tests::rete::rules::witness-of s (:wat-tests::rete::rules::q-two))
    (:wat-tests::rete::rules::witness-of s (:wat-tests::rete::rules::q-alt))))

(:wat::core::defrecord :wat-tests::rete::rules::Case
  [ord    <- :wat::core::i64   ;; :then kwargs order — declaration vs reversed
   arity  <- :wat::core::i64   ;; 1 or 2 derived facts per activation
   nrules <- :wat::core::i64   ;; 1, or 2 sharing one alpha
   srcs   <- :wat::core::i64   ;; how many Src facts (distinct x,y per fact)
   qparam <- :wat::core::i64]) ;; 0 unparameterised · 1 param SELECTS a row · 2 param matches NOTHING

(:wat::core::defn :wat-tests::rete::rules::seed
  [c <- :wat-tests::rete::rules::Case] -> :wat::rete::Session
  (:wat::core::let
    [rs (:wat-tests::rete::rules::rule-set
          (:wat-tests::rete::rules::Case/ord c)
          (:wat-tests::rete::rules::Case/arity c)
          (:wat-tests::rete::rules::Case/nrules c))
     s0 (:wat::rete::compile-all rs
          (:wat::core::PersistentVector
            (:wat-tests::rete::rules::q-two) (:wat-tests::rete::rules::q-alt)
            (:wat-tests::rete::rules::q-two-at)))
     ;; x and y DIFFER per fact (i and i+7), so a transposed write moves the witness rather than
     ;; landing on the same number by symmetry.
     facts (:wat::core::into (:wat::core::PersistentVector)
             (:wat::core::mapv
               (:wat::core::fn [i <- :wat::core::i64] -> :wat-tests::rete::rules::Src
                 (:wat-tests::rete::rules::Src :x i :y (:wat::i64::+ i 7)))
               (:wat::core::range 0 (:wat-tests::rete::rules::Case/srcs c))))]
    (:wat::rete::insert-all s0 facts)))

;; The parameterised readout. `qparam 1` selects `?a = 0`, which the first Src (x=0) derives;
;; `qparam 2` selects a value no Src produces, so the correct answer is the EMPTY sum — a row that
;; would hide a param being ignored entirely, since ignoring it returns every row instead of none.
(:wat::core::defn :wat-tests::rete::rules::param-witness
  [s <- :wat::rete::Session  qparam <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64  p <- :wat::core::PersistentMap] -> :wat::core::i64
      (:wat::core::let [a (:wat::core::Option/expect (:wat::map::get p "?a") "?a")
                        b (:wat::core::Option/expect (:wat::map::get p "?b") "?b")]
        (:wat::i64::+ acc (:wat::i64::+ (:wat::i64::* a 1000) b))))
    0
    (:wat::core::if (:wat::core::= qparam 1)
      (:wat::rete::query s (:wat-tests::rete::rules::q-two-at) :?a 0)
      (:wat::rete::query s (:wat-tests::rete::rules::q-two-at) :?a 999))))

(:wat::core::defn :wat-tests::rete::rules::readout
  [s <- :wat::rete::Session  c <- :wat-tests::rete::rules::Case] -> :wat::core::i64
  (:wat::core::let [qp (:wat-tests::rete::rules::Case/qparam c)]
    (:wat::core::if (:wat::core::= qp 0)
      (:wat-tests::rete::rules::witness s)
      (:wat-tests::rete::rules::param-witness s qp))))

(:wat::core::defn :wat-tests::rete::rules::prop [c <- :wat-tests::rete::rules::Case]
  -> :wat::core::bool
  (:wat::core::let [st (:wat-tests::rete::rules::seed c)]
    (:wat::core::= (:wat-tests::rete::rules::readout (:wat::rete::fire-rules st) c)
                   (:wat-tests::rete::rules::readout (:wat::rete::fire-rules$oracle st) c))))

(:wat::core::defn :wat-tests::rete::rules::space []
  -> (:wat::gen::Gen :- [:wat-tests::rete::rules::Case])
  (:wat::gen::record :wat-tests::rete::rules::Case
    (:wat::gen::ints 0 2)
    (:wat::gen::ints 1 3)
    (:wat::gen::ints 1 3)
    (:wat::gen::ints 1 4)
    (:wat::gen::ints 0 3)))

;; ── the gates ────────────────────────────────────────────────────────────────
(:wat::test::time-limit "60s")
(:wat::test::deftest :wat-tests::rete::rules::test-native-matches-oracle-on-then-shapes
  (:wat::core::match (:wat::gen::check (:wat-tests::rete::rules::space) :wat-tests::rete::rules::prop)
    ((:wat::gen::CheckOutcome::Checked cases bad _first)
      (:wat::core::let [_ (:wat::test::assert-true (:wat::core::> cases 0))]
        (:wat::test::assert-eq bad 0)))
    (:wat::gen::CheckOutcome::EmptySpace (:wat::test::assert-true false))))

;; NON-VACUITY, and here it is not optional in the usual way — it is the file's ONLY defence
;; against measuring nothing. The whole design rests on the witness being able to SEE a
;; transposition; if `a*1000+b` were replaced by `a+b` in some future edit, every case would still
;; agree, `violations` would still read 0, and the file would certify an engine that had swapped
;; every field it wrote. So the discrimination is asserted directly rather than assumed.
(:wat::core::defn :wat-tests::rete::rules::witness-of-pair
  [a <- :wat::core::i64  b <- :wat::core::i64] -> :wat::core::i64
  (:wat-tests::rete::rules::witness
    (:wat::rete::fire-rules
      (:wat::rete::insert
        (:wat::rete::compile-all
          (:wat::core::PersistentVector)
          (:wat::core::PersistentVector
            (:wat-tests::rete::rules::q-two) (:wat-tests::rete::rules::q-alt)
            (:wat-tests::rete::rules::q-two-at)))
        (:wat-tests::rete::rules::Two :a a :b b)))))

(:wat::test::time-limit "60s")
(:wat::test::deftest :wat-tests::rete::rules::test-the-witness-can-see-a-transposition
  (:wat::core::let
    [straight (:wat-tests::rete::rules::witness-of-pair 1 8)
     swapped  (:wat-tests::rete::rules::witness-of-pair 8 1)
     ;; The instrument discriminates: the SAME two values in the other fields is a different
     ;; number. Without this row, `a+b` would pass every other assertion in this file.
     _ (:wat::test::assert-true (:wat::core::not (:wat::core::= straight swapped)))
     ;; And it is the asymmetry doing the work, not luck: 1*1000+8 = 1008, 8*1000+1 = 8001.
     _ (:wat::test::assert-eq straight 1008)]
    (:wat::test::assert-eq swapped 8001)))

;; THE REORDER'S OWN PROMISE, as a VALUE rather than as agreement between engines.
;;
;; ⚠ THIS GATE FOUND A LIVE DEFECT ON ITS FIRST RUN, 2026-08-27: it read 3024 vs 24003 — the same
;; three pairs with every field TRANSPOSED. `:then` kwargs in a runtime-built `Rule` were consumed
;; POSITIONALLY, because the reorder lived in the freeze-time `defrule` wall that such a rule never
;; passes. Fixed at the one door (`rete_kwargs_value_asts` now resolves BY NAME); tracked as
;; RETE-FIX-LIST entry E, and gated permanently by
;; `tests/rete/probe_arc278_then_kwargs_positional.rs`.
;;
;; Note what could NOT have caught it. The differential above passes either way — both engines
;; transposed identically, so they agreed perfectly on the wrong answer. A row count is blind too:
;; a transposed `:then` derives exactly as many facts. Only an independent VALUE witness sees it,
;; which is why this file compares one.
(:wat::test::time-limit "60s")
(:wat::test::deftest :wat-tests::rete::rules::test-kwargs-order-does-not-change-the-fact
  (:wat::core::let
    ;; `:qparam 0` — the UNPARAMETERISED readout, deliberately: this gate is about `:then` kwargs
    ;; order, and filtering the rows would let a param bug mask a transposition or vice versa.
    [base (:wat-tests::rete::rules::Case :ord 0 :arity 1 :nrules 1 :srcs 3 :qparam 0)
     rev  (:wat-tests::rete::rules::Case :ord 1 :arity 1 :nrules 1 :srcs 3 :qparam 0)
     w0   (:wat-tests::rete::rules::witness
            (:wat::rete::fire-rules (:wat-tests::rete::rules::seed base)))
     w1   (:wat-tests::rete::rules::witness
            (:wat::rete::fire-rules (:wat-tests::rete::rules::seed rev)))
     ;; Non-vacuous on its own terms: a witness of 0 on both sides would make "equal" meaningless.
     _    (:wat::test::assert-true (:wat::core::> w0 0))
     ;; And PINNED, not merely equal — 3024 is (a=0,b=7) (1,8) (2,9). If both sides drifted to the
     ;; transposed 24003 together, "w0 == w1" would still hold and certify the defect as fixed.
     _    (:wat::test::assert-eq w0 3024)]
    (:wat::test::assert-eq w1 w0)))

;; PARAMS ACTUALLY FILTER — non-vacuity for the `qparam` dimension.
;;
;; The differential above cannot see a param being IGNORED: if `query`'s `:?a` were dropped on the
;; floor, both engines would return every row and agree perfectly. So the filtering is asserted
;; against pinned values rather than inferred from agreement.
;;
;; With srcs=3 the Src facts are (0,7) (1,8) (2,9), so `Two` is (a=0,b=7) (1,8) (2,9):
;;   unparameterised  7 + 1008 + 2009 = 3024   (every row)
;;   :?a 0            0*1000 + 7      =    7   (a PROPER subset — one row)
;;   :?a 999                               0   (no row matches)
;; The 999 row is the load-bearing one: a param that is ignored returns EVERYTHING here, so 0 is
;; the only value that proves it was consulted.
(:wat::test::time-limit "60s")
(:wat::test::deftest :wat-tests::rete::rules::test-query-params-actually-filter
  (:wat::core::let
    [mk (:wat::core::fn [qp <- :wat::core::i64] -> :wat-tests::rete::rules::Case
          (:wat-tests::rete::rules::Case :ord 0 :arity 1 :nrules 1 :srcs 3 :qparam qp))
     fire (:wat::core::fn [c <- :wat-tests::rete::rules::Case] -> :wat::core::i64
            (:wat-tests::rete::rules::readout
              (:wat::rete::fire-rules (:wat-tests::rete::rules::seed c)) c))
     all  (fire (mk 0))
     one  (fire (mk 1))
     none (fire (mk 2))
     _ (:wat::test::assert-eq all 3024)
     ;; A PROPER subset: non-empty, and strictly less than the unfiltered readout. Either half
     ;; alone would pass with the param ignored (which returns `all`) or over-filtering (0).
     _ (:wat::test::assert-true (:wat::core::> one 0))
     _ (:wat::test::assert-true (:wat::core::< one all))
     _ (:wat::test::assert-eq one 7)]
    (:wat::test::assert-eq none 0)))
