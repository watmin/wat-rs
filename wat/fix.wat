;; wat/fix.wat — `fix-source`: the wat-to-wat faithful-Clojure converter.
;;
;; THE PROVING POINT: wat writes wat. `fix-source` recursively rewrites a form tree
;; (read by `read-string`) from the rust-scheme surface into a faithful-Clojure dialect,
;; rebuilding faithfully with `with-children` so only what a rule changes changes.
;;
;; It is written in CURRENT rust-scheme wat (so it loads on today's runtime); when the
;; corpus drive runs, fix-source fixes ITSELF (homoiconic self-application).
;;
;; Rules (each probe-gated; grown one at a time):
;;   strip-if   — drop the now-redundant `-> :T` return annotation from an `if`.
;;   head-rule  — a list-head `::`-keyword (a rust-scheme call head) → a faithful-Clojure
;;                symbol via `keyword/to-symbol` (e.g. `:wat::core::if` → `wat.core/if`).
;;   arrow-rule — a bare `<-` / `->` symbol (annotation arrow) → `:-`.
;;   type-rule  — a keyword right after an arrow, or a structurally-type-shaped keyword
;;                (name contains `<` or `(`), → faithful type form via `keyword/to-type-form`.
;;
;; The walk is position-aware: `fix-seq` carries `prev-arrow?` so post-arrow keywords are
;; converted as types. `strip-if` recognises the `:wat::core::if` KEYWORD head, so it must
;; run BEFORE the head-rule turns that head into the `wat.core/if` symbol.

;; structural? — a node whose children we recurse into (list/vector/set/map).
(:wat::core::defn :wat::fix::structural? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::let [k (:wat::core::ast-kind node)]
    (:wat::core::if (:wat::core::= k "list") true
      (:wat::core::if (:wat::core::= k "vector") true
        (:wat::core::if (:wat::core::= k "map") true
          (:wat::core::if (:wat::core::= k "set") true false))))))

;; annotated-if? — a List whose head is the `:wat::core::if` keyword and whose child[2] is
;; the bare Symbol `->` (the redundant return annotation). Keys on the EXACT head so an
;; `Option/expect -> :T` (different head) is never mistaken for an if annotation.
(:wat::core::defn :wat::fix::annotated-if? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? (:wat::core::drop ch 2))
        false
        (:wat::core::let [head (:wat::core::Option/expect -> :wat::WatAST (:wat::core::first ch) "head")
                          c2   (:wat::core::Option/expect -> :wat::WatAST (:wat::core::first (:wat::core::drop ch 2)) "c2")]
          (:wat::core::if (:wat::core::= (:wat::core::ast-name head) ":wat::core::if")
            (:wat::core::if (:wat::core::= (:wat::core::ast-kind c2) "symbol")
              (:wat::core::= (:wat::core::ast-name c2) "->")
              false)
            false))))
    false))

;; strip-if — rebuild the bare `(if cond then else)` from `(if cond -> :T then else)`,
;; dropping children [2] (`->`) and [3] (the type).
(:wat::core::defn :wat::fix::strip-if [node <- :wat::WatAST] -> :wat::WatAST
  (:wat::core::with-children node
    (:wat::core::concat (:wat::core::take (:wat::core::ast->children node) 2)
                        (:wat::core::drop (:wat::core::ast->children node) 4))))

;; head-keyword? — a `::`-namespaced keyword: a rust-scheme call head / reference, the kind
;; `keyword/to-symbol` converts. Bare data keywords (`:else`) have no `::` and are left alone.
(:wat::core::defn :wat::fix::head-keyword? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "keyword")
    (:wat::core::string::contains? (:wat::core::ast-name node) "::")
    false))

;; arrow? — a bare binder/return annotation arrow SYMBOL (<- or ->). NOTE: the threading
;; macro head is the KEYWORD :wat::core::-> ; a bare `->` SYMBOL is always an annotation arrow.
(:wat::core::defn :wat::fix::arrow? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "symbol")
    (:wat::core::if (:wat::core::= (:wat::core::ast-name node) "<-") true
      (:wat::core::= (:wat::core::ast-name node) "->"))
    false))

;; type-shaped-keyword? — a keyword STRUCTURALLY a type: a parametric `Head<...>` or a
;; tuple/fn `(...)`. The discriminator requires a MATCHING close — a parametric has BOTH `<`
;; and `>`, a tuple/fn has BOTH `(` and `)` — so the comparison operators `:wat::core::<` /
;; `:wat::core::<=` (which contain `<` but no `>`) are NOT mistaken for types.
(:wat::core::defn :wat::fix::type-shaped-keyword? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "keyword")
    (:wat::core::let [name (:wat::core::ast-name node)]
      (:wat::core::if (:wat::core::if (:wat::core::string::contains? name "<")
                        (:wat::core::string::contains? name ">")
                        false)
        true
        (:wat::core::if (:wat::core::string::contains? name "(")
          (:wat::core::string::contains? name ")")
          false)))
    false))

;; fix-seq — position-aware left-to-right walk over a child vector, carrying prev-arrow?.
;; Order matters: post-arrow type, then structural type, then arrow, then head/ref, then recurse.
(:wat::core::defn :wat::fix::fix-seq [items <- :wat::core::Vector<wat::WatAST> prev-arrow? <- :wat::core::bool] -> :wat::core::Vector<wat::WatAST>
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector :wat::WatAST)
    (:wat::core::let [h   (:wat::core::Option/expect -> :wat::WatAST (:wat::core::first items) "fix-seq: head")
                      tl  (:wat::core::rest items)
                      out (:wat::core::if (:wat::core::if prev-arrow? (:wat::core::= (:wat::core::ast-kind h) "keyword") false)
                            (:wat::core::keyword/to-type-form h)
                          (:wat::core::if (:wat::fix::type-shaped-keyword? h)
                            (:wat::core::keyword/to-type-form h)
                          (:wat::core::if (:wat::fix::arrow? h)
                            (:wat::core::keyword-node ":-")
                          (:wat::core::if (:wat::fix::head-keyword? h)
                            (:wat::core::keyword/to-symbol h)
                            (:wat::fix::fix-source h)))))]
      (:wat::core::concat (:wat::core::Vector :wat::WatAST out)
                          (:wat::fix::fix-seq tl (:wat::fix::arrow? h))))))

;; fix-source — strip an if-annotation (recognises the ::if KEYWORD head, so BEFORE the head
;; gets symbol-ised), then the position-aware walk.
(:wat::core::defn :wat::fix::fix-source [node <- :wat::WatAST] -> :wat::WatAST
  (:wat::core::if (:wat::fix::structural? node)
    (:wat::core::let [stripped (:wat::core::if (:wat::fix::annotated-if? node) (:wat::fix::strip-if node) node)]
      (:wat::core::with-children stripped (:wat::fix::fix-seq (:wat::core::ast->children stripped) false)))
    node))
