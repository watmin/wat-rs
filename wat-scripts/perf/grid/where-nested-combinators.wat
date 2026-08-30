;; wat-scripts/perf/grid/where-nested-combinators.wat — combinators NESTED inside combinators.
;;
;; Twin of where-nested-combinators.clj. THE GAP THIS AXIS HOLDS: the existing combinator axes are
;; each ONE level and ONE world — `where-not-and`, `where-not-or`, `where-not-not`, `where-or-and`.
;; None nests a `:not` INSIDE another combinator, where its truth is consumed by an enclosing
;; boolean rather than by the rule, and none walks the full presence table.
;;
;; Generated exhaustively by `wat-tests/rete/differential-fuzz-nesting.wat` (8 shapes x 8 worlds,
;; native vs `$oracle`). This axis carries THREE of those shapes against Clara, because two wat
;; engines agreeing proves nothing if they share an assumption — which the `:then` kwargs finding
;; (RETE-FIX-LIST entry E) demonstrated the same day: native and `$oracle` transposed identically
;; and agreed perfectly on the wrong answer.
;;
;; Each row is one WORLD: a 3-bit presence mask over A, B, C.
;;
;;     ./target/release/wat  wat-scripts/perf/grid/where-nested-combinators.wat
;;     clojure -Sdeps '{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}}}' \
;;             -M wat-scripts/perf/grid/where-nested-combinators.clj

(:wat::core::defrecord :wnc::A [k <- :wat::core::i64])
(:wat::core::defrecord :wnc::B [k <- :wat::core::i64])
(:wat::core::defrecord :wnc::C [k <- :wat::core::i64])

;; 5 — a `:not` nested inside an `:and`, itself negated.
(:wat::rete::defquery :wnc::q5 :params []
  :when [(:wat::rete::not (:wat::rete::and (:wnc::A) (:wat::rete::not (:wnc::B))))])
;; 6 — an `:and` carrying a `:not`, as one arm of an `:or`. World 3 yields TWO activations.
(:wat::rete::defquery :wnc::q6 :params []
  :when [(:wat::rete::or (:wnc::A) (:wat::rete::and (:wnc::B) (:wat::rete::not (:wnc::C))))])
;; 7 — two negations conjoined.
(:wat::rete::defquery :wnc::q7 :params []
  :when [(:wat::rete::and (:wat::rete::not (:wnc::A)) (:wat::rete::not (:wnc::B)))])

(:wat::core::defn :wnc::has [w <- :wat::core::i64  bit <- :wat::core::i64] -> :wat::core::bool
  (:wat::core::= 1 (:wat::core::i64::rem (:wat::core::i64::quot w bit) 2)))

(:wat::core::defn :wnc::n [w <- :wat::core::i64  q <- :wat::rete::Query] -> :wat::core::i64
  (:wat::core::let
    [s0 (:wat::rete::compile-all (:wat::core::PersistentVector)
          (:wat::core::PersistentVector (:wnc::q5) (:wnc::q6) (:wnc::q7)))
     s1 (:wat::core::if (:wnc::has w 1) (:wat::core::match (:wat::rete::insert s0 (:wnc::A :k 1)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))) s0)
     s2 (:wat::core::if (:wnc::has w 2) (:wat::core::match (:wat::rete::insert s1 (:wnc::B :k 1)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))) s1)
     s3 (:wat::core::if (:wnc::has w 4) (:wat::core::match (:wat::rete::insert s2 (:wnc::C :k 1)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))) s2)]
    (:wat::core::length (:wat::rete::query (:wat::core::match (:wat::rete::fire-rules s3) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))) q))))

(:wat::core::defn :wnc::line [row <- :wat::core::i64 name <- :wat::core::String n <- :wat::core::i64] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::String/concat
      (:wat::core::String/concat "row " (:wat::core::i64::to-string row))
      (:wat::core::String/concat
        (:wat::core::String/concat " " name)
        (:wat::core::String/concat " n=" (:wat::core::i64::to-string n))))))

(:wat::core::defn :wnc::sweep [base <- :wat::core::i64  name <- :wat::core::String  q <- :wat::rete::Query] -> :wat::core::nil
  (:wat::core::foldl
    (:wat::core::fn [_a <- :wat::core::nil  w <- :wat::core::i64] -> :wat::core::nil
      (:wnc::line (:wat::core::i64::+ base w)
        (:wat::core::String/concat name (:wat::core::i64::to-string w))
        (:wnc::n w q)))
    nil (:wat::core::range 0 8)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wnc::sweep 1  "not-and-not-w"  (:wnc::q5))
  (:wnc::sweep 9  "or-and-not-w"   (:wnc::q6))
  (:wnc::sweep 17 "and-not-not-w"  (:wnc::q7)))
