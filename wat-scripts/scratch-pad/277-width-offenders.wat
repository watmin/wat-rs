;; Scratch — print every formatted line that exceeds the ruled 120, with its width
;; and its indent, so the offenders can be classified by SHAPE rather than counted.
;; Measured inside wat: the fmt drivers println their output EDN-escaped, and decoding
;; that in the shell is a second instrument that can lie about where lines break.
(:wat::load-file! "../fmt/rules/defn.wat")
(:wat::load-file! "../fmt/rules/siblings.wat")
(:wat::load-file! "../fmt/rules/match.wat")
(:wat::load-file! "../fmt/rules/let.wat")
(:wat::load-file! "../fmt/rules/let-blank.wat")
(:wat::load-file! "../fmt/rules/kwargs.wat")
(:wat::load-file! "../fmt/rules/table.wat")
(:wat::load-file! "../fmt/rules/atoms.wat")
(:wat::load-file! "../fmt/rules/defrecord.wat")

;; Leading-space count of a line.
(:wat::core::defn :user::indent-of
  [line <- :wat::core::String
   i    <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::if (:wat::i64::>= i (:wat::string::length line))
    i
    (:wat::core::if (:wat::core::= (:wat::string::subs line i (:wat::i64::+ i 1)) " ")
      (:user::indent-of line (:wat::i64::+ i 1))
      i)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [argv  (:wat::runtime::argv)
     path  (:wat::core::Option/expect (:wat::core::get argv 2)
             "usage: wat wat-scripts/scratch-pad/277-width-offenders.wat <file.wat>")
     src   (:wat::io::read-file path)
     rules (:wat::rete::collect-rules :fmt)
     out   (:wat::fmt::format-source path src rules)]
    (:wat::core::foldl
      (:wat::core::fn [acc  <- :wat::core::nil
                       line <- :wat::core::String]
        -> :wat::core::nil
        (:wat::core::if (:wat::i64::> (:wat::string::length line) 120)
          (:wat::kernel::println
            (:wat::string::interpolate
              "w={w} indent={i} :: {t}"
              :w (:wat::i64::to-string (:wat::string::length line))
              :i (:wat::i64::to-string (:user::indent-of line 0))
              :t (:wat::string::subs line (:user::indent-of line 0) (:wat::string::length line))))
          acc))
      nil
      (:wat::string::split out "\n"))))
