;; pprintln of a :wat::doc::Row — the runtime printer. Capture is stdout.
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

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::pprintln
    (:wat::doc::from-map
      (:wat::core::Option/expect
        (:wat::runtime::metadata-of :wat::rete::step-payload)
        "metadata-of"))))
