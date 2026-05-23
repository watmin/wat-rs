;; :wat::holon::defclass — arc 227 stone 227.1.
;;
;; Mints a user-defined classifier-wrapped typed entity in the user's
;; declared namespace. Single-arg form only (stone 227.1); inheritance
;; via classifier-chain is stone 227.2.
;;
;; Usage:
;;   (:wat::holon::defclass :myapp::Voltage)
;;
;; Expands to two definitions in the user's DECLARED namespace:
;;
;;   ;; Constructor — wraps HolonAST payload in classifier-tagged Bind
;;   (:wat::core::defn :myapp::Voltage [v <- :wat::holon::HolonAST] -> :wat::holon::HolonAST
;;     (:wat::holon::Bind
;;       (:wat::holon::Atom (:wat::holon::to-holon "myapp::Voltage"))
;;       (:wat::holon::Atom v)))
;;
;;   ;; Predicate — delegates to arc 226's polymorphic is?
;;   (:wat::core::defn :myapp::is-Voltage? [v <- :wat::holon::HolonAST] -> :wat::core::bool
;;     (:wat::holon::is? v "myapp::Voltage"))
;;
;; The constructor expects a :wat::holon::HolonAST payload (use
;; :wat::holon::to-holon to lift primitive values before passing).
;;
;; Naming rules (all derived from the user-declared FQDN at macro-expand
;; time via :wat::core::keyword/to-string + string manipulation):
;;
;;   | Input FQDN            | Constructor           | Predicate                  | Classifier string      |
;;   |-----------------------|-----------------------|----------------------------|------------------------|
;;   | :myapp::Voltage       | :myapp::Voltage       | :myapp::is-Voltage?        | "myapp::Voltage"       |
;;   | :awesome::lib::Sensor | :awesome::lib::Sensor | :awesome::lib::is-Sensor?  | "awesome::lib::Sensor" |
;;   | :test::Foo            | :test::Foo            | :test::is-Foo?             | "test::Foo"            |
;;
;; Classifier string = FQDN without leading colon. Distinct across namespaces:
;;   (:defclass :appA::Voltage) → classifier "appA::Voltage"
;;   (:defclass :appB::Voltage) → classifier "appB::Voltage"
;;   These are NOT the same classifier — predicate discrimination is honest.
;;
;; FQDN doctrine (:feedback_fqdn_is_the_namespace): users declare their own
;; namespace. The macro NEVER inserts into :user::* or any auto-namespace.
;;
;; Depends on:
;;   - :wat::holon::Bind       (arc 228 — classifier-wrapped constructor)
;;   - :wat::holon::Atom       (arc 225 — narrow HolonAST wrapping)
;;   - :wat::holon::to-holon   (arc 225 — polymorphic lift to HolonAST)
;;   - :wat::holon::is?        (arc 226 — polymorphic type predicate)
;;   - :wat::core::keyword/to-string / keyword/from-string (arc 170 slice 3)
;;   - :wat::core::string::split / join / concat  (stdlib)
;;   - :wat::core::Vector/length / last / take    (stdlib)
;;
;; STOP-5: NO new substrate primitives. Pure macro expansion.
;; STOP-6: Single-arg defclass only. Inheritance is stone 227.2.
;; STOP-8: NO :user::* insertion. FQDN is user-declared.

(:wat::core::defmacro
  (:wat::holon::defclass
    (fqdn :AST<wat::core::nil>)
    -> :AST<wat::core::nil>)
  `(:wat::core::do
     (:wat::core::defn ~fqdn [v <- :wat::holon::HolonAST] -> :wat::holon::HolonAST
       (:wat::holon::Bind
         (:wat::holon::Atom (:wat::holon::to-holon ~(:wat::core::keyword/to-string fqdn)))
         (:wat::holon::Atom v)))
     (:wat::core::defn ~(:wat::core::let
                          [fqdn-str  (:wat::core::keyword/to-string fqdn)
                           parts     (:wat::core::string::split fqdn-str "::")
                           n         (:wat::core::Vector/length parts)
                           basename  (:wat::core::Option/expect -> :wat::core::string
                                       (:wat::core::last parts)
                                       "defclass: FQDN must have at least one segment")
                           pfx-parts (:wat::core::take parts (:wat::core::i64::-'2 n 1))
                           pfx-str   (:wat::core::string::join "::" pfx-parts)]
                          (:wat::core::keyword/from-string
                            (:wat::core::string::concat pfx-str "::" "is-" basename "?")))
                        [v <- :wat::holon::HolonAST] -> :wat::core::bool
       (:wat::holon::is? v ~(:wat::core::keyword/to-string fqdn)))))
