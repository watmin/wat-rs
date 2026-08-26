;; wat-scripts/scratch-pad/probe-rule-defn-shape.wat — reconnaissance for the where-shapes
;; defrule migration (arc 278): print the child-kind list of a rule-defn top-level form and its
;; nested let-body, so the codemod's index assumptions (defn shape, let-bindings shape) are
;; grounded against the real AST rather than guessed from reading source text.

(:wat::core::defn :user::kinds [ch <- (:wat::core::Vector :- [:wat::WatAST])] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String c <- :wat::WatAST] -> :wat::core::String
      (:wat::core::String/concat acc
        (:wat::core::String/concat " " (:wat::core::ast-kind c))))
    ""
    ch))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [src   (:wat::io::read-file "wat-scripts/perf/grid/where-shapes.wat")
     tree  (:wat::core::match (:wat::core::read-string src)
             ((:wat::core::ReadOutcome::Forms __f) __f)
             ((:wat::core::ReadOutcome::Malformed __c) (:wat::kernel::assertion-failed! (:wat::core::Error/message __c) :wat::core::None :wat::core::None)))
     forms (:wat::core::ast->children tree)
     ;; find the first form whose ast-name is ":wsh::rule-arith"
     ;; Stone 118.B4-iii — THE WALL: `filter` returns a lazy (Stream :- [T]) (arc 118.2a) and `first`
     ;; no longer accepts one. `forms` is already a fully-realized, finite (Vector :- [WatAST]) — this
     ;; is reconnaissance, not a force-count probe — so `into []` materializes the filtered
     ;; Stream back to a Vector so `first` still applies, byte-identical answer.
     rule  (:wat::core::first
             (:wat::core::into []
               (:wat::core::filter
                 (:wat::core::fn [f <- :wat::WatAST] -> :wat::core::bool
                   (:wat::core::if (:wat::core::= (:wat::core::ast-kind f) "list")
                     (:wat::core::let [ch (:wat::core::ast->children f)]
                       (:wat::core::if (:wat::core::>= (:wat::core::length ch) 2)
                         (:wat::core::= (:wat::core::ast-name (:wat::core::Option/expect (:wat::core::get ch 1) "n")) ":wsh::rule-arith")
                         false))
                     false))
                 forms)))
     rch   (:wat::core::ast->children rule)]
    (:wat::core::do
      (:wat::kernel::println (:wat::core::String/concat "rule-defn child kinds:" (:user::kinds rch)))
      (:wat::kernel::println (:wat::core::String/concat "rule-defn child count: " (:wat::i64::to-string (:wat::core::length rch))))
      (:wat::core::let [body (:wat::core::Option/expect (:wat::core::get rch 5) "body")]
        (:wat::core::do
          (:wat::kernel::println (:wat::core::String/concat "body kind: " (:wat::core::ast-kind body)))
          (:wat::core::let [bch (:wat::core::ast->children body)]
            (:wat::core::do
              (:wat::kernel::println (:wat::core::String/concat "body child kinds:" (:user::kinds bch)))
              (:wat::core::let [bindings (:wat::core::Option/expect (:wat::core::get bch 1) "bindings")]
                (:wat::core::do
                  (:wat::kernel::println (:wat::core::String/concat "bindings kind: " (:wat::core::ast-kind bindings)))
                  (:wat::core::let [bindch (:wat::core::ast->children bindings)]
                    (:wat::core::do
                      (:wat::kernel::println (:wat::core::String/concat "bindings child kinds:" (:user::kinds bindch)))
                      (:wat::kernel::println (:wat::core::String/concat "bindings child count: " (:wat::i64::to-string (:wat::core::length bindch))))
                      (:wat::core::let [conds-val (:wat::core::Option/expect (:wat::core::get bindch 1) "conds-val")]
                        (:wat::core::do
                          (:wat::kernel::println (:wat::core::String/concat "conds-val kind: " (:wat::core::ast-kind conds-val)))
                          (:wat::core::let [cvch (:wat::core::ast->children conds-val)]
                            (:wat::kernel::println (:wat::core::String/concat "conds-val child kinds:" (:user::kinds cvch))))))
                      (:wat::core::let [rule-call (:wat::core::Option/expect (:wat::core::get bch 2) "rule-call")]
                        (:wat::core::let [rcch (:wat::core::ast->children rule-call)]
                          (:wat::core::do
                            (:wat::kernel::println (:wat::core::String/concat "rule-call child kinds:" (:user::kinds rcch)))
                            (:wat::core::let [namestr (:wat::core::Option/expect (:wat::core::get rcch 2) "namestr")]
                              (:wat::core::do
                                (:wat::kernel::println (:wat::core::String/concat "name-node kind: " (:wat::core::ast-kind namestr)))
                                (:wat::kernel::println (:wat::core::String/concat "name-node ast-name: " (:wat::core::ast-name namestr)))))))))))))))))))
