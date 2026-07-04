;; Oracle regen for the clj-expressiveness equality matrix.
;; For each corpus row (a faithful `wat.core/…` expression), swap `wat.core` -> `clojure.core`,
;; eval under clj, and bake `RESULT<TAB>ROW` to the golden. RESULT is `(pr-str value)` — clj's
;; canonical, TYPE-DISCRIMINATING EDN (1 vs 1N vs 1.0 vs 1/2 print distinctly) — or `:THROW`.
;; The ward runs WITHOUT clj against this baked golden; regenerate whenever corpus.txt grows:
;;
;;   CORPUS=tests/clj_expr_oracle/corpus.txt GOLDEN=tests/clj_expr_oracle/golden.txt \
;;     clojure -M tests/clj_expr_oracle/regen.clj
(require '[clojure.string :as str])
(with-open [w (clojure.java.io/writer (System/getenv "GOLDEN"))]
  (doseq [raw (str/split-lines (slurp (System/getenv "CORPUS")))
          :let [line (str/trim raw)]
          :when (and (not (str/blank? line)) (not (str/starts-with? line ";")))]
    (let [clj-form (str/replace line "wat.core/" "clojure.core/")
          result (try (pr-str (eval (read-string clj-form)))
                      (catch Throwable _ ":THROW"))]
      (.write w (str result "\t" line "\n")))))
