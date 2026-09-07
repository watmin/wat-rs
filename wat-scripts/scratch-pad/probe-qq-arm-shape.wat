(:wat::core::defn :user::dump [label <- :wat::core::String n <- :wat::WatAST] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::string::concat label
      (:wat::string::concat " kind="
        (:wat::string::concat (:wat::core::ast-kind n)
          (:wat::string::concat " head="
            (:wat::string::concat (:wat::fix::head-name n)
              (:wat::string::concat " nch="
                (:wat::string::interpolate "{n}" :n (:wat::core::length (:wat::core::ast->children n)))))))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [src (:wat::io::read-file "wat-scripts/scratch-pad/probe-qq-arm-shape-src.wat")
     tree (:wat::core::match (:wat::core::read-string src)
             [:wat::core::ReadOutcome::Forms {:forms __forms} __forms]
             [:wat::core::ReadOutcome::Malformed {:cause __cause}
               (:wat::kernel::assertion-failed!
                 (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)])
     forms (:wat::core::ast->children tree)
     f0 (:wat::core::Option/expect (:wat::core::get forms 0) "f0")
     f1 (:wat::core::Option/expect (:wat::core::get forms 1) "f1")]
    (:wat::core::do
      (:user::dump "f0" f0)
      (:user::dump "f0.0" (:wat::core::Option/expect (:wat::core::get (:wat::core::ast->children f0) 0) "f0.0"))
      (:user::dump "f0.1" (:wat::core::Option/expect (:wat::core::get (:wat::core::ast->children f0) 1) "f0.1"))
      (:user::dump "f1" f1)
      (:wat::kernel::println (:wat::core::write-forms tree)))))
