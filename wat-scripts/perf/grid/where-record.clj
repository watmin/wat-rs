;; wat-scripts/perf/grid/where-record.clj — the RECORDS-AND-ACCESSOR-CHAINS family of the
;; `where`-expressivity corpus, Clara side. Twin of where-record.wat — read its header FIRST
;; (record shapes, seed formulas, and all 13 rows are documented there; this file mirrors it
;; verbatim, predicate for predicate).
;;
;; ── HOW IT RUNS ───────────────────────────────────────────────────────────────────────────────
;;
;;     ./target/release/wat  wat-scripts/perf/grid/where-record.wat   > /tmp/ours
;;     clojure -Sdeps '{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}}}' \
;;             -M wat-scripts/perf/grid/where-record.clj              > /tmp/theirs
;;     diff /tmp/ours /tmp/theirs        # empty ⇒ every row agrees
;;
;; ── MIRRORING wat's enum/Option, which Clojure has no native equivalent for ─────────────────────
;;
;; `:wr::Status` (Active[level] | Inactive | Pending[reason]) has no Clojure sum-type analogue, so
;; it is mirrored as a plain map with a `:tag` key (`{:tag "active" :level N}`, `{:tag "inactive"}`,
;; `{:tag "pending" :reason N}`) dispatched with `case` — the idiomatic Clojure shape for a closed
;; tag set, exactly as `defrecord`/`i64` are already how records/ints are mirrored elsewhere in this
;; corpus. `(:wat::core::Option :- [i64])` is mirrored the standard Clojure way: `nil` for `None`, the bare
;; value for `Some v` — this is rule 4 (mirror the OPERATION, not the vocabulary): neither language
;; has the other's exact construct, so each side uses its own idiomatic representation of the SAME
;; three-way (Active/Inactive/Pending) and two-way (Some/None) case split, and `match`/`case` walk
;; the identical branches.
;;
;; ── FAITHFULNESS ──────────────────────────────────────────────────────────────────────────────
;;
;; Every arithmetic formula uses Clojure's `mod`, which mirrors wat's `:wat::core::i64::mod` exactly
;; for all operands used here — every seed value is >= 0, so there is no floor-vs-truncate subtlety
;; to reason about (see where-multivar.clj's note on `i64::mod`).

(ns where-record
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]))

(def items 200)                                    ;; the stream size, both sides

(defrecord L4     [v])
(defrecord L3     [l4 w])
(defrecord L2     [l3 u])
(defrecord Bag    [items label])
(defrecord Client [l2 rep tags bag])
(defrecord Req    [k client client2 status note])
(defrecord Hit    [k])

;; client-of i — mirrors :wr::client-of EXACTLY: same four scalar formulas, same two collections.
(defn client-of [i]
  (let [v4          (mod i 9)
        w3          (+ (mod i 11) 1)
        u2          (mod i 13)
        rep         (- (mod i 5) 2)
        tagslen     (mod i 5)
        bagitemslen (mod i 4)
        l4          (->L4 v4)
        l3          (->L3 l4 w3)
        l2          (->L2 l3 u2)
        tags        (vec (range tagslen))
        bagitems    (vec (range bagitemslen))
        bag         (->Bag bagitems (str "b" i))]
    (->Client l2 rep tags bag)))

;; status-of i — mirrors :wr::status-of: m=0 Active(i mod 5), m=1 Inactive, m=2 Pending(i mod 4).
(defn status-of [i]
  (let [m (mod i 3)]
    (cond
      (= m 0) {:tag "active"   :level  (mod i 5)}
      (= m 1) {:tag "inactive"}
      :else   {:tag "pending"  :reason (mod i 4)})))

;; note-of i — mirrors :wr::note-of: (i mod 4)==0 -> None (nil), else Some(i mod 6).
(defn note-of [i]
  (let [nm (mod i 4)]
    (if (= nm 0)
      nil
      (mod i 6))))

;; rep-pos? / pos? — row 8 vs row 9's contrast: same constraint, whole-record vs scalar call shape.
(defn rep-pos? [c] (> (:rep c) 0))
(defn pos? [x] (> x 0))

;; is-risky? — mirrors :wr::is-risky?'s match over the enum, via `case` on the `:tag` key.
(defn is-risky? [st]
  (case (:tag st)
    "active"   (> (:level st) 3)
    "inactive" false
    "pending"  (> (:reason st) 1)))

;; note-positive? — mirrors :wr::note-positive?'s match over Option, via nil-check.
(defn note-positive? [nt]
  (if (nil? nt) false (> nt 2)))

;; THE SHARED LEADING CONDITION — every row binds all five fields off one [Req ...] pattern.
;; (Written out per rule because Clara's defrule takes the pattern literally.)

;; ROW 1 — 2-level accessor chain. u2(i) > 8 <=> i mod 13 in {9,10,11,12} -> 60 of 200.
(defrule chain2
  [Req (= ?k k) (= ?c client) (= ?c2 client2) (= ?st status) (= ?nt note)]
  [:test (> (:u (:l2 ?c)) 8)]
  => (insert! (->Hit ?k)))

;; ROW 2 — 3-level accessor chain. w3(i) > 7 <=> i mod 11 in {7,8,9,10} -> 72 of 200.
(defrule chain3
  [Req (= ?k k) (= ?c client) (= ?c2 client2) (= ?st status) (= ?nt note)]
  [:test (> (:w (:l3 (:l2 ?c))) 7)]
  => (insert! (->Hit ?k)))

;; ROW 3 — 4-level accessor chain. v4(i) > 5 <=> i mod 9 in {6,7,8} -> 66 of 200.
(defrule chain4
  [Req (= ?k k) (= ?c client) (= ?c2 client2) (= ?st status) (= ?nt note)]
  [:test (> (:v (:l4 (:l3 (:l2 ?c)))) 5)]
  => (insert! (->Hit ?k)))

;; ROW 4 — a record field that IS a collection, reached and measured. tagslen(i) > 2
;; <=> i mod 5 in {3,4} -> 80 of 200.
(defrule collection
  [Req (= ?k k) (= ?c client) (= ?c2 client2) (= ?st status) (= ?nt note)]
  [:test (> (count (:tags ?c)) 2)]
  => (insert! (->Hit ?k)))

;; ROW 5 — a record field that holds ANOTHER RECORD holding a collection. bagitemslen(i) > 1
;; <=> i mod 4 in {2,3} -> 100 of 200.
(defrule record-collection
  [Req (= ?k k) (= ?c client) (= ?c2 client2) (= ?st status) (= ?nt note)]
  [:test (> (count (:items (:bag ?c))) 1)]
  => (insert! (->Hit ?k)))

;; ROW 6 — SAME var, two different chains compared: rep(c) > v4-chain(c) -> 15 of 200.
(defrule same-var-two-chains
  [Req (= ?k k) (= ?c client) (= ?c2 client2) (= ?st status) (= ?nt note)]
  [:test (> (:rep ?c) (:v (:l4 (:l3 (:l2 ?c)))))]
  => (insert! (->Hit ?k)))

;; ROW 7 — TWO vars, same one-level chain compared: rep(c) > rep(c2) -> 80 of 200.
(defrule cross-var-scalar
  [Req (= ?k k) (= ?c client) (= ?c2 client2) (= ?st status) (= ?nt note)]
  [:test (> (:rep ?c) (:rep ?c2))]
  => (insert! (->Hit ?k)))

;; ROW 8 — a pure fn taking the WHOLE RECORD: (rep-pos? ?c) -> 80 of 200.
(defrule whole-record-fn
  [Req (= ?k k) (= ?c client) (= ?c2 client2) (= ?st status) (= ?nt note)]
  [:test (rep-pos? ?c)]
  => (insert! (->Hit ?k)))

;; ROW 9 — the CONTRAST with row 8: the caller reaches in and passes a bare scalar.
;; Same derived set as row 8 (80 of 200) — the call SHAPE differs, not the constraint.
(defrule scalar-fn
  [Req (= ?k k) (= ?c client) (= ?c2 client2) (= ?st status) (= ?nt note)]
  [:test (pos? (:rep ?c))]
  => (insert! (->Hit ?k)))

;; ROW 10 — an enum/variant field, matched inside the predicate: (is-risky? ?st) -> 46 of 200.
(defrule enum-match
  [Req (= ?k k) (= ?c client) (= ?c2 client2) (= ?st status) (= ?nt note)]
  [:test (is-risky? ?st)]
  => (insert! (->Hit ?k)))

;; ROW 11 — an Option-typed field, matched inside the predicate: (note-positive? ?nt) -> 82 of 200.
(defrule option-match
  [Req (= ?k k) (= ?c client) (= ?c2 client2) (= ?st status) (= ?nt note)]
  [:test (note-positive? ?nt)]
  => (insert! (->Hit ?k)))

;; ROW 12 — combined: deep chain AND shallow field, same var -> 44 of 200.
(defrule combined-and
  [Req (= ?k k) (= ?c client) (= ?c2 client2) (= ?st status) (= ?nt note)]
  [:test (and (> (:rep ?c) 0) (> (:v (:l4 (:l3 (:l2 ?c)))) 3))]
  => (insert! (->Hit ?k)))

;; ROW 13 — TWO vars, the SAME 2-level chain compared: u2-chain(c) > u2-chain(c2) -> 55 of 200.
(defrule cross-var-chain
  [Req (= ?k k) (= ?c client) (= ?c2 client2) (= ?st status) (= ?nt note)]
  [:test (> (:u (:l2 ?c)) (:u (:l2 ?c2)))]
  => (insert! (->Hit ?k)))

(defquery hit-q [] [Hit (= ?k k)])

;; THE ROW TABLE — mirrors where-record.wat's `build-rules` cond.
(def rows
  [[1  "chain2"              chain2]
   [2  "chain3"              chain3]
   [3  "chain4"              chain4]
   [4  "collection"          collection]
   [5  "record-collection"   record-collection]
   [6  "same-var-two-chains" same-var-two-chains]
   [7  "cross-var-scalar"    cross-var-scalar]
   [8  "whole-record-fn"     whole-record-fn]
   [9  "scalar-fn"           scalar-fn]
   [10 "enum-match"          enum-match]
   [11 "option-match"        option-match]
   [12 "combined-and"        combined-and]
   [13 "cross-var-chain"     cross-var-chain]])

;; seed-req i — the SAME formulas as :wr::seed, computed independently:
;;   client(i) = client-of(i)
;;   client2(i) = client-of(j),  j = (i + 97) mod items
;;   status(i) = status-of(i)
;;   note(i)   = note-of(i)
(defn seed-req [i]
  (let [j (mod (+ i 97) items)]
    (->Req i (client-of i) (client-of j) (status-of i) (note-of i))))

(def seeds (mapv seed-req (range items)))

(defn run-row [[row nm rule]]
  (let [session (apply insert (mk-session [rule hit-q] :cache false) seeds)
        codes   (sort (map :?k (query (fire-rules session) hit-q)))]
    ;; Mirrors the wat side's `render-ints` fold EXACTLY — one leading space per element.
    (str "row " row " " nm " n=" (count codes) " ->"
         (apply str (map #(str " " %) codes)))))

;; `prn`, not `println` — see where-shapes.clj's note (wat's println EDN-encodes Strings).
(defn -main [& _] (doseq [r rows] (prn (run-row r))))

(-main)
