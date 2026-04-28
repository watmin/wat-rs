;; Reads JSON from stdin, converts to Clojure value via wat-edn-clj,
;; then converts back to JSON and writes. Used for cross-language
;; JSON-EDN-JSON round-trip:
;;
;;   echo "<edn>" | json_producer | json_passthrough.clj | json_consumer
;;
;; Each leg verifies both sides agree on the wire convention.

(require '[wat-edn.json :as wj])

(let [in (slurp *in*)
      v  (wj/from-json-string in)
      out (wj/to-json-string v)]
  (binding [*out* *err*]
    (println "─── Clojure parsed ───")
    (println v)
    (println "─── Clojure re-serializing ───"))
  (print out))
