;; wat-scripts/fixes/wrap-nested-forms-recv-in-recvoutcome.wat — BRIEF-3 D2 (a).
;; SCOPE: wat-scripts/probes/arc-170/
;;
;; Self-hosted wrap-family codemod. a recv / client-method result matched as its message
;; is wrapped in its RecvOutcome match. Today's `.` spelling. Rewrites ONLY inside a
;; nested `(:wat::core::forms …)` child (walks every AST child; parent already faces RecvOutcome).
;;
;;   (match SCRUT <message arms>)
;;     ->  (match SCRUT
;;           [:wat::kernel::RecvOutcome.Message {:msg __recv}
;;             (match __recv <the SAME message arms, verbatim>)]
;;           [:wat::kernel::RecvOutcome.Lost {:cause __cause} …]
;;           [:wat::kernel::RecvOutcome.Stopped {} …]
;;           [:wat::kernel::RecvOutcome.Closed {} …])
;;
;; Matcher: a `:wat::core::match` whose arms do not already mention `RecvOutcome.`,
;; whose scrutinee is not `__recv` (idempotency), and that is either a `recv` call
;; or has an arm head containing `Response.`.
;;
;; Usage:
;;   printf '["pathA" …]\n' | ./target/release/wat ./wat-scripts/fixes/wrap-nested-forms-recv-in-recvoutcome.wat

(:wat::core::defn :user::start-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-span n) lines))

(:wat::core::defn :user::end-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-end-span n) lines))

(:wat::core::defn :user::kw-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "keyword")
    (:wat::core::ast-name n) ""))

;; Arm is a vector `[:Head …]` or a list `(pattern body…)`.
(:wat::core::defn :user::arm-head-name [arm <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind arm) "vector")
    (:wat::core::let [ch (:wat::core::ast->children arm)]
      (:wat::core::if (:wat::core::empty? ch) "" (:user::kw-name (:wat::core::first ch))))
    (:wat::fix::arm-head-name arm)))

(:wat::core::defn :user::any-arm-head-contains?
  [arms <- (:wat::core::Vector :- [:wat::WatAST])  needle <- :wat::core::String] -> :wat::core::bool
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::bool  arm <- :wat::WatAST] -> :wat::core::bool
      (:wat::core::if acc true
        (:wat::string::contains? (:user::arm-head-name arm) needle)))
    false arms))

(:wat::core::defn :user::scrut-is-recv? [scrut <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::= (:wat::fix::head-name scrut) ":wat::kernel::recv"))

(:wat::core::defn :user::message-match? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::fix::head-name node) ":wat::core::match")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::< (:wat::core::length ch) 3)
        false
        (:wat::core::let
          [scrut (:wat::core::Option/expect (:wat::core::get ch 1) "scrut")
           arms  (:wat::core::into [] (:wat::core::drop ch 2))
           skip  (:wat::core::if (:wat::core::= (:wat::core::ast-kind scrut) "symbol")
                   (:wat::core::= (:wat::core::ast-name scrut) "__recv") false)]
          (:wat::core::if skip
            false
            (:wat::core::if (:user::any-arm-head-contains? arms "RecvOutcome.")
              false
              (:wat::core::if (:user::any-arm-head-contains? arms "SendOutcome.")
                false
                (:wat::core::or (:user::scrut-is-recv? scrut)
                  (:user::any-arm-head-contains? arms "Response."))))))))
    false))

(:wat::core::defn :user::wrap-edits
  [ch <- (:wat::core::Vector :- [:wat::WatAST])  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::let
    [scrut    (:wat::core::Option/expect (:wat::core::get ch 1) "scrut")
     last-arm (:wat::core::Option/expect (:wat::core::get ch (:wat::core::- (:wat::core::length ch) 1)) "last")]
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
      (:wat::core::Tuple (:user::end-off scrut lines) ""
        " [:wat::kernel::RecvOutcome.Message {:msg __recv} (:wat::core::match __recv")
      (:wat::core::Tuple (:user::end-off last-arm lines) ""
        ")] [:wat::kernel::RecvOutcome.Lost {:cause __cause} (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message __cause))] [:wat::kernel::RecvOutcome.Stopped {} (:wat::kernel::assertion-failed! :message \"recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open\")] [:wat::kernel::RecvOutcome.Closed {} (:wat::kernel::assertion-failed! :message \"recv': peer closed\")]"))))

(:wat::core::defn :user::node-edits
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])  in-forms <- :wat::core::bool]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::let
    [in-forms (:wat::core::or in-forms (:wat::core::= (:wat::fix::head-name node) ":wat::core::forms"))
     this (:wat::core::if (:wat::core::if in-forms (:user::message-match? node) false)
            (:user::wrap-edits (:wat::core::ast->children node) lines)
            (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))]
    (:wat::core::if (:wat::fix::structural? node)
      (:wat::core::concat this (:user::seq-edits (:wat::core::ast->children node) lines in-forms))
      this)))

(:wat::core::defn :user::seq-edits
  [items <- (:wat::core::Vector :- [:wat::WatAST])  lines <- (:wat::core::Vector :- [:wat::core::String])  in-forms <- :wat::core::bool]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]) it <- :wat::WatAST]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
      (:wat::core::concat acc (:user::node-edits it lines in-forms)))
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
    items))

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let
    [lines (:wat::string::split src "\n")
     forms (:wat::core::ast->children (:wat::core::match (:wat::core::read-string src) [:wat::core::ReadOutcome.Forms {:forms __forms} __forms] [:wat::core::ReadOutcome.Malformed {:cause __cause} (:wat::kernel::assertion-failed! :message (:wat::core::Error/message __cause))]))
     eds   (:user::seq-edits forms lines false)
     rev   (:wat::core::reverse (:wat::core::sort eds))]
    (:wat::fix::fix-text-apply src rev)))

(:wat::core::defn :user::apply-each [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[wrap-nested-recv] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each (:wat::core::match (:wat::kernel::readln)
    [:wat::kernel::ReadlnOutcome.Datum {:v __datum} __datum]
    [:wat::kernel::ReadlnOutcome.Eof {} (:wat::kernel::assertion-failed! :message "readln: end of input")]
    [:wat::kernel::ReadlnOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "readln: stop requested")])))
