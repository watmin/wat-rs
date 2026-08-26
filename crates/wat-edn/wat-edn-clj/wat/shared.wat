;; shared.wat — fixture used by load-types! tests.
;;
;; Acts as a HEADER FILE for cross-language type sharing. The same
;; file IS consumed by wat-rs's type checker (as code — gated by
;; tests/lint/every_ungated_wat_checks.rs, which never lets this file
;; silently stop checking) and by wat-edn-clj's load-types! (as
;; schema, via the scanner in src/wat_edn/scanner.clj).
;;
;; Only :wat::core::defstruct forms are read by the Clojure scanner.
;; Function definitions, macros, etc. are silently skipped.

(:wat::core::defstruct :enterprise::config::SizeAdjust
  [asset    <- :wat::core::keyword
   factor   <- :wat::core::f64
   reason   <- :wat::core::String])

(:wat::core::defstruct :enterprise::observer::market::TradeSignal
  [asset       <- :wat::core::keyword
   side        <- :wat::core::keyword
   size        <- :wat::core::f64
   confidence  <- :wat::core::f64
   proposed-at <- :wat::time::Instant])

(:wat::core::defstruct :enterprise::treasury::events::Fill
  [order-id     <- :wat::core::i64
   asset        <- :wat::core::keyword
   filled-size  <- :wat::core::f64
   filled-price <- :wat::core::f64])

;; A function definition — should be ignored by the scanner.
(:wat::core::defn :enterprise::observer::market::TradeSignal/show
  [sig <- :enterprise::observer::market::TradeSignal]
  -> :wat::core::String
  (:wat::core::format "[{asset}] {side} @ {size}"
    :asset (:enterprise::observer::market::TradeSignal/asset sig)
    :side (:enterprise::observer::market::TradeSignal/side sig)
    :size (:enterprise::observer::market::TradeSignal/size sig)))
