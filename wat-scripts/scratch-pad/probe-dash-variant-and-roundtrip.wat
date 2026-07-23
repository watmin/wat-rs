;; probe-dash-variant-and-roundtrip.wat — arc 278 Stone 2-A:
;;  (1) a leading-dash enum variant :Svc::Op::-Tick constructs + matches + type-checks;
;;  (2) the ast->source → split/join → read-string round-trip resolves `:op :-tick` to a
;;      variant-constructor form (the macro's keyword-:op resolution mechanism).

(:wat::core::defenum :probe-dv::Op :wat::enum::Pure
  :Ping [req <- :wat::core::i64]
  :-Tick [])

;; (1) construct + match the dash variant
(:wat::core::defn :probe-dv::fire [] -> :wat::core::i64
  (:wat::core::match (:probe-dv::Op::-Tick) 
    ((:probe-dv::Op::-Tick) 42)
    ((:probe-dv::Op::Ping n) n)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:probe-dv::fire)))
