(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [p  (:wat::kernel::spawn-program' (:wat::spawn::process)
          (:wat::core::forms (:wat::core::defn :user::main [] -> :wat::core::nil nil)))
     f  (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
          (:wat::core::let [_ (:wat::core::match (:wat::kernel::send' p x) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) ((:wat::kernel::SendOutcome::Lost _c) nil))] x))
     wf (:wat::kernel::fn-forms f :probe::work)]
    (:wat::kernel::println "LEAK: impure capture was reified without error")))
