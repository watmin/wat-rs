;; Twin of where-or-inline.wat. Clara 0.24.0 — an `or` / `not` INSIDE one
;; condition's constraint list (Clara allows arbitrary Clojure in a constraint,
;; so the disjunction is written directly rather than as a `[:or …]` clause,
;; which would be the top-level form this family exists to distinguish itself
;; from). n= is DISTINCT keys / locations.

(ns where-or-inline
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]))

(defrecord Reading [k v loc])
(defrecord Station [loc])
(defrecord Hit     [k])
(defrecord At      [loc])

(defrule extreme
  [Reading (= ?k k) (or (> v 30) (< v 10))]
  => (insert! (->Hit ?k)))

(defrule not-high
  [Reading (= ?k k) (not (> v 30))]
  => (insert! (->Hit ?k)))

(defrule mid
  [Reading (= ?k k) (not (or (> v 30) (< v 10)))]
  => (insert! (->Hit ?k)))

(defrule station-extreme
  [Station (= ?loc loc)]
  [Reading (= ?loc loc) (= ?k k) (or (> v 30) (< v 10))]
  => (insert! (->At ?loc)))

(defquery hit-q [] [Hit (= ?k k)])
(defquery at-q  [] [At (= ?loc loc)])

(defn run [rule q n-fn facts]
  (let [s (apply insert (mk-session [rule q] :cache false) facts)]
    (count (set (map n-fn (query (fire-rules s) q))))))

(defn n-hit [rule facts] (run rule hit-q :?k facts))
(defn n-at  [facts]      (run station-extreme at-q :?loc facts))

(defn -main [& _]
  (prn (str "row 1 or-hits-high n="
            (n-hit extreme [(->Reading 1 40 "MCI") (->Reading 2 20 "MCI")])))
  (prn (str "row 2 or-hits-low n="
            (n-hit extreme [(->Reading 1 5 "MCI") (->Reading 2 20 "MCI")])))
  (prn (str "row 3 not-high n="
            (n-hit not-high [(->Reading 1 40 "MCI") (->Reading 2 20 "MCI")])))
  (prn (str "row 4 demorgan-mid n="
            (n-hit mid [(->Reading 1 40 "MCI") (->Reading 2 20 "MCI") (->Reading 3 5 "MCI")])))
  (prn (str "row 5 prefix-or n="
            (n-at [(->Station "MCI") (->Reading 1 40 "MCI")]))))

(-main)
