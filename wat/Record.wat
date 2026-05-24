;; :wat::Record::def — arc 234 stone 234.2b.
;;
;; Probe references:
;;   - tests/probe_arc234_stone2b_defrecord_macro.rs   (commit 676e861)
;;     6 contracts:
;;       1. Single-field expansion + invocation: constructor returns Value::wat__Record
;;       2. Per-field accessor returns the correct field value
;;       3. Predicate true on matching class
;;       4. Predicate false on non-matching class (two types defined; cross-call)
;;       5. Multi-field (3 fields) expansion + all three accessors work in order
;;       6. Zero-field expansion: constructor + predicate work
;;
;; Substrate primitives consumed (stone 234.2a):
;;   - :wat::Record::of           — constructor → Value::wat__Record dual-form hologram
;;   - :wat::Record/field-at      — positional field accessor; TypeScheme :T via recipient inference
;;
;; Canonical instance shape (Value::wat__Record triple):
;;   class_fqdn:  Arc<String>         e.g. "myapp::Voltage"  (no leading colon)
;;   struct_form: Arc<Vec<Value>>     field values in declaration order
;;   holon_form:  Arc<HolonAST>       Bind(Atom(class), Bundle(N field Binds))
;;
;;   N=0: class_fqdn + [] + Bind(Atom("ns::Tag"), Bundle())
;;   N=1: class_fqdn + [v] + Bind(Atom("ns::W"), Bundle(Bind(Atom("f"), Atom(v))))
;;   N=k: class_fqdn + [v0..vk-1] + Bind(Atom("ns::T"), Bundle(k field-Binds))
;;
;; Expansion shape for:
;;   (:wat::Record::def :myapp::Voltage [magnitude <- :wat::core::f64])
;;
;; →
;;   (:wat::core::do
;;     ;; 1. Constructor
;;     (:wat::core::defn :myapp::Voltage [magnitude <- :wat::core::f64] -> :wat::Record
;;       (:wat::Record::of
;;         :myapp::Voltage
;;         [magnitude]
;;         (:wat::holon::Bind
;;           (:wat::holon::Atom (:wat::holon::to-holon "myapp::Voltage"))
;;           (:wat::core::Result/expect -> :wat::holon::HolonAST
;;             (:wat::holon::Bundle
;;               [(:wat::holon::Bind
;;                  (:wat::holon::Atom (:wat::holon::to-holon "magnitude"))
;;                  (:wat::holon::Atom (:wat::holon::to-holon magnitude)))])
;;             "Record::def :myapp::Voltage instance: Bundle capacity exceeded"))))
;;
;;     ;; 2. Per-field accessor (one per declared field, spliced)
;;     (:wat::core::defn :myapp::Voltage/magnitude [v <- :wat::Record] -> :wat::core::f64
;;       (:wat::Record/field-at v 0))
;;
;;     ;; 3. Predicate
;;     (:wat::core::defn :myapp::is-Voltage? [v <- :wat::Record] -> :wat::core::bool
;;       (:wat::core::=
;;         (:wat::core::type v)
;;         "myapp::Voltage")))
;;
;; Naming rules (all derived at macro-expand time via keyword/to-string + string manipulation):
;;
;;   | Input FQDN            | Constructor           | Predicate                  | Classifier string      |
;;   |-----------------------|-----------------------|----------------------------|------------------------|
;;   | :myapp::Voltage       | :myapp::Voltage       | :myapp::is-Voltage?        | "myapp::Voltage"       |
;;   | :awesome::lib::Sensor | :awesome::lib::Sensor | :awesome::lib::is-Sensor?  | "awesome::lib::Sensor" |
;;   | :test::Foo            | :test::Foo            | :test::is-Foo?             | "test::Foo"            |
;;
;; Accessor naming: <class-fqdn>/<field-name> as keyword (e.g. :myapp::Voltage/magnitude).
;; Accessor signature: [v <- :wat::Record] -> :<declared-field-type>.
;; Recipient inference on :wat::Record/field-at drives T unification at check time.
;;
;; FQDN doctrine (feedback_fqdn_is_the_namespace): users declare their own namespace.
;; The macro NEVER inserts into :user::* or any auto-namespace.
;;
;; D10: Runtime class-safety check in accessor bodies is OUT OF SCOPE for this stone.
;;      Named follow-up: Stone 234.2c. User-facing safety pattern: check predicate first.
;;
;; D11: Field-type constraint enforcement at expand time is OUT OF SCOPE.
;;      Non-atomizable field type fails at constructor call site (clear runtime error).
;;
;; D12: Co-exists with :wat::holon::defrecord (DIFFERENT behavior: that macro → HolonAST;
;;      this macro → Value::wat__Record dual-form hologram). Retirement: Stone 234.6.
;;
;; D14: HARD CUT — no aliases. No single-arg form. Users MUST provide the field vector.
;;
;; Depends on:
;;   - :wat::Record::of              (arc 234 stone 234.2a — dual-form constructor)
;;   - :wat::Record/field-at         (arc 234 stone 234.2a — positional field accessor)
;;   - :wat::core::type              (arc 234 stone 234.0 — polymorphic type dispatch)
;;   - :wat::holon::Bind             (arc 228 — classifier-wrapped constructor)
;;   - :wat::holon::Atom             (arc 225 — narrow HolonAST wrapping)
;;   - :wat::holon::to-holon         (arc 225 — polymorphic lift to HolonAST)
;;   - :wat::holon::from-holon       (arc 225 — lower HolonAST to Value)
;;   - :wat::holon::from-wat         (arc 225 — WatAST → HolonAST)
;;   - :wat::holon::to-wat           (arc 225 — HolonAST → WatAST)
;;   - :wat::holon::Bundle           (arc 037 — multi-child Bundle; returns Result)
;;   - :wat::holon::Bundle/children  (arc 201 — Vec<HolonAST> from Bundle)
;;   - :wat::holon::statement-length (arc 037 — HolonAST surface arity)
;;   - :wat::core::Result/expect     (arc 108 — unwrap Result or panic)
;;   - :wat::core::Option/expect     (arc 108 — unwrap Option or panic)
;;   - :wat::core::keyword/to-string / keyword/from-string (arc 170 slice 3)
;;   - :wat::core::string::split / join / concat  (stdlib)
;;   - :wat::core::Vector/length / last / take / get (stdlib)
;;   - :wat::core::map / range       (stdlib iteration)
;;   - :wat::core::i64::*'2 / /'2 / +'2 / -'2  (integer arithmetic)
;;   - :wat::core::quasiquote        (runtime quasiquote; arc 091 slice 8)
;;   - :wat::core::= / :wat::core::let  (core)
;;
;; STOP-5: NO new substrate primitives beyond src/stdlib.rs WatSource entry.
;; STOP-6: No runtime class-safety check (D10 deferred to Stone 234.2c).
;; STOP-14: NO aliases for :wat::Record::def. HARD CUT.

(:wat::core::defmacro
  (:wat::Record::def
    (fqdn   :AST<wat::core::nil>)
    (fields :AST<wat::core::nil>)
    -> :AST<wat::core::nil>)
  `(:wat::core::do
     (:wat::core::defn ~fqdn [~@fields] -> :wat::Record
       (:wat::Record::of
         (:wat::core::keyword/from-string ~(:wat::core::keyword/to-string fqdn))
         [~@(:wat::core::let
               [fields-h (:wat::holon::from-wat (:wat::core::quote fields))
                n        (:wat::holon::statement-length fields-h)
                nf       (:wat::core::i64::/'2 n 3)
                children (:wat::holon::Bundle/children fields-h)
                syms     (:wat::core::map
                           (:wat::core::range 0 nf)
                           (:wat::core::fn [fi <- :wat::core::i64] -> :wat::WatAST
                             (:wat::core::let
                               [idx   (:wat::core::i64::*'2 fi 3)
                                name-h (:wat::core::Option/expect -> :wat::holon::HolonAST
                                         (:wat::core::Vector/get children idx)
                                         "Record::def: struct_form field name index out of range")
                                var-w (:wat::holon::to-wat name-h)]
                               var-w)))]
               syms)]
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
                                                  "Record::def: field name index out of range")
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
                 "Record::def "
                 (:wat::core::keyword/to-string fqdn)
                 " instance: Bundle capacity exceeded")))))
     ~@(:wat::core::let
           [fields-h    (:wat::holon::from-wat (:wat::core::quote fields))
            n           (:wat::holon::statement-length fields-h)
            nf          (:wat::core::i64::/'2 n 3)
            children    (:wat::holon::Bundle/children fields-h)
            fqdn-str    (:wat::core::keyword/to-string fqdn)
            accessors   (:wat::core::map
                          (:wat::core::range 0 nf)
                          (:wat::core::fn [fi <- :wat::core::i64] -> :wat::WatAST
                            (:wat::core::let
                              [idx          (:wat::core::i64::*'2 fi 3)
                               name-h       (:wat::core::Option/expect -> :wat::holon::HolonAST
                                              (:wat::core::Vector/get children idx)
                                              "Record::def: field name index out of range")
                               name-s       (:wat::core::keyword/to-string
                                              (:wat::holon::from-holon name-h))
                               type-h       (:wat::core::Option/expect -> :wat::holon::HolonAST
                                              (:wat::core::Vector/get children
                                                (:wat::core::i64::+'2 idx 2))
                                              "Record::def: field type index out of range")
                               type-w       (:wat::holon::to-wat type-h)
                               accessor-name (:wat::core::keyword/from-string
                                               (:wat::core::string::concat
                                                 fqdn-str
                                                 "/"
                                                 name-s))
                               msg-prefix   (:wat::core::string::concat
                                               ":"
                                               fqdn-str
                                               "/"
                                               name-s
                                               ": expected receiver of class :"
                                               fqdn-str
                                               ", got class :")]
                              (:wat::core::quasiquote
                                (:wat::core::defn
                                  (:wat::core::unquote accessor-name)
                                  [v <- :wat::Record] -> (:wat::core::unquote type-w)
                                  (:wat::Record/field-at
                                    (:wat::core::Option/expect -> :wat::Record
                                      (:wat::core::if
                                        (:wat::core::=
                                          (:wat::core::type v)
                                          (:wat::core::unquote fqdn-str))
                                        -> :wat::core::Option<wat::Record>
                                        (:wat::core::Some v)
                                        :wat::core::None)
                                      (:wat::core::string::concat
                                        (:wat::core::unquote msg-prefix)
                                        (:wat::core::type v)))
                                    (:wat::core::unquote fi)))))))]
           accessors)
     (:wat::core::defn ~(:wat::core::let
                           [fqdn-str  (:wat::core::keyword/to-string fqdn)
                            parts     (:wat::core::string::split fqdn-str "::")
                            n         (:wat::core::Vector/length parts)
                            basename  (:wat::core::Option/expect -> :wat::core::String
                                        (:wat::core::last parts)
                                        "Record::def: FQDN must have at least one segment")
                            pfx-parts (:wat::core::take parts (:wat::core::i64::-'2 n 1))
                            pfx-str   (:wat::core::string::join "::" pfx-parts)]
                           (:wat::core::keyword/from-string
                             (:wat::core::string::concat pfx-str "::" "is-" basename "?")))
                       [v <- :wat::Record] -> :wat::core::bool
       (:wat::core::=
         (:wat::core::type v)
         ~(:wat::core::keyword/to-string fqdn)))))
