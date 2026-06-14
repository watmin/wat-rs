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

;; ─── Stone 251.5 / Slice 4.2 — comment-faithful span-edit codemod ───────────
;;
;; fix-text(src) → migrated-src
;;
;; Algorithm: parse src → locate edits (via ast-span) → splice ORIGINAL text
;; right-to-left so comments + formatting survive byte-identical.
;;
;; edit = Tuple(off, old-len, new-text) : :(i64,i64,String)
;;   off      — flat 0-indexed char offset in src
;;   old-len  — char length of token being replaced/deleted
;;   new-text — replacement string; "" means pure deletion
;;
;; Edits are collected left-to-right (ascending offset order), reversed to
;; right-to-left, then applied with fix-text-apply.

;; fix-text-line-start — char offset of the first character of line N (1-indexed).
;; lines = result of (string::split src "\n"); each element excludes the newline.
;; line 1 starts at 0; line N starts at: sum over k=1..N-1 of (length(lines[k-1]) + 1).
(:wat::core::defn :wat::fix::fix-text-line-start
  [n     <- :wat::core::i64
   lines <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::i64
  (:wat::core::if (:wat::core::= n 1)
    0
    (:wat::core::let [fst (:wat::core::Option/expect -> :wat::core::String
                              (:wat::core::first lines)
                              "fix-text-line-start: no line")]
      (:wat::core::+ (:wat::core::string::length fst)
        (:wat::core::+ 1
          (:wat::fix::fix-text-line-start
            (:wat::core::- n 1)
            (:wat::core::rest lines)))))))

;; fix-text-offset-of — convert an ast-span {:line N :col C} map to a flat char offset.
;; loc is the HashMap<keyword,i64> from (ast-span node); lines = (split src "\n").
;; offset = line-start(line) + (col - 1)  (col is 1-indexed char count from line start).
(:wat::core::defn :wat::fix::fix-text-offset-of
  [loc   <- :wat::core::HashMap<wat::core::keyword,wat::core::i64>
   lines <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::i64
  (:wat::core::let [ln (:wat::core::Option/expect -> :wat::core::i64
                           (:wat::core::HashMap/get loc :line)
                           "fix-text-offset-of: :line")
                    co (:wat::core::Option/expect -> :wat::core::i64
                           (:wat::core::HashMap/get loc :col)
                           "fix-text-offset-of: :col")]
    (:wat::core::+ (:wat::fix::fix-text-line-start ln lines)
                   (:wat::core::- co 1))))

;; fix-text-deletion-edit — a one-element Vector holding a deletion edit for node.
;; Deletion covers exactly the token text (ast-name char length); surrounding whitespace stays.
(:wat::core::defn :wat::fix::fix-text-deletion-edit
  [node  <- :wat::WatAST
   lines <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::Vector<(wat::core::i64,wat::core::i64,wat::core::String)>
  (:wat::core::let [off     (:wat::fix::fix-text-offset-of (:wat::core::ast-span node) lines)
                    old-len (:wat::core::string::length (:wat::core::ast-name node))]
    (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String)
      (:wat::core::Tuple off old-len ""))))

;; fix-text-leaf-edits — apply the same rule order as fix-seq to a leaf node,
;; emitting zero or one edit (never recurses into children).
;; post-arrow type > structural type > arrow > head-keyword > no-op.
(:wat::core::defn :wat::fix::fix-text-leaf-edits
  [node        <- :wat::WatAST
   prev-arrow? <- :wat::core::bool
   lines       <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::Vector<(wat::core::i64,wat::core::i64,wat::core::String)>
  (:wat::core::let [kind (:wat::core::ast-kind node)]
    (:wat::core::if (:wat::core::= kind "keyword")
      ;; keyword leaf — check type-annotation and head-keyword rules
      (:wat::core::let [span    (:wat::core::ast-span node)
                        off     (:wat::fix::fix-text-offset-of span lines)
                        nm      (:wat::core::ast-name node)
                        old-len (:wat::core::string::length nm)]
        (:wat::core::if prev-arrow?
          ;; post-arrow keyword is a type annotation → convert to type form
          (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String)
            (:wat::core::Tuple off old-len
              (:wat::core::write-forms (:wat::core::keyword/to-type-form node))))
          (:wat::core::if (:wat::fix::type-shaped-keyword? node)
            ;; parametric/tuple keyword → type form
            (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String)
              (:wat::core::Tuple off old-len
                (:wat::core::write-forms (:wat::core::keyword/to-type-form node))))
            (:wat::core::if (:wat::fix::head-keyword? node)
              ;; ::-namespaced call head → faithful-Clojure symbol
              (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String)
                (:wat::core::Tuple off old-len
                  (:wat::core::ast-name (:wat::core::keyword/to-symbol node))))
              ;; bare data keyword (no ::, not type-shaped) — no edit
              (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String))))))
      (:wat::core::if (:wat::core::= kind "symbol")
        ;; symbol leaf — only arrow rule applies
        (:wat::core::if (:wat::fix::arrow? node)
          (:wat::core::let [span    (:wat::core::ast-span node)
                            off     (:wat::fix::fix-text-offset-of span lines)
                            nm      (:wat::core::ast-name node)
                            old-len (:wat::core::string::length nm)]
            (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String)
              (:wat::core::Tuple off old-len ":-")))
          ;; non-arrow symbol — no edit
          (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String)))
        ;; int, float, bool, string, nil — no edit
        (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String))))))

;; fix-text-node-edits — dispatch: structural nodes → fix-text-struct-edits;
;; leaf nodes → fix-text-leaf-edits with position context.
(:wat::core::defn :wat::fix::fix-text-node-edits
  [node        <- :wat::WatAST
   prev-arrow? <- :wat::core::bool
   lines       <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::Vector<(wat::core::i64,wat::core::i64,wat::core::String)>
  (:wat::core::if (:wat::fix::structural? node)
    (:wat::fix::fix-text-struct-edits node lines)
    (:wat::fix::fix-text-leaf-edits node prev-arrow? lines)))

;; fix-text-seq-edits — position-aware left-to-right walk over a child sequence.
;; Mirrors fix-seq's rule order; collects edits in ascending offset order.
(:wat::core::defn :wat::fix::fix-text-seq-edits
  [items       <- :wat::core::Vector<wat::WatAST>
   prev-arrow? <- :wat::core::bool
   lines       <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::Vector<(wat::core::i64,wat::core::i64,wat::core::String)>
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String))
    (:wat::core::let [h  (:wat::core::Option/expect -> :wat::WatAST
                             (:wat::core::first items)
                             "fix-text-seq-edits: empty")
                      tl (:wat::core::rest items)]
      (:wat::core::concat
        (:wat::fix::fix-text-node-edits h prev-arrow? lines)
        (:wat::fix::fix-text-seq-edits tl (:wat::fix::arrow? h) lines)))))

;; fix-text-struct-edits — collect edits from a structural node.
;; For annotated-if: emit deletion edits for child[2](arrow) + child[3](type),
;; and leaf-edit for the head keyword; recurse into cond and branches normally.
;; For all other structural nodes: delegate to fix-text-seq-edits on children.
(:wat::core::defn :wat::fix::fix-text-struct-edits
  [node  <- :wat::WatAST
   lines <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::Vector<(wat::core::i64,wat::core::i64,wat::core::String)>
  (:wat::core::if (:wat::fix::annotated-if? node)
    ;; strip-if: manually process children to emit deletions for -> and :T
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::let [head     (:wat::core::Option/expect -> :wat::WatAST
                                     (:wat::core::first ch)
                                     "fix-text-struct-edits: head")
                        c1       (:wat::core::Option/expect -> :wat::WatAST
                                     (:wat::core::first (:wat::core::rest ch))
                                     "fix-text-struct-edits: cond")
                        c2       (:wat::core::Option/expect -> :wat::WatAST
                                     (:wat::core::first (:wat::core::drop ch 2))
                                     "fix-text-struct-edits: arrow")
                        c3       (:wat::core::Option/expect -> :wat::WatAST
                                     (:wat::core::first (:wat::core::drop ch 3))
                                     "fix-text-struct-edits: type")
                        branches (:wat::core::drop ch 4)]
        ;; edits in ascending text order: head, cond, arrow-del, type-del, branches
        (:wat::core::concat
          (:wat::fix::fix-text-leaf-edits head false lines)
          (:wat::core::concat
            (:wat::fix::fix-text-node-edits c1 false lines)
            (:wat::core::concat
              (:wat::fix::fix-text-deletion-edit c2 lines)
              (:wat::core::concat
                (:wat::fix::fix-text-deletion-edit c3 lines)
                (:wat::fix::fix-text-seq-edits branches false lines)))))))
    ;; normal structural node — walk children with fix-text-seq-edits
    (:wat::fix::fix-text-seq-edits (:wat::core::ast->children node) false lines)))

;; fix-text-apply — apply a list of edits (in right-to-left order) to src.
;; Each edit is Tuple(off, old-len, new-text); replaces src[off..off+old-len] with new-text.
(:wat::core::defn :wat::fix::fix-text-apply
  [src   <- :wat::core::String
   edits <- :wat::core::Vector<(wat::core::i64,wat::core::i64,wat::core::String)>]
  -> :wat::core::String
  (:wat::core::if (:wat::core::empty? edits)
    src
    (:wat::core::let [edit     (:wat::core::Option/expect -> :(wat::core::i64,wat::core::i64,wat::core::String)
                                   (:wat::core::first edits)
                                   "fix-text-apply: edit")
                      off      (:wat::core::first edit)
                      old-len  (:wat::core::second edit)
                      new-text (:wat::core::third edit)
                      tl       (:wat::core::rest edits)
                      new-src  (:wat::core::string::concat
                                  (:wat::core::string::subs src 0 off)
                                  new-text
                                  (:wat::core::string::subs src
                                    (:wat::core::+ off old-len)
                                    (:wat::core::string::length src)))]
      (:wat::fix::fix-text-apply new-src tl))))

;; fix-text — comment-faithful codemod: src string → migrated-src string.
;; Parses src to collect span-located edits (left-to-right / ascending offset),
;; reverses the list to right-to-left, then splices the ORIGINAL text for each edit.
;; Comments and formatting between edited tokens survive byte-identical.
(:wat::core::defn :wat::fix::fix-text
  [src <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::let [lines     (:wat::core::string::split src "\n")
                    tree      (:wat::core::read-string src)
                    forms     (:wat::core::ast->children tree)
                    all-edits (:wat::fix::fix-text-seq-edits forms false lines)
                    rev-edits (:wat::core::reverse all-edits)]
    (:wat::fix::fix-text-apply src rev-edits)))
