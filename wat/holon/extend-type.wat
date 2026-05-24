;; :wat::holon::extend-type — arc 232 stone 232.1 (bundled with defprotocol).
;;
;; Extends a type with method implementations for a protocol. Generates ONE
;; defn per declared method-body at the mangled keyword name
;; :<type-fqdn>/<proto-basename>-<method> that the defprotocol dispatcher
;; routes to via apply + runtime-built keyword.
;;
;; This is pure macro sugar over already-sufficient substrate primitives.
;; No Rust changes; no new substrate extensions.
;;
;; Usage:
;;   (:wat::holon::extend-type :ns::Voltage :ns::Formattable
;;     (format [self] -> :wat::core::String "voltage-formatted"))
;;
;;   (:wat::holon::extend-type :ns::Celsius :ns::Formattable
;;     (format [self] -> :wat::core::String "celsius-formatted"))
;;
;; Form shape:
;;   (extend-type <type-fqdn> <protocol-fqdn> <method-body>...)
;;
;; Method-body shape: (method-name [params] -> :ReturnType body)
;;   - method-name: symbol at index 0
;;   - params: vector at index 1 (discarded — dispatcher always routes
;;     to [self <- :wat::holon::HolonAST] per D8; impl mirrors that shape)
;;   - `->` symbol at index 2
;;   - ReturnType keyword at index 3
;;   - body: the implementation at index 4
;;
;; Each method-body expands to ONE defn at the mangled name:
;;
;;   (:wat::core::defn :ns::Voltage/Formattable-format
;;     [self <- :wat::holon::HolonAST] -> :wat::core::String
;;     "voltage-formatted")
;;
;; Mangling convention (D2):
;;   <type-fqdn>/<proto-basename>-<method>
;;   e.g. :myapp::Voltage + :myapp::Formattable + format
;;        → :myapp::Voltage/Formattable-format
;;
;; Return type annotation — explicit `-> :T` in the method-body form.
;; Verbose-is-honest: the macro cannot look up the protocol's declared
;; return type without a registry (a future Rust-side enhancement, D7
;; deferral; see SCORE). Callers MUST provide the return type in the
;; extend-type method-body; the type checker validates consistency with
;; the dispatcher's annotation when apply is called.
;;
;; The method bodies are parsed via from-wat + Bundle/children on the
;; rest-param WatAST::List. Per-method iteration uses range + Vector/get
;; (same pattern as defrecord field iteration and defprotocol).
;;
;; D7 deferral — method-name validation against the protocol's declared
;; method list requires a registry stored on SymbolTable. Since Stone
;; 232.1 is pure wat-side macro work, validation defers to runtime:
;; typos in method names surface as UnknownFunction in the dispatcher
;; (arc 233 names the missing mangled verb + span).
;;
;; D8 — per-class impl self parameter is :wat::holon::HolonAST.
;;
;; Open extension — extend-type may be called BEFORE or AFTER defprotocol.
;; The dispatcher uses runtime lookup via apply + keyword/from-string; no
;; pre-registration of extending types is needed. This is the structural
;; property that makes protocols open.
;;
;; Depends on:
;;   - :wat::core::keyword/to-string    (arc 170 slice 3)
;;   - :wat::core::keyword/from-string  (arc 170 slice 3)
;;   - :wat::core::string::split        (stdlib)
;;   - :wat::core::string::concat       (stdlib)
;;   - :wat::core::last                 (stdlib)
;;   - :wat::core::map                  (stdlib)
;;   - :wat::core::range                (stdlib)
;;   - :wat::holon::from-wat            (arc 225)
;;   - :wat::holon::from-holon          (arc 225)
;;   - :wat::holon::to-wat              (arc 225)
;;   - :wat::holon::Bundle/children     (arc 201)
;;   - :wat::holon::statement-length    (arc 037)
;;   - :wat::core::Vector/get           (stdlib)
;;   - :wat::core::Option/expect        (arc 108)
;;   - :wat::core::quote                (core)

(:wat::core::defmacro
  (:wat::holon::extend-type
    (type-name  :AST<wat::core::nil>)
    (proto-name :AST<wat::core::nil>)
    & (methods :AST<wat::core::Vector<wat::WatAST>>)
    -> :AST<wat::core::nil>)
  `(:wat::core::do
     ~@(:wat::core::let
          [methods-h    (:wat::holon::from-wat (:wat::core::quote methods))
           n-methods    (:wat::holon::statement-length methods-h)
           methods-vec  (:wat::holon::Bundle/children methods-h)
           type-name-s  (:wat::core::keyword/to-string type-name)
           proto-name-s (:wat::core::keyword/to-string proto-name)
           proto-parts  (:wat::core::string::split proto-name-s "::")
           proto-base   (:wat::core::Option/expect -> :wat::core::String
                          (:wat::core::last proto-parts)
                          "extend-type: protocol FQDN must have at least one segment")
           impl-defns   (:wat::core::map
                          (:wat::core::range 0 n-methods)
                          (:wat::core::fn [i <- :wat::core::i64] -> :wat::WatAST
                            (:wat::core::let
                              [method-h   (:wat::core::Option/expect -> :wat::holon::HolonAST
                                            (:wat::core::Vector/get methods-vec i)
                                            "extend-type: method index out of range")
                               children   (:wat::holon::Bundle/children method-h)
                               mname-h    (:wat::core::Option/expect -> :wat::holon::HolonAST
                                            (:wat::core::Vector/get children 0)
                                            "extend-type: method body has no name at index 0")
                               mname-s    (:wat::core::keyword/to-string
                                            (:wat::holon::from-holon mname-h))
                               ret-h      (:wat::core::Option/expect -> :wat::holon::HolonAST
                                            (:wat::core::Vector/get children 3)
                                            "extend-type: method body has no return type at index 3")
                               ret-v      (:wat::holon::from-holon ret-h)
                               body-h     (:wat::core::Option/expect -> :wat::holon::HolonAST
                                            (:wat::core::Vector/get children 4)
                                            "extend-type: method body has no implementation at index 4")
                               body-w     (:wat::holon::to-wat body-h)
                               impl-s     (:wat::core::string::concat
                                            type-name-s "/" proto-base "-" mname-s)
                               impl-kw    (:wat::core::keyword/from-string impl-s)]
                              (:wat::core::quasiquote
                                (:wat::core::defn
                                  (:wat::core::unquote impl-kw)
                                  [self <- :wat::holon::HolonAST] ->
                                  (:wat::core::unquote ret-v)
                                  (:wat::core::unquote body-w)))))
                         )]
          impl-defns)))
