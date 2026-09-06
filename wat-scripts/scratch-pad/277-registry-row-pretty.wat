;; Scratch — query the REGISTRY for one symbol and pretty-print its example with the
;; fmt rules. Not a parse of a Rust doc comment (that is examples/render_one.rs) — this
;; asks the live registry, which is the thing arc 255's migration will drive.
;;
;;   (:wat::intrinsic::examples)  ->  Vector<Example{fqdn, expr <- :wat::WatAST, …}>
;;
;; ⚠ ENUMERATE-AND-FILTER, and that is the API gap this probe documents. The DIRECT
;; lookup :wat::runtime::metadata-of <fqdn> exists and returns 13 keys — :doc :ret :arity
;; :purity … — but carries NEITHER :examples NOR :args, which are exactly the two slots
;; the formatter is for. Only the enumeration verbs reach them.
;;   Example/expr                 ->  the parsed form
;;   ast->source + format-source  ->  the ruled shape
(:wat::load-file! "../fmt/rules/defn.wat")
(:wat::load-file! "../fmt/rules/siblings.wat")
(:wat::load-file! "../fmt/rules/match.wat")
(:wat::load-file! "../fmt/rules/if.wat")
(:wat::load-file! "../fmt/rules/cond.wat")
(:wat::load-file! "../fmt/rules/let.wat")
(:wat::load-file! "../fmt/rules/let-blank.wat")
(:wat::load-file! "../fmt/rules/kwargs.wat")
(:wat::load-file! "../fmt/rules/table.wat")
(:wat::load-file! "../fmt/rules/atoms.wat")
(:wat::load-file! "../fmt/rules/defrecord.wat")

(:wat::core::defn :user::widest
  [s <- :wat::core::String]
  -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [m <- :wat::core::i64  line <- :wat::core::String] -> :wat::core::i64
      (:wat::core::if (:wat::i64::> (:wat::string::length line) m)
        (:wat::string::length line)
        m))
    0
    (:wat::string::split s "\n")))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [argv  (:wat::runtime::argv)
     want  (:wat::core::Option/expect (:wat::core::get argv 2)
             "usage: wat 277-registry-row-pretty.wat <fqdn-as-string>")
     rules (:wat::rete::collect-rules :fmt)
     all   (:wat::intrinsic::examples)
     mine  (:wat::core::into (:wat::core::Vector :- [:wat::intrinsic::Example])
             (:wat::core::filter
               (:wat::core::fn [e <- :wat::intrinsic::Example] -> :wat::core::bool
                 (:wat::core::= (:wat::keyword::to-string (:wat::intrinsic::Example/fqdn e)) want))
               all))]
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::nil  e <- :wat::intrinsic::Example] -> :wat::core::nil
        (:wat::core::let
          [src (:wat::core::ast->source (:wat::intrinsic::Example/expr e))
           out (:wat::fmt::format-source "<registry>" src rules)]
          (:wat::core::do
            (:wat::kernel::println
              (:wat::string::interpolate "SOURCE  lines=1 widest={a}   FORMATTED widest={b}"
                :a (:wat::i64::to-string (:user::widest src))
                :b (:wat::i64::to-string (:user::widest out))))
            (:wat::kernel::println out))))
      nil
      mine)))
