;; Reads EDN from stdin (produced by Rust wat-edn), parses it
;; using clojure.edn (PURE Clojure, no wat-edn-clj dep — proves
;; the bytes are spec-conforming EDN that Clojure's reference
;; reader handles natively).
;;
;; Asserts the parsed shape matches what we expect from a
;; TradeSignal blob.

(require '[clojure.edn :as edn])

;; Default reader fn: any tag we don't explicitly handle becomes
;; a tagged-literal pair (Clojure's standard graceful interop).
(def parsed
  (edn/read
    {:default tagged-literal
     :readers {}}
    (java.io.PushbackReader. *in*)))

(println "─── parsed Clojure value ───")
(println parsed)
(println)

;; The outer form is a tagged-literal: (tag, body)
(println "─── structure assertions ───")

(assert (tagged-literal? parsed)
  "outer form should be a tagged-literal")

(assert (= 'enterprise.observer.market/TradeSignal (:tag parsed))
  (str "expected enterprise.observer.market/TradeSignal, got " (:tag parsed)))

(let [body (:form parsed)]
  (assert (map? body) "body should be a map")
  (println "  outer tag:    " (:tag parsed))
  (println "  body type:    " (type body))
  (println "  asset:        " (:asset body))
  (assert (= :BTC (:asset body)))
  (assert (= :Buy (:side body)))
  (assert (= 0.025 (:size body)))
  (assert (= 0.73 (:confidence body)))
  (assert (= "550e8400-e29b-41d4-a716-446655440000"
             (str (:id body))))

  ;; Built-in #inst comes through as java.util.Date by default
  (assert (instance? java.util.Date (:proposed-at body)))
  (println "  proposed-at:  " (:proposed-at body))

  ;; The reasoning is a tagged-literal wrapping a vector
  (let [reasoning (:reasoning body)]
    (assert (tagged-literal? reasoning))
    (assert (= 'wat.core/Vec<wat.holon.HolonAST> (:tag reasoning)))
    (let [items (:form reasoning)]
      (assert (vector? items))
      (assert (= 2 (count items)))
      ;; Each item is a tagged-literal with tag wat.holon/Atom
      (doseq [item items]
        (assert (tagged-literal? item))
        (assert (= 'wat.holon/Atom (:tag item))))
      (println "  reasoning:    " (count items) "Atom-tagged items"))))

(println)
(println "✓ Clojure read wat-edn output cleanly.")
(println "✓ Spec built-ins (#inst, #uuid) round-tripped to Date/UUID.")
(println "✓ User tags (enterprise.observer.market/TradeSignal,")
(println "             wat.core/Vec<...>, wat.holon/Atom)")
(println "  preserved as tagged-literal pairs.")
