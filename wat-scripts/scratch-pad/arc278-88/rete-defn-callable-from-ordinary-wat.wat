;; Arc 278 #88 — STOP-5 empirical confirmation: a rete-defn stays callable from ORDINARY wat,
;; like Postgres IMMUTABLE — it is a fn carrying an extra guarantee, never a separate callable
;; namespace. Declare a clean rete-defn and call it directly, outside any `where`/rule context.

(:wat::rete::core::defn :probe88::big? [n <- :wat::core::i64] -> :wat::core::bool
  (:wat::rete::i64::> n 100))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:probe88::big? 150)))
