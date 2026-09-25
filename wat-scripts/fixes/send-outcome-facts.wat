;; wat-scripts/fixes/send-outcome-facts.wat — stone 255.36.
;; SCOPE: corpus
;;
;; SendOutcome.Closed and TrySendOutcome.Closed become HandleClosed. The
;; arm body stays. Those variants are nullary, and every arm binds {}.
;; SendOutcome.Lost and TrySendOutcome.Lost become two arms, Closed and
;; Failed, each with that same body. The cause those arms bind is a
;; Failure now, so LociDiedError/message inside the copied arm becomes
;; Failure/message. Inside the SendOutcome and TrySendOutcome defenums
;; only, a unit :Closed becomes :HandleClosed and :Lost becomes :Closed
;; plus :Failed, each [cause <- Failure]. Other enums keep :Closed and :Lost.
;;
;; Idempotent: a second run finds neither old head. The new :Closed carries
;; a field vector, so it is not the unit keyword this pass renames.
;;
;; spec [:wat::kernel::SendOutcome.HandleClosed {} "same"] :: SendOutcome.Closed arm → a SendOutcome.HandleClosed arm (same body)
;; spec [:wat::kernel::SendOutcome.Closed {:cause c} "same"] [:wat::kernel::SendOutcome.Failed {:cause c} "same"])) :: SendOutcome.Lost arm → two arms, Closed and Failed, each with the same body
;; spec :HandleClosed :: a unit :Closed becomes :HandleClosed inside the SendOutcome defenum
;; spec :Closed [cause <- :wat::kernel::Failure] :Failed [cause <- :wat::kernel::Failure]) :: :Lost becomes :Closed plus :Failed inside the SendOutcome defenum
;; spec-before :Closed :: the SendOutcome defenum's unit :Closed variant
;; spec-before :Lost [cause <- :wat::kernel::LociDiedError]) :: the SendOutcome defenum's :Lost variant
;; spec-before [:wat::kernel::SendOutcome.Closed {} "same"] :: SendOutcome.Closed arm → a SendOutcome.HandleClosed arm (same body)
;; spec-before [:wat::kernel::SendOutcome.Lost {:cause c} "same"])) :: SendOutcome.Lost arm → two arms, Closed and Failed, each with the same body
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["pathA" "pathB"]\n' | ./target/release/wat ./wat-scripts/fixes/send-outcome-facts.wat

(:wat::core::defn :user::no-edits []
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::Vector :- [:wat::fix::Edit]))

(:wat::core::defn :user::slice
  [src <- :wat::core::String
   node <- :wat::WatAST
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::String
  (:wat::string::subs src
    (:wat::fix::node-start-offset node lines)
    (:wat::fix::node-end-offset node lines)))

(:wat::core::defn :user::send-defenum?
  [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::fix::calls-to? node ":wat::core::defenum")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::< (:wat::core::length ch) 2)
        false
        (:wat::core::let [name (:wat::fix::kw-name (:wat::core::nth ch 1))]
          (:wat::core::or
            (:wat::core::= name ":wat::kernel::SendOutcome")
            (:wat::core::= name ":wat::kernel::TrySendOutcome")))))
    false))

(:wat::core::defn :user::unit-closed-edit
  [kw <- :wat::WatAST
   src <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::Vector :- [:wat::fix::Edit]
    (:wat::core::Tuple
      (:wat::fix::node-start-offset kw lines)
      (:user::slice src kw lines)
      ":HandleClosed")))

(:wat::core::defn :user::lost-variant-edit
  [kw <- :wat::WatAST
   vec <- :wat::WatAST
   src <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::let
    [old (:wat::string::subs src
           (:wat::fix::node-start-offset kw lines)
           (:wat::fix::node-end-offset vec lines))
     new ":Closed [cause <- :wat::kernel::Failure] :Failed [cause <- :wat::kernel::Failure]"]
    (:wat::core::Vector :- [:wat::fix::Edit]
      (:wat::core::Tuple
        (:wat::fix::node-start-offset kw lines)
        old
        new))))

(:wat::core::defn :user::variant-edits
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   src <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:wat::core::empty? items)
    (:user::no-edits)
    (:wat::core::let [h (:wat::core::first items)
                      tl (:wat::core::rest items)]
      (:wat::core::if (:wat::core::= (:wat::core::ast-kind h) "keyword")
        (:wat::core::let [name (:wat::fix::kw-name h)
                          nxt (:wat::core::if (:wat::core::empty? tl) "" (:wat::core::ast-kind (:wat::core::first tl)))]
          (:wat::core::if (:wat::core::if (:wat::core::= name ":Closed")
                            (:wat::core::not (:wat::core::= nxt "vector"))
                            false)
            (:wat::core::concat
              (:user::unit-closed-edit h src lines)
              (:user::variant-edits tl src lines))
            (:wat::core::if (:wat::core::if (:wat::core::= name ":Lost")
                              (:wat::core::= nxt "vector")
                              false)
              (:wat::core::concat
                (:user::lost-variant-edit h (:wat::core::first tl) src lines)
                (:user::variant-edits (:wat::core::rest tl) src lines))
              (:wat::core::if (:wat::core::= nxt "vector")
                (:user::variant-edits (:wat::core::rest tl) src lines)
                (:user::variant-edits tl src lines)))))
        (:user::variant-edits tl src lines)))))

(:wat::core::defn :user::arm-head [arm <- :wat::WatAST] -> :wat::core::String
  (:wat::core::let [k (:wat::core::ast-kind arm)]
    (:wat::core::if (:wat::core::or (:wat::core::= k "list") (:wat::core::= k "vector"))
      (:wat::core::let [ch (:wat::core::ast->children arm)]
        (:wat::core::if (:wat::core::empty? ch)
          ""
          (:wat::core::let [pat (:wat::core::first ch)
                            pk (:wat::core::ast-kind pat)]
            (:wat::core::if (:wat::core::or (:wat::core::= pk "list") (:wat::core::= pk "vector"))
              (:wat::core::let [pch (:wat::core::ast->children pat)]
                (:wat::core::if (:wat::core::empty? pch) "" (:wat::fix::kw-name (:wat::core::first pch))))
              (:wat::fix::kw-name pat)))))
      "")))

(:wat::core::defn :user::handle-closed-name [head <- :wat::core::String] -> :wat::core::String
  (:wat::string::join ".HandleClosed" (:wat::string::split head ".Closed")))

(:wat::core::defn :user::closed-arm-edit
  [arm <- :wat::WatAST
   src <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::let [kw (:wat::core::first (:wat::core::ast->children arm))]
    (:wat::core::Vector :- [:wat::fix::Edit]
      (:wat::core::Tuple
        (:wat::fix::node-start-offset kw lines)
        (:user::slice src kw lines)
        (:user::handle-closed-name (:user::arm-head arm))))))

(:wat::core::defn :user::fix-accessor [text <- :wat::core::String] -> :wat::core::String
  (:wat::string::join ":wat::kernel::Failure/message"
    (:wat::string::split text ":wat::kernel::LociDiedError/message")))

(:wat::core::defn :user::lost-arm-edit
  [arm <- :wat::WatAST
   src <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::let
    [text (:user::slice src arm lines)
     head (:user::arm-head arm)
     closed-head (:wat::string::join ".Closed" (:wat::string::split head ".Lost"))
     failed-head (:wat::string::join ".Failed" (:wat::string::split head ".Lost"))
     one (:user::fix-accessor (:wat::string::join closed-head (:wat::string::split text head)))
     two (:user::fix-accessor (:wat::string::join failed-head (:wat::string::split text head)))]
    (:wat::core::Vector :- [:wat::fix::Edit]
      (:wat::core::Tuple
        (:wat::fix::node-start-offset arm lines)
        text
        (:wat::string::concat one (:wat::string::concat " " two))))))

(:wat::core::defn :user::is-closed-head [head <- :wat::core::String] -> :wat::core::bool
  (:wat::core::or
    (:wat::core::= head ":wat::kernel::SendOutcome.Closed")
    (:wat::core::= head ":wat::kernel::TrySendOutcome.Closed")))

(:wat::core::defn :user::is-lost-head [head <- :wat::core::String] -> :wat::core::bool
  (:wat::core::or
    (:wat::core::= head ":wat::kernel::SendOutcome.Lost")
    (:wat::core::= head ":wat::kernel::TrySendOutcome.Lost")))

(:wat::core::defn :user::nullary-arm?
  [arm <- :wat::WatAST
   src <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::bool
  (:wat::core::let [ch (:wat::core::ast->children arm)]
    (:wat::core::if (:wat::core::< (:wat::core::length ch) 2)
      false
      (:wat::core::= (:user::slice src (:wat::core::nth ch 1) lines) "{}"))))

(:wat::core::defn :user::arm-edit
  [arm <- :wat::WatAST
   src <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::let [head (:user::arm-head arm)]
    (:wat::core::if (:wat::core::if (:user::is-closed-head head)
                      (:user::nullary-arm? arm src lines)
                      false)
      (:user::closed-arm-edit arm src lines)
      (:wat::core::if (:user::is-lost-head head)
        (:user::lost-arm-edit arm src lines)
        (:user::no-edits)))))

(:wat::core::defn :user::walk
  [node <- :wat::WatAST
   src <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:user::send-defenum? node)
    (:wat::core::let [ch (:wat::core::ast->children node)
                      start (:wat::fix::defenum-variant-start ch)
                      body (:wat::core::into [] (:wat::core::drop ch start))]
      (:user::variant-edits body src lines))
    (:wat::core::if (:wat::core::= (:wat::fix::head-name node) ":wat::core::match")
      (:wat::core::let [ch (:wat::core::ast->children node)]
        (:wat::core::if (:wat::core::< (:wat::core::length ch) 3)
          (:user::no-edits)
          (:wat::core::concat
            (:user::walk (:wat::core::nth ch 1) src lines)
            (:user::arm-walk (:wat::core::into [] (:wat::core::drop ch 2)) src lines))))
      (:wat::core::if (:wat::fix::structural? node)
        (:user::walk-list (:wat::core::ast->children node) src lines)
        (:user::no-edits)))))

(:wat::core::defn :user::arm-walk
  [arms <- (:wat::core::Vector :- [:wat::WatAST])
   src <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:wat::core::empty? arms)
    (:user::no-edits)
    (:wat::core::let [arm (:wat::core::first arms)
                      head (:user::arm-head arm)]
      (:wat::core::concat
        (:user::arm-edit arm src lines)
        (:wat::core::concat
          (:wat::core::if (:user::is-lost-head head)
            (:user::no-edits)
            (:user::walk arm src lines))
          (:user::arm-walk (:wat::core::rest arms) src lines))))))

(:wat::core::defn :user::walk-list
  [nodes <- (:wat::core::Vector :- [:wat::WatAST])
   src <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:wat::core::empty? nodes)
    (:user::no-edits)
    (:wat::core::concat
      (:user::walk (:wat::core::first nodes) src lines)
      (:user::walk-list (:wat::core::rest nodes) src lines))))

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let
    [lines (:wat::string::split src "\n")
     tree  (:wat::core::match (:wat::core::read-string src)
             [:wat::core::ReadOutcome.Forms {:forms forms} forms]
             [:wat::core::ReadOutcome.Malformed {:cause cause}
               (:wat::kernel::assertion-failed! :message (:wat::core::Error/message cause))])
     edits (:user::walk-list (:wat::core::ast->children tree) src lines)]
    (:wat::fix::fix-text-apply src (:wat::core::reverse edits))))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[send-outcome-facts] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln)
      [:wat::kernel::ReadlnOutcome.Datum {:v paths} paths]
      [:wat::kernel::ReadlnOutcome.Eof {}
        (:wat::kernel::assertion-failed! :message "readln: end of input")]
      [:wat::kernel::ReadlnOutcome.Stopped {}
        (:wat::kernel::assertion-failed! :message "readln: stop requested")])))
