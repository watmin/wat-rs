;; shared.wat — fixture used by load-types! tests.
;;
;; Acts as a HEADER FILE for cross-language type sharing. The same
;; file would be consumed by wat-rs's type checker (as code) and
;; by wat-edn-clj's load-types! (as schema).
;;
;; Only :wat::core::struct forms are read by the Clojure scanner.
;; Function definitions, macros, etc. are silently skipped.

(:wat::core::use! :rust::wat_edn::write-str)

(:wat::core::struct :enterprise::config::SizeAdjust
  (asset    :Keyword)
  (factor   :f64)
  (reason   :String))

(:wat::core::struct :enterprise::observer::market::TradeSignal
  (asset       :Keyword)
  (side        :Keyword)
  (size        :f64)
  (confidence  :f64)
  (proposed-at :wat::time::Instant))

(:wat::core::struct :enterprise::treasury::events::Fill
  (order-id     :i64)
  (asset        :Keyword)
  (filled-size  :f64)
  (filled-price :f64))

;; A function definition — should be ignored by the scanner.
(:wat::core::define (:enterprise::observer::market::TradeSignal/show
                     (sig :enterprise::observer::market::TradeSignal)
                     -> :String)
  (:wat::core::format "[%s] %s @ %f"
    (:enterprise::observer::market::TradeSignal/asset sig)
    (:enterprise::observer::market::TradeSignal/side sig)
    (:enterprise::observer::market::TradeSignal/size sig)))
