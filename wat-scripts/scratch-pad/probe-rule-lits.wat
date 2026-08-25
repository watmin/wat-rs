;; Probe (v2): sift-rules-defsvc's two macro-time extraction problems, end to end:
;;   (1) build Rule VALUES from raw (defrule …) forms (no top-level defn needed) — via make-rule.
;;   (2) macro-emit a per-derived-type QueryNode `query` flat-map (Session/facts set-diff does
;;       NOT carry derived facts — proven false below; make-query + query is the mouth).
;; Both probed together, then wired into a deduce-one that flat-maps ALL derived types into one
;; (PersistentVector :- [Value]) — the exact shape sift-rules' op needs per Log/seed.

(:wat::core::defrecord :usr::Temp [c <- :wat::core::i64])
(:wat::core::defrecord :usr::Hot  [c <- :wat::core::i64])
(:wat::core::defrecord :usr::Warn [c <- :wat::core::i64])

;; take-rules — builds (1) the compiled-rules Vector-of-Rule-VALUES call AND (2) a flat-map
;; query expression over the UNIQUE derived types found across all rules' :then forms. Returns
;; a 2-elem Vector literal `[rules-call flatmap-fn-call]` is awkward across macro boundary, so
;; instead expand STRAIGHT to the full deduce-one defn (mirrors what sift-rules-defsvc's :init +
;; op body will do, minus the service wrapper).
(:wat::core::defmacro :probe::mk-deduce
  [rules-vec <- :wat::WatAST] -> :wat::WatAST
  (:wat::core::let
    [rules-children (:wat::core::ast->children rules-vec)
     rule-lits (:wat::core::foldl
                 (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST]) rf <- :wat::WatAST]
                   -> (:wat::core::Vector :- [:wat::WatAST])
                   (:wat::core::let
                     [rch       (:wat::core::ast->children rf)
                      rname     (:wat::core::Option/expect (:wat::core::get rch 1) "mk-deduce: rule missing name")
                      raw-name  (:wat::core::ast-name rname)
                      name-str  (:wat::core::if (:wat::core::= (:wat::string::subs raw-name 0 1) ":")
                                   (:wat::string::subs raw-name 1 (:wat::string::length raw-name))
                                   raw-name)
                      when-vec  (:wat::core::Option/expect (:wat::core::get rch 3) "mk-deduce: rule missing :when")
                      then-vec  (:wat::core::Option/expect (:wat::core::get rch 5) "mk-deduce: rule missing :then")
                      rule-lit  `(:wat::rete::make-rule ~name-str (:wat::core::quote ~when-vec) (:wat::core::quote ~then-vec))]
                     (:wat::core::conj acc rule-lit)))
                 (:wat::core::Vector :wat::WatAST)
                 rules-children)
     ;; derived-type-strs: unique type names across every rule's :then fact-forms (arc 278 Stone
     ;; A: bare facts, no more `(:wat::rete::insert (:Type …))` wrapper).
     derived-type-strs
               (:wat::core::foldl
                 (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) rf <- :wat::WatAST]
                   -> (:wat::core::Vector :- [:wat::core::String])
                   (:wat::core::let
                     [rch (:wat::core::ast->children rf)
                      then-vec   (:wat::core::Option/expect (:wat::core::get rch 5) "mk-deduce: rule missing :then")
                      then-forms (:wat::core::ast->children then-vec)]
                     (:wat::core::foldl
                       (:wat::core::fn [acc2 <- (:wat::core::Vector :- [:wat::core::String]) tf <- :wat::WatAST]
                         -> (:wat::core::Vector :- [:wat::core::String])
                         (:wat::core::let
                           [cch  (:wat::core::ast->children tf)
                            tkw  (:wat::core::Option/expect (:wat::core::get cch 0) "mk-deduce: :then fact-form missing a type")
                            traw (:wat::core::ast-name tkw)
                            tstr (:wat::core::if (:wat::core::= (:wat::string::subs traw 0 1) ":")
                                   (:wat::string::subs traw 1 (:wat::string::length traw))
                                   traw)]
                           (:wat::core::if (:wat::core::Vector/contains? acc2 tstr) acc2 (:wat::core::conj acc2 tstr))))
                       acc
                       then-forms)))
                 (:wat::core::Vector :wat::core::String)
                 rules-children)
     fired-sym  (:wat::core::symbol-node "fired")
     query-lits
               (:wat::core::foldl
                 (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST]) tstr <- :wat::core::String]
                   -> (:wat::core::Vector :- [:wat::WatAST])
                   (:wat::core::let
                     [tkw  (:wat::core::keyword-node (:wat::string::concat ":" tstr))
                      cond `(~tkw)]
                     (:wat::core::conj acc
                       `(:wat::rete::make-query ~tstr
                          (:wat::core::quote [])
                          (:wat::core::quote [~cond])))))
                 (:wat::core::Vector :wat::WatAST)
                 derived-type-strs)
     query-calls
               (:wat::core::foldl
                 (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST]) lit <- :wat::WatAST]
                   -> (:wat::core::Vector :- [:wat::WatAST])
                   (:wat::core::conj acc `(:wat::rete::query ~fired-sym ~lit)))
                 (:wat::core::Vector :wat::WatAST)
                 query-lits)]
    `(:wat::core::do
       (:wat::core::defn :usr::rules-template [] -> :wat::rete::Session
         (:wat::rete::compile-all
           (:wat::core::PersistentVector ~@rule-lits)
           (:wat::core::PersistentVector ~@query-lits)))
       (:wat::core::defn :usr::deduce-one
         [template <- :wat::rete::Session  seed <- :usr::Temp]
         -> (:wat::core::PersistentVector :- [:wat::core::Value])
         (:wat::core::let
           [~fired-sym (:wat::rete::fire-rules (:wat::rete::insert template seed))]
           (:wat::core::concat ~@query-calls))))))

(:probe::mk-deduce
  [(:wat::rete::defrule :usr::hot-rule
     :when [(:usr::Temp (?c <- :c) (:wat::rete::core::i64::> ?c 50))]
     :then [(:usr::Hot :c ?c)])
   (:wat::rete::defrule :usr::warn-rule
     :when [(:usr::Temp (?c <- :c) (:wat::rete::core::i64::> ?c 50))]
     :then [(:usr::Warn :c ?c)])])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [template (:usr::rules-template)
     hot   (:usr::deduce-one template (:usr::Temp :c 60))
     cold  (:usr::deduce-one template (:usr::Temp :c 10))]
    (:wat::core::do
      (:wat::kernel::println (:wat::string::concat "hot="  (:wat::core::str (:wat::core::length hot))))
      (:wat::kernel::println (:wat::string::concat "cold=" (:wat::core::str (:wat::core::length cold)))))))
