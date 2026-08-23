;; probe-locus1-generic-surface-method.wat — disconfirming probe for the Locus→surface flip, risk 1:
;; does a GENERIC method member on an AGGREGATE (:nature :Struct) defsurface, satisfied by a
;; defstruct via extend-type, type-check AND dispatch? (Locus/launch is generic: launch<S,R,St,Sh,Lu>.)
;;
;; Mirrors tests/types/probe_arc232_generic_method.wat but on a defsurface, not a defprotocol.
;; GREEN target: prints "5".

(:wat::core::defsurface :probe::Maker :nature :wat::core::Struct
  :features [(make<T> [self <- :probe::Maker  x <- :T] -> (:wat::core::Vector :- [T]))])

(:wat::core::defstruct :probe::Dup [])
(:wat::core::extend-type :probe::Dup :probe::Maker (make [self x] [x x]))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::i64::to-string
      (:wat::core::nth (:probe::Maker/make (:probe::Dup) 5) 0))))  ;; expect 5
