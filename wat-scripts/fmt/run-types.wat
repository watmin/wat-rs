;; Probe — print how many type applications the predicate recognises.
;; A green over zero proves nothing.
(:wat::load-file! "rules/defn.wat")
(:wat::load-file! "rules/siblings.wat")
(:wat::load-file! "rules/match.wat")
(:wat::load-file! "rules/let.wat")
(:wat::load-file! "rules/let-blank.wat")

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [argv (:wat::runtime::argv)
     path (:wat::core::Option/expect (:wat::core::get argv 2) "usage: wat run-types.wat <file.wat>")
     src  (:wat::io::read-file path)]
    (:wat::core::match (:wat::core::read-string-with-comments src)
      ((:wat::core::ReadWithCommentsOutcome::Forms forms comments)
        (:wat::core::let
          [decls (:wat::fmt::count-type-apps forms)
           colon (:wat::fmt::count-colon-args forms)
           rules (:wat::rete::collect-rules :fmt)
           out (:wat::fmt::format-source path src rules)]
          (:wat::core::do
            (:wat::kernel::println
              (:wat::string::interpolate "TYPE_DECLS={d} COLON_ARGS={c} COMMENTS={k}"
                :d (:wat::i64::to-string decls)
                :c (:wat::i64::to-string colon)
                :k (:wat::i64::to-string (:wat::core::length comments))))
            (:wat::kernel::println out))))
      ((:wat::core::ReadWithCommentsOutcome::Malformed cause)
        (:wat::kernel::assertion-failed! (:wat::core::Error/message cause) :wat::core::None :wat::core::None)))))
