;; Produces EDN from Clojure side using pr-str on a hand-built
;; tagged-literal structure. Output goes to stdout.
;;
;; The Rust reader (cargo run --bin reader) parses this and asserts
;; the round-trip survived the cross-tool boundary.

(let [signal
      (tagged-literal
        'enterprise.config/SizeAdjust
        {:asset :BTC
         :factor 1.5
         :reason "drawdown breach — increase position cautiously"
         :issued-at #inst "2026-04-27T16:00:00Z"
         :ticket #uuid "12345678-1234-5678-1234-567812345678"
         :nested (tagged-literal
                   'wat.core/Vec<wat.holon.HolonAST>
                   [(tagged-literal 'wat.holon/Atom :stop-loss-hit)
                    (tagged-literal 'wat.holon/Atom :volatility-spike)])})]
  (binding [*print-dup* false]
    (print (pr-str signal))
    (newline)))
