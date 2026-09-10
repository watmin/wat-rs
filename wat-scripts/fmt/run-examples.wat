;; Format each top-level form in isolation (rules collected once).
(:wat::load-file! "rules/defn.wat")
(:wat::load-file! "rules/siblings.wat")
(:wat::load-file! "rules/match.wat")
(:wat::load-file! "rules/if.wat")
(:wat::load-file! "rules/cond.wat")
(:wat::load-file! "rules/let.wat")
(:wat::load-file! "rules/let-blank.wat")
(:wat::load-file! "rules/kwargs.wat")
(:wat::load-file! "rules/table.wat")
(:wat::load-file! "rules/atoms.wat")
(:wat::load-file! "rules/defrecord.wat")

(:wat::core::defrecord :user::Ex
  [n       <- :wat::core::i64
   changed <- :wat::core::i64
   inline  <- :wat::core::i64
   over    <- :wat::core::i64
   worst   <- :wat::core::i64])

(:wat::core::defn :user::line-stats
  [s <- :wat::core::String]
  -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::i64])
                     line <- :wat::core::String]
      -> (:wat::core::PersistentVector :- [:wat::core::i64])
      (:wat::core::let [w (:wat::string::length line)
                        over (:wat::core::nth acc 0)
                        worst (:wat::core::nth acc 1)]
        (:wat::core::PersistentVector :- [:wat::core::i64]
          (:wat::core::if (:wat::i64::> w 120) (:wat::i64::+ over 1) over)
          (:wat::core::if (:wat::i64::> w worst) w worst))))
    (:wat::core::PersistentVector :- [:wat::core::i64] 0 0)
    (:wat::string::split s "\n")))

(:wat::core::defn :user::one
  [acc   <- :user::Ex
   piece <- :wat::core::String
   rules <- (:wat::core::PersistentVector :- [:wat::rete::Rule])]
  -> :user::Ex
  (:wat::core::let
    [out   (:wat::fmt::format-source "<ex>" piece rules)
     again (:wat::fmt::format-source "<ex>" out rules)
     stats (:user::line-stats out)
     over  (:wat::core::nth stats 0)
     w     (:wat::core::nth stats 1)
     lines (:wat::string::split out "\n")
     nlines (:wat::core::length lines)
     one?  (:wat::core::if (:wat::i64::= nlines 1)
             true
             (:wat::core::if (:wat::i64::= nlines 2)
               (:wat::string::empty? (:wat::core::nth lines 1))
               false))]
    (:user::Ex
      :n       (:wat::i64::+ (:user::Ex/n acc) 1)
      :changed (:wat::core::if (:wat::core::= piece out)
                 (:user::Ex/changed acc)
                 (:wat::i64::+ (:user::Ex/changed acc) 1))
      :inline  (:wat::core::if one?
                 (:wat::i64::+ (:user::Ex/inline acc) 1)
                 (:user::Ex/inline acc))
      :over    (:wat::i64::+ (:user::Ex/over acc) over)
      :worst   (:wat::core::if (:wat::i64::> w (:user::Ex/worst acc)) w (:user::Ex/worst acc)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [argv (:wat::runtime::argv)
     path (:wat::core::Option/expect (:wat::core::get argv 2) "usage: wat run-examples.wat <file.wat>")
     src  (:wat::io::read-file path)
     rules (:wat::rete::collect-rules :fmt)]
    (:wat::core::match (:wat::core::read-string src)
      [:wat::core::ReadOutcome.Forms {:forms forms}
        (:wat::core::let
          [acc (:wat::core::foldl
                 (:wat::core::fn [a <- :user::Ex  f <- :wat::WatAST] -> :user::Ex
                   (:user::one a (:wat::core::ast->source f) rules))
                 (:user::Ex :n 0 :changed 0 :inline 0 :over 0 :worst 0)
                 (:wat::core::ast->children forms))]
          (:wat::kernel::println
            (:wat::string::interpolate
              "N={n} CHANGED={c} INLINE={i} OVER120={o} WORST={w}"
              :n (:wat::i64::to-string (:user::Ex/n acc))
              :c (:wat::i64::to-string (:user::Ex/changed acc))
              :i (:wat::i64::to-string (:user::Ex/inline acc))
              :o (:wat::i64::to-string (:user::Ex/over acc))
              :w (:wat::i64::to-string (:user::Ex/worst acc)))))]
      [:wat::core::ReadOutcome.Malformed {:cause cause}
        (:wat::kernel::assertion-failed! :message (:wat::core::Error/message cause))])))
