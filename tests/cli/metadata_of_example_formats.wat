;; Drive format-source from the metadata-of LOOKUP, not an :wat::intrinsic::examples scan.
;; Row 12 of STONE-metadata-of-answers-with-the-whole-row.
(:wat::load-file! "../../wat-scripts/fmt/rules/defn.wat")
(:wat::load-file! "../../wat-scripts/fmt/rules/siblings.wat")
(:wat::load-file! "../../wat-scripts/fmt/rules/match.wat")
(:wat::load-file! "../../wat-scripts/fmt/rules/if.wat")
(:wat::load-file! "../../wat-scripts/fmt/rules/cond.wat")
(:wat::load-file! "../../wat-scripts/fmt/rules/let.wat")
(:wat::load-file! "../../wat-scripts/fmt/rules/let-blank.wat")
(:wat::load-file! "../../wat-scripts/fmt/rules/kwargs.wat")
(:wat::load-file! "../../wat-scripts/fmt/rules/table.wat")
(:wat::load-file! "../../wat-scripts/fmt/rules/atoms.wat")
(:wat::load-file! "../../wat-scripts/fmt/rules/defrecord.wat")

(:wat::core::defn :user::widest
  [s <- :wat::core::String]
  -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [m <- :wat::core::i64  line <- :wat::core::String] -> :wat::core::i64
      (:wat::core::if (:wat::i64::> (:wat::string::length line) m)
        (:wat::string::length line)
        m))
    0
    (:wat::string::split s "\n")))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [rules (:wat::rete::collect-rules :fmt)
     meta  (:wat::core::Option/expect
             (:wat::runtime::metadata-of :wat::rete::step-payload)
             "metadata-of step-payload")
     examples (:wat::core::Option/expect
                (:wat::core::get meta :examples)
                "metadata-of :examples")
     ex0   (:wat::core::Option/expect (:wat::core::get examples 0) "examples[0]")
     expr  (:wat::core::Option/expect (:wat::core::get ex0 0) "examples[0][0]")
     src   (:wat::core::ast->source expr)
     out   (:wat::fmt::format-source "<registry>" src rules)]
    (:wat::core::do
      (:wat::kernel::println
        (:wat::string::interpolate "SOURCE  widest={a}   FORMATTED widest={b}"
          :a (:wat::i64::to-string (:user::widest src))
          :b (:wat::i64::to-string (:user::widest out))))
      nil)))
