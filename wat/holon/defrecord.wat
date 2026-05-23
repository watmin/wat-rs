;; :wat::holon::defrecord — arc 227 stone 227.2 v2.
;;
;; Mints a user-defined classifier-wrapped typed entity in the user's
;; declared namespace. MANDATED 2-arg form (stone 227.2 v2 hard cut):
;;   (defrecord <fqdn> <field-list>)
;;
;; The single-arg form (defrecord :fqdn) is RETIRED per stone 227.2 v2.
;; Users MUST provide the field-list (possibly empty []).
;;
;; Usage:
;;   (:wat::holon::defrecord :myapp::Tag [])           ;; tagged unit
;;   (:wat::holon::defrecord :myapp::Voltage
;;     [magnitude <- :wat::core::f64])                 ;; single-field
;;
;; Field-list cases:
;;
;;   [] empty                    → zero-arg constructor; no accessors; predicate
;;   [name <- :Type]  N=1 field  → one-arg constructor; predicate
;;   [a <- :T1, b <- :T2] N≥2   → STOP-5b deferred; errors at expand time
;;
;; Expands to definitions in the user's DECLARED namespace:
;;
;;   For [] (tagged unit):
;;     (:wat::core::defn :myapp::Tag [] -> :wat::holon::HolonAST
;;       (:wat::holon::Bind
;;         (:wat::holon::Atom (:wat::holon::to-holon "myapp::Tag"))
;;         (:wat::holon::Atom (:wat::holon::to-holon :wat::core::nil))))
;;
;;     (:wat::core::defn :myapp::is-Tag? [v <- :wat::holon::HolonAST] -> :wat::core::bool
;;       (:wat::holon::is? v "myapp::Tag"))
;;
;;   For [magnitude <- :wat::core::f64] (single-field):
;;     (:wat::core::defn :myapp::Voltage [magnitude <- :wat::core::f64] -> :wat::holon::HolonAST
;;       (:wat::holon::Bind
;;         (:wat::holon::Atom (:wat::holon::to-holon "myapp::Voltage"))
;;         (:wat::holon::Bind
;;           (:wat::holon::Atom (:wat::holon::to-holon "magnitude"))
;;           (:wat::holon::Atom (:wat::holon::to-holon magnitude)))))
;;
;;     (:wat::core::defn :myapp::is-Voltage? [v <- :wat::holon::HolonAST] -> :wat::core::bool
;;       (:wat::holon::is? v "myapp::Voltage"))
;;
;; Inner structure (STOP-5b finding — stone 227.2 v2):
;;   The inner slot uses Atom(nil) for tagged units and Bind(Atom(name), Atom(value))
;;   for single-field records. :wat::holon::Bundle is NOT used in the inner slot —
;;   Bundle returns Result<HolonAST, CapacityExceeded> which is incompatible with
;;   Bind's HolonAST argument. Bind-as-inner-slot is the honest substrate choice.
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
;;   (:defrecord :appA::Voltage []) → classifier "appA::Voltage"
;;   (:defrecord :appB::Voltage []) → classifier "appB::Voltage"
;;   These are NOT the same classifier — predicate discrimination is honest.
;;
;; FQDN doctrine (:feedback_fqdn_is_the_namespace): users declare their own
;; namespace. The macro NEVER inserts into :user::* or any auto-namespace.
;;
;; STOP-5b finding (stone 227.2 v2):
;;   Accessor synthesis (:ns::Type/field-name functions) is deferred.
;;   The substrate lacks an ergonomic Bind-decomposition primitive
;;   (:wat::holon::Bind/inner or similar) needed to walk the inner Bundle
;;   of a defrecord instance at runtime. Named-field accessors are
;;   future work pending a Bind/inner substrate primitive.
;;
;;   N≥2 fields are also deferred: generating N named-field Bundle children
;;   at macro expand time requires substrate iteration support that does not
;;   yet exist. Macro errors at expand time for N≥2.
;;
;; Depends on:
;;   - :wat::holon::Bind              (arc 228 — classifier-wrapped constructor)
;;   - :wat::holon::Atom              (arc 225 — narrow HolonAST wrapping)
;;   - :wat::holon::to-holon          (arc 225 — polymorphic lift to HolonAST)
;;   - :wat::holon::is?               (arc 226 — polymorphic type predicate)
;;   - :wat::holon::from-holon        (arc 225 — lower HolonAST to Value)
;;   - :wat::holon::from-wat          (arc 225 — WatAST → HolonAST)
;;   - :wat::holon::to-wat            (arc 225 — HolonAST → WatAST)
;;   - :wat::holon::Bundle/first      (arc 201 — first child of Bundle)
;;   - :wat::holon::statement-length  (arc 037 — HolonAST surface arity)
;;   - :wat::core::keyword/to-string / keyword/from-string (arc 170 slice 3)
;;   - :wat::core::string::split / join / concat  (stdlib)
;;   - :wat::core::Vector/length / last / take / get (stdlib)
;;   - :wat::core::quasiquote (runtime quasiquote; arc 091 slice 8)
;;   - :wat::core::= / :wat::core::if / :wat::core::let  (core)
;;
;; STOP-5: NO new substrate primitives. Pure macro expansion.
;; STOP-6: Methods stay separate defns. defrecord mints data-only type.
;; STOP-8: NO :user::* insertion. FQDN is user-declared.
;; HARD CUT: Single-arg (defrecord :fqdn) form RETIRED. Users write [].

(:wat::core::defmacro
  (:wat::holon::defrecord
    (fqdn :AST<wat::core::nil>)
    (fields :AST<wat::core::nil>)
    -> :AST<wat::core::nil>)
  `(:wat::core::do
     (:wat::core::defn ~fqdn [~@fields] -> :wat::holon::HolonAST
       (:wat::holon::Bind
         (:wat::holon::Atom (:wat::holon::to-holon ~(:wat::core::keyword/to-string fqdn)))
         ~(:wat::core::let
             [fields-h  (:wat::holon::from-wat (:wat::core::quote fields))
              n         (:wat::holon::statement-length fields-h)]
             (:wat::core::if (:wat::core::= n 0) -> :wat::WatAST
               ;; N=0: tagged unit — inner Atom wrapping nil (Bundle avoided; returns Result)
               (:wat::core::quote (:wat::holon::Atom (:wat::holon::to-holon :wat::core::nil)))
               (:wat::core::if (:wat::core::= n 3) -> :wat::WatAST
                 ;; N=1: single-field — inner Bind(Atom(name), Atom(value))
                 (:wat::core::let
                   [first-h  (:wat::holon::Bundle/first fields-h)
                    var0     (:wat::holon::to-wat first-h)
                    name0    (:wat::core::keyword/to-string
                               (:wat::holon::from-holon first-h))]
                   (:wat::core::quasiquote
                     (:wat::holon::Bind
                       (:wat::holon::Atom (:wat::holon::to-holon ~name0))
                       (:wat::holon::Atom (:wat::holon::to-holon ~var0)))))
                 ;; N≥2: STOP-5b — deferred; surface diagnostic at expand time
                 (:wat::core::Option/expect -> :wat::WatAST
                   (:wat::core::Vector/get (:wat::core::Vector :wat::WatAST) 0)
                   "defrecord v2 STOP-5b: N>1 fields require substrate iteration at macro expand time; deferred to future stone. Use N=0 [] or N=1 [field <- :Type] for now."))))))
     (:wat::core::defn ~(:wat::core::let
                          [fqdn-str  (:wat::core::keyword/to-string fqdn)
                           parts     (:wat::core::string::split fqdn-str "::")
                           n         (:wat::core::Vector/length parts)
                           basename  (:wat::core::Option/expect -> :wat::core::string
                                       (:wat::core::last parts)
                                       "defrecord: FQDN must have at least one segment")
                           pfx-parts (:wat::core::take parts (:wat::core::i64::-'2 n 1))
                           pfx-str   (:wat::core::string::join "::" pfx-parts)]
                          (:wat::core::keyword/from-string
                            (:wat::core::string::concat pfx-str "::" "is-" basename "?")))
                        [v <- :wat::holon::HolonAST] -> :wat::core::bool
       (:wat::holon::is? v ~(:wat::core::keyword/to-string fqdn)))))
