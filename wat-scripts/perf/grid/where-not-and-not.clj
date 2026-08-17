;; Twin of where-not-and-not.wat. Clara 0.24.0 test-complex-negation nested.
;; n= is 1 when the outer :not fires.

(ns where-not-and-not
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]))

(defrecord Wind [kph loc])
(defrecord Temp [c loc])
(defrecord Cold [c])
(defrecord Hit  [k])
(defrecord At   [loc])

(defrule lead
  [:not [:and
         [?t <- Temp]
         [:not [Cold (= c (:c ?t))]]]]
  => (insert! (->Hit 1)))

(defrule nested
  [Wind (= ?l loc)]
  [:not [:and
         [?t <- Temp (= ?l loc)]
         [:not [Cold (= c (:c ?t))]]]]
  => (insert! (->At ?l)))

(defquery hit-q [] [Hit (= ?k k)])
(defquery at-q  [] [At (= ?loc loc)])

(defn run [rule q n-fn facts]
  (let [s (mk-session [rule q] :cache false)
        s (if (seq facts) (apply insert s facts) s)]
    (count (set (map n-fn (query (fire-rules s) q))))))

(defn n-hit [facts] (run lead hit-q :?k facts))
(defn n-at  [facts] (run nested at-q :?loc facts))

(defn -main [& _]
  (prn (str "row 1 lead-empty n=" (n-hit [])))
  (prn (str "row 2 lead-temp n=" (n-hit [(->Temp 10 "MCI")])))
  (prn (str "row 3 lead-mismatch n=" (n-hit [(->Temp 10 "MCI") (->Cold 20)])))
  (prn (str "row 4 lead-match n=" (n-hit [(->Temp 10 "MCI") (->Cold 10)])))
  (prn (str "row 5 nest-wind n=" (n-at [(->Wind 10 "MCI")])))
  (prn (str "row 6 nest-cold20 n=" (n-at [(->Wind 10 "MCI") (->Temp 10 "MCI") (->Cold 20)])))
  (prn (str "row 7 nest-cold10 n=" (n-at [(->Wind 10 "MCI") (->Temp 10 "MCI") (->Cold 10)])))
  (prn (str "row 8 nest-issue304 n=" (n-at [(->Wind 10 "MCI") (->Temp 10 "MCI") (->Cold 20)
                                           (->Wind 20 "ORD") (->Temp 20 "ORD")]))))

(-main)
