(require '[clojure.edn :as edn] '[clojure.string :as str])
(with-open [w (clojure.java.io/writer (System/getenv "GOLDEN"))]
  (doseq [line (str/split-lines (slurp (System/getenv "CORPUS")))
          :when (not (str/blank? line))]
    (let [v (try (do (edn/read-string line) "OK") (catch Throwable _ "ERR"))]
      (.write w (str v "\t" line "\n")))))
