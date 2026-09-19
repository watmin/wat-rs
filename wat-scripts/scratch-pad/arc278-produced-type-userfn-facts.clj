;; wat-scripts/scratch-pad/arc278-produced-type-userfn-facts.clj — CLARA REFEREE of
;; arc278-produced-type-userfn-facts.wat. Read that file's header FIRST.
;;
;; This is the instrument for SCORE row 6. A number without this file cannot be re-run.
;;
;; Run (from wat-rs/, or any cwd with this file on the path):
;;   clojure -Sdeps '{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}}}' \
;;           -M wat-scripts/scratch-pad/arc278-produced-type-userfn-facts.clj
;;
;; ── MODELLING POINT 1 — THE USER FN IS INLINED ──────────────────────────────────────────
;;
;; Clara 0.24.0 has no `:then`-head-is-a-fn. wat's `:a2::via` RHS is `(:a2::mk-rate ?k)`,
;; and `:a2::mk-rate` is
;;
;;   (:wat::rete::core::defn :a2::mk-rate [k <- :wat::core::i64] -> :a2::Rate
;;     (:a2::Rate :count k))
;;
;; The body is exactly `(->Rate k)`. The Clara `via` rule inlines that construction as
;; `(insert! (->Rate ?k))`. Nothing else of the fn is dropped: one argument, one record,
;; the bound `k` is the `count` field. If `mk-rate` ever computes, this file is no longer
;; the same rule and must be rewritten, not trusted.
;;
;; ── MODELLING POINT 2 — `Bad` NEVER FIRES ───────────────────────────────────────────────
;;
;; Seed is `Src(1)`. `bad` fires only when `k = 2`. The negation `[:not [Bad (= ?k k)]]`
;; is vacuously true for k=1 — there is no Bad(1) to retract. Clara's Rate is derived for
;; the same reason wat's is: Src(1) exists and Bad(1) does not. The referee is not
;; exercising Clara's negation under retraction; it is exercising the consumer of Rate.
;;
;; Expected print: `CLARA facts [Bad Rate Out]: [0 1 1]`
;; Native `[0 1 1]`. Oracle `[0 1 0]`. This number is which engine is wrong.
;;
;; Self-invokes `(-main)` at load: this is a scratch referee, not a three-way twin.

(ns arc278-produced-type-userfn-facts
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]))

(defrecord Src [k])
(defrecord Bad [k])
(defrecord Rate [count])
(defrecord Out [n])

(defrule bad
  [Src (= ?k k)]
  [:test (= ?k 2)]
  => (insert! (->Bad ?k)))

(defrule via
  [Src (= ?k k)]
  [:not [Bad (= ?k k)]]
  => (insert! (->Rate ?k)))

(defrule out
  [Rate (= ?n count)]
  => (insert! (->Out ?n)))

(defquery q-bad [] [Bad (= ?k k)])
(defquery q-rate [] [Rate (= ?n count)])
(defquery q-out [] [Out (= ?n n)])

(defn -main [& _]
  (let [s (fire-rules (insert (mk-session 'arc278-produced-type-userfn-facts :cache false)
                              (->Src 1)))]
    (println (str "CLARA facts [Bad Rate Out]: ["
                  (count (query s q-bad)) " "
                  (count (query s q-rate)) " "
                  (count (query s q-out)) "]"))))

(-main)
