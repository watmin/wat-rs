;; vigilatum: 2026-06-04T05:50:09Z — vigilia 4-spell L1+L2=0, checker-clean + deftest-green(record-def)
;;
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
;;     ;; 3. Per-field accessors (one per field; class-safety guarded, predicate
;;     ;;    auto-minted elsewhere — same as BASE)
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
  [fqdn   <- :wat::WatAST
   fields <- :wat::WatAST]
  -> :wat::WatAST
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
                               name-h (:wat::core::Option/expect  
                                        (:wat::core::Vector/get children idx)
                                        "Record::def: field name index out of range (recordtype emission)")
                               name-s (:wat::core::keyword/to-string
                                        (:wat::holon::from-holon name-h))]
                              (:wat::holon::to-wat (:wat::holon::to-holon name-s))))
                          (:wat::core::range 0 nf))]
             name-strs)])
     (:wat::core::defn ~fqdn [~@fields] -> ~fqdn
       (:wat::Record::of
         (:wat::core::keyword/from-string ~(:wat::core::keyword/to-string fqdn))
         [~@(:wat::core::let
               ;; Arc 291 hygiene fix: use (ast->children (quote fields)) to get the original
               ;; AST nodes with scope preserved, not the holon round-trip (which strips scope).
               ;; The binders in [~@fields] carry the original scope (e.g. scope 433 when this
               ;; defn is emitted inside another macro's quasiquote); the body references must
               ;; carry the SAME scope — reuse the original nodes from (quote fields) directly.
               ;; (quote fields) is needed: substitute_bindings replaces `fields` with the raw
               ;; WatAST::Vector node; quote wraps it as Value::wat__WatAST for ast->children.
               [raw-ch  (:wat::core::ast->children (:wat::core::quote fields))
                nf      (:wat::core::i64::/ (:wat::core::length raw-ch) 3)
                syms    (:wat::core::map
                           (:wat::core::fn [fi <- :wat::core::i64] -> :wat::WatAST
                             (:wat::core::let
                               [idx   (:wat::core::i64::* fi 3)
                                var-w (:wat::core::Option/expect
                                         (:wat::core::Vector/get raw-ch idx)
                                         "Record::def: struct_form field name index out of range")]
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
                               name-h       (:wat::core::Option/expect  
                                              (:wat::core::Vector/get children idx)
                                              "Record::def: field name index out of range")
                               name-s       (:wat::core::keyword/to-string
                                              (:wat::holon::from-holon name-h))
                               type-h       (:wat::core::Option/expect  
                                              (:wat::core::Vector/get children
                                                (:wat::core::i64::+ idx 2))
                                              "Record::def: field type index out of range")
                               type-w       (:wat::holon::to-wat type-h)
                               accessor-name (:wat::core::keyword/from-string
                                               (:wat::core::string::interpolate "{fqdn-str}/{name-s}" :fqdn-str fqdn-str :name-s name-s))
                               msg-prefix   (:wat::core::string::interpolate ":{fqdn-str}/{name-s}: expected receiver of class :{fqdn-str}, got class :" :fqdn-str fqdn-str :name-s name-s)]
                              (:wat::core::quasiquote
                                (:wat::core::defn
                                  (:wat::core::unquote accessor-name)
                                  [v <- :wat::Record] -> (:wat::core::unquote type-w)
                                  (:wat::Record/field-at
                                    (:wat::core::Option/expect  
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
  [fqdn   <- :wat::WatAST
   fields <- :wat::WatAST]
  -> :wat::WatAST
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
                               name-h (:wat::core::Option/expect  
                                        (:wat::core::Vector/get children idx)
                                        "Record::def: field name index out of range (recordtype emission)")
                               name-s (:wat::core::keyword/to-string
                                        (:wat::holon::from-holon name-h))]
                              (:wat::holon::to-wat (:wat::holon::to-holon name-s))))
                          (:wat::core::range 0 nf))]
             name-strs)])
     (:wat::core::defn ~fqdn [~@fields] -> ~fqdn
       (:wat::holon::Record::of
         (:wat::core::keyword/from-string ~(:wat::core::keyword/to-string fqdn))
         [~@(:wat::core::let
               ;; Arc 291 hygiene fix: use (ast->children (quote fields)) to get the original
               ;; AST nodes with scope preserved, not the holon round-trip (which strips scope).
               ;; The binders in [~@fields] carry the original scope; the body references must
               ;; carry the SAME scope — reuse the original nodes from (quote fields) directly.
               [raw-ch  (:wat::core::ast->children (:wat::core::quote fields))
                nf      (:wat::core::i64::/ (:wat::core::length raw-ch) 3)
                syms    (:wat::core::map
                           (:wat::core::fn [fi <- :wat::core::i64] -> :wat::WatAST
                             (:wat::core::let
                               [idx   (:wat::core::i64::* fi 3)
                                var-w (:wat::core::Option/expect
                                         (:wat::core::Vector/get raw-ch idx)
                                         "Record::def: struct_form field name index out of range")]
                               var-w))
                           (:wat::core::range 0 nf))]
               syms)]
         (:wat::holon::Bind
           (:wat::holon::Atom (:wat::holon::to-holon ~(:wat::core::keyword/to-string fqdn)))
           (:wat::core::Result/expect  
             (:wat::holon::Bundle
               [~@(:wat::core::let
                     ;; Arc 291 hygiene fix: use (ast->children (quote fields)) so var-w
                     ;; carries the original AST node (scope-preserving), matching the binder
                     ;; in [~@fields]. name-s still derives from holon round-trip (safe: it is
                     ;; a String, not a symbol reference in the emitted code).
                     [raw-ch     (:wat::core::ast->children (:wat::core::quote fields))
                      nf         (:wat::core::i64::/ (:wat::core::length raw-ch) 3)
                      ;; holon children still needed for name-s (field keyword → string)
                      fields-h   (:wat::holon::from-wat (:wat::core::quote fields))
                      h-children (:wat::holon::Bundle/children fields-h)
                      field-binds (:wat::core::map
                                    (:wat::core::fn [fi <- :wat::core::i64] -> :wat::WatAST
                                      (:wat::core::let
                                        [idx    (:wat::core::i64::* fi 3)
                                         name-h (:wat::core::Option/expect
                                                  (:wat::core::Vector/get h-children idx)
                                                  "Record::def: field name index out of range")
                                         name-s (:wat::core::keyword/to-string
                                                  (:wat::holon::from-holon name-h))
                                         ;; Arc 291: reuse original AST node (scope-preserving)
                                         var-w  (:wat::core::Option/expect
                                                  (:wat::core::Vector/get raw-ch idx)
                                                  "Record::def: holonic field var index out of range")]
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
                               name-h       (:wat::core::Option/expect  
                                              (:wat::core::Vector/get children idx)
                                              "Record::def: field name index out of range")
                               name-s       (:wat::core::keyword/to-string
                                              (:wat::holon::from-holon name-h))
                               type-h       (:wat::core::Option/expect  
                                              (:wat::core::Vector/get children
                                                (:wat::core::i64::+ idx 2))
                                              "Record::def: field type index out of range")
                               type-w       (:wat::holon::to-wat type-h)
                               accessor-name (:wat::core::keyword/from-string
                                               (:wat::core::string::interpolate "{fqdn-str}/{name-s}" :fqdn-str fqdn-str :name-s name-s))
                               msg-prefix   (:wat::core::string::interpolate ":{fqdn-str}/{name-s}: expected receiver of class :{fqdn-str}, got class :" :fqdn-str fqdn-str :name-s name-s)]
                              (:wat::core::quasiquote
                                (:wat::core::defn
                                  (:wat::core::unquote accessor-name)
                                  [v <- :wat::Record] -> (:wat::core::unquote type-w)
                                  (:wat::Record/field-at
                                    (:wat::core::Option/expect  
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
