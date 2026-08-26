;; tests/wat_lang/probe_undefined_builtin_resolves.wat
;; Control: valid dispatchable operator — must keep resolving (startup OK).

(:wat::core::defn :t::add [] -> :wat::core::i64
  (:wat::i64::+ 1 2))
