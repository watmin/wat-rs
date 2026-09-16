;; strike-nested-wall — KIND 1 of 4: `UnknownField`, at the NESTED-CONSTRUCTOR producer.
;;
;; Tuned so this kind fires ALONE. `:nwu::Inner` declares exactly `x`, and the nested form
;; SUPPLIES `x` — so `RhsMissingFields` has nothing to report and only the undeclared `:nope`
;; remains. That separation is the point: the mutation that disables this arm must redden this
;; probe and no other.
;;
;; The wall only sees this at all because it now reads the LOWERED head. `defrecord`'s companion
;; macro rewrites `(:nwu::Inner :x ?k :nope ?k)` to
;; `(:wat::core::kwargs-construct :nwu::Inner :x ?k :nope ?k)` before freeze, putting the type at
;; index 1; matched against `items[0]` this form resolved to no type and was accepted unvalidated.

(:wat::core::defrecord :nwu::Src   [k <- :wat::core::i64])
(:wat::core::defrecord :nwu::Inner [x <- :wat::core::i64])
(:wat::core::defrecord :nwu::Outer [k <- :wat::core::i64  inner <- :nwu::Inner])

(:wat::rete::defrule :nwu::r
  :when [(:nwu::Src (?k <- :k))]
  :then [(:nwu::Outer :k ?k :inner (:nwu::Inner :x ?k :nope ?k))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "the wall refuses before main runs"))
