;; Shape matrix consumer — reads the named-shape Map from wat-edn and
;; asserts each named shape parsed correctly through pure clojure.edn/read.
;;
;; Pure clojure.edn — no wat-edn-clj dep. Proves the wire is
;; strict-EDN-compliant for every shape wat-edn can emit.

(require '[clojure.edn :as edn])

(def parsed
  (edn/read
    {:default tagged-literal :readers {}}
    (java.io.PushbackReader. *in*)))

(println "─── Shape matrix received ───")
(println "  shape count:" (count parsed))
(println)

(defn assert-shape [k pred msg]
  (let [v (get parsed k)]
    (if (pred v)
      (println "  ✓" k)
      (do (println "  ✗" k "—" msg "— got:" v)
          (System/exit 1)))))

(println "─── Primitives ───")
(assert-shape :primitive-i64     #(= % 42) "i64 should be 42")
(assert-shape :primitive-string  #(= % "hello") "string should be \"hello\"")
(assert-shape :primitive-keyword #(= % :asset/BTC) "keyword should be :asset/BTC")
(assert-shape :primitive-bool    #(= % true) "bool should be true")
(assert-shape :primitive-nil     nil? "nil should be nil")
(assert-shape :primitive-f64     #(= % 2.5) "f64 should be 2.5")

(println)
(println "─── Collections ───")
(assert-shape :collection-vector #(= % [1 2 3]) "vector of ints")
(assert-shape :collection-set    #(= % #{:a :b :c}) "set of keywords")
(assert-shape :collection-map    #(= % {:k1 1 :k2 2}) "map keyword→int")

(println)
(println "─── Nested collections ───")
(assert-shape :nested-vec-of-vecs #(= % [[1 2] [3 4]]) "vec of vecs")
(assert-shape :nested-map-of-vec  #(= % {:numbers [1 2 3]}) "map of vec")

(println)
(println "─── EDN-spec built-in tags ───")
(assert-shape :builtin-inst  #(instance? java.util.Date %)
              "inst should be Date")
(assert-shape :builtin-uuid  #(instance? java.util.UUID %)
              "uuid should be UUID")

(println)
(println "─── FQDN tagged literals (2026-05-21b doctrine) ───")
(defn tagged? [v ns name]
  (and (tagged-literal? v)
       (= (:tag v) (symbol ns name))))

(assert-shape :tagged-some-i64
              #(and (tagged? % "wat.core" "Some") (= (:form %) 42))
              "Some<i64>=42 as tagged-literal")
(assert-shape :tagged-none
              #(and (tagged? % "wat.core" "None") (nil? (:form %)))
              "None as tagged-literal with nil body")
(assert-shape :tagged-ok-string
              #(and (tagged? % "wat.core" "Ok") (= (:form %) "fine"))
              "Ok<String>=\"fine\"")
(assert-shape :tagged-err-map
              #(and (tagged? % "wat.core" "Err")
                    (= (:form %) {:code 500 :msg "boom"}))
              "Err<Map>=error envelope")
(assert-shape :tagged-duration
              #(and (tagged? % "wat.time" "Duration") (= (:form %) "PT5M"))
              "Duration ISO 8601")

(println)
(println "─── Nested complex (the user's example + variants) ───")
(assert-shape :nested-some-set-of-maps
              #(and (tagged? % "wat.core" "Some")
                    (= (:form %) #{{:foo "baz"}}))
              "Some<Set<Map>>")
(assert-shape :nested-ok-vec-of-maps
              #(and (tagged? % "wat.core" "Ok")
                    (= (:form %) [{:a 1} {:b 2}]))
              "Ok<Vec<Map>>")
(assert-shape :nested-some-some-i64
              (fn [v]
                (and (tagged? v "wat.core" "Some")
                     (let [inner (:form v)]
                       (and (tagged? inner "wat.core" "Some")
                            (= (:form inner) 42)))))
              "Some<Some<i64>>")
(assert-shape :vec-of-options
              (fn [v]
                (and (vector? v)
                     (= 3 (count v))
                     (tagged? (nth v 0) "wat.core" "Some")
                     (tagged? (nth v 1) "wat.core" "None")
                     (tagged? (nth v 2) "wat.core" "Some")))
              "Vec<Some, None, Some>")

(println)
(println "─── Composite keys (arc 216 antidote) ───")
(assert-shape :map-with-tagged-keys
              (fn [v]
                (and (map? v)
                     (= 1 (count v))
                     (let [[k val] (first v)]
                       (and (tagged? k "wat.holon" "Atom")
                            (tagged? val "wat.holon" "Atom")
                            (= :role (:form k))
                            (= :filler (:form val))))))
              "Map<Atom<:role>, Atom<:filler>> — composite key")

(println)
(println "─── Arc 220 — :wat::core::Char (BMP-only) ───")
(assert-shape :char-bmp #(= % \x) "char \\x")

(println)
(println "─── Arc 220 Stone 220.4 — :wat::core::List<T> ───")
(assert-shape :list-3
              #(= % '(1 2 3))
              "list of 3 ints (1 2 3)")

(println)
(println "✓ All shapes parsed cleanly through clojure.edn/read.")
(println "✓ wat-edn output is strict-EDN compliant across the matrix.")
