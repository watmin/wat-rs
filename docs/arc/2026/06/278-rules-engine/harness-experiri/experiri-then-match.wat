;; experiri-then-match.wat — the D5 repro (see README.md beside this file, "What it proves" §2).
;;
;; rune:lint(red-by-design) — the refusal PROVES that `match` is rejected in `:then` while the
;;   byte-identical expression is accepted in the `where` fence (experiri-when-match.wat, which
;;   loads and prints "loaded"). `validate/mod.rs:747`'s `walk_nested_constructors` cannot tell a
;;   match ARM from a CALL: `(:probe::E::A true)` has an enum-variant head, so the arity check
;;   fires the variant's 0 declared fields against the arm's length 1. A reader can check the
;;   sentence: startup raises `RhsArityMismatch` naming a `:then` INSERT of `:probe::E::A` — an
;;   insert that appears nowhere in the source below. If this file ever loads, D5 is cured and the
;;   rune must go with it.
;;
(:wat::core::defenum :probe::E :wat::enum::Pure :A :B)

(:wat::core::defrecord :probe::In  [k <- :wat::core::String  v <- :probe::E])
(:wat::core::defrecord :probe::Out [k <- :wat::core::String  ok <- :wat::core::bool])

;; IDENTICAL match expression, three positions. Uncomment one rule at a time.
(:wat::rete::defrule :probe::in-then
  :when  [(:probe::In (?k <- :k) (?v <- :v))]
  :then  [(:probe::Out :k ?k :ok (:wat::rete::core::match ?v (:probe::E::A true) (:probe::E::B false)))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "loaded"))
