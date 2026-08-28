;; wat-scripts/scratch-pad/255-home4-verify-examples-failures.wat — arc 255 home #4
;; phase 2 (the string/uuid/char/regex/list carve): print each verify-examples
;; failure's fqdn + reason, so row 4 can be confirmed by NAME, not just count.
;; Scratch, per holon/CLAUDE.md's `.wat` scratch convention. `foldl`, not `map`,
;; so the println side effects are forced (map over a lazy Seqable doesn't
;; force without a consumer).

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [failures (:wat::doctest::verify-examples)]
    (:wat::core::do
      (:wat::kernel::println (:wat::string::interpolate "TOTAL FAILURES: {n}" :n (:wat::i64::to-string (:wat::core::length failures))))
      (:wat::core::foldl
        (:wat::core::fn [acc <- :wat::core::i64 f <- :wat::doctest::Failure] -> :wat::core::i64
          (:wat::core::do
            (:wat::kernel::println
              (:wat::string::interpolate "{fqdn}  ::  {reason}"
                :fqdn (:wat::keyword::to-string (:wat::doctest::Failure/fqdn f))
                :reason (:wat::doctest::Failure/reason f)))
            (:wat::i64::+ acc 1)))
        0
        failures)
      nil)))
