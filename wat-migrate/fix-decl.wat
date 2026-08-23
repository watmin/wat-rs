;; wat-migrate/fix-decl.wat — `fix-form`: the throwaway declaration migrator.
;;
;; Closes the two gaps `fix.wat`'s `fix-source` leaves in DECLARATION and DEFN forms:
;;   Gap A — a bare non-arrow'd type-slot (typealias/newtype/recordtype child[2]) hit
;;            `head-keyword?` → `keyword/to-symbol` instead of `keyword/to-type-form`.
;;   Gap B — a generic DEFN name `map<T>` misfired through `type-shaped-keyword?`
;;            → a parametric FORM instead of a PLAIN symbol.
;;
;; Fix: position-aware rewrite keyed on the declaration/defn head. NON-BLESSED.
;; Retires at the hard-cut (4.4). Calls the blessed `:wat::fix::fix-source` and
;; `:wat::fix::fix-seq` for the remainder of each declaration's children.
;;
;; Namespace: :migrate:: (non-:wat:: to avoid the reserved-prefix guard)

;; name-fix — strip any `<…>` suffix from a name keyword and produce a plain symbol.
;;
;; Two cases:
;;   (a) `:wat::stream::map<T>` — has `::` after stripping `<T>` →
;;       split on `<` → `:wat::stream::map` → keyword-node → keyword/to-symbol
;;       → `wat.stream/map` (namespaced symbol).
;;   (b) `:Foo<T>` — bare upper-case, no `::` after stripping `<T>` →
;;       split on `<` → `:Foo` → strip leading `:` via subs(1, length) →
;;       symbol-node("Foo") → bare symbol `Foo`.
;;
;; A name without `<` splits to a single piece; first = the full name; same path.
;; Always produces a Symbol, never a parametric form.
(:wat::core::defn :migrate::name-fix [kw <- :wat::WatAST] -> :wat::WatAST
  (:wat::core::let [stripped (:wat::core::first
                                (:wat::core::string::split (:wat::core::ast-name kw) "<"))]
    (:wat::core::if (:wat::core::string::contains? stripped "::")
      (:wat::core::keyword/to-symbol (:wat::core::keyword-node stripped))
      (:wat::core::symbol-node
        (:wat::core::string::subs stripped 1 (:wat::core::string::length stripped))))))

;; type-slot-2? — TRUE when the declaration head names a form whose child[2] is a
;; bare non-arrow'd type-slot (typealias, newtype, recordtype). All others have no
;; extra type-slot override beyond the name: their type positions are arrow'd or
;; parametric and fix-seq already handles them.
(:wat::core::defn :migrate::type-slot-2? [head-name <- :wat::core::String] -> :wat::core::bool
  (:wat::core::if (:wat::core::= head-name ":wat::core::typealias") true
    (:wat::core::if (:wat::core::= head-name ":wat::core::newtype") true
      (:wat::core::= head-name ":wat::core::recordtype"))))

;; name-head? — TRUE when the head names a declaration/defn form (child[1] is the
;; declaration name → name-fix applies).
(:wat::core::defn :migrate::name-head? [head-name <- :wat::core::String] -> :wat::core::bool
  (:wat::core::if (:wat::core::= head-name ":wat::core::defn") true
    (:wat::core::if (:wat::core::= head-name ":wat::core::def") true
      (:wat::core::if (:wat::core::= head-name ":wat::core::typealias") true
        (:wat::core::if (:wat::core::= head-name ":wat::core::newtype") true
          (:wat::core::if (:wat::core::= head-name ":wat::core::recordtype") true
            (:wat::core::if (:wat::core::= head-name ":wat::core::defstruct") true
              (:wat::core::if (:wat::core::= head-name ":wat::core::defclause") true
                (:wat::core::if (:wat::core::= head-name ":wat::core::defenum") true
                  (:wat::core::= head-name ":wat::core::typeunion"))))))))))

;; fix-types — type-convert each element of a member vector (typeunion's child[2] is a
;; `[type type …]` vector of NON-arrow'd member types; fix-seq would route them through
;; head-keyword? → keyword/to-symbol → the wrong `wat.core/i64`). Each keyword element →
;; keyword/to-type-form; any non-keyword → fix-source (defensive). Mirrors fix-seq's shape.
(:wat::core::defn :migrate::fix-types [items <- (:wat::core::Vector :- [:wat::WatAST])] -> (:wat::core::Vector :- [:wat::WatAST])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector :wat::WatAST)
    (:wat::core::let [h   (:wat::core::first items)
                      out (:wat::core::if (:wat::core::= (:wat::core::ast-kind h) "keyword")
                            (:wat::core::keyword/to-type-form h)
                            (:wat::fix::fix-source h))]
      (:wat::core::concat (:wat::core::Vector :wat::WatAST out)
                          (:migrate::fix-types (:wat::core::rest items))))))

;; fix-type-vector — rebuild a member vector with every element type-converted.
(:wat::core::defn :migrate::fix-type-vector [vec <- :wat::WatAST] -> :wat::WatAST
  (:wat::core::with-children vec (:migrate::fix-types (:wat::core::ast->children vec))))

;; fix-form — position-aware declaration/defn rewriter.
;;
;; If node is a List headed by a NAME-HEAD keyword:
;;   child[0] (head)  → keyword/to-symbol  (direct; head is guaranteed a ::  keyword)
;;   child[1] (name)  → name-fix    (strips <T>, produces a plain symbol)
;;   child[2] (type)  → keyword/to-type-form  IF type-slot-2? (typealias/newtype/recordtype)
;;                      followed by fix-seq(rest3, false) for children[3+]
;;             OR      fix-seq(rest2, false) for all of children[2+]  (defn/def/etc.)
;; Else: fix-source(node).
;;
;; fix-seq (blessed, position-aware) handles arrows + post-arrow type keywords in the
;; remainder so that `-> :T` and `<- :T` binders migrate correctly.
(:wat::core::defn :migrate::fix-form [node <- :wat::WatAST] -> :wat::WatAST
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch   (:wat::core::ast->children node)
                      head (:wat::core::first ch)]
      (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::ast-kind head) "keyword")
                        (:migrate::name-head? (:wat::core::ast-name head))
                        false)
        (:wat::core::let [ch1   (:wat::core::first (:wat::core::drop ch 1))
                          rest2  (:wat::core::drop ch 2)
                          fixed-head (:wat::core::keyword/to-symbol head)
                          fixed-name (:migrate::name-fix ch1)
                          fixed-rest (:wat::core::if (:migrate::type-slot-2? (:wat::core::ast-name head))
                                       (:wat::core::if (:wat::core::empty? rest2)
                                         (:wat::core::Vector :wat::WatAST)
                                         (:wat::core::let [ch2   (:wat::core::first rest2)
                                                           rest3  (:wat::core::rest rest2)]
                                           (:wat::core::concat
                                             (:wat::core::Vector :wat::WatAST
                                               (:wat::core::keyword/to-type-form ch2))
                                             (:wat::fix::fix-seq rest3 false))))
                                     (:wat::core::if (:wat::core::= (:wat::core::ast-name head) ":wat::core::typeunion")
                                       (:wat::core::if (:wat::core::empty? rest2)
                                         (:wat::core::Vector :wat::WatAST)
                                         (:wat::core::let [uch2  (:wat::core::first rest2)
                                                           urest (:wat::core::rest rest2)]
                                           (:wat::core::concat
                                             (:wat::core::Vector :wat::WatAST
                                               (:migrate::fix-type-vector uch2))
                                             (:wat::fix::fix-seq urest false))))
                                       (:wat::fix::fix-seq rest2 false)))]
          (:wat::core::with-children node
            (:wat::core::concat
              (:wat::core::Vector :wat::WatAST fixed-head)
              (:wat::core::concat
                (:wat::core::Vector :wat::WatAST fixed-name)
                fixed-rest))))
        (:wat::fix::fix-source node)))
    (:wat::fix::fix-source node)))
