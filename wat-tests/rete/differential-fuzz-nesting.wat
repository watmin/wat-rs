;; wat-tests/rete/differential-fuzz-nesting.wat — NESTED COMBINATORS, over every world.
;;
;; RETE-OPEN-WORK 1.4. The shape fuzzer's filter families are FLAT — `:not` of a fact, `:or` across
;; conditions, `:not` of a constraint. Composition is where boolean engines actually break, and
;; `:not (:and …)`, `:not (:or …)`, `:or (:and …)`, `:not (:not …)` existed only as hand-written
;; grid axes (`where-not-and`, `where-not-or`, `where-not-not`, `where-or-and`) — one world each.
;;
;; THE SPACE IS A TRUTH TABLE, which is what makes this cheap AND exhaustive: every shape against
;; EVERY world. Three fact classes, present or absent, is 8 worlds; 8 shapes over them is 64 cases
;; that cover the boolean semantics completely rather than sampling them. A hand-written axis pins
;; one row of one shape's table.
;;
;; THE LEAVES BIND NOTHING, deliberately. A bind inside a `:not` must be consumed there
;; (`src/rete/validate.rs`'s wall, added earlier this arc), and a bare `(:Class)` negation is the
;; form that says "this class is absent" without dragging a dead variable along. It also makes the
;; readout a clean 0/1 per case: a query whose only condition is the combinator either activates or
;; does not, so the row count IS the truth value — the one place in these fuzzers where a count is
;; the right instrument rather than a blind one.

(:wat::core::defrecord :wat-tests::rete::nest::A [k <- :wat::core::i64])
(:wat::core::defrecord :wat-tests::rete::nest::B [k <- :wat::core::i64])
(:wat::core::defrecord :wat-tests::rete::nest::C [k <- :wat::core::i64])

;; ── the shapes ───────────────────────────────────────────────────────────────
;; Chosen so each is a DIFFERENT composition, not a different spelling of one:
;;   0 not(and)   1 not(or)    2 or(and,·)  3 and(or,·)
;;   4 not(not)   5 not(and(·,not))  6 or(·,and(·,not))  7 and(not,not)
;; 5 and 6 nest a `:not` INSIDE another combinator, which is the arrangement flat families cannot
;; reach at all: a negation whose truth is consumed by an enclosing boolean rather than by the rule.
(:wat::core::defn :wat-tests::rete::nest::shape [i <- :wat::core::i64] -> :wat::WatAST
  (:wat::core::cond
    ((:wat::core::= i 0) (:wat::core::quasiquote
      (:wat::rete::not (:wat::rete::and (:wat-tests::rete::nest::A) (:wat-tests::rete::nest::B)))))
    ((:wat::core::= i 1) (:wat::core::quasiquote
      (:wat::rete::not (:wat::rete::or (:wat-tests::rete::nest::A) (:wat-tests::rete::nest::B)))))
    ((:wat::core::= i 2) (:wat::core::quasiquote
      (:wat::rete::or (:wat::rete::and (:wat-tests::rete::nest::A) (:wat-tests::rete::nest::B))
                      (:wat-tests::rete::nest::C))))
    ((:wat::core::= i 3) (:wat::core::quasiquote
      (:wat::rete::and (:wat::rete::or (:wat-tests::rete::nest::A) (:wat-tests::rete::nest::B))
                       (:wat-tests::rete::nest::C))))
    ((:wat::core::= i 4) (:wat::core::quasiquote
      (:wat::rete::not (:wat::rete::not (:wat-tests::rete::nest::A)))))
    ((:wat::core::= i 5) (:wat::core::quasiquote
      (:wat::rete::not (:wat::rete::and (:wat-tests::rete::nest::A)
                                        (:wat::rete::not (:wat-tests::rete::nest::B))))))
    ((:wat::core::= i 6) (:wat::core::quasiquote
      (:wat::rete::or (:wat-tests::rete::nest::A)
                      (:wat::rete::and (:wat-tests::rete::nest::B)
                                       (:wat::rete::not (:wat-tests::rete::nest::C))))))
    (:else (:wat::core::quasiquote
      (:wat::rete::and (:wat::rete::not (:wat-tests::rete::nest::A))
                       (:wat::rete::not (:wat-tests::rete::nest::B)))))))

(:wat::core::defn :wat-tests::rete::nest::n-shapes [] -> :wat::core::i64 8)

;; ── the worlds ───────────────────────────────────────────────────────────────
;; `world` is a 3-bit presence mask: bit0 A, bit1 B, bit2 C. All 8, so the truth table is complete.
(:wat::core::defn :wat-tests::rete::nest::has [world <- :wat::core::i64  bit <- :wat::core::i64]
  -> :wat::core::bool
  (:wat::core::= 1 (:wat::core::i64::rem (:wat::core::i64::quot world bit) 2)))

(:wat::core::defn :wat-tests::rete::nest::seed
  [world <- :wat::core::i64  q <- :wat::rete::Query] -> :wat::rete::Session
  (:wat::core::let
    [s0 (:wat::rete::compile-all (:wat::core::PersistentVector) (:wat::core::PersistentVector q))
     s1 (:wat::core::if (:wat-tests::rete::nest::has world 1)
          (:wat::core::match (:wat::rete::insert s0 (:wat-tests::rete::nest::A :k 1)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))) s0)
     s2 (:wat::core::if (:wat-tests::rete::nest::has world 2)
          (:wat::core::match (:wat::rete::insert s1 (:wat-tests::rete::nest::B :k 1)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))) s1)]
    (:wat::core::if (:wat-tests::rete::nest::has world 4)
      (:wat::core::match (:wat::rete::insert s2 (:wat-tests::rete::nest::C :k 1)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))) s2)))

;; ── the case, and the readout ────────────────────────────────────────────────
(:wat::core::defrecord :wat-tests::rete::nest::Case
  [shape <- :wat::core::i64   ;; which composition
   world <- :wat::core::i64]) ;; 3-bit presence mask over A, B, C

;; A query whose ONLY condition is the combinator: it activates or it does not, so the row count is
;; the truth value. Built fresh per case because the shape is the query.
(:wat::core::defn :wat-tests::rete::nest::query-for [shape <- :wat::core::i64] -> :wat::rete::Query
  (:wat::rete::Query :name "q" :params (:wat::core::PersistentVector)
    :lhs (:wat::core::PersistentVector (:wat-tests::rete::nest::shape shape))))

(:wat::core::defn :wat-tests::rete::nest::rows
  [c <- :wat-tests::rete::nest::Case  oracle? <- :wat::core::bool] -> :wat::core::i64
  (:wat::core::let [q  (:wat-tests::rete::nest::query-for (:wat-tests::rete::nest::Case/shape c))
                    st (:wat-tests::rete::nest::seed (:wat-tests::rete::nest::Case/world c) q)]
    (:wat::core::length
      (:wat::rete::query
        (:wat::core::if oracle?
          (:wat::core::match (:wat::rete::fire-rules$oracle st) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
          (:wat::core::match (:wat::rete::fire-rules st) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))))
        q))))

(:wat::core::defn :wat-tests::rete::nest::prop [c <- :wat-tests::rete::nest::Case] -> :wat::core::bool
  (:wat::core::= (:wat-tests::rete::nest::rows c false)
                 (:wat-tests::rete::nest::rows c true)))

(:wat::core::defn :wat-tests::rete::nest::space []
  -> (:wat::gen::Gen :- [:wat-tests::rete::nest::Case])
  (:wat::gen::record :wat-tests::rete::nest::Case
    (:wat::gen::ints 0 (:wat-tests::rete::nest::n-shapes))
    (:wat::gen::ints 0 8)))

;; ── the gates ────────────────────────────────────────────────────────────────
(:wat::test::time-limit "60s")
(:wat::test::deftest :wat-tests::rete::nest::test-native-matches-oracle-on-nested-combinators
  (:wat::core::match (:wat::gen::check (:wat-tests::rete::nest::space) :wat-tests::rete::nest::prop)
    ((:wat::gen::CheckOutcome::Checked cases bad _first)
      (:wat::core::let [_ (:wat::test::assert-true (:wat::core::> cases 0))]
        (:wat::test::assert-eq bad 0)))
    (:wat::gen::CheckOutcome::EmptySpace (:wat::test::assert-true false))))

;; NON-VACUITY, and for a truth table it has a sharp form: EVERY shape must CHANGE ITS MIND across
;; the worlds. A shape that answers the same in all 8 is either a tautology, a contradiction, or a
;; composition the engine collapsed — and all three make its 8 cases agree with the oracle for a
;; reason that has nothing to do with nesting. This is the check that would catch, say, a `:not` of
;; an `:and` being silently treated as an `:and` of `:not`s only in the cases where they coincide.
(:wat::core::defn :wat-tests::rete::nest::varies [shape <- :wat::core::i64] -> :wat::core::bool
  (:wat::core::let
    [rows (:wat::core::mapv
            (:wat::core::fn [w <- :wat::core::i64] -> :wat::core::i64
              (:wat-tests::rete::nest::rows
                (:wat-tests::rete::nest::Case :shape shape :world w) false))
            (:wat::core::range 0 8))
     lo (:wat::core::foldl
          (:wat::core::fn [a <- :wat::core::i64  b <- :wat::core::i64] -> :wat::core::i64
            (:wat::core::if (:wat::core::< a b) a b))
          99 rows)
     hi (:wat::core::foldl
          (:wat::core::fn [a <- :wat::core::i64  b <- :wat::core::i64] -> :wat::core::i64
            (:wat::core::if (:wat::core::> a b) a b))
          -1 rows)]
    (:wat::core::not (:wat::core::= lo hi))))

(:wat::test::time-limit "60s")
(:wat::test::deftest :wat-tests::rete::nest::test-every-shape-changes-its-mind
  (:wat::core::let
    [bad (:wat::core::foldl
           (:wat::core::fn [acc <- :wat::core::i64  i <- :wat::core::i64] -> :wat::core::i64
             (:wat::core::if (:wat-tests::rete::nest::varies i)
               acc
               (:wat::core::i64::+ acc 1)))
           0
           (:wat::core::range 0 (:wat-tests::rete::nest::n-shapes)))]
    ;; Every one of the 8 compositions must be sensitive to the world it runs in.
    (:wat::test::assert-eq bad 0)))
