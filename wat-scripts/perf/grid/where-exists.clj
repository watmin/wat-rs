;; Twin of where-exists.wat. Clara 0.24.0 test-simple-exists /
;; test-exists-with-conjunction / exists under :or / mid-chain exists after a left fact.
;; n= is DISTINCT locs (or Hit keys).

(ns where-exists
  (:require [clara.rules :refer [mk-session insert retract fire-rules query defrule defquery insert!]]
            [clara.rules.accumulators :as acc]))

(defrecord Temp [c loc])
(defrecord Wind [kph loc])
(defrecord Caw  [t w])
(defrecord Loc  [loc])
(defrecord At   [loc])
(defrecord Hit  [k])

(defrule lead-wind
  [:exists [Wind (= ?loc loc)]]
  => (insert! (->At ?loc)))

(defrule both-exist
  [:exists [Temp (= ?loc loc)]]
  [:exists [Wind (= ?loc loc)]]
  => (insert! (->At ?loc)))

(defrule or-exists
  [:or
   [:exists [Caw]]
   [:exists [Temp (< c 20)]]]
  => (insert! (->Hit 1)))

(defrule mid-wind
  [Loc (= ?loc loc)]
  [:exists [Wind (= ?loc loc)]]
  => (insert! (->At ?loc)))

(defrule mid-both
  [Loc (= ?loc loc)]
  [:exists [Temp (= ?loc loc)]]
  [:exists [Wind (= ?loc loc)]]
  => (insert! (->At ?loc)))

(defquery at-q  [] [At (= ?loc loc)])
(defquery hit-q [] [Hit (= ?k k)])

(defn run [rule q n-fn facts]
  (let [s (mk-session [rule q] :cache false)
        s (if (seq facts) (apply insert s facts) s)]
    (count (set (map n-fn (query (fire-rules s) q))))))

(defn n-at  [rule facts] (run rule at-q  :?loc facts))
(defn n-hit [facts]      (run or-exists hit-q :?k facts))

(defn n-at-retract []
  (let [a (->Wind 50 "MCI")
        b (->Wind 60 "MCI")
        s (-> (mk-session [lead-wind at-q] :cache false)
              (insert a b)
              (retract a b))]
    (count (set (map :?loc (query (fire-rules s) at-q))))))

(defn -main [& _]
  (prn (str "row 1 lead-empty n=" (n-at lead-wind [])))
  (prn (str "row 2 lead-two-same n=" (n-at lead-wind [(->Wind 50 "MCI") (->Wind 60 "MCI")])))
  (prn (str "row 3 lead-two-locs n=" (n-at lead-wind [(->Wind 50 "MCI") (->Wind 60 "ORD")])))
  (prn (str "row 4 lead-retract n=" (n-at-retract)))
  (prn (str "row 5 and-wind-only n=" (n-at both-exist [(->Wind 50 "MCI")])))
  (prn (str "row 6 and-diff-locs n=" (n-at both-exist [(->Wind 50 "MCI") (->Temp 60 "ORD")])))
  (prn (str "row 7 and-both-mci n=" (n-at both-exist [(->Wind 50 "MCI") (->Temp 60 "MCI")])))
  (prn (str "row 8 and-two-cities n=" (n-at both-exist [(->Wind 50 "MCI") (->Wind 60 "ORD")
                                                       (->Temp 60 "MCI") (->Temp 70 "ORD")])))
  (prn (str "row 9 or-empty n=" (n-hit [])))
  (prn (str "row 10 or-caw n=" (n-hit [(->Caw 10 10)])))
  (prn (str "row 11 or-temp n=" (n-hit [(->Temp 10 "MCI")])))
  (prn (str "row 12 or-both n=" (n-hit [(->Caw 10 10) (->Temp 10 "MCI")])))
  (prn (str "row 13 mid-loc-only n=" (n-at mid-wind [(->Loc "MCI")])))
  (prn (str "row 14 mid-wind-only n=" (n-at mid-wind [(->Wind 50 "MCI")])))
  (prn (str "row 15 mid-two-winds n=" (n-at mid-wind [(->Loc "MCI") (->Wind 50 "MCI") (->Wind 60 "MCI")])))
  (prn (str "row 16 mid-two-locs n=" (n-at mid-wind [(->Loc "MCI") (->Loc "ORD")
                                                     (->Wind 50 "MCI") (->Wind 60 "ORD")])))
  (prn (str "row 17 mid-both-one-city n=" (n-at mid-both [(->Loc "MCI") (->Loc "ORD")
                                                          (->Wind 50 "MCI") (->Temp 60 "MCI")
                                                          (->Wind 60 "ORD")])))
  (prn (str "row 18 mid-both-two-cities n=" (n-at mid-both [(->Loc "MCI") (->Loc "ORD")
                                                            (->Wind 50 "MCI") (->Temp 60 "MCI")
                                                            (->Wind 60 "ORD") (->Temp 70 "ORD")]))))

(-main)
