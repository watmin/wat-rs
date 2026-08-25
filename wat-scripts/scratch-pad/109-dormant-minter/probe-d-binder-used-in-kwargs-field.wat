;; Extra robustness check (not one of the brief's 3 rows): T consumed by a FIELD
;; INSIDE the trailing `& [...]` kwargs section itself, not just a leading positional
;; param — exercises the non-empty `kw-tp-syms` path on `::Kwargs`'s OWN declaration
;; (record-def), not just the $impl fn's signature. Expect 7.
(:wat::core::defn :dm109d::hold :- [T]
  [& [payload <- :T]]
  -> :T
  payload)

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::string::interpolate "D={d}"
      :d (:wat::core::i64::to-string (:dm109d::hold :payload 7)))))
