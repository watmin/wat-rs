;; probe-where-shape-spread.wat — the EXPRESSIVITY SURFACE, not our corpus.
;;
;; ORIGIN (builder, 2026-08-01): "do not optimize for our code - optimize for the users we have not
;; encountered yet - i do not trust your results when you only consider what code has been written
;; so far." Every number in DESIGN-STONE-compiled-where's Step 0 came from ONE predicate —
;; node-share's `(= 7 (- ?k (* (/ ?k 50) 50)))`, four i64 ops over one bound variable. "The walk is
;; 77%", "the env build is 23%", "arm F is 138 ns" are all statements about THAT expression.
;;
;; This fixture is the correction. Eight rules over ONE fact stream, each diverging only at its
;; trailing `where`, and the eight `where`s span what a USER IS ALLOWED TO WRITE rather than what we
;; happened to write: arithmetic, record accessor, nested accessor, string op, collection op, map
;; op, user-fn call, multi-variable, boolean/negation, deep nesting.
;;
;; The rete harness reads each TestNode's predicate out of a real fire and runs the decomposition
;; arms PER SHAPE, so the table says where the env build dominates, where the walk dominates, and —
;; the row that actually decides the design — WHICH SHAPES THE VERB-AGNOSTIC EXECUTOR CAN COMPILE
;; AT ALL. A shape it cannot compile is not a failure; it is coverage data, and it must be visible.
;;
;; Every rule shares the SAME leading conditions, so every TestNode sees the same token stream and
;; the same binding count — the only variable across rows is the predicate's own shape.

(:wat::core::defrecord :shape::Geo
  [country <- :wat::core::String])

(:wat::core::defrecord :shape::Client
  [id   <- :wat::core::i64
   geo  <- :shape::Geo
   rep  <- :wat::core::i64])

(:wat::core::defrecord :shape::Req
  [k       <- :wat::core::i64
   a       <- :wat::core::i64
   b       <- :wat::core::i64
   name    <- :wat::core::String
   flag    <- :wat::core::bool
   tags    <- (:wat::core::PersistentVector :- [:wat::core::i64])
   attrs   <- (:wat::core::PersistentMap :- [:wat::core::String :wat::core::i64])
   client  <- :shape::Client])

(:wat::core::defrecord :shape::Hit [k <- :wat::core::i64])

(:wat::rete::defquery :shape::q-Hit
  :params []
  :when [(?fact <- :shape::Hit)])


;; A user-defined predicate fn — the shape a compiled executor CANNOT model and must hand back to
;; the interpreter. Its presence in the spread is the point: users write these.
(:wat::core::defn :shape::big? [n <- :wat::core::i64] -> :wat::core::bool
  (:wat::i64::> n 4))

;; ── the NINE shapes ───────────────────────────────────────────────────────────────────────
;; Built the way node-share builds its rules — quasiquoted LHS/RHS forms folded into a `Rule`
;; record. Written as a live `defrule` the checker reads each condition as a CONSTRUCTOR call
;; and each `:then` form as a live `insert` (it taught me both, one located error each);
;; quoted forms are the shape that composes.

;; arithmetic (4 i64 ops, 1 var) — the shape EVERY Step-0 number came from
(:wat::core::defn :shape::rule-arith [] -> :wat::rete::Rule
  (:wat::core::let [conds   (:wat::core::quasiquote (:shape::Req (?k <- :k)))
                    where-c (:wat::core::quasiquote (:wat::rete::where (:wat::core::= 3 (:wat::i64::- ?k (:wat::i64::* (:wat::i64::/ ?k 10) 10)))))
                    ins     (:wat::core::quasiquote (:shape::Hit ?k))]
    (:wat::rete::Rule :name "arith"
      :lhs (:wat::core::PersistentVector conds where-c)
      :rhs (:wat::core::PersistentVector ins))))

;; record accessor, one level — the corpus's commonest non-arithmetic shape
(:wat::core::defn :shape::rule-accessor [] -> :wat::rete::Rule
  (:wat::core::let [conds   (:wat::core::quasiquote (:shape::Req (?k <- :k) (?c <- :client)))
                    where-c (:wat::core::quasiquote (:wat::rete::where (:wat::rete::i64::> (:shape::Client/rep ?c) 0)))
                    ins     (:wat::core::quasiquote (:shape::Hit ?k))]
    (:wat::rete::Rule :name "accessor"
      :lhs (:wat::core::PersistentVector conds where-c)
      :rhs (:wat::core::PersistentVector ins))))

;; accessor over accessor — the arena's Geo/country (Client/geo ?c)
(:wat::core::defn :shape::rule-accessor-nested [] -> :wat::rete::Rule
  (:wat::core::let [conds   (:wat::core::quasiquote (:shape::Req (?k <- :k) (?c <- :client)))
                    where-c (:wat::core::quasiquote (:wat::rete::where (:wat::core::= (:shape::Geo/country (:shape::Client/geo ?c)) "XX")))
                    ins     (:wat::core::quasiquote (:shape::Hit ?k))]
    (:wat::rete::Rule :name "accessor-nested"
      :lhs (:wat::core::PersistentVector conds where-c)
      :rhs (:wat::core::PersistentVector ins))))

;; a stdlib String verb over a String binding
(:wat::core::defn :shape::rule-string [] -> :wat::rete::Rule
  (:wat::core::let [conds   (:wat::core::quasiquote (:shape::Req (?k <- :k) (?n <- :name)))
                    where-c (:wat::core::quasiquote (:wat::rete::where (:wat::rete::string::starts-with? ?n "ad")))
                    ins     (:wat::core::quasiquote (:shape::Hit ?k))]
    (:wat::rete::Rule :name "string"
      :lhs (:wat::core::PersistentVector conds where-c)
      :rhs (:wat::core::PersistentVector ins))))

;; a PersistentVector verb over a collection binding
(:wat::core::defn :shape::rule-collection [] -> :wat::rete::Rule
  (:wat::core::let [conds   (:wat::core::quasiquote (:shape::Req (?k <- :k) (?t <- :tags)))
                    where-c (:wat::core::quasiquote (:wat::rete::where (:wat::rete::i64::> (:wat::rete::vector::length ?t) 1)))
                    ins     (:wat::core::quasiquote (:shape::Hit ?k))]
    (:wat::rete::Rule :name "collection"
      :lhs (:wat::core::PersistentVector conds where-c)
      :rhs (:wat::core::PersistentVector ins))))

;; a PersistentMap verb — the HAMT lookup path
(:wat::core::defn :shape::rule-map [] -> :wat::rete::Rule
  (:wat::core::let [conds   (:wat::core::quasiquote (:shape::Req (?k <- :k) (?m <- :attrs)))
                    where-c (:wat::core::quasiquote (:wat::rete::where (:wat::rete::map::contains-key? ?m "hot")))
                    ins     (:wat::core::quasiquote (:shape::Hit ?k))]
    (:wat::rete::Rule :name "map"
      :lhs (:wat::core::PersistentVector conds where-c)
      :rhs (:wat::core::PersistentVector ins))))

;; a USER-DEFINED fn — no compiler can model this; it must fall back
(:wat::core::defn :shape::rule-userfn [] -> :wat::rete::Rule
  (:wat::core::let [conds   (:wat::core::quasiquote (:shape::Req (?k <- :k)))
                    where-c (:wat::core::quasiquote (:wat::rete::where (:shape::big? ?k)))
                    ins     (:wat::core::quasiquote (:shape::Hit ?k))]
    (:wat::rete::Rule :name "userfn"
      :lhs (:wat::core::PersistentVector conds where-c)
      :rhs (:wat::core::PersistentVector ins))))

;; 3 bound vars, 5 levels — separates per-EVALUATION cost from per-NODE cost
(:wat::core::defn :shape::rule-multivar-deep [] -> :wat::rete::Rule
  (:wat::core::let [conds   (:wat::core::quasiquote (:shape::Req (?k <- :k) (?a <- :a) (?b <- :b)))
                    where-c (:wat::core::quasiquote (:wat::rete::where (:wat::rete::i64::> (:wat::i64::+ ?a (:wat::i64::* ?b (:wat::i64::- ?k (:wat::i64::/ (:wat::i64::+ ?a ?b) 2)))) 0)))
                    ins     (:wat::core::quasiquote (:shape::Hit ?k))]
    (:wat::rete::Rule :name "multivar-deep"
      :lhs (:wat::core::PersistentVector conds where-c)
      :rhs (:wat::core::PersistentVector ins))))

;; a bare bound bool under `not` — the cheapest predicate a user can write
(:wat::core::defn :shape::rule-bool [] -> :wat::rete::Rule
  (:wat::core::let [conds   (:wat::core::quasiquote (:shape::Req (?k <- :k) (?f <- :flag)))
                    where-c (:wat::core::quasiquote (:wat::rete::where (:wat::rete::core::not ?f)))
                    ins     (:wat::core::quasiquote (:shape::Hit ?k))]
    (:wat::rete::Rule :name "bool"
      :lhs (:wat::core::PersistentVector conds where-c)
      :rhs (:wat::core::PersistentVector ins))))

(:wat::core::defn :shape::build-rules [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::PersistentVector
    (:shape::rule-arith)
    (:shape::rule-accessor)
    (:shape::rule-accessor-nested)
    (:shape::rule-string)
    (:shape::rule-collection)
    (:shape::rule-map)
    (:shape::rule-userfn)
    (:shape::rule-multivar-deep)
    (:shape::rule-bool)
  ))

;; seed n — stage n Req facts. Values are chosen so each predicate has a MIX of pass and fail
;; across the stream: a predicate that is constantly true or constantly false would let the branch
;; predictor flatter one shape over another, and the row would be an artifact.
(:wat::core::defn :shape::seed
  [session <- :wat::rete::Session  n <- :wat::core::i64] -> :wat::rete::Session
  (:wat::rete::insert-all
    session
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  i <- :wat::core::i64]
                      -> (:wat::core::PersistentVector :- [:wat::core::Record])
        (:wat::vector::conj acc
          (:shape::Req
            :k      i
            :a      (:wat::i64::- i 1)
            :b      (:wat::i64::+ i 2)
            :name   (:wat::core::if (:wat::core::= 0 (:wat::i64::- i (:wat::i64::* (:wat::i64::/ i 2) 2))) "admin" "guest")
            :flag   (:wat::core::= 0 (:wat::i64::- i (:wat::i64::* (:wat::i64::/ i 3) 3)))
            :tags   (:wat::core::PersistentVector i (:wat::i64::+ i 1))
            :attrs  (:wat::core::PersistentMap "hot" i)
            :client (:shape::Client
                      :id  i
                      :geo (:shape::Geo :country (:wat::core::if (:wat::core::= 0 (:wat::i64::- i (:wat::i64::* (:wat::i64::/ i 5) 5))) "XX" "US"))
                      :rep (:wat::i64::- i 5)))))
      (:wat::core::PersistentVector)
      (:wat::core::range 0 n))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::i64::to-string
      (:wat::core::length
        (:wat::rete::query
          (:wat::rete::fire-rules
            (:shape::seed
              (:wat::rete::compile-all
                (:shape::build-rules)
                (:wat::core::PersistentVector (:shape::q-Hit)))
              50))
          (:shape::q-Hit))))))
