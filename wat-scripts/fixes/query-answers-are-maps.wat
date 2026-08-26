;; wat-scripts/fixes/query-answers-are-maps.wat
;;
;; Query returns binding maps. A leftover consumer that types the answer as a
;; fact is talking to the old mouth. Re-architect:
;;
;;   1. defquery `:when [(:ns::T …)]`  →  `:when [(?fact <- :ns::T …)]`
;;   2. map/filter/foldl fns whose ITEM is a user record  →  PersistentMap,
;;      then bind the old name to `(get p "?fact")`.
;;
;; `:wat::rete::{not,exists,where,and,or}` when-entries are left alone.
;; Already-fact-bound conditions (`?x <- :ns::T`) are left alone.
;; Idempotent.
;;
;;   printf '["pathA" "pathB"]\n' | ./target/release/wat wat-scripts/fixes/query-answers-are-maps.wat

(:wat::core::defn :user::quoted?
  [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::fix::calls-to? node ":wat::core::quote")
    true
    (:wat::fix::calls-to? node ":wat::core::quasiquote")))

(:wat::core::defn :user::user-type-kw?
  [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "keyword")
    (:wat::core::let [nm (:wat::core::ast-name node)]
      (:wat::core::if (:wat::string::contains? nm "::")
        (:wat::core::not (:wat::string::starts-with? nm ":wat::"))
        false))
    false))

(:wat::core::defn :user::plain-type-cond?
  [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::= (:wat::core::length ch) 1)
        (:user::user-type-kw? (:wat::core::first ch))
        false))
    false))

;; `(?fact <- :Type …extra)` — we over-wrapped a field pattern. Strip the
;; shared binder so two conditions do not join on the same `?fact`.
(:wat::core::defn :user::overwrapped?
  [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)
                      n  (:wat::core::length ch)]
      (:wat::core::if (:wat::i64::> n 3)
        (:wat::core::if (:wat::core::= (:wat::core::ast-name (:wat::core::first ch)) "?fact")
          (:wat::core::if (:wat::core::= (:wat::core::ast-name
                                           (:wat::core::Option/expect
                                             (:wat::core::get ch 1)
                                             "overwrapped?: arrow"))
                                         "<-")
            (:user::user-type-kw?
              (:wat::core::Option/expect (:wat::core::get ch 2) "overwrapped?: type"))
            false)
          false)
        false))
    false))

(:wat::core::defn :user::node-text
  [node  <- :wat::WatAST
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::String
  (:wat::string::subs src
    (:wat::fix::node-start-offset node lines)
    (:wat::fix::node-end-offset node lines)))

;; old-text = fix-text-span-text over the WHOLE matched node's OWN span (arc 282) —
;; sanctioned: every caller of span-edit has already structurally verified `node`'s
;; identity (e.g. calls-to? ":wat::core::fn") before calling this, and it is a List's
;; own span — never a reader-synthesized leaf's — being replaced wholesale.
(:wat::core::defn :user::span-edit
  [node  <- :wat::WatAST
   text  <- :wat::core::String
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::fix::Edit
  (:wat::core::let [off      (:wat::fix::node-start-offset node lines)
                    old-text (:wat::fix::fix-text-span-text (:wat::core::ast-span node) (:wat::core::ast-end-span node) lines src)]
    (:wat::core::Tuple off old-text text)))

(:wat::core::defn :user::insert-fact-bind
  [node  <- :wat::WatAST
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::fix::Edit
  (:wat::core::Tuple
    (:wat::i64::+ (:wat::fix::node-start-offset node lines) 1)
    ""
    "?fact <- "))

(:wat::core::defn :user::when-edits
  [node  <- :wat::WatAST
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:wat::fix::calls-to? node ":wat::rete::defquery")
    (:wat::core::let [ch (:wat::core::ast->children node)
                      n  (:wat::core::length ch)]
      (:user::when-edits-scan ch 0 n lines))
    (:wat::core::Vector :wat::fix::Edit)))

(:wat::core::defn :user::when-edits-scan
  [ch    <- (:wat::core::Vector :- [:wat::WatAST])
   i     <- :wat::core::i64
   n     <- :wat::core::i64
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:wat::i64::>= i n)
    (:wat::core::Vector :wat::fix::Edit)
    (:wat::core::let [kid (:wat::core::Option/expect
                            (:wat::core::get ch i)
                            "when-edits-scan")]
      (:wat::core::if
        (:wat::core::if (:wat::core::= (:wat::core::ast-kind kid) "keyword")
          (:wat::core::= (:wat::core::ast-name kid) ":when")
          false)
        (:wat::core::if (:wat::i64::< (:wat::i64::+ i 1) n)
          (:user::when-vec-edits
            (:wat::core::Option/expect
              (:wat::core::get ch (:wat::i64::+ i 1))
              "when-edits-scan: vec")
            lines)
          (:wat::core::Vector :wat::fix::Edit))
        (:user::when-edits-scan ch (:wat::i64::+ i 1) n lines)))))

(:wat::core::defn :user::when-vec-edits
  [vec   <- :wat::WatAST
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind vec) "vector")
    (:user::when-vec-scan (:wat::core::ast->children vec) 0
      (:wat::core::length (:wat::core::ast->children vec)) lines)
    (:wat::core::Vector :wat::fix::Edit)))

(:wat::core::defn :user::when-vec-scan
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   i     <- :wat::core::i64
   n     <- :wat::core::i64
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:wat::i64::>= i n)
    (:wat::core::Vector :wat::fix::Edit)
    (:wat::core::let [c (:wat::core::Option/expect (:wat::core::get items i) "when-vec-scan")]
      (:wat::core::concat
        (:wat::core::if (:user::plain-type-cond? c)
          (:wat::core::Vector :wat::fix::Edit (:user::insert-fact-bind c lines))
          (:wat::core::if (:user::overwrapped? c)
            ;; old-text = the literal "?fact <- " (9 chars) — overwrapped? already verified
            ;; c's shape is `(?fact <- :Type …)`, so this is exactly what the rule believes
            ;; immediately follows the opening paren; NEVER span text (this claims a SPECIFIC
            ;; literal, not "whatever's there").
            (:wat::core::Vector :wat::fix::Edit
              (:wat::core::Tuple
                (:wat::i64::+ (:wat::fix::node-start-offset c lines) 1)
                "?fact <- "
                ""))
            (:wat::core::Vector :wat::fix::Edit)))
        (:user::when-vec-scan items (:wat::i64::+ i 1) n lines)))))

(:wat::core::defn :user::record-item-name
  [params <- :wat::WatAST] -> (:wat::core::Option :- [:wat::core::String])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind params) "vector")
    (:wat::core::let [pch (:wat::core::ast->children params)
                      pn  (:wat::core::length pch)]
      (:wat::core::if (:wat::core::not (:wat::core::= pn 3))
        :wat::core::None
        (:wat::core::let [last3 0
                          a (:wat::core::Option/expect (:wat::core::get pch last3) "rec-name a")
                          b (:wat::core::Option/expect (:wat::core::get pch (:wat::i64::+ last3 1)) "rec-name b")
                          c (:wat::core::Option/expect (:wat::core::get pch (:wat::i64::+ last3 2)) "rec-name c")]
          (:wat::core::if
            (:wat::core::if (:wat::core::= (:wat::core::ast-kind a) "symbol")
              (:wat::core::if (:wat::core::= (:wat::core::ast-name b) "<-")
                (:user::user-type-kw? c)
                false)
              false)
            (:wat::core::Some (:wat::core::ast-name a))
            :wat::core::None))))
    :wat::core::None))

(:wat::core::defn :user::rewrite-fn-text
  [fn-node <- :wat::WatAST
   src     <- :wat::core::String
   lines   <- (:wat::core::Vector :- [:wat::core::String])
   item    <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::let [ch (:wat::core::ast->children fn-node)
                    n  (:wat::core::length ch)
                    body (:wat::core::Option/expect
                           (:wat::core::get ch (:wat::i64::- n 1))
                           "rewrite-fn: body")
                    body-t (:user::node-text body src lines)
                    has-arrow
                      (:wat::core::if (:wat::i64::> n 3)
                        (:wat::core::= (:wat::core::ast-name
                                         (:wat::core::Option/expect
                                           (:wat::core::get ch 2)
                                           "rewrite-fn: arrow"))
                                       "->")
                        false)
                    ret
                      (:wat::core::if has-arrow
                        (:user::node-text
                          (:wat::core::Option/expect
                            (:wat::core::get ch 3)
                            "rewrite-fn: ret")
                          src lines)
                        "")
                    head
                      (:wat::core::if has-arrow
                        (:wat::string::concat
                          "(:wat::core::fn [p <- :wat::core::PersistentMap] -> "
                          (:wat::string::concat ret " "))
                        "(:wat::core::fn [p <- :wat::core::PersistentMap] ")]
    (:wat::string::concat head
      (:wat::string::concat
        "(:wat::core::let ["
        (:wat::string::concat item
          (:wat::string::concat
            " (:wat::core::Option/expect (:wat::core::PersistentMap/get p \"?fact\") \"query: ?fact\")] "
            (:wat::string::concat body-t "))")))))))

(:wat::core::defn :user::hof-fn-edits
  [node  <- :wat::WatAST
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if
    (:wat::core::if (:wat::fix::calls-to? node ":wat::core::map")
      true
      (:wat::core::if (:wat::fix::calls-to? node ":wat::core::filter")
        true
        (:wat::fix::calls-to? node ":wat::core::foldl")))
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::i64::< (:wat::core::length ch) 2)
        (:wat::core::Vector :wat::fix::Edit)
        (:wat::core::let [fn-node (:wat::core::Option/expect
                                    (:wat::core::get ch 1)
                                    "hof-fn-edits: fn")]
          (:wat::core::if (:wat::fix::calls-to? fn-node ":wat::core::fn")
            (:wat::core::let [params (:wat::core::Option/expect
                                       (:wat::core::get (:wat::core::ast->children fn-node) 1)
                                       "hof-fn-edits: params")]
              (:wat::core::match (:user::record-item-name params)
                ((:wat::core::Some nm)
                 (:wat::core::Vector :wat::fix::Edit
                   (:user::span-edit fn-node
                     (:user::rewrite-fn-text fn-node src lines nm)
                     src lines)))
                (:wat::core::None (:wat::core::Vector :wat::fix::Edit))))
            (:wat::core::Vector :wat::fix::Edit)))))
    (:wat::core::Vector :wat::fix::Edit)))

(:wat::core::defn :user::walk-edits
  [node  <- :wat::WatAST
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:user::quoted? node)
    (:wat::core::Vector :wat::fix::Edit)
    (:wat::core::let [this (:wat::core::concat
                             (:user::when-edits node lines)
                             (:user::hof-fn-edits node src lines))]
      (:wat::core::if (:wat::fix::structural? node)
        (:wat::core::concat this
          (:user::walk-seq (:wat::core::ast->children node) src lines))
        this))))

(:wat::core::defn :user::walk-seq
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector :wat::fix::Edit)
    (:wat::core::concat
      (:user::walk-edits (:wat::core::first items) src lines)
      (:user::walk-seq (:wat::core::into [] (:wat::core::rest items)) src lines))))

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let
    [lines (:wat::string::split src "\n")
     tree  (:wat::core::match (:wat::core::read-string src)
             ((:wat::core::ReadOutcome::Forms __forms) __forms)
             ((:wat::core::ReadOutcome::Malformed __cause)
               (:wat::kernel::assertion-failed!
                 (:wat::core::Error/message __cause)
                 :wat::core::None :wat::core::None)))
     forms (:wat::core::ast->children tree)
     edits (:user::walk-seq forms src lines)]
    (:wat::core::if (:wat::core::empty? edits)
      src
      (:wat::fix::fix-text-apply src
        (:wat::core::reverse (:wat::core::sort edits))))))

(:wat::core::defn :user::rewrite-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)
                      src  (:wat::io::read-file path)
                      out  (:user::migrate src)]
      (:wat::core::do
        (:wat::io::write-file path out)
        (:wat::kernel::println
          (:wat::string::concat
            (:wat::core::if (:wat::core::= src out) "[unchanged] " "[rewritten] ")
            path))
        (:user::rewrite-each (:wat::core::into [] (:wat::core::rest paths)))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::rewrite-each
    (:wat::core::match (:wat::kernel::readln)
      ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
      (:wat::kernel::ReadlnOutcome::Eof
        (:wat::kernel::assertion-failed! "readln: end of input"
          :wat::core::None :wat::core::None))
      (:wat::kernel::ReadlnOutcome::Stopped
        (:wat::kernel::assertion-failed! "readln: stop requested"
          :wat::core::None :wat::core::None)))))
