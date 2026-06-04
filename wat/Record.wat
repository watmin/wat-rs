;; :wat::Record::def — BASE record macro.
;;
;; Defines a BASE record: struct_form only, NO holon_form.
;; The unmarked name is the cheap common case.
;;
;; :wat::holon::Record::def — HOLONIC record macro.
;;
;; Defines a HOLONIC record: struct_form + holon_form (opt-in for holon-ops).
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
;;     ;; 3. Per-field accessor (one per field), receiver class-safety guarded:
;;     ;;    field-at runs only after (= (type v) :myapp::Pt) is checked.
;;     (:wat::core::defn :myapp::Pt/x [v <- :wat::Record] -> :wat::core::i64
;;       (:wat::Record/field-at <class-checked v> 0))
;;
;;     ;; 4. Predicate (auto-minted elsewhere; NOT emitted by this macro)
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
;; (The Predicate column is shown for FQDN-derivation reference; the predicate is
;;  auto-minted elsewhere, not emitted by this macro.)
;;
;; Accessor naming: <class-fqdn>/<field-name> as keyword (e.g. :myapp::Pt/x).
;; Accessor signature: [v <- :wat::Record] -> :<declared-field-type>.
;;
;; FQDN doctrine (feedback_fqdn_is_the_namespace): users declare their own namespace.
;; The macro NEVER inserts into :user::* or any auto-namespace.
;;
;; Accessor bodies are class-safety guarded: each runs (:wat::Record/field-at …)
;; only after checking (= (type v) <fqdn>), panicking with a "got class …" message
;; on a mismatched receiver (see the accessor expansion in the macro body below).
;; Field-type constraints are NOT enforced at expand time. No aliases, no single-arg
;; form — users MUST provide the field vector.

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
                          (:wat::core::fn [fi <- :wat::core::i64] -> :wat::WatAST
                            (:wat::core::let
                              [idx    (:wat::core::i64::* fi 3)
                               name-h (:wat::core::Option/expect -> :wat::holon::HolonAST
                                        (:wat::core::Vector/get children idx)
                                        "Record::def: field name index out of range (recordtype emission)")
                               name-s (:wat::core::keyword/to-string
                                        (:wat::holon::from-holon name-h))]
                              (:wat::holon::to-wat (:wat::holon::to-holon name-s))))
                          (:wat::core::range 0 nf))]
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
                           (:wat::core::fn [fi <- :wat::core::i64] -> :wat::WatAST
                             (:wat::core::let
                               [idx   (:wat::core::i64::* fi 3)
                                name-h (:wat::core::Option/expect -> :wat::holon::HolonAST
                                         (:wat::core::Vector/get children idx)
                                         "Record::def: struct_form field name index out of range")
                                var-w (:wat::holon::to-wat name-h)]
                               var-w))
                           (:wat::core::range 0 nf))]
               syms)]))
     ~@(:wat::core::let
           [fields-h    (:wat::holon::from-wat (:wat::core::quote fields))
            n           (:wat::holon::statement-length fields-h)
            nf          (:wat::core::i64::/ n 3)
            children    (:wat::holon::Bundle/children fields-h)
            fqdn-str    (:wat::core::keyword/to-string fqdn)
            accessors   (:wat::core::map
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
                                    (:wat::core::unquote fi))))))
                          (:wat::core::range 0 nf))]
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
                          (:wat::core::fn [fi <- :wat::core::i64] -> :wat::WatAST
                            (:wat::core::let
                              [idx    (:wat::core::i64::* fi 3)
                               name-h (:wat::core::Option/expect -> :wat::holon::HolonAST
                                        (:wat::core::Vector/get children idx)
                                        "Record::def: field name index out of range (recordtype emission)")
                               name-s (:wat::core::keyword/to-string
                                        (:wat::holon::from-holon name-h))]
                              (:wat::holon::to-wat (:wat::holon::to-holon name-s))))
                          (:wat::core::range 0 nf))]
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
                           (:wat::core::fn [fi <- :wat::core::i64] -> :wat::WatAST
                             (:wat::core::let
                               [idx   (:wat::core::i64::* fi 3)
                                name-h (:wat::core::Option/expect -> :wat::holon::HolonAST
                                         (:wat::core::Vector/get children idx)
                                         "Record::def: struct_form field name index out of range")
                                var-w (:wat::holon::to-wat name-h)]
                               var-w))
                           (:wat::core::range 0 nf))]
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
                                                (:wat::core::unquote var-w)))))))
                                    (:wat::core::range 0 nf))]
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
                                    (:wat::core::unquote fi))))))
                          (:wat::core::range 0 nf))]
           accessors)))
