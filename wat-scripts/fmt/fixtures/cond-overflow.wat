;; Nested cond whose ALIGNED width exceeds 120. Built first (STOP-2).
;; Widest test and widest body sit on DIFFERENT clauses: each clause fits
;; alone; padding the short tests toward the wide one would not.
(:wat::core::defn :fix::cond-overflow
  [k <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::cond
    ((:wat::string::= k "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx") "a")
    ((:wat::string::= k "short") "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")
    (:else "c")))
