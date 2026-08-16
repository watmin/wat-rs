;; probe-198-ctor-macro-span.wat — arc 198 span-fix rider: measure the spans the
;; kwargs-construct companion macro actually emits for `(:my::Token 7)`, node by node,
;; via macroexpand + ast-span/ast-end-span (never executed as real code, so the
;; restriction check never fires — this is pure introspection of the expansion).
(:wat::core::defstruct :my::Token
  {:restricted-to [:my::issuer::]}
  [id <- :wat::core::i64])

(:wat::core::defn :probe::pos-str [p <- :wat::core::HashMap<wat::core::keyword,wat::core::i64>] -> :wat::core::String
  (:wat::core::string::concat "l" (:wat::core::i64::to-string (:wat::core::Option/expect (:wat::core::HashMap/get p :line) "line"))
    ":c" (:wat::core::i64::to-string (:wat::core::Option/expect (:wat::core::HashMap/get p :col) "col"))))

(:wat::core::defn :probe::dump-children [kids <- :wat::core::Vector<wat::WatAST> i <- :wat::core::i64 depth <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::match (:wat::core::Vector/get kids i)
    ((:wat::core::Some c) (:wat::core::do (:probe::dump c depth) (:probe::dump-children kids (:wat::core::+ i 1) depth)))
    (:wat::core::None nil)))

(:wat::core::defn :probe::dump [node <- :wat::WatAST depth <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println
      (:wat::core::string::concat "depth=" (:wat::core::i64::to-string depth)
        " kind=" (:wat::core::ast-kind node)
        " form=" (:wat::core::ast->source node)
        "  span=[" (:probe::pos-str (:wat::core::ast-span node))
        " .. " (:probe::pos-str (:wat::core::ast-end-span node))
        "]"))
    (:probe::dump-children (:wat::core::ast->children node) 0 (:wat::core::+ depth 1))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [forms (:wat::core::match (:wat::core::read-string "(:my::Token 7)")
              ((:wat::core::ReadOutcome::Forms __forms) __forms)
              ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
     form (:wat::core::first forms)
     exp  (:wat::core::macroexpand form)]
    (:wat::core::do
      (:wat::kernel::println "==== ORIGINAL FORM ====")
      (:probe::dump form 0)
      (:wat::kernel::println "==== EXPANDED FORM ====")
      (:probe::dump exp 0))))
