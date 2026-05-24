;; :wat::holon::defprotocol — arc 232 stone 232.1 (bundled with extend-type).
;;
;; Declares a protocol: a named set of method signatures. Generates ONE
;; polymorphic dispatcher defn per declared method. Each dispatcher routes
;; per-first-arg-classifier via the canonical composition proven by the
;; FM 2-bis probe (tests/probe_diagnostic_defprotocol_dispatch.rs, commit f38e120):
;;
;;   extract-classifier → string::concat → keyword/from-string → apply
;;
;; This is pure macro sugar over already-sufficient substrate primitives.
;; No Rust changes; no new substrate extensions.
;;
;; Usage:
;;   (:wat::holon::defprotocol :ns::Formattable
;;     (format [self] -> :wat::core::String))
;;
;;   (:wat::holon::defprotocol :ns::Readable
;;     (read  [self] -> :ns::Readable)
;;     (label [self] -> :wat::core::String))
;;
;; Form shape: (defprotocol <protocol-fqdn> <method-decl>...)
;; Method decl shape: (method-name [params] -> :ReturnType)
;;   - method-name: symbol at index 0
;;   - params: vector at index 1 (not forwarded — dispatcher always takes
;;     [self <- :wat::holon::HolonAST] per D8)
;;   - `->` symbol at index 2
;;   - ReturnType keyword at index 3
;;
;; Each method-decl expands to ONE dispatcher defn at :<proto-fqdn>/<method>.
;;
;; The method declarations are parsed via from-wat + Bundle/children on the
;; rest-param WatAST::List. Per-method iteration uses range + Vector/get
;; (same pattern as defrecord field iteration).
;;
;; Mangling convention (D2): the dispatcher lives at :<proto-fqdn>/<method>.
;; Impl (from extend-type) lives at :<type-fqdn>/<proto-basename>-<method>.
;;
;; D7 deferral — method-name validation at expand time requires a registry
;; stored on SymbolTable (a Rust-side change). Since Stone 232.1 is pure
;; wat-side, validation defers to runtime: missing impls surface as
;; UnknownFunction naming the missing mangled keyword (per FM 2-bis probe 3).
;;
;; D8 — dispatcher self parameter is always :wat::holon::HolonAST.
;;
;; Depends on:
;;   - :wat::holon::extract-classifier  (arc 232 stone 232.0a)
;;   - :wat::core::apply                (arc 232 stone 232.0)
;;   - :wat::core::Option/expect        (arc 108)
;;   - :wat::core::keyword/to-string    (arc 170 slice 3)
;;   - :wat::core::keyword/from-string  (arc 170 slice 3)
;;   - :wat::core::string::split        (stdlib)
;;   - :wat::core::string::concat       (stdlib)
;;   - :wat::core::last                 (stdlib)
;;   - :wat::core::map                  (stdlib)
;;   - :wat::core::range                (stdlib)
;;   - :wat::holon::from-wat            (arc 225)
;;   - :wat::holon::from-holon          (arc 225)
;;   - :wat::holon::Bundle/children     (arc 201)
;;   - :wat::holon::statement-length    (arc 037)
;;   - :wat::core::Vector/get           (stdlib)
;;   - :wat::core::quote                (core)

(:wat::core::defmacro
  (:wat::holon::defprotocol
    (proto-name :AST<wat::core::nil>)
    & (methods :AST<wat::core::Vector<wat::WatAST>>)
    -> :AST<wat::core::nil>)
  `(:wat::core::do
     ~@(:wat::core::let
          [methods-h    (:wat::holon::from-wat (:wat::core::quote methods))
           n-methods    (:wat::holon::statement-length methods-h)
           methods-vec  (:wat::holon::Bundle/children methods-h)
           proto-name-s (:wat::core::keyword/to-string proto-name)
           proto-parts  (:wat::core::string::split proto-name-s "::")
           proto-base   (:wat::core::Option/expect -> :wat::core::String
                          (:wat::core::last proto-parts)
                          "defprotocol: protocol FQDN must have at least one segment")
           dispatchers  (:wat::core::map
                          (:wat::core::range 0 n-methods)
                          (:wat::core::fn [i <- :wat::core::i64] -> :wat::WatAST
                            (:wat::core::let
                              [method-h   (:wat::core::Option/expect -> :wat::holon::HolonAST
                                            (:wat::core::Vector/get methods-vec i)
                                            "defprotocol: method index out of range")
                               children   (:wat::holon::Bundle/children method-h)
                               mname-h    (:wat::core::Option/expect -> :wat::holon::HolonAST
                                            (:wat::core::Vector/get children 0)
                                            "defprotocol: method has no name at index 0")
                               mname-s    (:wat::core::keyword/to-string
                                            (:wat::holon::from-holon mname-h))
                               ret-h      (:wat::core::Option/expect -> :wat::holon::HolonAST
                                            (:wat::core::Vector/get children 3)
                                            "defprotocol: method has no return type at index 3")
                               ret-v      (:wat::holon::from-holon ret-h)
                               disp-s     (:wat::core::string::concat proto-name-s "/" mname-s)
                               disp-kw    (:wat::core::keyword/from-string disp-s)
                               suffix-s   (:wat::core::string::concat "/" proto-base "-" mname-s)
                               error-msg  (:wat::core::string::concat
                                            proto-base "/" mname-s ": no classifier on arg")]
                              (:wat::core::quasiquote
                                (:wat::core::defn
                                  (:wat::core::unquote disp-kw)
                                  [self <- :wat::holon::HolonAST] ->
                                  (:wat::core::unquote ret-v)
                                  (:wat::core::let
                                    [classifier-opt (:wat::holon::extract-classifier self)
                                     classifier     (:wat::core::Option/expect -> :wat::core::String
                                                      classifier-opt
                                                      (:wat::core::unquote error-msg))
                                     mangled-str    (:wat::core::string::concat
                                                      classifier
                                                      (:wat::core::unquote suffix-s))
                                     mangled-kw     (:wat::core::keyword/from-string mangled-str)]
                                    (:wat::core::apply ->
                                      (:wat::core::unquote ret-v)
                                      mangled-kw
                                      [self])            ;; close apply
                                  )                      ;; close inner let2
                                )                        ;; close defn
                              )                          ;; close quasiquote
                            )                            ;; close fn-body let1
                          )                              ;; close fn
                         )                               ;; close map
                         ]                               ;; close outer let binding vec
          dispatchers)))
