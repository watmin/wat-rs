;; BRIEF-cond-the-first-macro-backed-rete-row.md — scorecard rows 2/3/4/5/7/9, ordinary
;; (non-`where`) evaluation. Row 6 (composes in a real `defrule`'s `where`) is a SEPARATE probe
;; (`probe-cond-rete-where.wat`) — grounding proved a `where` clause is never macro-expanded, so
;; it exercises a genuinely different mechanism than this file's calls.
;;
;; Row 2 — the tier ladder, read from wat/core.wat:1237 before writing this clause (paren clauses,
;; terminal (:else body), exactly core's syntax) — spelled with the RETE alias throughout.
(:wat::core::defn :probe::tier-score [tier <- :wat::core::keyword] -> :wat::core::f64
  (:wat::rete::core::cond
    ((:wat::rete::core::keyword::= tier :gold)   0.5)
    ((:wat::rete::core::keyword::= tier :silver) 0.7)
    (:else                                        0.9)))

;; Row 3 — first-match wins: TWO true tests, reorder so the earlier one should win. If :silver's
;; arm (first, always true here since both tests use the same tier) fires ahead of :gold's arm
;; (second, would also match — 1 = 1 always true), the "first" should be taken.
(:wat::core::defn :probe::first-match [x <- :wat::core::i64] -> :wat::core::String
  (:wat::rete::core::cond
    ((:wat::i64::= x x) "first")
    ((:wat::i64::= x x) "second")
    (:else                    "else")))

;; Row 4 — :else terminal fires when every test is false.
(:wat::core::defn :probe::else-fires [x <- :wat::core::i64] -> :wat::core::String
  (:wat::rete::core::cond
    ((:wat::i64::> x 1000000) "huge")
    (:else                          "normal")))

;; Row 5 — non-exhaustive cond (no terminal :else) is a LOCATED macro-expansion error, not a
;; silent nil. Exercised by :user::main catching the StartupError via a dedicated binary path is
;; overkill for a scratch probe — instead this arm is commented OUT of the live program and
;; verified separately (see the report — a non-exhaustive rete cond is written to its own file and
;; run through --check, expecting a macro-error naming "cond: non-exhaustive").

;; Row 7 — core's cond, UNCHANGED, spelled with the CORE (non-rete) name, same shapes.
(:wat::core::defn :probe::core-tier-score [tier <- :wat::core::keyword] -> :wat::core::f64
  (:wat::core::cond
    ((:wat::core::= tier :gold)   0.5)
    ((:wat::core::= tier :silver) 0.7)
    (:else                        0.9)))

;; Row 9 — every other rete family unregressed: one Alias (i64::>), one Fallback (i64::+ with
;; :undefined), one other Form (if), one Redispatch (foldl).
(:wat::core::defn :probe::alias-check [] -> :wat::core::bool
  (:wat::rete::i64::> 5 3))

(:wat::core::defn :probe::fallback-check [] -> :wat::core::i64
  (:wat::rete::i64::+ 1 2 :undefined -1))

(:wat::core::defn :probe::form-if-check [] -> :wat::core::i64
  (:wat::rete::core::if true 10 20))

(:wat::core::defn :probe::redispatch-check [] -> :wat::core::i64
  (:wat::rete::core::foldl
    (:wat::core::fn [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::i64 (:wat::i64::+ a b))
    0
    [1 2 3 4]))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:wat::string::concat "row2 gold="   (:wat::core::str (:probe::tier-score :gold))))
    (:wat::kernel::println (:wat::string::concat "row2 silver=" (:wat::core::str (:probe::tier-score :silver))))
    (:wat::kernel::println (:wat::string::concat "row2 other="  (:wat::core::str (:probe::tier-score :bronze))))
    (:wat::kernel::println (:wat::string::concat "row3 first="  (:probe::first-match 5)))
    (:wat::kernel::println (:wat::string::concat "row4 else="   (:probe::else-fires 5)))
    (:wat::kernel::println (:wat::string::concat "row7 core-silver=" (:wat::core::str (:probe::core-tier-score :silver))))
    (:wat::kernel::println (:wat::string::concat "row9 alias="  (:wat::core::str (:probe::alias-check))))
    (:wat::kernel::println (:wat::string::concat "row9 fallback=" (:wat::core::str (:probe::fallback-check))))
    (:wat::kernel::println (:wat::string::concat "row9 form-if=" (:wat::core::str (:probe::form-if-check))))
    (:wat::kernel::println (:wat::string::concat "row9 redispatch=" (:wat::core::str (:probe::redispatch-check))))))
