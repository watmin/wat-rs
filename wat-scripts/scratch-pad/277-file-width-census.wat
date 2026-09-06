;; Scratch — width census of a WHOLE file's formatted output, measured INSIDE wat.
;; The fmt drivers print the output through println, which EDN-escapes it; decoding
;; that in the shell is a second instrument that can lie about line breaks.
;; This measures source and formatted width on the same string the emitter produced.
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

(:wat::core::defn :user::stats
  [s <- :wat::core::String]
  -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::foldl
    (:wat::core::fn [acc  <- (:wat::core::PersistentVector :- [:wat::core::i64])
                     line <- :wat::core::String]
      -> (:wat::core::PersistentVector :- [:wat::core::i64])
      (:wat::core::let
        [w     (:wat::string::length line)
         over  (:wat::core::nth acc 0)
         worst (:wat::core::nth acc 1)
         n     (:wat::core::nth acc 2)
         cmt   (:wat::core::nth acc 3)]
        (:wat::core::PersistentVector :- [:wat::core::i64]
          (:wat::core::if (:wat::i64::> w 120) (:wat::i64::+ over 1) over)
          (:wat::core::if (:wat::i64::> w worst) w worst)
          (:wat::i64::+ n 1)
          (:wat::core::if (:wat::core::and (:wat::i64::> w 120)
                            (:wat::string::contains? (:wat::string::trim line) ";;"))
            (:wat::i64::+ cmt 1)
            cmt))))
    (:wat::core::PersistentVector :- [:wat::core::i64] 0 0 0 0)
    (:wat::string::split s "\n")))

(:wat::core::defn :user::report
  [tag <- :wat::core::String
   s   <- :wat::core::String]
  -> :wat::core::nil
  (:wat::core::let [st (:user::stats s)]
    (:wat::kernel::println
      (:wat::string::interpolate
        "{t} over120={o} worst={w} lines={n} over120-with-comment={c}"
        :t tag
        :o (:wat::i64::to-string (:wat::core::nth st 0))
        :w (:wat::i64::to-string (:wat::core::nth st 1))
        :n (:wat::i64::to-string (:wat::core::nth st 2))
        :c (:wat::i64::to-string (:wat::core::nth st 3))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [argv  (:wat::runtime::argv)
     path  (:wat::core::Option/expect (:wat::core::get argv 2)
             "usage: wat wat-scripts/scratch-pad/277-file-width-census.wat <file.wat>")
     src   (:wat::io::read-file path)
     rules (:wat::rete::collect-rules :fmt)
     out   (:wat::fmt::format-source path src rules)]
    (:wat::core::do
      (:user::report "SOURCE   " src)
      (:user::report "FORMATTED" out))))
