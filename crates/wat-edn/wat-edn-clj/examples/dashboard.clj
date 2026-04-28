(ns dashboard
  "Example: a Clojure dashboard that loads its types from a
  shared `.wat` file (the same file wat-rs would consume), then
  emits configuration updates back via wat/emit."
  (:require [wat-edn.core :as wat]))

(defn -main [& args]
  ;; Single source of truth for the schema.
  (wat/load-types! "wat/shared.wat")

  (println "Loaded types:")
  (doseq [t (wat/list-types)]
    (println "  " t))

  ;; Build and emit a SizeAdjust — validation happens before
  ;; pr-str ever sees the data.
  (let [edn (wat/emit 'enterprise.config/SizeAdjust
                      {:asset :BTC
                       :factor 1.5
                       :reason "drawdown breach — increase size"})]
    (println)
    (println "Emitted EDN:")
    (println "  " edn))

  ;; Demonstrate a validation error caught at construction time.
  (println)
  (println "Validation demo (should throw):")
  (try
    (wat/gen 'enterprise.config/SizeAdjust
             {:asset "BTC"   ; ← string, schema says :Keyword
              :factor 1.5
              :reason "x"})
    (catch Throwable t
      (println "  caught:" (ex-message t))
      (println "  data:  " (ex-data t)))))
