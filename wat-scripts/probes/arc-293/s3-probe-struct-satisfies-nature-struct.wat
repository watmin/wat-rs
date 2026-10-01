;; s3-probe-struct-satisfies-nature-struct.wat — gate row 3 for 293 S3-Nature-2.
;; QUESTION: after adding the fourth Nature variant (Peer), does an ORDINARY struct still
;; extend-type-satisfy a `:nature :wat::core::Struct` surface (the aggregate rank-ladder path,
;; UNCHANGED by this stone — the `else` branch of `nature_floor_ok`)? MUST type-check.
;; CLAIM: (Bumper/bump c) == 42.

(:wat::core::defstruct :probe::Counter [n <- wat.type/i64])

(:wat::core::defsurface :probe::Bumper :nature wat.type/Struct
  :features [(bump [self <- :probe::Bumper] -> wat.type/i64)])

(:wat::core::extend-type :probe::Counter :probe::Bumper
  (bump [self] (:wat::core::+ (:probe::Counter/n self) 1)))

(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [c (:probe::Counter 41)
     r (:probe::Bumper/bump c)]
    (:wat::core::do
      (:wat::test::assert-eq r 42)
      (:wat::kernel::println (:wat::string::concat "struct-as-Bumper bump = " (:wat::i64::to-string r))))))
