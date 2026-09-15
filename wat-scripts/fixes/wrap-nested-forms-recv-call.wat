;; wat-scripts/fixes/wrap-nested-forms-recv-call.wat — BRIEF-3 D2 (a) call form.
;; SCOPE: wat-scripts/probes/arc-170/
;;
;; WRAP family: wrap every `:wat::kernel::recv` call in its RecvOutcome match so a
;; result used as the message (not already matched) extracts Message. Rewrites
;; ONLY inside a nested `(:wat::core::forms …)` child — the parent already faces
;; RecvOutcome. Today's `.` spelling.
;;
;;   (recv self)  ->  (match (recv self)
;;                      [:wat::kernel::RecvOutcome.Message {:msg __d} __d]
;;                      [:wat::kernel::RecvOutcome.Lost …]
;;                      [:wat::kernel::RecvOutcome.Stopped …]
;;                      [:wat::kernel::RecvOutcome.Closed …])
;;
;; Idempotent: a match whose VECTOR arm heads already contain `RecvOutcome.` skips
;; its scrutinee (today's bracket arms; wrap-calls-in-match only saw list arms).
;;
;; Usage:
;;   printf '["pathA" …]\n' | ./target/release/wat ./wat-scripts/fixes/wrap-nested-forms-recv-call.wat

(:wat::core::defn :user::kw-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "keyword")
    (:wat::core::ast-name n) ""))

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

(:wat::core::defn :user::wrapped-recv-match? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::fix::head-name node) ":wat::core::match")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::< (:wat::core::length ch) 3)
        false
        (:user::any-arm-head-contains? (:wat::core::into [] (:wat::core::drop ch 2)) "RecvOutcome.")))
    false))

(:wat::core::defn :user::node-edits
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])  in-forms <- :wat::core::bool]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::let
    [in-forms (:wat::core::or in-forms (:wat::core::= (:wat::fix::head-name node) ":wat::core::forms"))]
    (:wat::core::if (:wat::core::if in-forms (:user::wrapped-recv-match? node) false)
      (:wat::core::let [ch (:wat::core::ast->children node)]
        (:user::seq-edits
          (:wat::core::concat
            (:wat::core::into [] (:wat::core::take ch 1))
            (:wat::core::into [] (:wat::core::drop ch 2)))
          lines true))
      (:wat::core::let
        [this (:wat::core::if (:wat::core::if in-forms (:wat::fix::calls-to? node ":wat::kernel::recv") false)
                (:wat::fix::wrap-edits node
                  "(:wat::core::match "
                  " [:wat::kernel::RecvOutcome.Message {:msg __d} __d] [:wat::kernel::RecvOutcome.Lost {:cause __c} (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message __c))] [:wat::kernel::RecvOutcome.Stopped {} (:wat::kernel::assertion-failed! :message \"recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open\")] [:wat::kernel::RecvOutcome.Closed {} (:wat::kernel::assertion-failed! :message \"recv': peer closed\")])"
                  lines)
                (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))]
        (:wat::core::if (:wat::fix::structural? node)
          (:wat::core::concat this (:user::seq-edits (:wat::core::ast->children node) lines in-forms))
          this)))))

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
        (:wat::kernel::println (:wat::string::concat "[wrap-nested-recv-call] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each (:wat::core::match (:wat::kernel::readln)
    [:wat::kernel::ReadlnOutcome.Datum {:v __datum} __datum]
    [:wat::kernel::ReadlnOutcome.Eof {} (:wat::kernel::assertion-failed! :message "readln: end of input")]
    [:wat::kernel::ReadlnOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "readln: stop requested")])))
