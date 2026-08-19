;; wat-scripts/scratch-pad/probe-rule-defn-shape-b.wat — reconnaissance for Shape B (hoisted
;; conds/ins helper) rule-defns, e.g. where-boolean.wat's :wsb::rule-and2. Grounds the codemod's
;; Shape-B index assumptions (bindings-vec arity, helper-defn body shape) against the real AST.

(:wat::core::defn :user::kinds [ch <- :wat::core::Vector<wat::WatAST>] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String c <- :wat::WatAST] -> :wat::core::String
      (:wat::core::String/concat acc
        (:wat::core::String/concat " " (:wat::core::ast-kind c))))
    ""
    ch))

;; Stone 118.B4-iii — THE WALL: `filter` returns a lazy `Stream<T>` (arc 118.2a) and `first` no
;; longer accepts one. `forms` is already a fully-realized, finite `Vector<WatAST>`
;; (`ast->children`'s return type) and this is reconnaissance over it, not a force-count probe —
;; `into []` materializes the filtered Stream back to a Vector so `first` still applies, same
;; "find the named top-level form" semantics, byte-identical answer.
(:wat::core::defn :user::find-named [forms <- :wat::core::Vector<wat::WatAST> nm <- :wat::core::String] -> :wat::WatAST
  (:wat::core::first
    (:wat::core::into []
      (:wat::core::filter
        (:wat::core::fn [f <- :wat::WatAST] -> :wat::core::bool
          (:wat::core::if (:wat::core::= (:wat::core::ast-kind f) "list")
            (:wat::core::let [ch (:wat::core::ast->children f)]
              (:wat::core::if (:wat::core::>= (:wat::core::length ch) 2)
                (:wat::core::= (:wat::core::ast-name (:wat::core::Option/expect (:wat::core::get ch 1) "n")) nm)
                false))
            false))
        forms))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [src   (:wat::io::read-file "wat-scripts/perf/grid/where-boolean.wat")
     tree  (:wat::core::match (:wat::core::read-string src)
             ((:wat::core::ReadOutcome::Forms __f) __f)
             ((:wat::core::ReadOutcome::Malformed __c) (:wat::kernel::assertion-failed! (:wat::core::Error/message __c) :wat::core::None :wat::core::None)))
     forms (:wat::core::ast->children tree)
     rule  (:user::find-named forms ":wsb::rule-and2")
     condsfn (:user::find-named forms ":wsb::conds")
     rch   (:wat::core::ast->children rule)
     cch   (:wat::core::ast->children condsfn)]
    (:wat::core::do
      (:wat::kernel::println (:wat::core::String/concat "rule-and2 child kinds:" (:user::kinds rch)))
      (:wat::core::let [body (:wat::core::Option/expect (:wat::core::get rch 5) "body")]
        (:wat::core::let [bch (:wat::core::ast->children body)]
          (:wat::core::do
            (:wat::kernel::println (:wat::core::String/concat "body child kinds:" (:user::kinds bch)))
            (:wat::core::let [bindings (:wat::core::Option/expect (:wat::core::get bch 1) "bindings")]
              (:wat::core::let [bindch (:wat::core::ast->children bindings)]
                (:wat::core::do
                  (:wat::kernel::println (:wat::core::String/concat "bindings child kinds:" (:user::kinds bindch)))
                  (:wat::kernel::println (:wat::core::String/concat "bindings child count: " (:wat::core::i64::to-string (:wat::core::length bindch))))))))))
      (:wat::kernel::println (:wat::core::String/concat "conds-fn child kinds:" (:user::kinds cch)))
      (:wat::core::let [cbody (:wat::core::Option/expect (:wat::core::get cch 5) "cbody")]
        (:wat::core::let [cbch (:wat::core::ast->children cbody)]
          (:wat::kernel::println (:wat::core::String/concat "conds-fn body child kinds:" (:user::kinds cbch))))))))
