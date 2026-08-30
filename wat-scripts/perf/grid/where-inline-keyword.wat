;; where-inline-keyword — KEYWORD AND ENUM CONSTANTS, in both predicate positions.
;;
;; ── WHY THIS AXIS EXISTS ──────────────────────────────────────────────────────────────────────
;;
;; `(keyword::= :tag :alpha)` and `(enum::= :grade :wik::G::Hi)` were REFUSED as inline constraints
;; for the life of the engine, with "`:wik::Req` has no field `:alpha`". A keyword in operand
;; position was read as a FIELD REFERENCE unconditionally, so a keyword or enum CONSTANT could not
;; be written there at all.
;;
;; ⚠ **THE ENGINE WAS ALREADY DECIDING THIS CORRECTLY ONE LEVEL DOWN.** The identical comparison,
;; nested as an operand of another call, fired and answered correctly — because that path runs the
;; same `field_names.position(...)` lookup and falls through to a constant. Two answers to one
;; question, ~120 lines apart in one file. Measured 2026-08-28.
;;
;; The rule is now ONE rule, the one the nested path always used: **a keyword operand is a field
;; reference if it names a declared field; otherwise it is a constant.** Rows 5-6 are the proof
;; that this is purely ADDITIVE — there, the constant's spelling IS a declared field name, and the
;; field must still win.
;;
;; ── THE ROWS ──────────────────────────────────────────────────────────────────────────────────
;;
;; k ranges 0..209. `tag` is :alpha for k < 100, else :beta. `grade` is Hi for k < 60, else Lo.
;;
;;   1 inline-kw    (keyword::= tag :alpha)  inline  => k < 100        => 100/210
;;   2 fence-kw     identical                fence   => identical      => 100/210
;;   3 inline-enum  (enum::= grade Hi)       inline  => k < 60         => 60/210
;;   4 fence-enum   identical                fence   => identical      => 60/210
;;   5 inline-shadow (keyword::= tag :beta)  inline  => `beta` IS A DECLARED FIELD => reads the
;;                                                     FIELD, not the constant    => 110/210
;;   6 fence-shadow  identical               fence   => identical      => 110/210
;;
;; ⛔ ROWS 5-6 ARE THE BACKWARD-COMPATIBILITY PROOF and the sharpest rows here. `beta` is declared
;; as a field holding the same keyword as `tag` for k >= 100. If the new rule read `:beta` as a
;; CONSTANT, rows 5-6 would select k >= 100 (110 rows) by comparing tag to the constant :beta — the
;; SAME count. So the count alone cannot tell the two readings apart, which is exactly why the
;; field is seeded to DIFFER from the constant reading: `beta` holds :alpha for k < 100 and :beta
;; for k >= 100, and `tag` holds the same, so field-reading selects ALL 210 while constant-reading
;; selects 110. One number, two readings, no overlap.

(:wat::core::defenum :wik::G :wat::enum::Pure :Hi :Lo)

(:wat::core::defn :wik::items [] -> :wat::core::i64 210)

(:wat::core::defn :wik::row-count [] -> :wat::core::i64 6)

(:wat::core::defrecord :wik::Req
  [k     <- :wat::core::i64
   tag   <- :wat::core::keyword
   beta  <- :wat::core::keyword
   grade <- :wik::G])
(:wat::core::defrecord :wik::Hit [k <- :wat::core::i64])

;; ROW 1 — INLINE keyword constant. Refused outright until 2026-08-28.
(:wat::rete::defrule :wik::inline-kw
  :when [(:wik::Req (?k <- :k) (:wat::rete::core::keyword::= :tag :alpha))]
  :then [(:wik::Hit :k ?k)])

;; ROW 2 — FENCE. This position always worked, which is how the inline refusal stayed invisible.
(:wat::rete::defrule :wik::fence-kw
  :when [(:wik::Req (?k <- :k) (?t <- :tag))
         (:wat::rete::where (:wat::rete::core::keyword::= ?t :alpha))]
  :then [(:wik::Hit :k ?k)])

;; ROW 3 — INLINE enum constant. `:wik::G::Hi` carries `::` and so could NEVER have been a field
;; name — there was no ambiguity here to resolve, only a question nobody asked.
(:wat::rete::defrule :wik::inline-enum
  :when [(:wik::Req (?k <- :k) (:wat::rete::core::enum::= :grade :wik::G::Hi))]
  :then [(:wik::Hit :k ?k)])

;; ROW 4 — FENCE.
(:wat::rete::defrule :wik::fence-enum
  :when [(:wik::Req (?k <- :k) (?g <- :grade))
         (:wat::rete::where (:wat::rete::core::enum::= ?g :wik::G::Hi))]
  :then [(:wik::Hit :k ?k)])

;; ROW 5 — ⛔ THE FIELD STILL WINS. `:beta` names a declared field, so this compares tag AGAINST
;; THE FIELD `beta` — never against the constant `:beta`. Seeded so the two readings disagree.
(:wat::rete::defrule :wik::inline-shadow
  :when [(:wik::Req (?k <- :k) (:wat::rete::core::keyword::= :tag :beta))]
  :then [(:wik::Hit :k ?k)])

;; ROW 6 — FENCE, the same comparison written with binds.
(:wat::rete::defrule :wik::fence-shadow
  :when [(:wik::Req (?k <- :k) (?t <- :tag) (?b <- :beta))
         (:wat::rete::where (:wat::rete::core::keyword::= ?t ?b))]
  :then [(:wik::Hit :k ?k)])

(:wat::rete::defquery :wik::q-Hit :params [] :when [(?fact <- :wik::Hit)])

(:wat::core::defn :wik::rule-for [row <- :wat::core::i64] -> :wat::core::String
  (:wat::core::cond
    ((:wat::core::= row 1) "inline-kw")
    ((:wat::core::= row 2) "fence-kw")
    ((:wat::core::= row 3) "inline-enum")
    ((:wat::core::= row 4) "fence-enum")
    ((:wat::core::= row 5) "inline-shadow")
    (:else "fence-shadow")))

(:wat::core::defn :wik::rules-for [row <- :wat::core::i64]
  -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::PersistentVector
    (:wat::core::cond
      ((:wat::core::= row 1) (:wik::inline-kw))
      ((:wat::core::= row 2) (:wik::fence-kw))
      ((:wat::core::= row 3) (:wik::inline-enum))
      ((:wat::core::= row 4) (:wik::fence-enum))
      ((:wat::core::= row 5) (:wik::inline-shadow))
      (:else (:wik::fence-shadow)))))

(:wat::core::defn :wik::seed [session <- :wat::rete::Session  items <- :wat::core::i64]
  -> :wat::rete::Session
  (:wat::rete::insert-all
    session
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  i <- :wat::core::i64]
                      -> (:wat::core::PersistentVector :- [:wat::core::Record])
        (:wat::vector::conj acc
          (:wik::Req
            :k i
            :tag   (:wat::core::if (:wat::i64::< i 100) :alpha :beta)
            :beta  (:wat::core::if (:wat::i64::< i 100) :alpha :beta)
            :grade (:wat::core::if (:wat::i64::< i 60) :wik::G::Hi :wik::G::Lo))))
      (:wat::core::PersistentVector)
      (:wat::core::range 0 items))))

(:wat::core::defn :wik::derived-ints [fired <- :wat::rete::Session]
  -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::sort
    (:wat::core::into (:wat::core::Vector :- [:wat::core::i64])
      (:wat::core::map
        (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64
          (:wat::core::let [f (:wat::core::Option/expect (:wat::map::get p "?fact") "query: ?fact")]
            (:wik::Hit/k f)))
        (:wat::rete::query fired (:wik::q-Hit))))))

(:wat::core::defn :wik::render-ints [v <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String  x <- :wat::core::i64] -> :wat::core::String
      (:wat::string::concat acc
        (:wat::string::concat " " (:wat::i64::to-string x))))
    ""
    v))

(:wat::core::defn :wik::run-row [row <- :wat::core::i64] -> :wat::core::String
  (:wat::core::let [rules   (:wik::rules-for row)
                    staged  (:wik::seed (:wat::rete::compile-all rules (:wat::core::PersistentVector (:wik::q-Hit))) (:wik::items))
                    fired   (:wat::rete::fire-rules staged)
                    derived (:wik::derived-ints fired)
                    n       (:wat::vec::length derived)]
    (:wat::string::concat
      (:wat::string::concat
        (:wat::string::concat "row " (:wat::i64::to-string row))
        (:wat::string::concat " " (:wik::rule-for row)))
      (:wat::string::concat
        (:wat::string::concat " n=" (:wat::i64::to-string n))
        (:wat::string::concat " ->" (:wik::render-ints derived))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::nil  row <- :wat::core::i64] -> :wat::core::nil
      (:wat::kernel::println (:wik::run-row row)))
    nil
    (:wat::core::range 1 (:wat::i64::+ (:wik::row-count) 1))))
