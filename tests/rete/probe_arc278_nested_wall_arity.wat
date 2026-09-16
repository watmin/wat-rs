;; strike-nested-wall — KIND 3 of 4: `RhsArityMismatch`, at the NESTED-CONSTRUCTOR producer.
;;
;; The single-arg positional passthrough. `eval_kwargs_construct` sends `rest.len() <= 1` straight
;; to `construct_aggregate`, and the walker mirrors that arm: one positional value against a
;; TWO-field record is a shape that cannot construct.
;;
;; ⚠ `RhsArityMismatch` has TWO producers in this walker — this one and the enum-variant branch,
;; which was always live because an enum variant is not lowered. This fixture drives the AGGREGATE
;; one; `probe_arc278_enum_variant_typo.rs` covers the other. A mutation to either must not redden
;; the other's probe, which is what makes them two producers rather than one.

(:wat::core::defrecord :nwa::Src   [k <- :wat::core::i64])
(:wat::core::defrecord :nwa::Inner [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::core::defrecord :nwa::Outer [k <- :wat::core::i64  inner <- :nwa::Inner])

(:wat::rete::defrule :nwa::r
  :when [(:nwa::Src (?k <- :k))]
  :then [(:nwa::Outer :k ?k :inner (:nwa::Inner ?k))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "the wall refuses before main runs"))
