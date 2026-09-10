;; wat-scripts/scratch-pad/census-join-scope-where.wat — read-only recon for
;; stop-teaching-the-join-blowup: enumerate every RULE (defrule/defquery) whose `:when` is
;; IN SCOPE for `wat-scripts/fixes/hoist-where-into-condition.wat`'s join-only gate — i.e.
;; BOTH (1) it carries a trailing `(:wat::rete::where …)` whose vars are bound by EXACTLY ONE
;; ordinary condition (the codemod's own var-binding-superset hoist criterion) AND (2) its
;; `:when` has >=2 ordinary fact conditions (`collect-hoist-targets` count) — a real join to
;; blow up. This is a REPORT tool only: it never writes a file. Population census for
;; docs/arc/2026/06/278-rules-engine/stop-teaching-the-join-blowup/SCORE.md.
;;
;; Shape-classification / collection helpers are copied VERBATIM from the codemod (not
;; re-derived) so this census agrees with what the codemod itself will and will not touch.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["a.wat" "b.wat" …]\n' | cargo wat ./wat-scripts/scratch-pad/census-join-scope-where.wat

(:wat::core::defn :user::bind-clause-var [c <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind c) "list")
    (:wat::core::let [ch (:wat::core::ast->children c)]
      (:wat::core::if (:wat::core::= (:wat::core::length ch) 3)
        (:wat::core::let [h0 (:wat::core::first ch)
                          h1 (:wat::core::nth ch 1)
                          h2 (:wat::core::nth ch 2)]
          (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::ast-kind h0) "symbol")
                            (:wat::core::string::starts-with? (:wat::core::ast-name h0) "?") false)
            (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::ast-kind h1) "symbol")
                              (:wat::core::= (:wat::core::ast-name h1) "<-") false)
              (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::ast-kind h2) "keyword")
                                (:wat::core::string::contains? (:wat::core::ast-name h2) "::") false)
                ""
                (:wat::core::if (:wat::core::= (:wat::core::ast-kind h2) "keyword")
                  (:wat::core::ast-name h0)
                  ""))
              "")
            ""))
        ""))
    ""))

(:wat::core::defn :user::top-shape-tag [node <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch)
        "other"
        (:wat::core::let [head (:wat::core::first ch)]
          (:wat::core::if (:wat::core::= (:wat::core::ast-kind head) "keyword")
            (:wat::core::let [hn (:wat::core::ast-name head)]
              (:wat::core::cond
                ((:wat::core::= hn ":wat::rete::where") "where")
                ((:wat::core::= hn ":wat::rete::not") "not")
                ((:wat::core::= hn ":wat::rete::exists") "exists")
                ((:wat::core::= hn ":wat::rete::or") "or")
                ((:wat::core::= hn ":wat::rete::and") "and")
                (:else "plain")))
            (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::ast-kind head) "symbol")
                              (:wat::core::string::starts-with? (:wat::core::ast-name head) "?") false)
              (:wat::core::let [n (:wat::core::length ch)]
                (:wat::core::if (:wat::core::if (:wat::core::>= n 3)
                                  (:wat::core::if (:wat::core::= (:wat::core::ast-kind (:wat::core::nth ch 1)) "symbol")
                                    (:wat::core::= (:wat::core::ast-name (:wat::core::nth ch 1)) "<-") false)
                                  false)
                  (:wat::core::let [third (:wat::core::nth ch 2)]
                    (:wat::core::cond
                      ((:wat::core::if (:wat::core::= (:wat::core::ast-kind third) "keyword")
                         (:wat::core::string::contains? (:wat::core::ast-name third) "::") false)
                        "factbind")
                      ((:wat::core::if (:wat::core::= n 5)
                         (:wat::core::if (:wat::core::= (:wat::core::ast-kind (:wat::core::nth ch 3)) "keyword")
                           (:wat::core::= (:wat::core::ast-name (:wat::core::nth ch 3)) ":from") false)
                         false)
                        "accumulate")
                      (:else "other")))
                  "other"))
              "other")))))
    "other"))

(:wat::core::defn :user::bound-vars-of-plain [node <- :wat::WatAST] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::let [ch (:wat::core::ast->children node)
                    clauses (:wat::core::into [] (:wat::core::rest ch))]
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) c <- :wat::WatAST]
        -> (:wat::core::Vector :- [:wat::core::String])
        (:wat::core::let [v (:user::bind-clause-var c)]
          (:wat::core::if (:wat::core::= v "") acc (:wat::core::conj acc v))))
      (:wat::core::Vector :wat::core::String)
      clauses)))

(:wat::core::defn :user::bound-vars-of-factbind [node <- :wat::WatAST] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::let [ch (:wat::core::ast->children node)
                    head-var (:wat::core::ast-name (:wat::core::first ch))
                    clauses (:wat::core::into [] (:wat::core::drop ch 3))]
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) c <- :wat::WatAST]
        -> (:wat::core::Vector :- [:wat::core::String])
        (:wat::core::let [v (:user::bind-clause-var c)]
          (:wat::core::if (:wat::core::= v "") acc (:wat::core::conj acc v))))
      (:wat::core::conj (:wat::core::Vector :wat::core::String) head-var)
      clauses)))

(:wat::core::defn :user::var-occurrences [node <- :wat::WatAST] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "symbol")
    (:wat::core::if (:wat::core::string::starts-with? (:wat::core::ast-name node) "?")
      (:wat::core::Vector :wat::core::String (:wat::core::ast-name node))
      (:wat::core::Vector :wat::core::String))
    (:wat::core::if (:wat::fix::structural? node)
      (:wat::core::foldl
        (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) c <- :wat::WatAST]
          -> (:wat::core::Vector :- [:wat::core::String])
          (:wat::core::concat acc (:user::var-occurrences c)))
        (:wat::core::Vector :wat::core::String)
        (:wat::core::into [] (:wat::core::ast->children node)))
      (:wat::core::Vector :wat::core::String))))

(:wat::core::defn :user::dedup [items <- (:wat::core::Vector :- [:wat::core::String])] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) x <- :wat::core::String]
      -> (:wat::core::Vector :- [:wat::core::String])
      (:wat::core::if (:wat::core::contains? acc x) acc (:wat::core::conj acc x)))
    (:wat::core::Vector :wat::core::String)
    items))

(:wat::core::defn :user::subset? [small <- (:wat::core::Vector :- [:wat::core::String]) big <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::bool
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::bool x <- :wat::core::String]
      -> :wat::core::bool
      (:wat::core::if acc (:wat::core::contains? big x) false))
    true
    small))

(:wat::core::defn :user::collect-hoist-targets [items <- (:wat::core::Vector :- [:wat::WatAST])] -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::WatAST (:wat::core::Vector :- [:wat::core::String])])])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::WatAST (:wat::core::Vector :- [:wat::core::String])]))
    (:wat::core::let [it (:wat::core::first items)
                      tl (:wat::core::into [] (:wat::core::rest items))
                      tag (:user::top-shape-tag it)]
      (:wat::core::cond
        ((:wat::core::= tag "and")
          (:wat::core::concat
            (:user::collect-hoist-targets (:wat::core::into [] (:wat::core::rest (:wat::core::ast->children it))))
            (:user::collect-hoist-targets tl)))
        ((:wat::core::= tag "plain")
          (:wat::core::conj (:user::collect-hoist-targets tl) (:wat::core::Tuple it (:user::bound-vars-of-plain it))))
        ((:wat::core::= tag "factbind")
          (:wat::core::conj (:user::collect-hoist-targets tl) (:wat::core::Tuple it (:user::bound-vars-of-factbind it))))
        (:else (:user::collect-hoist-targets tl))))))

(:wat::core::defn :user::collect-where-sites [items <- (:wat::core::Vector :- [:wat::WatAST])] -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::WatAST :wat::WatAST])])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::WatAST :wat::WatAST]))
    (:wat::core::let [it (:wat::core::first items)
                      tl (:wat::core::into [] (:wat::core::rest items))
                      tag (:user::top-shape-tag it)]
      (:wat::core::cond
        ((:wat::core::= tag "where")
          (:wat::core::concat
            (:wat::core::Vector (:wat::core::Tuple :- [:wat::WatAST :wat::WatAST])
              (:wat::core::Tuple it (:wat::core::nth (:wat::core::ast->children it) 1)))
            (:user::collect-where-sites tl)))
        ((:wat::core::= tag "and")
          (:wat::core::concat
            (:user::collect-where-sites (:wat::core::into [] (:wat::core::rest (:wat::core::ast->children it))))
            (:user::collect-where-sites tl)))
        (:else (:user::collect-where-sites tl))))))

(:wat::core::defn :user::find-when-vector-at [ch <- (:wat::core::Vector :- [:wat::WatAST]) i <- :wat::core::i64] -> (:wat::core::Option :wat::WatAST)
  (:wat::core::if (:wat::core::>= (:wat::core::i64::+ i 1) (:wat::core::length ch))
    (:wat::core::None :wat::WatAST)
    (:wat::core::let [c (:wat::core::Option/expect (:wat::core::get ch i) "find-when-vector-at c")]
      (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::ast-kind c) "keyword")
                        (:wat::core::= (:wat::core::ast-name c) ":when") false)
        (:wat::core::Some (:wat::core::Option/expect (:wat::core::get ch (:wat::core::i64::+ i 1)) "find-when-vector-at next"))
        (:user::find-when-vector-at ch (:wat::core::i64::+ i 1))))))

(:wat::core::defn :user::find-when-vector [ch <- (:wat::core::Vector :- [:wat::WatAST])] -> (:wat::core::Option :wat::WatAST)
  (:user::find-when-vector-at ch 0))

;; rule-name — the keyword/symbol immediately after the defrule/defquery head, or "?" if the
;; shape is unexpected (never fatal — this is a report tool).
(:wat::core::defn :user::rule-name [ch <- (:wat::core::Vector :- [:wat::WatAST])] -> :wat::core::String
  (:wat::core::if (:wat::core::< (:wat::core::length ch) 2)
    "?"
    (:wat::core::ast-name (:wat::core::Option/expect (:wat::core::get ch 1) "rule-name"))))

;; how-many-single-target-hoistable — count of `where` sites among `wheres` whose var-occurrence
;; set matches EXACTLY ONE candidate in `targets` (the codemod's own hoist criterion).
(:wat::core::defn :user::count-hoistable
  [wheres <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::WatAST :wat::WatAST])])
   targets <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::WatAST (:wat::core::Vector :- [:wat::core::String])])])]
  -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64 ws <- (:wat::core::Tuple :- [:wat::WatAST :wat::WatAST])]
      -> :wat::core::i64
      (:wat::core::let [pred (:wat::core::second ws)
                        vars (:user::dedup (:user::var-occurrences pred))
                        matches (:wat::core::foldl
                                  (:wat::core::fn [macc <- :wat::core::i64 t <- (:wat::core::Tuple :- [:wat::WatAST (:wat::core::Vector :- [:wat::core::String])])]
                                    -> :wat::core::i64
                                    (:wat::core::if (:user::subset? vars (:wat::core::second t)) (:wat::core::i64::+ macc 1) macc))
                                  0 targets)]
        (:wat::core::if (:wat::core::= matches 1) (:wat::core::i64::+ acc 1) acc)))
    0
    wheres))

;; rule-report — "" if this rule is OUT of scope; else one report line. IN SCOPE iff (1) there is
;; at least one where-site hoistable to exactly one target (criterion 1) AND (2) >=2 ordinary
;; conditions total (criterion 2, the join-only gate the codemod now enforces).
(:wat::core::defn :user::rule-report [node <- :wat::WatAST path <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [ch (:wat::core::into [] (:wat::core::ast->children node))
                    wvopt (:user::find-when-vector ch)]
    (:wat::core::match wvopt
      (:wat::core::None "")
      ((:wat::core::Some wv)
        (:wat::core::let
          [items (:wat::core::into [] (:wat::core::ast->children wv))
           targets (:user::collect-hoist-targets items)
           wheres (:user::collect-where-sites items)
           n-targets (:wat::core::length targets)
           n-hoistable (:user::count-hoistable wheres targets)]
          (:wat::core::if (:wat::core::if (:wat::core::> n-hoistable 0) (:wat::core::>= n-targets 2) false)
            (:wat::core::string::concat path
              (:wat::core::string::concat " :: "
                (:wat::core::string::concat (:user::rule-name ch)
                  (:wat::core::string::concat " :: targets="
                    (:wat::core::string::concat (:wat::core::i64::to-string n-targets)
                      (:wat::core::string::concat " :: hoistable-wheres="
                        (:wat::core::i64::to-string n-hoistable)))))))
            ""))))))

(:wat::core::defn :user::walk-node [node <- :wat::WatAST path <- :wat::core::String] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [hn (:wat::fix::head-name node)
                      this (:wat::core::if (:wat::core::if (:wat::core::= hn ":wat::rete::defrule") true (:wat::core::= hn ":wat::rete::defquery"))
                             (:wat::core::let [r (:user::rule-report node path)]
                               (:wat::core::if (:wat::core::= r "") (:wat::core::Vector :wat::core::String) (:wat::core::Vector :wat::core::String r)))
                             (:wat::core::Vector :wat::core::String))]
      (:wat::core::concat this
        (:wat::core::foldl
          (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) c <- :wat::WatAST]
            -> (:wat::core::Vector :- [:wat::core::String])
            (:wat::core::concat acc (:user::walk-node c path)))
          (:wat::core::Vector :wat::core::String)
          (:wat::core::into [] (:wat::core::ast->children node)))))
    (:wat::core::if (:wat::fix::structural? node)
      (:wat::core::foldl
        (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) c <- :wat::WatAST]
          -> (:wat::core::Vector :- [:wat::core::String])
          (:wat::core::concat acc (:user::walk-node c path)))
        (:wat::core::Vector :wat::core::String)
        (:wat::core::into [] (:wat::core::ast->children node)))
      (:wat::core::Vector :wat::core::String))))

(:wat::core::defn :user::census-one [path <- :wat::core::String] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::let [src (:wat::io::read-file path)
                    tree (:wat::core::match (:wat::core::read-string src) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
                    forms (:wat::core::into [] (:wat::core::ast->children tree))]
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) f <- :wat::WatAST]
        -> (:wat::core::Vector :- [:wat::core::String])
        (:wat::core::concat acc (:user::walk-node f path)))
      (:wat::core::Vector :wat::core::String)
      forms)))

(:wat::core::defn :user::print-each [lines <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? lines)
    nil
    (:wat::core::do
      (:wat::kernel::println (:wat::core::first lines))
      (:user::print-each (:wat::core::into [] (:wat::core::rest lines))))))

(:wat::core::defn :user::census-each [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:user::print-each (:user::census-one path))
        (:user::census-each (:wat::core::into [] (:wat::core::rest paths)))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::census-each
    (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
