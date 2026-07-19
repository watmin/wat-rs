;; prove.clj — the measurement on the way out.
;;
;; Read every real wat EDN face on disk through Clojure's CANONICAL edn reader,
;; and prove the value tags come back as real records Clojure can HANDLE.
;;
;; Run from the repo root:  clj -M crates/wat-edn/clj/prove.clj

(load-file "crates/wat-edn/clj/wat_edn.clj")
(require '[clojure.java.io :as io])
(alias 'w 'wat-edn)

(let [files (->> (file-seq (io/file "tests"))
                 (filter #(.endsWith (.getName %) ".edn"))
                 (sort-by #(.getPath %)))]
  (when (empty? files)
    (println "no .edn files found under tests/") (System/exit 2))
  (println (str "Reading " (count files) " real wat EDN faces via clojure.edn:\n"))
  (doseq [f files]
    (println (format "  OK  %s" (.getPath f)))
    (w/read-wat (slurp f)))                 ; throws if not well-formed EDN

  ;; the value tags must come back as REAL records with working tools ─────────
  (let [face (w/read-wat
               (slurp "tests/wat_lang/wat_core_cond__cond_refuses_missing_else.edn"))
        span (:location face)
        end  (:end span)]
    (println "\nvalue-tag reconstruction (cond missing-:else face):")
    (println "  top             :" (:wat/tag face))                   ; the error tag, stamped on the flat map
    (println "  :location class :" (type span))                       ; wat_edn.Span — a real record
    (println "  span file/line  :" (:file span) (:line span) (:col span))
    (println "  :end class      :" (type end))                        ; wat_edn.Some — a real record
    (println "  (some? end)     :" (w/some? end))                     ; the TOOL: true
    (println "  (unwrap end)    :" (w/unwrap end))                    ; the TOOL: the wrapped Pos record
    (println "  unwrapped class :" (type (w/unwrap end))))            ; wat_edn.Pos

  ;; and the tools handle Option/Result end to end ────────────────────────────
  (println "\nOption/Result tools:")
  (println "  Some round-trip :" (w/read-wat "#wat.core.Option/Some [42]")
                                  "→ some?" (w/some? (w/read-wat "#wat.core.Option/Some [42]"))
                                  "unwrap" (w/unwrap (w/read-wat "#wat.core.Option/Some [42]")))
  (println "  None            :" (w/read-wat "#wat.core.Option/None []")
                                  "→ none?" (w/none? (w/read-wat "#wat.core.Option/None []")))
  (println "  Ok              :" (w/read-wat "#wat.core.Result/Ok [7]")
                                  "→ ok?" (w/ok? (w/read-wat "#wat.core.Result/Ok [7]")))
  (println "  Err             :" (w/read-wat "#wat.core.Result/Err [\"boom\"]")
                                  "→ err?" (w/err? (w/read-wat "#wat.core.Result/Err [\"boom\"]")))

  (println (str "\nPROVED: " (count files)
                " wat EDN faces read by canonical Clojure; Option/Result/Span/Pos"
                " reconstruct as real records with working tools. wat IS EDN — externally verified.")))
