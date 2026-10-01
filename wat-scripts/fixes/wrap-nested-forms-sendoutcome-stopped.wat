;; wat-scripts/fixes/wrap-nested-forms-sendoutcome-stopped.wat — BRIEF-3 D2 (b).
;; SCOPE: wat-scripts/probes/arc-170/
;;
;; Self-hosted wrap-family codemod. a SendOutcome match lacking Stopped gains the arm.
;; Today's `.` spelling. Rewrites ONLY inside a nested `(:wat::core::forms …)` child.
;;
;;   (match (send …) [:wat::kernel::SendOutcome.Sent {} nil] [SendOutcome.Closed …] [SendOutcome.Lost …])
;;     ->  same, plus [:wat::kernel::SendOutcome.Stopped {} nil]
;;
;; Matcher: a `:wat::core::match` whose arm heads mention `SendOutcome.` and none
;; mention `Stopped`. Idempotent: a second run finds Stopped and edits nothing.
;;
;; Usage:
;;   printf '["pathA" …]\n' | ./target/release/wat ./wat-scripts/fixes/wrap-nested-forms-sendoutcome-stopped.wat

(:wat::core::defn :user::end-off [n <- wat.type/AST  lines <- (wat.type/Vector :- [wat.type/String])]
  -> wat.type/i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-end-span n) lines))

(:wat::core::defn :user::kw-name [n <- wat.type/AST] -> wat.type/String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "keyword")
    (:wat::core::ast-name n) ""))

(:wat::core::defn :user::arm-head-name [arm <- wat.type/AST] -> wat.type/String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind arm) "vector")
    (:wat::core::let [ch (:wat::core::ast->children arm)]
      (:wat::core::if (:wat::core::empty? ch) "" (:user::kw-name (:wat::core::first ch))))
    (:wat::fix::arm-head-name arm)))

(:wat::core::defn :user::any-arm-head-contains?
  [arms <- (wat.type/Vector :- [wat.type/AST])  needle <- wat.type/String] -> wat.type/bool
  (:wat::core::foldl
    (:wat::core::fn [acc <- wat.type/bool  arm <- wat.type/AST] -> wat.type/bool
      (:wat::core::if acc true
        (:wat::string::contains? (:user::arm-head-name arm) needle)))
    false arms))

(:wat::core::defn :user::needs-stopped? [node <- wat.type/AST] -> wat.type/bool
  (:wat::core::if (:wat::core::= (:wat::fix::head-name node) ":wat::core::match")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::< (:wat::core::length ch) 3)
        false
        (:wat::core::let [arms (:wat::core::into [] (:wat::core::drop ch 2))]
          (:wat::core::if (:user::any-arm-head-contains? arms "SendOutcome.")
            (:wat::core::not (:user::any-arm-head-contains? arms "Stopped"))
            false))))
    false))

(:wat::core::defn :user::stopped-edits
  [ch <- (wat.type/Vector :- [wat.type/AST])  lines <- (wat.type/Vector :- [wat.type/String])]
  -> (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])])
  (:wat::core::let
    [last-arm (:wat::core::Option/expect (:wat::core::get ch (:wat::core::- (:wat::core::length ch) 1)) "last")]
    (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])]
      (wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String] (:user::end-off last-arm lines) ""
        " [:wat::kernel::SendOutcome.Stopped {} nil]"))))

(:wat::core::defn :user::node-edits
  [node <- wat.type/AST  lines <- (wat.type/Vector :- [wat.type/String])  in-forms <- wat.type/bool]
  -> (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])])
  (:wat::core::let
    [in-forms (:wat::core::or in-forms (:wat::core::= (:wat::fix::head-name node) ":wat::core::forms"))
     this (:wat::core::if (:wat::core::if in-forms (:user::needs-stopped? node) false)
            (:user::stopped-edits (:wat::core::ast->children node) lines)
            (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])]))]
    (:wat::core::if (:wat::fix::structural? node)
      (:wat::core::concat this (:user::seq-edits (:wat::core::ast->children node) lines in-forms))
      this)))

(:wat::core::defn :user::seq-edits
  [items <- (wat.type/Vector :- [wat.type/AST])  lines <- (wat.type/Vector :- [wat.type/String])  in-forms <- wat.type/bool]
  -> (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])]) it <- wat.type/AST]
      -> (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])])
      (:wat::core::concat acc (:user::node-edits it lines in-forms)))
    (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])])
    items))

(:wat::core::defn :user::migrate [src <- wat.type/String] -> wat.type/String
  (:wat::core::let
    [lines (:wat::string::split src "\n")
     forms (:wat::core::ast->children (:wat::core::match (:wat::core::read-string src) [:wat::core::ReadOutcome.Forms {:forms __forms} __forms] [:wat::core::ReadOutcome.Malformed {:cause __cause} (:wat::kernel::assertion-failed! :message (:wat::core::Error/message __cause))]))
     eds   (:user::seq-edits forms lines false)
     rev   (:wat::core::reverse (:wat::core::sort eds))]
    (:wat::fix::fix-text-apply src rev)))

(:wat::core::defn :user::apply-each [paths <- (wat.type/Vector :- [wat.type/String])] -> wat.type/nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[wrap-nested-stopped] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> wat.type/nil
  (:user::apply-each (:wat::core::match (:wat::kernel::readln)
    [:wat::kernel::ReadlnOutcome.Datum {:v __datum} __datum]
    [:wat::kernel::ReadlnOutcome.Eof {} (:wat::kernel::assertion-failed! :message "readln: end of input")]
    [:wat::kernel::ReadlnOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "readln: stop requested")])))
