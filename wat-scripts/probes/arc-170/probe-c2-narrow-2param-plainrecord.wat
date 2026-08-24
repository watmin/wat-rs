;; NARROW: 2-param surface (Pair :- [A B]) satisfied by a PLAIN RECORD.
;; receiver clean + (Pair/fst b):i64 → the bug is the SERVICE-HANDLE satisfier
;; receiver errors ("expects :probe::Pair; got :probe::ISBox") → the bug is the 2-PARAM count
(:wat::core::defsurface :probe::Pair :- [A B] :nature :wat::core::Struct
  :features [(fst [self <- (:probe::Pair :- [A B])] -> :A)])
(:wat::core::defrecord :probe::ISBox [i <- :wat::core::i64  s <- :wat::core::String])
(:wat::core::extend-type :probe::ISBox (:probe::Pair :- [:wat::core::i64 :wat::core::String])
  (fst [self] (:probe::ISBox/i self)))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [b  (:probe::ISBox :i 42 :s "hi")
     ok (:wat::core::ann-form (:probe::Pair/fst b) :wat::core::i64)]
    (:wat::kernel::println "narrowed")))
