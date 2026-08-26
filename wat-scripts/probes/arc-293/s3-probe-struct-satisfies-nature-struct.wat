;; s3-probe-struct-satisfies-nature-struct.wat — gate row 3 for 293 S3-Nature-2.
;; QUESTION: after adding the fourth Nature variant (Peer), does an ORDINARY struct still
;; extend-type-satisfy a `:nature :wat::core::Struct` surface (the aggregate rank-ladder path,
;; UNCHANGED by this stone — the `else` branch of `nature_floor_ok`)? MUST type-check.

(:wat::core::defstruct :probe::Counter [n <- :wat::core::i64])

(:wat::core::defsurface :probe::Bumper :nature :wat::core::Struct
  :features [(bump [self <- :probe::Bumper] -> :wat::core::i64)])

(:wat::core::extend-type :probe::Counter :probe::Bumper
  (bump [self] (:wat::core::+ (:probe::Counter/n self) 1)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [c (:probe::Counter 41)
     r (:probe::Bumper/bump c)]
    (:wat::kernel::println (:wat::string::concat "struct-as-Bumper bump = " (:wat::i64::to-string r)))))
