;; tests/rete/probe_arc278_then_user_forms_notfact.wat — Stone B RED world, the SECOND check
;; ("returns-a-fact") independently of the axis check. Loaded via startup_from_file. The `:then`
;; item's head (`:tf::compute-scalar`) is pure ∧ deterministic ∧ rete-composed — it would PASS
;; `where`'s own fence outright — but its declared return type is `:wat::core::i64`, not a fact
;; type. `then-item-fence` must still refuse it (a `field-names-of` raise, not an axis panic —
;; this check has no axis to name; STOP-2's "an item head that is neither a fact-type keyword nor
;; a fn returning a fact type").

(:wat::core::defrecord :tf::In [n <- :wat::core::i64])

(:wat::rete::core::defn :tf::compute-scalar
  [n <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::rete::i64::+ n 1 :undefined 0))

(:wat::rete::defrule :tf::compute-bad
  :when [(:tf::In (?n <- :n))]
  :then [(:tf::compute-scalar ?n)])

(:wat::core::defn :user::run-compile [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :tf)
     session (:wat::rete::compile rules)]
    (:wat::core::length (:wat::rete::Session/facts session))))
