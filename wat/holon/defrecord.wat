;; :wat::holon::defrecord — arc 227 stone 227.2 v3.
;;
;; Design substrate (stone 227.2 v3 — both probes pass, composition proven):
;;   - tests/probe_diagnostic_macro_splice_from_let.rs   (commit c18fa6b)
;;     Proves ~@(let [forms (map xs fn)] forms) splices Vec<WatAST> built
;;     via :wat::core::map + runtime quasiquote at macro expand time.
;;   - tests/probe_diagnostic_bundle_result_compose.rs   (commit 72367f1)
;;     Proves (:wat::holon::Bind classifier (:wat::core::Result/expect
;;     (:wat::holon::Bundle items) "msg")) produces canonical Bind(Atom, Bundle)
;;     instance shape with inner children preserved.
;;
;; Both probes pass. The STOP-5b framing from v2 is retired.
;; Task #477 and Task #478 are DISCONFIRMED by the probes.
;;
;; Canonical instance shape per typed-entities doctrine (ALL N):
;;   N=0: Bind(Atom("ns::Tag"), Bundle())
;;   N=1: Bind(Atom("ns::W"),   Bundle(Bind(Atom("v"),  Atom(value))))
;;   N=2: Bind(Atom("ns::P"),   Bundle(Bind(Atom("a"), Atom(av)),
;;                                      Bind(Atom("b"), Atom(bv))))
;;   N=k: Bind(Atom("ns::T"),   Bundle(... k field-Binds ...))
;;
;; Result/expect discipline per arc 037: Bundle returns
;;   Result<HolonAST, CapacityExceeded>; the macro uses
;;   :wat::core::Result/expect to acknowledge the Kanerva-capacity discipline.
;;
;; Mandated 2-arg form (stone 227.2 v2 hard cut — preserved in v3):
;;   (defrecord <fqdn> <field-list>)
;;
;; The single-arg form (defrecord :fqdn) is RETIRED per stone 227.2 v2.
;; Users MUST provide the field-list (possibly empty []).
;;
;; Usage:
;;   (:wat::holon::defrecord :myapp::Tag [])                        ;; tagged unit
;;   (:wat::holon::defrecord :myapp::Voltage
;;     [magnitude <- :wat::core::f64])                              ;; single-field
;;   (:wat::holon::defrecord :myapp::Point
;;     [x <- :wat::core::i64  y <- :wat::core::i64])               ;; two-field
;;   (:wat::holon::defrecord :myapp::Triple
;;     [a <- :wat::core::i64  b <- :wat::core::String  c <- :wat::core::bool])
;;
;; Field-list mechanics (commas are EDN whitespace, not tokens):
;;   [a <- :T1  b <- :T2]  →  6 children after from-wat:
;;     [symbol(a), symbol(<-), keyword(T1), symbol(b), symbol(<-), keyword(T2)]
;;   N = total-tokens / 3 = (statement-length fields-h) / 3
;;   field-name token at children-index: fi * 3  (fi in [0, N))
;;
;; Expands to:
;;
;;   (:wat::core::do
;;     (:wat::core::defn :myapp::Point [x <- :i64  y <- :i64] -> :wat::holon::HolonAST
;;       (:wat::holon::Bind
;;         (:wat::holon::Atom (:wat::holon::to-holon "myapp::Point"))
;;         (:wat::core::Result/expect -> :wat::holon::HolonAST
;;           (:wat::holon::Bundle
;;             [(:wat::holon::Bind (:wat::holon::Atom (to-holon "x")) (Atom (to-holon x)))
;;              (:wat::holon::Bind (:wat::holon::Atom (to-holon "y")) (Atom (to-holon y)))])
;;           "defrecord :myapp::Point instance: Bundle capacity exceeded")))
;;
;;     (:wat::core::defn :myapp::is-Point? [v <- :wat::holon::HolonAST] -> :wat::core::bool
;;       (:wat::holon::is? v "myapp::Point")))
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
;; Accessor synthesis (:ns::Type/field-name functions) is deferred.
;; The substrate lacks an ergonomic Bind-decomposition primitive
;; (:wat::holon::Bind/inner or similar) needed to walk the inner Bundle
;; of a defrecord instance at runtime. Named-field accessors are
;; future work pending a Bind/inner substrate primitive.
;;
;; Depends on:
;;   - :wat::holon::Bind              (arc 228 — classifier-wrapped constructor)
;;   - :wat::holon::Atom              (arc 225 — narrow HolonAST wrapping)
;;   - :wat::holon::to-holon          (arc 225 — polymorphic lift to HolonAST)
;;   - :wat::holon::is?               (arc 226 — polymorphic type predicate)
;;   - :wat::holon::from-holon        (arc 225 — lower HolonAST to Value)
;;   - :wat::holon::from-wat          (arc 225 — WatAST → HolonAST)
;;   - :wat::holon::to-wat            (arc 225 — HolonAST → WatAST)
;;   - :wat::holon::Bundle            (arc 037 — multi-child Bundle; returns Result)
;;   - :wat::holon::Bundle/children   (arc 201 — Vec<HolonAST> from Bundle)
;;   - :wat::holon::statement-length  (arc 037 — HolonAST surface arity)
;;   - :wat::core::Result/expect      (arc 108 — unwrap Result or panic)
;;   - :wat::core::Option/expect      (arc 108 — unwrap Option or panic)
;;   - :wat::core::keyword/to-string / keyword/from-string (arc 170 slice 3)
;;   - :wat::core::string::split / join / concat  (stdlib)
;;   - :wat::core::Vector/length / last / take / get (stdlib)
;;   - :wat::core::map / range        (stdlib iteration)
;;   - :wat::core::i64::*'2 / /'2 / -'2  (integer arithmetic)
;;   - :wat::core::quasiquote (runtime quasiquote; arc 091 slice 8)
;;   - :wat::core::= / :wat::core::if / :wat::core::let  (core)
;;
;; STOP-5: NO new substrate primitives. Pure macro expansion.
;; STOP-6: Methods stay separate defns. defrecord mints data-only type.
;; STOP-8: NO :user::* insertion. FQDN is user-declared.
;; HARD CUT: Single-arg (defrecord :fqdn) form RETIRED. Users write [].

(:wat::core::defmacro
  (:wat::holon::defrecord
    (fqdn   :AST<wat::core::nil>)
    (fields :AST<wat::core::nil>)
    -> :AST<wat::core::nil>)
  `(:wat::core::do
     (:wat::core::defn ~fqdn [~@fields] -> :wat::holon::HolonAST
       (:wat::holon::Bind
         (:wat::holon::Atom (:wat::holon::to-holon ~(:wat::core::keyword/to-string fqdn)))
         (:wat::core::Result/expect -> :wat::holon::HolonAST
           (:wat::holon::Bundle
             [~@(:wat::core::let
                   [fields-h    (:wat::holon::from-wat (:wat::core::quote fields))
                    n           (:wat::holon::statement-length fields-h)
                    nf          (:wat::core::i64::/'2 n 3)
                    children    (:wat::holon::Bundle/children fields-h)
                    field-binds (:wat::core::map
                                  (:wat::core::range 0 nf)
                                  (:wat::core::fn [fi <- :wat::core::i64] -> :wat::WatAST
                                    (:wat::core::let
                                      [idx    (:wat::core::i64::*'2 fi 3)
                                       name-h (:wat::core::Option/expect -> :wat::holon::HolonAST
                                                (:wat::core::Vector/get children idx)
                                                "defrecord: field name index out of range")
                                       name-s (:wat::core::keyword/to-string
                                                (:wat::holon::from-holon name-h))
                                       var-w  (:wat::holon::to-wat name-h)]
                                      (:wat::core::quasiquote
                                        (:wat::holon::Bind
                                          (:wat::holon::Atom
                                            (:wat::holon::to-holon
                                              (:wat::core::unquote name-s)))
                                          (:wat::holon::Atom
                                            (:wat::holon::to-holon
                                              (:wat::core::unquote var-w))))))))]
                   field-binds)])
           ~(:wat::core::string::concat
               "defrecord "
               (:wat::core::keyword/to-string fqdn)
               " instance: Bundle capacity exceeded"))))
     (:wat::core::defn ~(:wat::core::let
                          [fqdn-str  (:wat::core::keyword/to-string fqdn)
                           parts     (:wat::core::string::split fqdn-str "::")
                           n         (:wat::core::Vector/length parts)
                           basename  (:wat::core::Option/expect -> :wat::core::String
                                       (:wat::core::last parts)
                                       "defrecord: FQDN must have at least one segment")
                           pfx-parts (:wat::core::take parts (:wat::core::i64::-'2 n 1))
                           pfx-str   (:wat::core::string::join "::" pfx-parts)]
                          (:wat::core::keyword/from-string
                            (:wat::core::string::concat pfx-str "::" "is-" basename "?")))
                        [v <- :wat::holon::HolonAST] -> :wat::core::bool
       (:wat::holon::is? v ~(:wat::core::keyword/to-string fqdn)))))
