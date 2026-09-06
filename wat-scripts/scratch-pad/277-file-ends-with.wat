;; Scratch — how does the formatted file END? Measured inside wat, because the fmt
;; drivers println their output EDN-escaped and a shell decode is a second instrument
;; that can lie about trailing newlines. Splitting on "\n" makes the tail countable:
;;   "…)\n"    -> [… ")" ""]        one trailing empty part
;;   "…)\n\n"  -> [… ")" "" ""]     two — the blank-line bug
;;   "…)"      -> [… ")"]           zero — the overcorrection
(:wat::load-file! "../fmt/rules/defn.wat")
(:wat::load-file! "../fmt/rules/siblings.wat")
(:wat::load-file! "../fmt/rules/match.wat")
(:wat::load-file! "../fmt/rules/let.wat")
(:wat::load-file! "../fmt/rules/let-blank.wat")
(:wat::load-file! "../fmt/rules/kwargs.wat")
(:wat::load-file! "../fmt/rules/table.wat")
(:wat::load-file! "../fmt/rules/atoms.wat")
(:wat::load-file! "../fmt/rules/defrecord.wat")

(:wat::core::defn :user::trailing-empties
  [parts <- (:wat::core::Vector :- [:wat::core::String])
   i     <- :wat::core::i64
   n     <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::if (:wat::i64::< i 0)
    n
    (:wat::core::if (:wat::string::empty? (:wat::core::nth parts i))
      (:user::trailing-empties parts (:wat::i64::- i 1) (:wat::i64::+ n 1))
      n)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [argv  (:wat::runtime::argv)
     path  (:wat::core::Option/expect (:wat::core::get argv 2)
             "usage: wat wat-scripts/scratch-pad/277-file-ends-with.wat <file.wat>")
     src   (:wat::io::read-file path)
     rules (:wat::rete::collect-rules :fmt)
     out   (:wat::fmt::format-source path src rules)
     sparts (:wat::string::split src "\n")
     sn     (:wat::core::length sparts)
     sempt  (:user::trailing-empties sparts (:wat::i64::- sn 1) 0)
     parts (:wat::string::split out "\n")
     n     (:wat::core::length parts)
     empties   (:user::trailing-empties parts (:wat::i64::- n 1) 0)
     last-text (:wat::core::nth parts (:wat::i64::- (:wat::i64::- n empties) 1))]
    (:wat::kernel::println
      (:wat::string::interpolate
        "SOURCE-trailing-empties={s} FORMATTED-trailing-empties={e} (1 = ONE newline) last={l}"
        :s (:wat::i64::to-string sempt)
        :e (:wat::i64::to-string empties)
        :l last-text))))
