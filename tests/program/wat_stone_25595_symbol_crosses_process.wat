;; Stone 255.95 — a symbol value built in the parent is spliced into the
;; child program as `(quote <symbol>)`, the form the process-wall encoder
;; emits. The child `apply`s that value and resolves the function it names.
(:wat::core::defn :my::test::symbol-applies-across-a-process [] -> wat.type/i64
  (:wat::core::let
    [sym (:wat::core::symbol "user.probe/inc")
     inc (:wat::core::quasiquote
           (:wat::core::defn user.probe/inc [n :- wat.type/i64] -> wat.type/i64
             (:wat::i64::+ n 1)))
     entry (:wat::core::quasiquote
             (:wat::core::defn :user::main [] -> wat.type/nil
               (:wat::kernel::println
                 (:wat::core::apply
                   (:wat::core::quote (:wat::core::unquote sym))
                   2 []))))
     prog (wat.type/Vector :- [wat.type/AST] inc entry)
     p (:wat::test::spawn-peer (:wat::spawn::process) prog)]
    (:wat::core::match (:wat::kernel::recv p)
      [:wat::kernel::RecvOutcome.Message {:msg m} m]
      [:wat::kernel::RecvOutcome.Lost {:cause cause}
        (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message cause))]
      [:wat::kernel::RecvOutcome.Stopped {}
        (:wat::kernel::assertion-failed! :message "symbol apply: stop requested before the child sent its value")]
      [:wat::kernel::RecvOutcome.Closed {}
        (:wat::kernel::assertion-failed! :message "symbol apply: child closed before sending its value")])))
