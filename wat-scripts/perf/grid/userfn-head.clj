;; wat-scripts/perf/grid/userfn-head.clj — the CLARA TWIN of userfn-head.wat.
;; Read that file's header FIRST: Src(k) for k in [0,items); Bad :- Src, k = -1 (never
;; fires); Rate :- Src, (not Bad), :then (mk-rate ?k); Out :- Rate. The encoding
;; (enc kind*1e15 + id), the sorted-NOT-deduped :derived carrying BOTH Rate and Out,
;; and the user-fn head are all documented there and mirrored here.
;;
;; ── ⛔ WHY THIS IS A STATIC `.clj` AND NOT A `gen-userfn-head.sh` ──────────────────────
;;
;; `run-all.sh:81-87` discovers a PERF axis as `<axis>.wat` WITH a `gen-<axis>.sh` twin, and
;; `:89-99` exits 2 for any discovered axis with no LADDER rung. A generator here would
;; therefore drag a CORRECTNESS proof onto the perf ladder and onto `check-grid-speed.sh`,
;; which treats `:accuracy :MISMATCH` as a gate failure. A static `.clj` is invisible to
;; that discovery (same convention as `accum-over-derived.clj` / `retract-multiplicity.clj`).
;;
;; It does NOT self-invoke `(-main)` at load: `check-grid-three-way.sh` `require`s this into
;; a shared JVM and drives `-main` with the size args, so a load-time print would emit its
;; row twice.
;;
;; ── MODELLING POINT 1 — THE USER FN IS INLINED ──────────────────────────────────────────
;;
;; Clara 0.24.0 has no `:then`-head-is-a-fn. wat's `:ufh::via` RHS is `(:ufh::mk-rate ?k)`,
;; and `:ufh::mk-rate` is
;;
;;   (:wat::rete::core::defn :ufh::mk-rate [k <- :wat::core::i64] -> :ufh::Rate
;;     (:ufh::Rate :count k))
;;
;; The body is exactly `(->Rate k)`. The Clara `via` rule inlines that construction as
;; `(insert! (->Rate ?k))`. Nothing else of the fn is dropped: one argument, one record,
;; the bound `k` is the `count` field. If `mk-rate` ever computes, this file is no longer
;; the same rule and must be rewritten, not trusted.
;;
;; ── MODELLING POINT 2 — `Bad` NEVER FIRES ───────────────────────────────────────────────
;;
;; Seed is `Src(k)` for k in [0, items). `bad` fires only when `k = -1`. No key in
;; [0, items) is -1, so the negation `[:not [Bad (= ?k k)]]` is vacuously true for every
;; seeded k. Clara's Rate is derived for the same reason wat's is: Src(k) exists and
;; Bad(k) does not. The referee is not exercising Clara's negation under retraction; it
;; is exercising the consumer of Rate.
;;
;; ⛔ MULTIPLICITY IS THE WITNESS. `all-codes` is `(sort (concat …))` — no `set`, no
;; `distinct`. Rate AND Out. A dropped Out is a missing enc(1,k).

(ns userfn-head
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]))

(defrecord Src [k])
(defrecord Bad [k])
(defrecord Rate [count])
(defrecord Out [n])

(defrule bad
  [Src (= ?k k)]
  [:test (= ?k -1)]
  => (insert! (->Bad ?k)))

(defrule via
  [Src (= ?k k)]
  [:not [Bad (= ?k k)]]
  => (insert! (->Rate ?k)))

(defrule out
  [Rate (= ?n count)]
  => (insert! (->Out ?n)))

(defquery q-rate [] [Rate (= ?n count)])
(defquery q-out [] [Out (= ?n n)])

(defn enc [kind id] (+ (* kind 1000000000000000) id))

;; Sorted bag of every Rate and every Out. sort does not dedup.
(defn all-codes [s]
  (sort
    (concat
      (map (fn [r] (enc 0 (:?n r))) (query s q-rate))
      (map (fn [r] (enc 1 (:?n r))) (query s q-out)))))

(defn -main [& args]
  (let [items (Long/parseLong (or (first args) "5"))
        seeds (map ->Src (range items))
        build (fn [] (apply insert (mk-session 'userfn-head :cache false) seeds))]
    (dotimes [_ 3] (count (all-codes (fire-rules (build)))))
    (let [s (apply insert (mk-session 'userfn-head :cache false) seeds)
          t0 (System/nanoTime)
          f (fire-rules s)
          t1 (System/nanoTime)
          codes (all-codes f)]
      (println (str "#grid/Result {:axis \"userfn-head\" :size [" items "] :derived ["
                    (clojure.string/join " " codes) "] :clara-ns " (- t1 t0) "}")))))
