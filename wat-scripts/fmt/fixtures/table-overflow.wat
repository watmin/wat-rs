;; Nested same-head group whose ALIGNED width exceeds 120.
;; Compound values so each member's own rule is kwargs, not AllAtoms.
(:wat::core::defn :fix::table-overflow
  [p <- :wat::core::String]
  -> :wat::core::nil
  (:wat::core::do
    (:wat::grep::Unreadable :file p :reason (:wat::string::concat p "one") :line 0 :col 0)
    (:wat::grep::Unreadable :file p :reason (:wat::string::concat p "two-much-longer-reason-string-XXXX") :line 1 :col 1)
    nil))
