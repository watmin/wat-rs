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
;;   1 inline-gt      (k+2) > 100        inline  => k >= 99        => 111/210
;;   2 fence-gt       (k+2) > 100        fence   => identical      => 111/210
;;   3 inline-eq      (k+2) = 100        inline  => k = 98 exactly => 1/210
;;   4 fence-eq       (k+2) = 100        fence   => identical      => 1/210
;;   5 inline-let-gt  (let [x k] x) > 100  inline  => k >= 101     => 109/210
;;   6 fence-let-gt   (let [x k] x) > 100  fence   => identical    => 109/210
;;   7 inline-let-eq  (let [x k] x) = 100  inline  => k = 100      => 1/210
;;   8 fence-let-eq   (let [x k] x) = 100  fence   => identical    => 1/210
;;   9 inline-cond    cond as the HEAD     inline  => k > 100      => 109/210
;;  10 fence-cond     identical            fence   => identical    => 109/210
;;  11 inline-let-head let as the HEAD     inline  => k > 100      => 109/210
;;  12 fence-let-head identical            fence   => identical    => 109/210
;;  13 inline-match   match as the HEAD    inline  => k = 100      => 1/210
;;  14 fence-match    identical            fence   => identical    => 1/210
;;
;; Rows 3/4 and 7/8 are the sharp ones and are not redundant with their `>` partners: n=1 is
;; bracketed on BOTH sides, so a silent never-match (n=0, entry F's signature) and an always-match
;; (n=210, the opposite failure a naive fix could introduce) are each one row away from the truth.
;; A row that can only be wrong in one direction is half an instrument.
;;
;; ── WHY ROWS 5-8 EXIST — A SECOND DEFECT OF THE SAME CLASS, FOUND 2026-08-28 ──────────────────
;;
;; Rows 1-4 put the field reference inside a nested CALL. Rows 5-8 put it inside a `[...]` — a
;; `let` binder — and that was a SEPARATE live silent never-match, still open on the day entry F
;; was declared closed. `bind_field_refs` walked only `Keyword` and `List` and ended in
;; `other => clone`, so `WatAST::Vector` was cloned untouched, `:k` never became a slot read, and
;; the rule compiled, fired and matched nothing.
;;
;; ── WHY ROWS 9-14 EXIST — THREE FORMS WE HAD DENIED OURSELVES ────────────────────────
;;
;; Rows 1-8 put the field reference in a nested position. Rows 9-14 put `cond`, `let` and `match`
;; in the HEAD position of an inline constraint, where all three were REFUSED until 2026-08-28 —
;; on the stated ground that they are "polymorphic in their body's type and the inline position has
;; no type check that could demand bool of them."
;;
;; That ground was wrong twice over. Polymorphic-in-the-body means the type is a FUNCTION of the
;; body, and the body is right there in the AST. And `cond` was not failing a type test at all: the
;; macro expander descended into `where` bodies ONLY, so an inline `cond` was never expanded to
;; nested `if` and reached the compiler as a head with no lowering arm. The builder's ruling:
;; *"we very carefully crafted rete's DSL to ensure every form a user can express can be
;; compiled... we just inappropriately denied access, poorly, to tooling we fully intended to
;; support."*
;;
;; Clara has all three natively — `cond`, `let`, and `case` for a match on a boolean — so these
;; rows are a real comparison, not a claim about our own two engines.
;;
;; ⛔ **AND `$oracle` AND `$native` WERE BOTH WRONG AGAIN.** `matcher.rs`'s `rewrite_field_refs`
;; carried the identical two arms and the identical catch-all. Two engines agreeing is not
;; evidence when the thing they agree on is the bug — that is the repeat failure mode of this
;; arc, and it is why these rows exist HERE, against Clara, rather than only as a Rust probe.
;; Clara is the only party to this comparison that did not inherit our mistake.

(:wat::core::defn :wic::items [] -> :wat::core::i64 210)

(:wat::core::defn :wic::row-count [] -> :wat::core::i64 14)

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

;; ROW 5 — INLINE, the field reference inside a `let` BINDER VECTOR. `[x :k]` is a
;; `WatAST::Vector`, which the rewriter's `other => clone` catch-all passed through untouched:
;; `:k` stayed a bare keyword, compared unequal to every i64, and this row read n=0 in BOTH
;; engines while every gate was green.
(:wat::rete::defrule :wic::inline-let-gt
  :when [(:wic::Req (?k <- :k)
           (:wat::rete::core::i64::> (:wat::rete::core::let [x :k] x) 100))]
  :then [(:wic::Hit :k ?k)])

;; ROW 6 — FENCE, the identical predicate. This position always worked, which is exactly what made
;; row 5's silence invisible: the same expression answered correctly two lines away.
(:wat::rete::defrule :wic::fence-let-gt
  :when [(:wic::Req (?k <- :k))
         (:wat::rete::where (:wat::rete::core::i64::> (:wat::rete::core::let [x ?k] x) 100))]
  :then [(:wic::Hit :k ?k)])

;; ROW 7 — INLINE, exact equality. n=1 brackets a never-match and an always-match from both sides.
(:wat::rete::defrule :wic::inline-let-eq
  :when [(:wic::Req (?k <- :k)
           (:wat::rete::core::i64::= (:wat::rete::core::let [x :k] x) 100))]
  :then [(:wic::Hit :k ?k)])

;; ROW 8 — FENCE, exact equality.
(:wat::rete::defrule :wic::fence-let-eq
  :when [(:wic::Req (?k <- :k))
         (:wat::rete::where (:wat::rete::core::i64::= (:wat::rete::core::let [x ?k] x) 100))]
  :then [(:wic::Hit :k ?k)])

;; ROW 9 — `cond` as the inline HEAD. Refused until 2026-08-28 with `"alpha 0 cond did not
;; compile"` — a refusal that named nothing, because the expander never reached this position.
(:wat::rete::defrule :wic::inline-cond
  :when [(:wic::Req (?k <- :k)
           (:wat::rete::core::cond ((:wat::rete::core::i64::> :k 100) true) (:else false)))]
  :then [(:wic::Hit :k ?k)])

;; ROW 10 — FENCE. `cond` ALWAYS worked here, which is precisely how the inline refusal survived.
(:wat::rete::defrule :wic::fence-cond
  :when [(:wic::Req (?k <- :k))
         (:wat::rete::where
           (:wat::rete::core::cond ((:wat::rete::core::i64::> ?k 100) true) (:else false)))]
  :then [(:wic::Hit :k ?k)])

;; ROW 11 — `let` as the inline HEAD. Provably bool because its BODY is.
(:wat::rete::defrule :wic::inline-let-head
  :when [(:wic::Req (?k <- :k)
           (:wat::rete::core::let [x :k] (:wat::rete::core::i64::> x 100)))]
  :then [(:wic::Hit :k ?k)])

;; ROW 12 — FENCE.
(:wat::rete::defrule :wic::fence-let-head
  :when [(:wic::Req (?k <- :k))
         (:wat::rete::where (:wat::rete::core::let [x ?k] (:wat::rete::core::i64::> x 100)))]
  :then [(:wic::Hit :k ?k)])

;; ROW 13 — `match` as the inline HEAD, on `= 100` so n=1 brackets it from both sides.
(:wat::rete::defrule :wic::inline-match
  :when [(:wic::Req (?k <- :k)
           (:wat::rete::core::match (:wat::rete::core::i64::= :k 100) (true true) (false false)))]
  :then [(:wic::Hit :k ?k)])

;; ROW 14 — FENCE.
(:wat::rete::defrule :wic::fence-match
  :when [(:wic::Req (?k <- :k))
         (:wat::rete::where
           (:wat::rete::core::match (:wat::rete::core::i64::= ?k 100) (true true) (false false)))]
  :then [(:wic::Hit :k ?k)])

(:wat::rete::defquery :wic::q-Hit :params [] :when [(?fact <- :wic::Hit)])

(:wat::core::defn :wic::rule-for [row <- :wat::core::i64] -> :wat::core::String
  (:wat::core::cond
    ((:wat::core::= row 1) "inline-gt")
    ((:wat::core::= row 2) "fence-gt")
    ((:wat::core::= row 3) "inline-eq")
    ((:wat::core::= row 4) "fence-eq")
    ((:wat::core::= row 5) "inline-let-gt")
    ((:wat::core::= row 6) "fence-let-gt")
    ((:wat::core::= row 7) "inline-let-eq")
    ((:wat::core::= row 8) "fence-let-eq")
    ((:wat::core::= row 9) "inline-cond")
    ((:wat::core::= row 10) "fence-cond")
    ((:wat::core::= row 11) "inline-let-head")
    ((:wat::core::= row 12) "fence-let-head")
    ((:wat::core::= row 13) "inline-match")
    (:else "fence-match")))

(:wat::core::defn :wic::rules-for [row <- :wat::core::i64]
  -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::PersistentVector
    (:wat::core::cond
      ((:wat::core::= row 1) (:wic::inline-gt))
      ((:wat::core::= row 2) (:wic::fence-gt))
      ((:wat::core::= row 3) (:wic::inline-eq))
      ((:wat::core::= row 4) (:wic::fence-eq))
      ((:wat::core::= row 5) (:wic::inline-let-gt))
      ((:wat::core::= row 6) (:wic::fence-let-gt))
      ((:wat::core::= row 7) (:wic::inline-let-eq))
      ((:wat::core::= row 8) (:wic::fence-let-eq))
      ((:wat::core::= row 9) (:wic::inline-cond))
      ((:wat::core::= row 10) (:wic::fence-cond))
      ((:wat::core::= row 11) (:wic::inline-let-head))
      ((:wat::core::= row 12) (:wic::fence-let-head))
      ((:wat::core::= row 13) (:wic::inline-match))
      (:else (:wic::fence-match)))))

(:wat::core::defn :wic::seed [session <- :wat::rete::Session  items <- :wat::core::i64]
  -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert-all
    session
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  i <- :wat::core::i64]
                      -> (:wat::core::PersistentVector :- [:wat::core::Record])
        (:wat::core::PersistentVector/conj acc (:wic::Req :k i)))
      (:wat::core::PersistentVector)
      (:wat::core::range 0 items))) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))

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
                    fired   (:wat::core::match (:wat::rete::fire-rules staged) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
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
