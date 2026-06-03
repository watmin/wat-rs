;; :wat::Record::def — BASE record macro — arc 237 Stone S-C.3.
;;
;; Defines a BASE record: struct_form only, NO holon_form.
;; The unmarked name is the cheap common case.
;;
;; :wat::holon::Record::def — HOLONIC record macro — arc 237 Stone S-C.3.
;;
;; Defines a HOLONIC record: struct_form + holon_form (opt-in for holon-ops).
;;
;; Probe references:
;;   - tests/probe_arc237_sC3_macro_split.rs   (18 contracts; arc 237 Stone S-C.3)
;;   - tests/probe_arc234_stone2b_defrecord_macro.rs   (commit 676e861)
;;     6 contracts:
;;       1. Single-field expansion + invocation: constructor returns Value::wat__Record
;;       2. Per-field accessor returns the correct field value
;;       3. Predicate true on matching class
;;       4. Predicate false on non-matching class (two types defined; cross-call)
;;       5. Multi-field (3 fields) expansion + all three accessors work in order
;;       6. Zero-field expansion: constructor + predicate work
;;
;; Substrate primitives consumed:
;;   BASE:
;;   - :wat::Record::of             — 2-arg constructor → Value::wat__Record (struct only)
;;   - :wat::Record/field-at        — positional field accessor
;;   HOLONIC:
;;   - :wat::holon::Record::of      — 3-arg constructor → Value::wat__holon__Record
;;   - :wat::Record/field-at        — positional field accessor (same; variant-agnostic)
;;
;; Flavor hierarchy (Liskov):
;;   :wat::Record                   — base parent (all records are :wat::Record)
;;   :wat::holon::Record            — holonic parent (inherits from :wat::Record)
;;   A func wanting :wat::Record    accepts BOTH base and holonic instances.
;;   A func wanting :wat::holon::Record accepts ONLY holonic instances.
;;
;; Expansion shape for BASE:
;;   (:wat::Record::def :myapp::Pt [x <- :wat::core::i64  y <- :wat::core::i64])
;;
;; →
;;   (:wat::core::do
;;     ;; 1. recordtype declaration (parent = :wat::Record)
;;     (:wat::core::recordtype :myapp::Pt :wat::Record [x y])
;;
;;     ;; 2. Constructor (2-arg :wat::Record::of — no holon_form)
;;     (:wat::core::defn :myapp::Pt [x <- :wat::core::i64  y <- :wat::core::i64] -> :wat::Record
;;       (:wat::Record::of
;;         :myapp::Pt
;;         [x y]))
;;
;;     ;; 3. Per-field accessor (one per declared field)
;;     (:wat::core::defn :myapp::Pt/x [v <- :wat::Record] -> :wat::core::i64
;;       (:wat::Record/field-at v 0))
;;
;;     ;; 4. Predicate (auto-minted by arc 237.6; not emitted by macro)
;;     )
;;
;; Expansion shape for HOLONIC:
;;   (:wat::holon::Record::def :myapp::HPt [x <- :wat::core::i64  y <- :wat::core::i64])
;;
;; →
;;   (:wat::core::do
;;     ;; 1. recordtype declaration (parent = :wat::holon::Record)
;;     (:wat::core::recordtype :myapp::HPt :wat::holon::Record [x y])
;;
;;     ;; 2. Constructor (3-arg :wat::holon::Record::of — with holon_form)
;;     (:wat::core::defn :myapp::HPt [x <- :wat::core::i64  y <- :wat::core::i64] -> :wat::holon::Record
;;       (:wat::holon::Record::of
;;         :myapp::HPt
;;         [x y]
;;         (:wat::holon::Bind ...)))
;;
;;     ;; 3. Per-field accessor + Predicate
;;     )
;;
;; Naming rules (derived at macro-expand time via keyword/to-string + string manipulation):
;;
;;   | Input FQDN            | Constructor           | Predicate                  | Classifier string      |
;;   |-----------------------|-----------------------|----------------------------|------------------------|
;;   | :myapp::Pt            | :myapp::Pt            | :myapp::is-Pt?             | "myapp::Pt"            |
;;   | :awesome::lib::Sensor | :awesome::lib::Sensor | :awesome::lib::is-Sensor?  | "awesome::lib::Sensor" |
;;
;; Accessor naming: <class-fqdn>/<field-name> as keyword (e.g. :myapp::Pt/x).
;; Accessor signature: [v <- :wat::Record] -> :<declared-field-type>.
;;
;; FQDN doctrine (feedback_fqdn_is_the_namespace): users declare their own namespace.
;; The macro NEVER inserts into :user::* or any auto-namespace.
;;
;; D10: Runtime class-safety check in accessor bodies is OUT OF SCOPE for this stone.
;; D11: Field-type constraint enforcement at expand time is OUT OF SCOPE.
;; D14: HARD CUT — no aliases. No single-arg form. Users MUST provide the field vector.

;; ─── BASE macro (:wat::Record::def) ──────────────────────────────────────────

(:wat::core::defmacro :wat::Record::def
  [fqdn   <- :AST<wat::core::nil>
   fields <- :AST<wat::core::nil>]
  -> :AST<wat::core::nil>
  `(:wat::core::do
     (:wat::core::recordtype ~fqdn :wat::Record
       [~@(:wat::core::let
             [fields-h  (:wat::holon::from-wat (:wat::core::quote fields))
              n         (:wat::holon::statement-length fields-h)
              nf        (:wat::core::i64::/ n 3)
              children  (:wat::holon::Bundle/children fields-h)
              name-strs (:wat::core::map
                          (:wat::core::range 0 nf)
                          (:wat::core::fn [fi <- :wat::core::i64] -> :wat::WatAST
                            (:wat::core::let
                              [idx    (:wat::core::i64::* fi 3)
                               name-h (:wat::core::Option/expect -> :wat::holon::HolonAST
                                        (:wat::core::Vector/get children idx)
                                        "Record::def: field name index out of range (recordtype emission)")
                               name-s (:wat::core::keyword/to-string
                                        (:wat::holon::from-holon name-h))]
                              (:wat::holon::to-wat (:wat::holon::to-holon name-s)))))]
             name-strs)])
     (:wat::core::defn ~fqdn [~@fields] -> :wat::Record
       (:wat::Record::of
         (:wat::core::keyword/from-string ~(:wat::core::keyword/to-string fqdn))
         [~@(:wat::core::let
               [fields-h (:wat::holon::from-wat (:wat::core::quote fields))
                n        (:wat::holon::statement-length fields-h)
                nf       (:wat::core::i64::/ n 3)
                children (:wat::holon::Bundle/children fields-h)
                syms     (:wat::core::map
                           (:wat::core::range 0 nf)
                           (:wat::core::fn [fi <- :wat::core::i64] -> :wat::WatAST
                             (:wat::core::let
                               [idx   (:wat::core::i64::* fi 3)
                                name-h (:wat::core::Option/expect -> :wat::holon::HolonAST
                                         (:wat::core::Vector/get children idx)
                                         "Record::def: struct_form field name index out of range")
                                var-w (:wat::holon::to-wat name-h)]
                               var-w)))]
               syms)]))
     ~@(:wat::core::let
           [fields-h    (:wat::holon::from-wat (:wat::core::quote fields))
            n           (:wat::holon::statement-length fields-h)
            nf          (:wat::core::i64::/ n 3)
            children    (:wat::holon::Bundle/children fields-h)
            fqdn-str    (:wat::core::keyword/to-string fqdn)
            accessors   (:wat::core::map
                          (:wat::core::range 0 nf)
                          (:wat::core::fn [fi <- :wat::core::i64] -> :wat::WatAST
                            (:wat::core::let
                              [idx          (:wat::core::i64::* fi 3)
                               name-h       (:wat::core::Option/expect -> :wat::holon::HolonAST
                                              (:wat::core::Vector/get children idx)
                                              "Record::def: field name index out of range")
                               name-s       (:wat::core::keyword/to-string
                                              (:wat::holon::from-holon name-h))
                               type-h       (:wat::core::Option/expect -> :wat::holon::HolonAST
                                              (:wat::core::Vector/get children
                                                (:wat::core::i64::+ idx 2))
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
           accessors)))

;; ─── HOLONIC macro (:wat::holon::Record::def) ────────────────────────────────

(:wat::core::defmacro :wat::holon::Record::def
  [fqdn   <- :AST<wat::core::nil>
   fields <- :AST<wat::core::nil>]
  -> :AST<wat::core::nil>
  `(:wat::core::do
     (:wat::core::recordtype ~fqdn :wat::holon::Record
       [~@(:wat::core::let
             [fields-h  (:wat::holon::from-wat (:wat::core::quote fields))
              n         (:wat::holon::statement-length fields-h)
              nf        (:wat::core::i64::/ n 3)
              children  (:wat::holon::Bundle/children fields-h)
              name-strs (:wat::core::map
                          (:wat::core::range 0 nf)
                          (:wat::core::fn [fi <- :wat::core::i64] -> :wat::WatAST
                            (:wat::core::let
                              [idx    (:wat::core::i64::* fi 3)
                               name-h (:wat::core::Option/expect -> :wat::holon::HolonAST
                                        (:wat::core::Vector/get children idx)
                                        "Record::def: field name index out of range (recordtype emission)")
                               name-s (:wat::core::keyword/to-string
                                        (:wat::holon::from-holon name-h))]
                              (:wat::holon::to-wat (:wat::holon::to-holon name-s)))))]
             name-strs)])
     (:wat::core::defn ~fqdn [~@fields] -> :wat::holon::Record
       (:wat::holon::Record::of
         (:wat::core::keyword/from-string ~(:wat::core::keyword/to-string fqdn))
         [~@(:wat::core::let
               [fields-h (:wat::holon::from-wat (:wat::core::quote fields))
                n        (:wat::holon::statement-length fields-h)
                nf       (:wat::core::i64::/ n 3)
                children (:wat::holon::Bundle/children fields-h)
                syms     (:wat::core::map
                           (:wat::core::range 0 nf)
                           (:wat::core::fn [fi <- :wat::core::i64] -> :wat::WatAST
                             (:wat::core::let
                               [idx   (:wat::core::i64::* fi 3)
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
                      nf          (:wat::core::i64::/ n 3)
                      children    (:wat::holon::Bundle/children fields-h)
                      field-binds (:wat::core::map
                                    (:wat::core::range 0 nf)
                                    (:wat::core::fn [fi <- :wat::core::i64] -> :wat::WatAST
                                      (:wat::core::let
                                        [idx    (:wat::core::i64::* fi 3)
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
            nf          (:wat::core::i64::/ n 3)
            children    (:wat::holon::Bundle/children fields-h)
            fqdn-str    (:wat::core::keyword/to-string fqdn)
            accessors   (:wat::core::map
                          (:wat::core::range 0 nf)
                          (:wat::core::fn [fi <- :wat::core::i64] -> :wat::WatAST
                            (:wat::core::let
                              [idx          (:wat::core::i64::* fi 3)
                               name-h       (:wat::core::Option/expect -> :wat::holon::HolonAST
                                              (:wat::core::Vector/get children idx)
                                              "Record::def: field name index out of range")
                               name-s       (:wat::core::keyword/to-string
                                              (:wat::holon::from-holon name-h))
                               type-h       (:wat::core::Option/expect -> :wat::holon::HolonAST
                                              (:wat::core::Vector/get children
                                                (:wat::core::i64::+ idx 2))
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
           accessors)))
