;; wat-scripts/scratch-pad/dot-flip-ask-residuals.wat — class ⑦ phase C, the inverted ask.
;; Reads bare NEW (dot-spelled) candidate names on stdin, asks variant-parent-of about each,
;; prints "<bare> -> <parent or NONE>".
(:wat::core::defn :user::ask [bare <- :wat::core::String] -> :wat::core::String
  (:wat::core::match
    (:wat::runtime::variant-parent-of (:wat::keyword::from-string bare))
    [:wat::core::Option.Some {:value parent}
      (:wat::string::concat bare (:wat::string::concat "  VARIANT of " (:wat::keyword::to-string parent)))]
    [:wat::core::Option.None {}
      (:wat::string::concat bare "  -- NONE")]))

(:wat::core::defn :user::ask-each [names <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? names)
    nil
    (:wat::core::do
      (:wat::kernel::println (:user::ask (:wat::core::first names)))
      (:user::ask-each (:wat::core::rest names)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::match (:wat::kernel::readln)
    [:wat::kernel::ReadlnOutcome.Datum {:v names} (:user::ask-each names)]
    [:wat::kernel::ReadlnOutcome.Eof {} (:wat::kernel::assertion-failed! :message "readln: end of input")]
    [:wat::kernel::ReadlnOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "readln: stop requested")]))
