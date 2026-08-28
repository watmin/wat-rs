;; where-inline-computed — THE POSITION AXIS. The grid was blind to this and it cost a live defect.
;;
;; ── WHY THIS AXIS EXISTS ──────────────────────────────────────────────────────────────────────
;;
;; Every one of the other 36 `where-*` axes writes its predicate in a `(:wat::rete::where …)`
;; FENCE. Measured 2026-08-28: the wat side of the whole grid contained ZERO inline constraints.
;; So the corpus measured ONE of rete's two predicate positions, on every axis, forever — and
;; fix-list entry F lived in the other one: an inline constraint whose operand was a nested call
;; compiled to a permanent, SILENT never-match. Accepted at every gate, fired, matched nothing,
;; exit 0, no diagnostic. 39 of 77 vocabulary rows wide.
;;
;; A shape corpus that only ever spells things one way cannot see a defect in the other spelling,
;; no matter how many shapes it holds. THAT is the hole this axis closes, and the row pairs below
;; are the instrument: each predicate appears TWICE, once per position, and both must agree with
;; the one answer Clara gives.
;;
;; ── WHY CLARA CAN ARBITRATE IT ────────────────────────────────────────────────────────────────
;;
;; Clara has both positions too, which is what makes this checkable rather than merely self-
;; consistent: a predicate written INSIDE the bracket — `[Req (> (+ k 2) 100)]` — is its inline
;; constraint, and `[:test …]` is its fence. The corpus already leans hard on `:test` (128 rows)
;; and barely uses the in-pattern form (7). So the .clj twin mirrors the pairing exactly, and a
;; disagreement between OUR two positions shows up as a diff against ONE Clara answer rather than
;; as two of our rows quietly agreeing with each other.
;;
;; ── THE ROWS ──────────────────────────────────────────────────────────────────────────────────
;;
;; k ranges 0..209. The predicate always contains a COMPUTED operand — `(k + 2)` — because that is
;; the exact shape `compile_operand_expr` could not lower.
;;
;;   1 inline-gt   (k+2) > 100  inline  => k >= 99          => 111/210
;;   2 fence-gt    (k+2) > 100  fence   => identical         => 111/210
;;   3 inline-eq   (k+2) = 100  inline  => k = 98 exactly    => 1/210
;;   4 fence-eq    (k+2) = 100  fence   => identical         => 1/210
;;
;; Rows 3 and 4 are the sharp ones and are not redundant with 1 and 2: n=1 is bracketed on BOTH
;; sides, so a silent never-match (n=0, entry F's signature) and an always-match (n=210, the
;; opposite failure a naive fix could introduce) are each one row away from the truth. A row that
;; can only be wrong in one direction is half an instrument.

(:wat::core::defn :wic::items [] -> :wat::core::i64 210)

(:wat::core::defn :wic::row-count [] -> :wat::core::i64 4)

(:wat::core::defrecord :wic::Req [k <- :wat::core::i64])
(:wat::core::defrecord :wic::Hit [k <- :wat::core::i64])

;; ROW 1 — INLINE, computed operand. The position and shape that were silently broken.
(:wat::rete::defrule :wic::inline-gt
  :when [(:wic::Req (?k <- :k)
           (:wat::rete::core::i64::> (:wat::rete::core::i64::+ :k 2 :undefined 0) 100))]
  :then [(:wic::Hit :k ?k)])

;; ROW 2 — FENCE, the identical predicate. This position always worked; it is the control that
;; makes row 1 a comparison rather than an assertion.
(:wat::rete::defrule :wic::fence-gt
  :when [(:wic::Req (?k <- :k))
         (:wat::rete::where (:wat::rete::core::i64::> (:wat::rete::core::i64::+ ?k 2 :undefined 0) 100))]
  :then [(:wic::Hit :k ?k)])

;; ROW 3 — INLINE, exact equality. n=1: brackets the answer from both sides.
(:wat::rete::defrule :wic::inline-eq
  :when [(:wic::Req (?k <- :k)
           (:wat::rete::core::i64::= (:wat::rete::core::i64::+ :k 2 :undefined 0) 100))]
  :then [(:wic::Hit :k ?k)])

;; ROW 4 — FENCE, exact equality.
(:wat::rete::defrule :wic::fence-eq
  :when [(:wic::Req (?k <- :k))
         (:wat::rete::where (:wat::rete::core::i64::= (:wat::rete::core::i64::+ ?k 2 :undefined 0) 100))]
  :then [(:wic::Hit :k ?k)])

(:wat::rete::defquery :wic::q-Hit :params [] :when [(?fact <- :wic::Hit)])

(:wat::core::defn :wic::rule-for [row <- :wat::core::i64] -> :wat::core::String
  (:wat::core::cond
    ((:wat::core::= row 1) "inline-gt")
    ((:wat::core::= row 2) "fence-gt")
    ((:wat::core::= row 3) "inline-eq")
    (:else "fence-eq")))

(:wat::core::defn :wic::rules-for [row <- :wat::core::i64]
  -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::PersistentVector
    (:wat::core::cond
      ((:wat::core::= row 1) (:wic::inline-gt))
      ((:wat::core::= row 2) (:wic::fence-gt))
      ((:wat::core::= row 3) (:wic::inline-eq))
      (:else (:wic::fence-eq)))))

(:wat::core::defn :wic::seed [session <- :wat::rete::Session  items <- :wat::core::i64]
  -> :wat::rete::Session
  (:wat::rete::insert-all
    session
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  i <- :wat::core::i64]
                      -> (:wat::core::PersistentVector :- [:wat::core::Record])
        (:wat::core::PersistentVector/conj acc (:wic::Req :k i)))
      (:wat::core::PersistentVector)
      (:wat::core::range 0 items))))

(:wat::core::defn :wic::derived-ints [fired <- :wat::rete::Session]
  -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::sort
    (:wat::core::into (:wat::core::Vector :wat::core::i64)
      (:wat::core::map
        (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64
          (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")]
            (:wic::Hit/k f)))
        (:wat::rete::query fired (:wic::q-Hit))))))

(:wat::core::defn :wic::render-ints [v <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String  x <- :wat::core::i64] -> :wat::core::String
      (:wat::core::String/concat acc
        (:wat::core::String/concat " " (:wat::core::i64::to-string x))))
    ""
    v))

(:wat::core::defn :wic::run-row [row <- :wat::core::i64] -> :wat::core::String
  (:wat::core::let [rules   (:wic::rules-for row)
                    staged  (:wic::seed (:wat::rete::compile-all rules (:wat::core::PersistentVector (:wic::q-Hit))) (:wic::items))
                    fired   (:wat::rete::fire-rules staged)
                    derived (:wic::derived-ints fired)
                    n       (:wat::core::Vector/length derived)]
    (:wat::core::String/concat
      (:wat::core::String/concat
        (:wat::core::String/concat "row " (:wat::core::i64::to-string row))
        (:wat::core::String/concat " " (:wic::rule-for row)))
      (:wat::core::String/concat
        (:wat::core::String/concat " n=" (:wat::core::i64::to-string n))
        (:wat::core::String/concat " ->" (:wic::render-ints derived))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::nil  row <- :wat::core::i64] -> :wat::core::nil
      (:wat::kernel::println (:wic::run-row row)))
    nil
    (:wat::core::range 1 (:wat::core::i64::+ (:wic::row-count) 1))))
