(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [src "`(:wat::kernel::Frame/file ~origin-sym)"
                    tree (:wat::core::match (:wat::core::read-string src) [:wat::core::ReadOutcome.Forms {:forms __forms} __forms] [:wat::core::ReadOutcome.Malformed {:cause __cause} (:wat::kernel::assertion-failed! :message "parse")])
                    forms (:wat::core::ast->children tree)
                    quasi (:wat::core::Option/expect (:wat::core::get forms 0) "quasi")
                    qch (:wat::core::ast->children quasi)
                    inner (:wat::core::Option/expect (:wat::core::get qch 1) "inner")
                    ich (:wat::core::ast->children inner)
                    arg (:wat::core::Option/expect (:wat::core::get ich 1) "arg")
                    argspan (:wat::core::ast-span arg)
                    argend (:wat::core::ast-end-span arg)
                    argl (:wat::core::Option/expect (:wat::hashmap::get argspan :line) "l")
                    argc (:wat::core::Option/expect (:wat::hashmap::get argspan :col) "c")
                    argel (:wat::core::Option/expect (:wat::hashmap::get argend :line) "el")
                    argec (:wat::core::Option/expect (:wat::hashmap::get argend :col) "ec")]
    (:wat::kernel::println (:wat::core::format "inner-children={n} arg-kind={k} start={l}:{c} end={el}:{ec}"
      :n (:wat::i64::to-string (:wat::core::length ich))
      :k (:wat::core::ast-kind arg)
      :l (:wat::i64::to-string argl) :c (:wat::i64::to-string argc)
      :el (:wat::i64::to-string argel) :ec (:wat::i64::to-string argec)))))
