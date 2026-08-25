;; BRIEF-STONE-the-dormant-minter.md — row 1, the row the STOP triggers guard: BOTH a
;; `:-` binder and a kwargs `& [...]` section on one defn. This is the dormant
;; minter's reach (`binder-tp`, wat/core.wat:736 before the fix) — the only feature
;; combination the corpus never used. Expect 3.
(:wat::core::defn :dm109c::hold :- [T]
  [seed <- :T
   & [times <- :wat::core::i64]]
  -> :wat::core::i64
  times)

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::string::interpolate "C={c}"
      :c (:wat::core::i64::to-string (:dm109c::hold 1 :times 3)))))
