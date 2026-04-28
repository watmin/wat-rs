;; Full-pipeline cross-tool proof:
;;   1. Load schema from shared.wat (the same artifact wat-rs reads)
;;   2. Build a typed value via wat/gen (validates against schema)
;;   3. Emit to stdout
;;   4. /tmp/cross-test/target/release/reader (Rust + wat-edn) parses it

(require '[wat-edn.core :as wat])

;; Path resolves relative to where you ran `clojure`. Per the
;; README, you run from the interop-tests/ directory, so the wat
;; schema sibling lives at ../wat-edn-clj/wat/shared.wat.
(wat/load-types! "../wat-edn-clj/wat/shared.wat")

(let [edn (wat/emit 'enterprise.config/SizeAdjust
                    {:asset  :BTC
                     :factor 1.5
                     :reason "drawdown breach — increase size"})]
  (println edn))
