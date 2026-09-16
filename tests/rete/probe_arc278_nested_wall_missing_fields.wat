;; strike-nested-wall — KIND 2 of 4: `RhsMissingFields`, at the NESTED-CONSTRUCTOR producer.
;;
;; Tuned so this kind fires ALONE. Every field NAME written is declared, so `UnknownField` has
;; nothing to report; `:nwm::Inner` declares two fields and the nested form supplies one, so the
;; only finding is the missing `y`.
;;
;; Its span is the whole nested form, not any one keyword — deliberately, and asserted so: "missing
;; `y`" is a property of the form, not of a field in it.

(:wat::core::defrecord :nwm::Src   [k <- :wat::core::i64])
(:wat::core::defrecord :nwm::Inner [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::core::defrecord :nwm::Outer [k <- :wat::core::i64  inner <- :nwm::Inner])

(:wat::rete::defrule :nwm::r
  :when [(:nwm::Src (?k <- :k))]
  :then [(:nwm::Outer :k ?k :inner (:nwm::Inner :x ?k))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "the wall refuses before main runs"))
