;; wat-scripts/fixes/type-query-to-defquery.wat — rip type-readout query.
;;
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — wat rewrites wat.
;;
;; TWO heretic call shapes (the old public mouth):
;;   (:wat::rete::query-by-type-string SESSION "ns::Type")
;;   (:wat::rete::query SESSION :ns::Type)          ; 3 children, child[2] is a type keyword
;;
;; become the ONE remaining mouth (Clara-shaped QueryNode lookup):
;;   (:wat::rete::query SESSION (:ns::q-Type))
;;
;; For each unique heretic type the file names, insert (if not already present):
;;   (:wat::rete::defquery :ns::q-Type
;;     :params []
;;     :when [(:ns::Type)])
;;
;; and rewrite every 1-arg `(:wat::rete::compile X)` in the file to
;;   (:wat::rete::compile-all X (:wat::core::PersistentVector (:ns::q-T1) …))
;; so the generated queries actually sit on the network. Unused QueryNodes
;; (a compile of a rule subset that never matches that type) are empty answers.
;;
;; Quote / quasiquote subtrees are skipped — they are data, not live calls
;; (wat/query.wat's expand-time generator is a HAND-CHECK, not this pass).
;; Rust-embedded wat strings are a HAND-CHECK (the 2026-07-24 class-4 lesson).
;; Callers that map query results as records (not binding maps) are a HAND-CHECK
;; after this pass — the floor names them.
;;
;; Idempotent: a legal `(query s (:ns::q))` has a LIST as child[2], so it is
;; not a heretic; a `compile-all` is not `compile`; an existing `:ns::q-Type`
;; defquery is not re-inserted. Re-run = 0 edits.
;;
;; STOP (never silent skip):
;;   - query-by-type-string whose arity is not (session "ns::Type")
;;   - a type with no `::` (Unnamespaced)
;;   - heretic calls in a file that has no compile / compile-all
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["tests/rete/a.wat" "wat-scripts/perf/grid/b.wat"]\n' \
;;     | ./target/debug/wat ./wat-scripts/fixes/type-query-to-defquery.wat
;;
;; Dry-run on a /tmp COPY first and `diff` it.

;; ── small predicates ────────────────────────────────────────────────────────

(:wat::core::defn :user::quoted?
  [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::fix::calls-to? node ":wat::core::quote")
    true
    (:wat::fix::calls-to? node ":wat::core::quasiquote")))

(:wat::core::defn :user::decl-form?
  [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::let [h (:wat::fix::head-name node)]
    (:wat::core::if (:wat::core::= h ":wat::core::defrecord") true
      (:wat::core::if (:wat::core::= h ":wat::rete::defrule") true
        (:wat::core::if (:wat::core::= h ":wat::rete::defquery") true
          (:wat::core::if (:wat::core::= h ":wat::core::defenum") true
            (:wat::core::if (:wat::core::= h ":wat::core::defstruct") true
              (:wat::core::= h ":wat::core::defholon"))))))))

(:wat::core::defn :user::type-query?
  [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::fix::calls-to? node ":wat::rete::query")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::= (:wat::core::count ch) 3)
        (:wat::core::= (:wat::core::ast-kind
                         (:wat::core::Option/expect
                           (:wat::core::get ch 2)
                           "type-query?: child 2"))
                       "keyword")
        false))
    false))

(:wat::core::defn :user::qbts?
  [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::fix::calls-to? node ":wat::rete::query-by-type-string"))

(:wat::core::defn :user::compile-1?
  [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::fix::calls-to? node ":wat::rete::compile")
    (:wat::core::= (:wat::core::count (:wat::core::ast->children node)) 2)
    false))

(:wat::core::defn :user::compile-any?
  [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::fix::calls-to? node ":wat::rete::compile")
    true
    (:wat::fix::calls-to? node ":wat::rete::compile-all")))

;; ── name algebra ────────────────────────────────────────────────────────────

;; "wnab::Hit" → "wnab::q-Hit". STOPS on a bare (unnamespaced) type.
(:wat::core::defn :user::type->qname
  [fqdn <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [parts (:wat::core::string::split fqdn "::")
                    n     (:wat::core::length parts)]
    (:wat::core::if (:wat::core::< n 2)
      (:wat::kernel::assertion-failed!
        (:wat::core::string::concat
          "type-query-to-defquery: type has no namespace: " fqdn)
        :wat::core::None :wat::core::None)
      (:wat::core::let [ty (:wat::core::Option/expect
                             (:wat::core::get parts (:wat::core::i64::- n 1))
                             "type->qname: last")
                        ns (:wat::core::foldl
                             (:wat::core::fn [acc <- :wat::core::String
                                              i   <- :wat::core::i64]
                               -> :wat::core::String
                               (:wat::core::let [seg (:wat::core::Option/expect
                                                       (:wat::core::get parts i)
                                                       "type->qname: ns")]
                                 (:wat::core::if (:wat::core::= acc "")
                                   seg
                                   (:wat::core::string::concat acc
                                     (:wat::core::string::concat "::" seg)))))
                             ""
                             (:wat::core::range 0 (:wat::core::i64::- n 1)))]
        (:wat::core::string::concat ns
          (:wat::core::string::concat "::q-" ty))))))

(:wat::core::defn :user::strip-colon
  [s <- :wat::core::String] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::string::subs s 0 1) ":")
    (:wat::core::string::subs s 1 (:wat::core::string::length s))
    s))

(:wat::core::defn :user::unique-conj
  [acc <- :wat::core::Vector<wat::core::String>
   x   <- :wat::core::String]
  -> :wat::core::Vector<wat::core::String>
  (:wat::core::if (:wat::fix::str-in? x acc)
    acc
    (:wat::core::conj acc x)))

;; ── collect types / existing defquery names / compile presence ───────────────

(:wat::core::defn :user::node-type
  [node <- :wat::WatAST] -> (:wat::core::Option :wat::core::String)
  (:wat::core::if (:user::type-query? node)
    (:wat::core::Some
      (:user::strip-colon
        (:wat::core::ast-name
          (:wat::core::Option/expect
            (:wat::core::get (:wat::core::ast->children node) 2)
            "node-type: type kw"))))
    (:wat::core::if (:user::qbts? node)
      (:wat::core::let [ch (:wat::core::ast->children node)]
        (:wat::core::if
          (:wat::core::if (:wat::core::= (:wat::core::count ch) 3)
            (:wat::core::= (:wat::core::ast-kind
                             (:wat::core::Option/expect
                               (:wat::core::get ch 2)
                               "node-type: qbts child"))
                           "string")
            false)
          (:wat::core::Some
            (:wat::core::ast-name
              (:wat::core::Option/expect
                (:wat::core::get ch 2)
                "node-type: qbts string")))
          (:wat::kernel::assertion-failed!
            "type-query-to-defquery: query-by-type-string must be (session \"ns::Type\")"
            :wat::core::None :wat::core::None)))
      :wat::core::None)))

(:wat::core::defn :user::collect-types
  [node  <- :wat::WatAST
   acc   <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::Vector<wat::core::String>
  (:wat::core::if (:user::quoted? node)
    acc
    (:wat::core::let [here (:wat::core::match (:user::node-type node)
                             ((:wat::core::Some t) (:user::unique-conj acc t))
                             (:wat::core::None acc))]
      (:wat::core::if (:wat::fix::structural? node)
        (:user::collect-types-seq (:wat::core::ast->children node) here)
        here))))

(:wat::core::defn :user::collect-types-seq
  [items <- :wat::core::Vector<wat::WatAST>
   acc   <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::Vector<wat::core::String>
  (:wat::core::if (:wat::core::empty? items)
    acc
    (:user::collect-types-seq
      (:wat::core::into [] (:wat::core::rest items))
      (:user::collect-types (:wat::core::first items) acc))))

(:wat::core::defn :user::collect-qnames
  [node <- :wat::WatAST
   acc  <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::Vector<wat::core::String>
  (:wat::core::if (:wat::fix::calls-to? node ":wat::rete::defquery")
    (:wat::core::let [nm (:wat::core::ast-name
                           (:wat::core::Option/expect
                             (:wat::core::get (:wat::core::ast->children node) 1)
                             "collect-qnames: name"))]
      (:user::unique-conj acc nm))
    (:wat::core::if (:wat::fix::structural? node)
      (:user::collect-qnames-seq (:wat::core::ast->children node) acc)
      acc)))

(:wat::core::defn :user::collect-qnames-seq
  [items <- :wat::core::Vector<wat::WatAST>
   acc   <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::Vector<wat::core::String>
  (:wat::core::if (:wat::core::empty? items)
    acc
    (:user::collect-qnames-seq
      (:wat::core::into [] (:wat::core::rest items))
      (:user::collect-qnames (:wat::core::first items) acc))))

(:wat::core::defn :user::has-compile?
  [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:user::quoted? node)
    false
    (:wat::core::if (:user::compile-any? node)
      true
      (:wat::core::if (:wat::fix::structural? node)
        (:user::has-compile-seq (:wat::core::ast->children node))
        false))))

(:wat::core::defn :user::has-compile-seq
  [items <- :wat::core::Vector<wat::WatAST>] -> :wat::core::bool
  (:wat::core::if (:wat::core::empty? items)
    false
    (:wat::core::if (:user::has-compile? (:wat::core::first items))
      true
      (:user::has-compile-seq (:wat::core::into [] (:wat::core::rest items))))))

;; ── text helpers ────────────────────────────────────────────────────────────

(:wat::core::defn :user::node-text
  [node  <- :wat::WatAST
   src   <- :wat::core::String
   lines <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::String
  (:wat::core::string::subs src
    (:wat::fix::node-start-offset node lines)
    (:wat::fix::node-end-offset node lines)))

(:wat::core::defn :user::span-edit
  [node  <- :wat::WatAST
   text  <- :wat::core::String
   lines <- :wat::core::Vector<wat::core::String>]
  -> :wat::fix::Edit
  (:wat::core::let [off (:wat::fix::node-start-offset node lines)
                    end (:wat::fix::node-end-offset node lines)]
    (:wat::core::Tuple off (:wat::core::i64::- end off) text)))

(:wat::core::defn :user::q-call
  [fqdn <- :wat::core::String] -> :wat::core::String
  (:wat::core::string::concat "(:"
    (:wat::core::string::concat (:user::type->qname fqdn) ")")))

(:wat::core::defn :user::q-kw
  [fqdn <- :wat::core::String] -> :wat::core::String
  (:wat::core::string::concat ":" (:user::type->qname fqdn)))

(:wat::core::defn :user::defquery-text
  [fqdn <- :wat::core::String] -> :wat::core::String
  (:wat::core::string::concat
    "(:wat::rete::defquery "
    (:wat::core::string::concat
      (:user::q-kw fqdn)
      (:wat::core::string::concat
        "\n  :params []\n  :when [(:"
        (:wat::core::string::concat fqdn ")])\n")))))

(:wat::core::defn :user::needed-texts
  [types    <- :wat::core::Vector<wat::core::String>
   existing <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String  t <- :wat::core::String]
      -> :wat::core::String
      (:wat::core::if (:wat::fix::str-in? (:user::q-kw t) existing)
        acc
        (:wat::core::string::concat acc
          (:wat::core::string::concat "\n\n" (:user::defquery-text t)))))
    ""
    types))

(:wat::core::defn :user::q-vec-text
  [types <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::String
  (:wat::core::string::concat
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::String  t <- :wat::core::String]
        -> :wat::core::String
        (:wat::core::string::concat acc
          (:wat::core::string::concat " " (:user::q-call t))))
      "(:wat::core::PersistentVector"
      types)
    ")"))

(:wat::core::defn :user::last-decl-end
  [forms <- :wat::core::Vector<wat::WatAST>
   lines <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64  f <- :wat::WatAST]
      -> :wat::core::i64
      (:wat::core::if (:user::decl-form? f)
        (:wat::fix::node-end-offset f lines)
        acc))
    0
    forms))

;; ── edits ───────────────────────────────────────────────────────────────────

(:wat::core::defn :user::call-edit
  [node  <- :wat::WatAST
   src   <- :wat::core::String
   lines <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::Vector<wat::fix::Edit>
  (:wat::core::if (:user::type-query? node)
    (:wat::core::let [ty (:wat::core::Option/expect
                           (:user::node-type node)
                           "call-edit: type-query")
                      arg (:wat::core::Option/expect
                            (:wat::core::get (:wat::core::ast->children node) 2)
                            "call-edit: type kw")]
      (:wat::core::Vector :wat::fix::Edit
        (:user::span-edit arg (:user::q-call ty) lines)))
    (:wat::core::if (:user::qbts? node)
      (:wat::core::let [ch (:wat::core::ast->children node)
                        sess (:wat::core::Option/expect
                               (:wat::core::get ch 1)
                               "call-edit: qbts session")
                        ty (:wat::core::Option/expect
                             (:user::node-type node)
                             "call-edit: qbts type")
                        new (:wat::core::string::concat
                              "(:wat::rete::query "
                              (:wat::core::string::concat
                                (:user::node-text sess src lines)
                                (:wat::core::string::concat " "
                                  (:user::q-call ty))))]
        (:wat::core::Vector :wat::fix::Edit
          (:user::span-edit node
            (:wat::core::string::concat new ")")
            lines)))
      (:wat::core::Vector :wat::fix::Edit))))

(:wat::core::defn :user::compile-edit
  [node  <- :wat::WatAST
   types <- :wat::core::Vector<wat::core::String>
   src   <- :wat::core::String
   lines <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::Vector<wat::fix::Edit>
  (:wat::core::if (:user::compile-1? node)
    (:wat::core::let [rules (:wat::core::Option/expect
                              (:wat::core::get (:wat::core::ast->children node) 1)
                              "compile-edit: rules")
                      new (:wat::core::string::concat
                            "(:wat::rete::compile-all "
                            (:wat::core::string::concat
                              (:user::node-text rules src lines)
                              (:wat::core::string::concat " "
                                (:user::q-vec-text types))))]
      (:wat::core::Vector :wat::fix::Edit
        (:user::span-edit node
          (:wat::core::string::concat new ")")
          lines)))
    (:wat::core::Vector :wat::fix::Edit)))

(:wat::core::defn :user::walk-edits
  [node  <- :wat::WatAST
   types <- :wat::core::Vector<wat::core::String>
   src   <- :wat::core::String
   lines <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::Vector<wat::fix::Edit>
  (:wat::core::if (:user::quoted? node)
    (:wat::core::Vector :wat::fix::Edit)
    (:wat::core::let [this (:wat::core::concat
                             (:user::call-edit node src lines)
                             (:user::compile-edit node types src lines))]
      (:wat::core::if (:wat::fix::structural? node)
        (:wat::core::concat this
          (:user::walk-seq-edits
            (:wat::core::ast->children node) types src lines))
        this))))

(:wat::core::defn :user::walk-seq-edits
  [items <- :wat::core::Vector<wat::WatAST>
   types <- :wat::core::Vector<wat::core::String>
   src   <- :wat::core::String
   lines <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::Vector<wat::fix::Edit>
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector :wat::fix::Edit)
    (:wat::core::concat
      (:user::walk-edits (:wat::core::first items) types src lines)
      (:user::walk-seq-edits (:wat::core::into [] (:wat::core::rest items)) types src lines))))

;; ── per-file migrate ────────────────────────────────────────────────────────

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let
    [lines (:wat::core::string::split src "\n")
     tree  (:wat::core::match (:wat::core::read-string src)
             ((:wat::core::ReadOutcome::Forms __forms) __forms)
             ((:wat::core::ReadOutcome::Malformed __cause)
               (:wat::kernel::assertion-failed!
                 (:wat::core::Error/message __cause)
                 :wat::core::None :wat::core::None)))
     forms (:wat::core::ast->children tree)
     types (:user::collect-types-seq forms
             (:wat::core::Vector :wat::core::String))
     existing (:user::collect-qnames-seq forms
                (:wat::core::Vector :wat::core::String))]
    (:wat::core::if (:wat::core::empty? types)
      src
      (:wat::core::let
        [_comp (:wat::core::if (:user::has-compile-seq forms)
                 nil
                 (:wat::kernel::assertion-failed!
                   "type-query-to-defquery: heretic query in a file with no compile"
                   :wat::core::None :wat::core::None))
         inserted (:user::needed-texts types existing)
         ins-edits
           (:wat::core::if (:wat::core::= inserted "")
             (:wat::core::Vector :wat::fix::Edit)
             (:wat::core::let [off (:user::last-decl-end forms lines)
                               at  (:wat::core::if (:wat::core::= off 0)
                                     (:wat::fix::node-end-offset
                                       (:wat::core::first forms) lines)
                                     off)]
               (:wat::core::Vector :wat::fix::Edit
                 (:wat::core::Tuple at 0 inserted))))
         call-edits (:user::walk-seq-edits forms types src lines)
         all (:wat::core::concat ins-edits call-edits)]
        (:wat::fix::fix-text-apply src
          (:wat::core::reverse (:wat::core::sort all)))))))

;; ── driver ──────────────────────────────────────────────────────────────────

(:wat::core::defn :user::rewrite-each
  [paths <- :wat::core::Vector<wat::core::String>] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)
                      src  (:wat::io::read-file path)
                      out  (:user::migrate src)]
      (:wat::core::do
        (:wat::io::write-file path out)
        (:wat::kernel::println
          (:wat::core::string::concat
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
