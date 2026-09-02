;; wat-scripts/fixes/outcome-composes.wat — arc 278: Outcome composes.
;;
;; Self-hosted fix-wat codemod — NO hand-editing of .wat files to migrate.
;; Rewrites the six hard-coded Outcome combinations to Continue/Stop with fields,
;; and internal-arm constructions to SelfOutcome (no reply field).
;;
;;   Outcome::Reply         s r       → Continue s (Some (Reply::Var r)) [] []
;;   Outcome::NoReply       s         → Continue s None [] []     (public)
;;                                    → SelfOutcome::Continue s [] []  (internal)
;;   Outcome::ReplyAndArm   s r arms  → Continue s (Some (Reply::Var r)) [] arms
;;   Outcome::NoReplyAndArm s arms    → Continue s None [] arms   (public)
;;                                    → SelfOutcome::Continue s [] arms (internal)
;;   Outcome::ReplyTo       s sends   → Continue s None sends []  (public)
;;                                    → SelfOutcome::Continue s sends [] (internal)
;;
;; Internal Outcome::Reply is LEFT UNTOUCHED so the type rejects it (rung 3).
;; Idempotent: Continue / SelfOutcome heads are skipped (ctor-kind does not name them).
;;
;; Wrap uses kebab->pascal, not kebab->pascal-in: an acronym-scoped op (create-web-acl →
;; CreateWebACL) needs a one-site follow-up. Parametric :satisfies `(Surface :- [K V])`
;; cannot be spliced as `{s}::Reply`; those empty-sends types are a follow-up too.
;;
;; STOP-1: uses only existing :wat::fix:: verbs (structural?, fix-text-offset-of,
;; fix-text-apply). No new fix verb. No Rust change. No stash-dance.
;;
;; Walker copies mandate-invocation-ctx-param.wat: every defservice, top-level OR nested
;; (a quasiquoted template is an ordinary List). Arms come from :impls, not from any
;; 3-child list — a `let` is not an arm.
;;
;; Usage:
;;   printf '["pathA" "pathB" …]\n' | ./target/release/wat ./wat-scripts/fixes/outcome-composes.wat
;; Exclude wat/service.wat — the enum and serve loop are the stone, not a construction site.

(:wat::core::defn :user::kw-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::ast->source n))

(:wat::core::defn :user::start-off
  [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-span n) lines))

(:wat::core::defn :user::end-off
  [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-end-span n) lines))

(:wat::core::defn :user::empty-sends [surface <- :wat::core::String] -> :wat::core::String
  (:wat::core::format
    "(:wat::core::Vector :- [(:wat::service::Directed :- [{s}::Reply])])"
    :s surface))

(:wat::core::defn :user::empty-arms [fqdn <- :wat::core::String] -> :wat::core::String
  (:wat::core::format
    "(:wat::core::Vector :- [(:wat::service::Alarm :- [{s}::Op])])"
    :s fqdn))

(:wat::core::defn :user::op-pascal [op <- :wat::core::String] -> :wat::core::String
  (:wat::string::kebab->pascal
    (:wat::core::if (:wat::string::starts-with? op "-")
      (:wat::string::subs op 1 (:wat::string::length op))
      op)))

;; Prefix/suffix so a multi-line reply is WRAPPED in place (insertions, empty old-text).
;; Replacing ast->source fails when the printer collapses newlines (sqs.wat arms vector).
(:wat::core::defn :user::wrap-prefix
  [surface <- :wat::core::String  op <- :wat::core::String  reply-src <- :wat::core::String] -> :wat::core::String
  (:wat::core::if (:wat::string::contains? reply-src "::Reply::")
    "(:wat::core::Some "
    (:wat::core::if (:wat::string::starts-with? surface ":")
      (:wat::core::format "(:wat::core::Some ({s}::Reply::{v} "
        :s surface :v (:user::op-pascal op))
      "(:wat::core::Some ")))

(:wat::core::defn :user::wrap-suffix
  [surface <- :wat::core::String  reply-src <- :wat::core::String] -> :wat::core::String
  (:wat::core::if (:wat::string::contains? reply-src "::Reply::")
    ")"
    (:wat::core::if (:wat::string::starts-with? surface ":") "))" ")")))

(:wat::core::defn :user::none-edits []
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))

(:wat::core::defn :user::defservice-form? [form <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::let [ch (:wat::core::ast->children form)]
    (:wat::core::if (:wat::core::empty? ch)
      false
      (:wat::core::= (:user::kw-name (:wat::core::first ch)) ":wat::service::defservice"))))

(:wat::core::defn :user::index-after-keyword
  [ch <- (:wat::core::Vector :- [:wat::WatAST])  kw <- :wat::core::String  i <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::if (:wat::i64::>= i (:wat::core::length ch))
    -1
    (:wat::core::if (:wat::core::= (:user::kw-name (:wat::core::nth ch i)) kw)
      (:wat::i64::+ i 1)
      (:user::index-after-keyword ch kw (:wat::i64::+ i 1)))))

(:wat::core::defn :user::satisfies-of [form <- :wat::WatAST] -> :wat::core::String
  (:wat::core::let [ch (:wat::core::ast->children form)
                    idx (:user::index-after-keyword ch ":satisfies" 0)]
    (:wat::core::if (:wat::i64::< idx 0)
      ""
      (:wat::core::if (:wat::i64::>= idx (:wat::core::length ch))
        ""
        (:user::kw-name (:wat::core::nth ch idx))))))

(:wat::core::defn :user::fqdn-of [form <- :wat::WatAST] -> :wat::core::String
  (:wat::core::let [ch (:wat::core::ast->children form)]
    (:wat::core::if (:wat::i64::< (:wat::core::length ch) 2)
      ""
      (:user::kw-name (:wat::core::nth ch 1)))))

(:wat::core::defn :user::arms-of
  [form <- :wat::WatAST] -> (:wat::core::Vector :- [:wat::WatAST])
  (:wat::core::let [ch  (:wat::core::ast->children form)
                    idx (:user::index-after-keyword ch ":impls" 0)]
    (:wat::core::if (:wat::i64::< idx 0)
      (:wat::core::Vector :- [:wat::WatAST])
      (:wat::core::if (:wat::i64::>= idx (:wat::core::length ch))
        (:wat::core::Vector :- [:wat::WatAST])
        (:wat::core::ast->children (:wat::core::nth ch idx))))))

(:wat::core::defn :user::ctor-kind [head <- :wat::core::String] -> :wat::core::String
  (:wat::core::if (:wat::core::= head ":wat::service::Outcome::Reply") "reply"
    (:wat::core::if (:wat::core::= head ":wat::service::Outcome::NoReply") "noreply"
      (:wat::core::if (:wat::core::= head ":wat::service::Outcome::ReplyAndArm") "reply-arm"
        (:wat::core::if (:wat::core::= head ":wat::service::Outcome::NoReplyAndArm") "noreply-arm"
          (:wat::core::if (:wat::core::= head ":wat::service::Outcome::ReplyTo") "replyto"
            ""))))))

;; Rewrite one construction list. Returns edits (offset, old, new).
(:wat::core::defn :user::ctor-edits
  [node <- :wat::WatAST
   lines <- (:wat::core::Vector :- [:wat::core::String])
   surface <- :wat::core::String
   fqdn <- :wat::core::String
   op <- :wat::core::String
   internal? <- :wat::core::bool]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::let
    [ch (:wat::core::ast->children node)]
    (:wat::core::if (:wat::core::empty? ch)
      (:user::none-edits)
      (:wat::core::let
        [kind (:user::ctor-kind (:user::kw-name (:wat::core::first ch)))]
        (:wat::core::if (:wat::core::= kind "")
          (:user::none-edits)
          (:wat::core::if (:wat::core::and internal? (:wat::core::= kind "reply"))
            (:user::none-edits)
            (:wat::core::let
              [head-n (:wat::core::first ch)
               head (:user::kw-name head-n)
               before-close (:wat::i64::- (:user::end-off node lines) 1)
               new-head (:wat::core::if internal?
                          ":wat::service::SelfOutcome::Continue"
                          ":wat::service::Outcome::Continue")
               head-edit (:wat::core::Tuple (:user::start-off head-n lines) head new-head)
               es (:user::empty-sends surface)
               ea (:user::empty-arms fqdn)]
              (:wat::core::if (:wat::core::= kind "reply")
                (:wat::core::let
                  [reply-n (:wat::core::nth ch 2)
                   rs (:wat::core::ast->source reply-n)]
                  (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
                    head-edit
                    (:wat::core::Tuple (:user::start-off reply-n lines) "" (:user::wrap-prefix surface op rs))
                    (:wat::core::Tuple before-close ""
                      (:wat::string::concat (:user::wrap-suffix surface rs)
                        (:wat::string::concat " " (:wat::string::concat es (:wat::string::concat " " ea)))))))
                (:wat::core::if (:wat::core::= kind "noreply")
                  (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
                    head-edit
                    (:wat::core::Tuple before-close ""
                      (:wat::core::if internal?
                        (:wat::string::concat " " (:wat::string::concat es (:wat::string::concat " " ea)))
                        (:wat::string::concat " :wat::core::None " (:wat::string::concat es (:wat::string::concat " " ea))))))
                  (:wat::core::if (:wat::core::= kind "reply-arm")
                    (:wat::core::let
                      [reply-n (:wat::core::nth ch 2)
                       rs (:wat::core::ast->source reply-n)
                       arms-n (:wat::core::nth ch 3)]
                      (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
                        head-edit
                        (:wat::core::Tuple (:user::start-off reply-n lines) "" (:user::wrap-prefix surface op rs))
                        (:wat::core::Tuple (:user::end-off reply-n lines) "" (:user::wrap-suffix surface rs))
                        (:wat::core::Tuple (:user::start-off arms-n lines) "" (:wat::string::concat es " "))))
                    (:wat::core::if (:wat::core::= kind "noreply-arm")
                      (:wat::core::let
                        [arms-n (:wat::core::nth ch 2)
                         prefix (:wat::core::if internal? es
                                  (:wat::string::concat ":wat::core::None " es))]
                        (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
                          head-edit
                          (:wat::core::Tuple (:user::start-off arms-n lines) ""
                            (:wat::string::concat prefix " "))))
                      (:wat::core::if (:wat::core::= kind "replyto")
                        (:wat::core::if internal?
                          (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
                            head-edit
                            (:wat::core::Tuple before-close "" (:wat::string::concat " " ea)))
                          (:wat::core::let
                            [sends-n (:wat::core::nth ch 2)]
                            (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
                              head-edit
                              (:wat::core::Tuple (:user::start-off sends-n lines) "" ":wat::core::None ")
                              (:wat::core::Tuple before-close "" (:wat::string::concat " " ea)))))
                        (:user::none-edits)))))))))))))

(:wat::core::defn :user::seq-edits
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   lines <- (:wat::core::Vector :- [:wat::core::String])
   surface <- :wat::core::String
   fqdn <- :wat::core::String
   op <- :wat::core::String
   internal? <- :wat::core::bool]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
                     it <- :wat::WatAST]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
      (:wat::core::concat acc (:user::body-edits it lines surface fqdn op internal?)))
    (:user::none-edits)
    items))

(:wat::core::defn :user::body-edits
  [node <- :wat::WatAST
   lines <- (:wat::core::Vector :- [:wat::core::String])
   surface <- :wat::core::String
   fqdn <- :wat::core::String
   op <- :wat::core::String
   internal? <- :wat::core::bool]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch)
        (:user::none-edits)
        (:wat::core::concat
          (:user::ctor-edits node lines surface fqdn op internal?)
          (:user::seq-edits ch lines surface fqdn op internal?))))
    (:wat::core::if (:wat::fix::structural? node)
      (:user::seq-edits (:wat::core::ast->children node) lines surface fqdn op internal?)
      (:user::none-edits))))

(:wat::core::defn :user::arm-edits
  [arm <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])
   surface <- :wat::core::String  fqdn <- :wat::core::String]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::let [ch (:wat::core::ast->children arm)]
    (:wat::core::if (:wat::i64::< (:wat::core::length ch) 3)
      (:user::none-edits)
      (:wat::core::let
        [op-str (:user::kw-name (:wat::core::first ch))
         internal? (:wat::string::starts-with? op-str "-")]
        (:user::body-edits arm lines surface fqdn op-str internal?)))))

(:wat::core::defn :user::arms-edits
  [arms <- (:wat::core::Vector :- [:wat::WatAST])  lines <- (:wat::core::Vector :- [:wat::core::String])
   surface <- :wat::core::String  fqdn <- :wat::core::String]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
                     arm <- :wat::WatAST]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
      (:wat::core::concat acc (:user::arm-edits arm lines surface fqdn)))
    (:user::none-edits)
    arms))

;; Nested defservice walk — same reason as mandate-invocation-ctx-param.wat.
(:wat::core::defn :user::node-edits
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch)
        (:user::none-edits)
        (:wat::core::let
          [this (:wat::core::if (:user::defservice-form? node)
                  (:user::arms-edits (:user::arms-of node) lines
                    (:user::satisfies-of node) (:user::fqdn-of node))
                  (:user::none-edits))]
          (:wat::core::concat this (:user::file-seq ch lines)))))
    (:wat::core::if (:wat::fix::structural? node)
      (:user::file-seq (:wat::core::ast->children node) lines)
      (:user::none-edits))))

(:wat::core::defn :user::file-seq
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
                     it <- :wat::WatAST]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
      (:wat::core::concat acc (:user::node-edits it lines)))
    (:user::none-edits)
    items))

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let
    [lines (:wat::string::split src "\n")
     forms (:wat::core::ast->children
             (:wat::core::match (:wat::core::read-string src)
               ((:wat::core::ReadOutcome::Forms __forms) __forms)
               ((:wat::core::ReadOutcome::Malformed __cause)
                 (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None))))
     eds (:user::file-seq forms lines)
     rev (:wat::core::reverse (:wat::core::sort eds))]
    (:wat::fix::fix-text-apply src rev)))

(:wat::core::defn :user::apply-each [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[outcome-composes] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln )
      ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
      (:wat::kernel::ReadlnOutcome::Eof
        (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
      (:wat::kernel::ReadlnOutcome::Stopped
        (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
