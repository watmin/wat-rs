;; Write formatted output to a path. println EDN-escapes; write-file does not.
(:wat::load-file! "../fmt/rules/defn.wat")
(:wat::load-file! "../fmt/rules/siblings.wat")
(:wat::load-file! "../fmt/rules/match.wat")
(:wat::load-file! "../fmt/rules/if.wat")
(:wat::load-file! "../fmt/rules/cond.wat")
(:wat::load-file! "../fmt/rules/let.wat")
(:wat::load-file! "../fmt/rules/let-blank.wat")
(:wat::load-file! "../fmt/rules/kwargs.wat")
(:wat::load-file! "../fmt/rules/table.wat")
(:wat::load-file! "../fmt/rules/atoms.wat")
(:wat::load-file! "../fmt/rules/defrecord.wat")
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [argv (:wat::runtime::argv)
     path (:wat::core::Option/expect (:wat::core::get argv 2) "need src")
     dest (:wat::core::Option/expect (:wat::core::get argv 3) "need dest")
     src  (:wat::io::read-file path)
     rules (:wat::rete::collect-rules :fmt)
     out   (:wat::fmt::format-source path src rules)]
    (:wat::io::write-file dest out)))
