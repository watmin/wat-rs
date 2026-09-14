;; Twin of where-not-derived-in-query.wat. `:not` over a DERIVED class, in a QUERY.
;;
;; THE ARBITRATION THIS FILE PROVIDED. The rete fuzzer found 54 divergences of one shape
;; (RETE-FIX-LIST entry C): a query whose only condition is `:not` over a class that exists ONLY
;; because a rule derived it. The wat `$oracle` blocked the query; native passed it. Everything
;; else on that list is native-is-wrong by the arc's standing rule — C was the one entry where the
;; two references could LEGITIMATELY disagree, because if a `defquery`'s negation is deliberately
;; NOT stratified the way a `defrule`'s is, the ORACLE would have been the wrong one and 54 of the
;; ratchet's 72 would not have been defects at all.
;;
;; Clara 0.24.0 decided it, 2026-08-26: row 2 and row 4 print the SAME number. A query's negation
;; IS stratified exactly the way a rule's is. The oracle was right, native was wrong, and the
;; arc's standing rule held.
;;
;;   row 1  query-no-chain   `:not S2`, S2 never derived     — the agreeing control
;;   row 2  query-chain-d1   `:not S2`, S2 derived           — THE RULING
;;   row 3  query-chain-d2   `:not S3`, S3 derived at depth 2 — the defect was ALL at depth >= 1,
;;                                                              so one level is not enough to hold
;;   row 4  rule-chain       the SAME negation in a defrule  — the contrast, in the output
;;   row 5  control-S2       is S2 actually derived at all   — without it, row 2 is unreadable
;;
;; WHY THE CORPUS COULD NOT SEE THIS. Every query-side `:not` in the grid negates an INSERTED
;; class (`where-query-compat`'s Wind); every derived-class `:not` lives in a RULE (`strat-neg`,
;; `negation`, `neg-consumer`). No axis crossed the two. That crossing is this file.
;;
;; ⚠ AN ACCEPTANCE DIVERGENCE, FOUND WRITING THIS FILE (2026-08-26). wat's own C fixture writes
;; the negation as `(:wat::rete::not (:user::S2 (?s <- :k)))` — a variable BOUND INSIDE the
;; negation and used nowhere else. wat compiles it. Clara REFUSES it, at compile time:
;;
;;     Using variable that is not previously bound. ... Note that variables used in negations
;;     are not bound for subsequent rules since the negation can never match.
;;     Unbound variables: #{?s}
;;
;; That is a divergence in what the two engines ACCEPT, not in what they compute, which is why
;; both sides here negate BARE. It is recorded in RETE-FIX-LIST as its own entry rather than
;; folded into C, because a fix for one is not a fix for the other.
;;
;;     ./target/release/wat  wat-scripts/perf/grid/where-not-derived-in-query.wat
;;     clojure -Sdeps '{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}}}' \
;;             -M wat-scripts/perf/grid/where-not-derived-in-query.clj

(ns where-not-derived-in-query
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]))

(defrecord S1  [k])
(defrecord S2  [k])
(defrecord S3  [k])
(defrecord Hit [k])

;; S2 and S3 exist ONLY by derivation. Nothing inserts either.
(defrule r1 [S1 (= ?k k)] => (insert! (->S2 ?k)))
(defrule r2 [S2 (= ?k k)] => (insert! (->S3 ?k)))

;; The same negation in a RULE — the contrast row.
(defrule r-not-S2 [:not [S2]] => (insert! (->Hit 1)))

(defquery q-not-S2 [] [:not [S2]])
(defquery q-not-S3 [] [:not [S3]])
(defquery q-Hit    [] [Hit (= ?k k)])
(defquery q-S2     [] [S2 (= ?k k)])

(defn n [rules q]
  (count (query (fire-rules (insert (mk-session (conj rules q) :cache false) (->S1 1))) q)))

(defn -main [& _]
  (prn (str "row 1 query-no-chain n=" (n []        q-not-S2)))
  (prn (str "row 2 query-chain-d1 n=" (n [r1]      q-not-S2)))
  (prn (str "row 3 query-chain-d2 n=" (n [r1 r2]   q-not-S3)))
  (prn (str "row 4 rule-chain n="     (n [r1 r-not-S2] q-Hit)))
  (prn (str "row 5 control-S2 n="     (n [r1]      q-S2))))

(-main)
