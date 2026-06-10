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
;;
;; The walk is bottom-up: fix the children first, then apply the node-level rules. Order
;; matters — `strip-if` recognises the `:wat::core::if` KEYWORD head, so it must run BEFORE
;; the head-rule turns that head into the `wat.core/if` symbol.

;; structural? — a node whose children we recurse into (list/vector/set/map).
(:wat::core::defn :fix::structural? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::let [k (:wat::core::ast-kind node)]
    (:wat::core::if (:wat::core::= k "list") true
      (:wat::core::if (:wat::core::= k "vector") true
        (:wat::core::if (:wat::core::= k "map") true
          (:wat::core::if (:wat::core::= k "set") true false))))))

;; annotated-if? — a List whose head is the `:wat::core::if` keyword and whose child[2] is
;; the bare Symbol `->` (the redundant return annotation). Keys on the EXACT head so an
;; `Option/expect -> :T` (different head) is never mistaken for an if annotation.
(:wat::core::defn :fix::annotated-if? [node <- :wat::WatAST] -> :wat::core::bool
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
(:wat::core::defn :fix::strip-if [node <- :wat::WatAST] -> :wat::WatAST
  (:wat::core::with-children node
    (:wat::core::concat (:wat::core::take (:wat::core::ast->children node) 2)
                        (:wat::core::drop (:wat::core::ast->children node) 4))))

;; head-keyword? — a `::`-namespaced keyword: a rust-scheme call head / reference, the kind
;; `keyword/to-symbol` converts. Bare data keywords (`:else`) have no `::` and are left alone.
(:wat::core::defn :fix::head-keyword? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "keyword")
    (:wat::core::string::contains? (:wat::core::ast-name node) "::")
    false))

;; convert-head — if `node` is a List whose head is a `::`-keyword, rewrite that head to a
;; faithful-Clojure symbol; otherwise leave it. A threading head (`->`/`->>`, a Symbol) and
;; an already-converted symbol head are not keywords, so they pass through untouched.
(:wat::core::defn :fix::convert-head [node <- :wat::WatAST] -> :wat::WatAST
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [kids (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? kids)
        node
        (:wat::core::let [h0 (:wat::core::Option/expect -> :wat::WatAST (:wat::core::first kids) "head")]
          (:wat::core::if (:fix::head-keyword? h0)
            (:wat::core::with-children node
              (:wat::core::concat (:wat::core::Vector :wat::WatAST (:wat::core::keyword/to-symbol h0))
                                  (:wat::core::drop kids 1)))
            node))))
    node))

;; fix-source — the recursive walk. Bottom-up: fix children, strip an if annotation, then
;; convert the head keyword.
(:wat::core::defn :fix::fix-source [node <- :wat::WatAST] -> :wat::WatAST
  (:wat::core::if (:fix::structural? node)
    (:wat::core::let [rebuilt  (:wat::core::with-children node
                                 (:wat::core::map
                                   (:wat::core::fn [c <- :wat::WatAST] -> :wat::WatAST (:fix::fix-source c))
                                   (:wat::core::ast->children node)))
                      stripped (:wat::core::if (:fix::annotated-if? rebuilt)
                                 (:fix::strip-if rebuilt)
                                 rebuilt)]
      (:fix::convert-head stripped))
    node))
