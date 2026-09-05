;; Driver — R1 + R11 + R3. let.wat is a new file; this driver only loads it.
(:wat::load-file! "rules/defn.wat")
(:wat::load-file! "rules/siblings.wat")
(:wat::load-file! "rules/let.wat")
(:wat::load-file! "rules/let-blank.wat")

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [argv (:wat::runtime::argv)
     path (:wat::core::Option/expect (:wat::core::get argv 2) "usage: wat wat-scripts/fmt/run-let.wat <file.wat>")
     src  (:wat::io::read-file path)
     rules (:wat::rete::collect-rules :fmt)
     out   (:wat::fmt::format-source path src rules)
     again (:wat::fmt::format-source path out rules)
     same? (:wat::core::= out again)]
    (:wat::core::match (:wat::core::read-string-with-comments src)
      ((:wat::core::ReadWithCommentsOutcome::Forms forms comments)
        (:wat::core::do
          (:wat::kernel::println
            (:wat::string::interpolate
              "FORMS={f} COMMENTS={c} IDEMPOTENT={i}"
              :f (:wat::i64::to-string (:wat::core::length (:wat::core::ast->children forms)))
              :c (:wat::i64::to-string (:wat::core::length comments))
              :i (:wat::core::if same? "true" "false")))
          (:wat::kernel::println out)))
      ((:wat::core::ReadWithCommentsOutcome::Malformed cause)
        (:wat::kernel::assertion-failed! (:wat::core::Error/message cause) :wat::core::None :wat::core::None)))))
