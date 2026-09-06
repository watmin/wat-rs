;; Scratch — count comments on BOTH sides of a format. The fmt drivers print the
;; count from the SOURCE only, so "no comment is lost" cannot be read off them; the
;; output has to be re-read through the same reader.
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

(:wat::core::defn :user::count-comments
  [tag <- :wat::core::String
   s   <- :wat::core::String]
  -> :wat::core::i64
  (:wat::core::match (:wat::core::read-string-with-comments s)
    ((:wat::core::ReadWithCommentsOutcome::Forms forms comments)
      (:wat::core::length comments))
    ((:wat::core::ReadWithCommentsOutcome::Malformed cause)
      (:wat::kernel::assertion-failed!
        (:wat::string::concat tag (:wat::core::Error/message cause))
        :wat::core::None :wat::core::None))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [argv  (:wat::runtime::argv)
     path  (:wat::core::Option/expect (:wat::core::get argv 2)
             "usage: wat wat-scripts/scratch-pad/277-comments-both-sides.wat <file.wat>")
     src   (:wat::io::read-file path)
     rules (:wat::rete::collect-rules :fmt)
     out   (:wat::fmt::format-source path src rules)]
    (:wat::kernel::println
      (:wat::string::interpolate "source={s} formatted={f} path={p}"
        :s (:wat::i64::to-string (:user::count-comments "source: " src))
        :f (:wat::i64::to-string (:user::count-comments "formatted: " out))
        :p path))))
