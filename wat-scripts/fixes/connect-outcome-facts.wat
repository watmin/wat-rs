;; wat-scripts/fixes/connect-outcome-facts.wat — stone 255.34.
;; SCOPE: corpus
;;
;; ConnectOutcome.Refused becomes ConnectOutcome.Closed. The arm body stays.
;; ConnectOutcome.Rejected becomes two arms, Undialable and WrongPeer, each
;; with that same body. Inside the ConnectOutcome defenum only, :Refused
;; becomes :Closed and :Rejected becomes :Undialable plus :WrongPeer.
;; Other enums keep their own :Refused and :Rejected.
;;
;; Idempotent: a second run finds neither old head.
;;
;; spec [:wat::kernel::ConnectOutcome.Closed {:cause c} "same"] :: ConnectOutcome.Refused arm → a ConnectOutcome.Closed arm (same body)
;; spec [:wat::kernel::ConnectOutcome.Undialable {:cause c} "same"] [:wat::kernel::ConnectOutcome.WrongPeer {:cause c} "same"] :: ConnectOutcome.Rejected arm → two arms, Undialable and WrongPeer, each with the same body
;; spec :Closed [cause <- :wat::kernel::Failure] :: :Refused becomes :Closed inside the ConnectOutcome defenum
;; spec :Undialable [cause <- :wat::kernel::Failure] :WrongPeer [cause <- :wat::kernel::Failure] :: :Rejected becomes :Undialable plus :WrongPeer inside the ConnectOutcome defenum
;; spec-before :Refused [cause <- :wat::kernel::Failure] :: the ConnectOutcome defenum's :Refused variant
;; spec-before :Rejected [cause <- :wat::kernel::Failure] :: the ConnectOutcome defenum's :Rejected variant
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["pathA" "pathB"]\n' | ./target/release/wat ./wat-scripts/fixes/connect-outcome-facts.wat

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

(:wat::core::defn :user::connect-defenum?
  [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::fix::calls-to? node ":wat::core::defenum")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::< (:wat::core::length ch) 2)
        false
        (:wat::core::= (:wat::fix::kw-name (:wat::core::nth ch 1)) ":wat::kernel::ConnectOutcome")))
    false))

(:wat::core::defn :user::variant-edit
  [kw <- :wat::WatAST
   vec <- :wat::WatAST
   src <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::let [name (:wat::fix::kw-name kw)]
    (:wat::core::if (:wat::core::= name ":Refused")
      (:wat::core::Vector :- [:wat::fix::Edit]
        (:wat::core::Tuple
          (:wat::fix::node-start-offset kw lines)
          (:user::slice src kw lines)
          ":Closed"))
      (:wat::core::if (:wat::core::= name ":Rejected")
        (:wat::core::let
          [ks  (:wat::fix::node-start-offset kw lines)
           ke  (:wat::fix::node-end-offset kw lines)
           vs  (:wat::fix::node-start-offset vec lines)
           ve  (:wat::fix::node-end-offset vec lines)
           gap (:wat::string::subs src ke vs)
           fields (:wat::string::subs src vs ve)
           old (:wat::string::subs src ks ve)
           one (:wat::string::concat ":Undialable" (:wat::string::concat gap fields))
           new (:wat::string::concat one (:wat::string::concat " :WrongPeer" (:wat::string::concat gap fields)))]
          (:wat::core::Vector :- [:wat::fix::Edit]
            (:wat::core::Tuple ks old new)))
        (:user::no-edits)))))

(:wat::core::defn :user::variant-edits
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   src <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:wat::core::empty? items)
    (:user::no-edits)
    (:wat::core::let [h (:wat::core::first items)
                      tl (:wat::core::rest items)]
      (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::ast-kind h) "keyword")
                        (:wat::core::if (:wat::core::not (:wat::core::empty? tl))
                          (:wat::core::= (:wat::core::ast-kind (:wat::core::first tl)) "vector")
                          false)
                        false)
        (:wat::core::concat
          (:user::variant-edit h (:wat::core::first tl) src lines)
          (:user::variant-edits (:wat::core::rest tl) src lines))
        (:user::variant-edits tl src lines)))))

(:wat::core::defn :user::refused-edit
  [arm <- :wat::WatAST
   src <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  ;; A match arm is a vector: [keyword map body…]. The head keyword is child 0.
  (:wat::core::let
    [kw (:wat::core::first (:wat::core::ast->children arm))]
    (:wat::core::Vector :- [:wat::fix::Edit]
      (:wat::core::Tuple
        (:wat::fix::node-start-offset kw lines)
        (:user::slice src kw lines)
        ":wat::kernel::ConnectOutcome.Closed"))))

(:wat::core::defn :user::rejected-edit
  [arm <- :wat::WatAST
   src <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::let
    [text (:user::slice src arm lines)
     und  (:wat::string::join ":wat::kernel::ConnectOutcome.Undialable"
            (:wat::string::split text ":wat::kernel::ConnectOutcome.Rejected"))
     wrong (:wat::string::join ":wat::kernel::ConnectOutcome.WrongPeer"
             (:wat::string::split text ":wat::kernel::ConnectOutcome.Rejected"))]
    (:wat::core::Vector :- [:wat::fix::Edit]
      (:wat::core::Tuple
        (:wat::fix::node-start-offset arm lines)
        text
        (:wat::string::concat und (:wat::string::concat " " wrong))))))

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

(:wat::core::defn :user::arm-edit
  [arm <- :wat::WatAST
   src <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::let [head (:user::arm-head arm)]
    (:wat::core::if (:wat::core::= head ":wat::kernel::ConnectOutcome.Refused")
      (:user::refused-edit arm src lines)
      (:wat::core::if (:wat::core::= head ":wat::kernel::ConnectOutcome.Rejected")
        (:user::rejected-edit arm src lines)
        (:user::no-edits)))))

(:wat::core::defn :user::walk
  [node <- :wat::WatAST
   src <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:user::connect-defenum? node)
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
          ;; A Rejected arm is replaced wholesale, so an edit inside it would
          ;; overlap that span. Refused only renames its head keyword.
          (:wat::core::if (:wat::core::= head ":wat::kernel::ConnectOutcome.Rejected")
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
        (:wat::kernel::println (:wat::string::concat "[connect-outcome-facts] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln)
      [:wat::kernel::ReadlnOutcome.Datum {:v paths} paths]
      [:wat::kernel::ReadlnOutcome.Eof {}
        (:wat::kernel::assertion-failed! :message "readln: end of input")]
      [:wat::kernel::ReadlnOutcome.Stopped {}
        (:wat::kernel::assertion-failed! :message "readln: stop requested")])))
