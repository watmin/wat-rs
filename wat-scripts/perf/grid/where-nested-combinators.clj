;; Twin of where-nested-combinators.wat. Combinators NESTED inside combinators, over every world.
;;
;; Clara is the third reference, and this axis exists because two wat engines agreeing proves
;; nothing when they share an assumption — demonstrated the same day by RETE-FIX-LIST entry E,
;; where native and the `$oracle` transposed `:then` kwargs identically and agreed perfectly on the
;; wrong answer. The `$oracle` says what we INTENDED; Clara says what a rules engine DOES.
;;
;; Each row is one WORLD: a 3-bit presence mask over A, B, C. Row 12 (`or-and-not-w3`) is the one
;; to watch — an `:or` with both arms satisfiable yields TWO activations, and activation
;; multiplicity is a shape this arc has been bitten by before.
(ns where-nested-combinators
  (:require [clara.rules :refer [mk-session insert fire-rules query defquery]]))

(defrecord A [k])
(defrecord B [k])
(defrecord C [k])

(defquery q5 [] [:not [:and [A] [:not [B]]]])
(defquery q6 [] [:or [A] [:and [B] [:not [C]]]])
(defquery q7 [] [:and [:not [A]] [:not [B]]])

(defn world [w]
  (cond-> []
    (bit-test w 0) (conj (->A 1))
    (bit-test w 1) (conj (->B 1))
    (bit-test w 2) (conj (->C 1))))

(defn n [w q]
  (let [facts (world w)
        s (mk-session [q] :cache false)
        s (if (seq facts) (apply insert s facts) s)]
    (count (query (fire-rules s) q))))

(defn sweep [base name q]
  (doseq [w (range 8)]
    (prn (str "row " (+ base w) " " name w " n=" (n w q)))))

(defn -main [& _]
  (sweep 1  "not-and-not-w" q5)
  (sweep 9  "or-and-not-w"  q6)
  (sweep 17 "and-not-not-w" q7))

(-main)
