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
;;
;; ════════════════════════════════════════════════════════════════════════════════════
;;  ⚠ BOOTSTRAP — running a codemod that ships ALONGSIDE a checker/runtime change.
;;     READ THIS before you give up and hand-edit. (A prior self did exactly that —
;;     abandoned this purpose-built tool because the dance below wasn't written down.)
;; ════════════════════════════════════════════════════════════════════════════════════
;;
;; The stdlib (wat/*.wat — including THIS file) is FROZEN INTO the wat binary at BUILD
;; time. Two facts bite when your codemod (a) calls a NEW `:wat::fix::…` verb you just
;; added here, AND (b) ships with a Rust change (src/check.rs / src/runtime.rs) that makes
;; the OLD corpus form ILLEGAL:
;;
;;   • The binary can't SEE your new verb until rebuilt — the on-disk edit is invisible to
;;     the embedded copy → `#wat.kernel/UnknownFunction {:path ":wat::fix::your-verb"}`.
;;   • But rebuilding NOW also bakes in the NEW checker, which then REJECTS the still-old
;;     stdlib at freeze → you can't build the binary you need to fix the stdlib. Chicken/egg.
;;
;;  THE STASH-DANCE (this is the supported path — do NOT hand-edit instead):
;;    1.  git stash push -m "rust change" src/check.rs src/runtime.rs   # old checker restored
;;    2.  cargo build --release                                          # old checker + your NEW verb
;;    3.  printf '["pathA" "pathB" …]\n' \                               # rewrite the WHOLE corpus
;;          | cargo wat ./wat-scripts/fixes/<your-fix>.wat    #   (list EVERY path; a
;;                                                                        #    missed file breaks the build)
;;    4.  git stash pop                                                  # restore the rust change
;;    5.  cargo build --release && cargo test                           # new checker; corpus now new-form
;;
;;  Dry-run step 3 on a `/tmp` COPY first and `diff` it — verify the rewrite is exactly the
;;  structural change you intend (the edits are token-span deletions; surrounding whitespace
;;  survives — that trailing whitespace is wat-fmt's job, not the codemod's).
;;
;;  No Rust change in your strike (e.g. a pure rename)? Skip the stash — just `cargo build
;;  --release` once to pick up your new verb, then run step 3.
;; ════════════════════════════════════════════════════════════════════════════════════

;; structural? — a node whose children we recurse into (list/vector/set/map).
(:wat::core::defn :wat::fix::structural? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::let [k (:wat::core::ast-kind node)]
    (:wat::core::contains? (:wat::core::HashSet :wat::type::Infer "list" "vector" "map" "set") k)))

;; annotated-if? — a List whose head is the `:wat::core::if` keyword and whose child[2] is
;; the bare Symbol `->` (the redundant return annotation). Keys on the EXACT head so an
;; `Option/expect -> :T` (different head) is never mistaken for an if annotation.
(:wat::core::defn :wat::fix::annotated-if? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? (:wat::core::drop ch 2))
        false
        (:wat::core::let [head (:wat::core::first ch)
                          c2   (:wat::core::first (:wat::core::drop ch 2))]
          (:wat::core::if (:wat::core::= (:wat::core::ast-name head) ":wat::core::if")
            (:wat::core::if (:wat::core::= (:wat::core::ast-kind c2) "symbol")
              (:wat::core::= (:wat::core::ast-name c2) "->")
              false)
            false))))
    false))

;; strip-if — rebuild the bare `(if cond then else)` from `(if cond -> :T then else)`,
;; dropping children [2] (`->`) and [3] (the type).
;; Arc 118.2a — `take`/`drop` flipped LAZY (return Stream); `concat` (unchanged, Vector/
;; PersistentVector/List-only) needs both sides eager.
(:wat::core::defn :wat::fix::strip-if [node <- :wat::WatAST] -> :wat::WatAST
  (:wat::core::with-children node
    (:wat::core::concat (:wat::core::into [] (:wat::core::take (:wat::core::ast->children node) 2))
                        (:wat::core::into [] (:wat::core::drop (:wat::core::ast->children node) 4)))))

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
    (:wat::core::let [h   (:wat::core::first items)
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
    (:wat::core::let [fst (:wat::core::first lines)]
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
  (:wat::core::let [ln (:wat::core::Option/expect  
                           (:wat::core::HashMap/get loc :line)
                           "fix-text-offset-of: :line")
                    co (:wat::core::Option/expect  
                           (:wat::core::HashMap/get loc :col)
                           "fix-text-offset-of: :col")]
    (:wat::core::+ (:wat::fix::fix-text-line-start ln lines)
                   (:wat::core::- co 1))))

;; fix-text-span-len — compute the char length of a source span (end offset minus start offset).
;; start-span and end-span are {:line N :col N} HashMaps (same shape as (ast-span node) /
;; (ast-end-span node)); lines = (string::split src "\n").
;; Returns offset-of(end) - offset-of(start): the number of chars the span covers.
(:wat::core::defn :wat::fix::fix-text-span-len
  [start-span <- :wat::core::HashMap<wat::core::keyword,wat::core::i64>
   end-span   <- :wat::core::HashMap<wat::core::keyword,wat::core::i64>
   lines      <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::i64
  (:wat::core::i64::-
    (:wat::fix::fix-text-offset-of end-span lines)
    (:wat::fix::fix-text-offset-of start-span lines)))

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
    (:wat::core::let [h  (:wat::core::first items)
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
      (:wat::core::let [head     (:wat::core::first ch)
                        c1       (:wat::core::first (:wat::core::rest ch))
                        c2       (:wat::core::first (:wat::core::drop ch 2))
                        c3       (:wat::core::first (:wat::core::drop ch 3))
                        ;; Arc 118.2a — `drop` flipped LAZY; `branches` feeds
                        ;; `fix-text-seq-edits`, which declares a `Vector<WatAST>` param.
                        branches (:wat::core::into [] (:wat::core::drop ch 4))]
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
    (:wat::core::let [edit     (:wat::core::first edits)
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

;; ─── Arc 258 — generic `-> :T` ascription stripper (reusable refactor tool) ──────────
;;
;; strip-arrow-ascription(src, heads) → migrated-src: deletes the `-> :T` return
;; ascription (a `->` SYMBOL + the type keyword following it) from every LIST whose
;; HEAD keyword is in `heads`. Comment-faithful (rides fix-text-deletion-edit + apply),
;; idempotent, head-GATED (a `->` inside an unrelated or nested form is left alone), and
;; position-agnostic (the arrow may sit at any child index — child[1] for `expect`,
;; child[2] for `if`/`match`). The checker/runtime change that makes the bare form legal
;; is per-form; THIS is the shared call-site rewriter every such kill reuses.

;; str-in? — String membership in a Vector<String> (explicit; not index-contains?).
(:wat::core::defn :wat::fix::str-in?
  [s <- :wat::core::String  xs <- :wat::core::Vector<wat::core::String>] -> :wat::core::bool
  (:wat::core::if (:wat::core::empty? xs)
    false
    (:wat::core::if (:wat::core::= s (:wat::core::first xs))
      true
      (:wat::fix::str-in? s (:wat::core::rest xs)))))

;; strip-arrow-scan — within a HEAD-MATCHED list's children, delete each `->` SYMBOL and
;; the child immediately after it (the type keyword); recurse (strip-arrow-edits) into
;; every other child so NESTED matched forms are caught too.
(:wat::core::defn :wat::fix::strip-arrow-scan
  [items       <- :wat::core::Vector<wat::WatAST>
   prev-arrow? <- :wat::core::bool
   heads       <- :wat::core::Vector<wat::core::String>
   lines       <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::Vector<(wat::core::i64,wat::core::i64,wat::core::String)>
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String))
    (:wat::core::let [h  (:wat::core::first items)
                      tl (:wat::core::rest items)]
      (:wat::core::if prev-arrow?
        ;; h is the type keyword after `->` → delete; do NOT recurse into it
        (:wat::core::concat (:wat::fix::fix-text-deletion-edit h lines)
                            (:wat::fix::strip-arrow-scan tl false heads lines))
        (:wat::core::if (:wat::fix::right-arrow? h)
          ;; h is the `->` → delete; mark prev-arrow for the next child
          (:wat::core::concat (:wat::fix::fix-text-deletion-edit h lines)
                              (:wat::fix::strip-arrow-scan tl true heads lines))
          ;; normal child → recurse for nested matched forms, no deletion here
          (:wat::core::concat (:wat::fix::strip-arrow-edits h heads lines)
                              (:wat::fix::strip-arrow-scan tl false heads lines)))))))

;; strip-arrow-seq — recurse strip-arrow-edits over each child (non-matched nodes).
(:wat::core::defn :wat::fix::strip-arrow-seq
  [items <- :wat::core::Vector<wat::WatAST>
   heads <- :wat::core::Vector<wat::core::String>
   lines <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::Vector<(wat::core::i64,wat::core::i64,wat::core::String)>
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String))
    (:wat::core::concat (:wat::fix::strip-arrow-edits (:wat::core::first items) heads lines)
                        (:wat::fix::strip-arrow-seq (:wat::core::rest items) heads lines))))

;; strip-arrow-edits — node → deletion edits for `-> :T` in lists headed by `heads`.
(:wat::core::defn :wat::fix::strip-arrow-edits
  [node  <- :wat::WatAST
   heads <- :wat::core::Vector<wat::core::String>
   lines <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::Vector<(wat::core::i64,wat::core::i64,wat::core::String)>
  (:wat::core::if (:wat::fix::structural? node)
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::if (:wat::core::empty? ch)
                        false
                        (:wat::core::if (:wat::core::= (:wat::core::ast-kind (:wat::core::first ch)) "keyword")
                          (:wat::fix::str-in? (:wat::core::ast-name (:wat::core::first ch)) heads)
                          false))
        (:wat::fix::strip-arrow-scan ch false heads lines)
        (:wat::fix::strip-arrow-seq ch heads lines)))
    (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String))))

;; strip-arrow-ascription — src → migrated-src for the given head-set.
(:wat::core::defn :wat::fix::strip-arrow-ascription
  [src   <- :wat::core::String
   heads <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::String
  (:wat::core::let [lines     (:wat::core::string::split src "\n")
                    tree      (:wat::core::read-string src)
                    forms     (:wat::core::ast->children tree)
                    all-edits (:wat::fix::strip-arrow-seq forms heads lines)]
    (:wat::fix::fix-text-apply src (:wat::core::reverse all-edits))))

;; ─── Stone 251.5 / Slice 4.2b — fix-macro-param-types: the first migration RULE ──────────
;;
;; fix-macro-param-types(src) → migrated-src
;;
;; A defmacro param/return is annotated with a type the macro engine discards;
;; the only honest type is `:wat::WatAST` (a macro arg is always a form). This
;; rule rewrites, COMMENT-FAITHFULLY (riding fix-text-apply's span-splice):
;;   - each FIXED param's type   → :wat::WatAST
;;   - the REST param's type     → :wat::core::Vector<wat::WatAST>
;;   - the RETURN type           → :wat::WatAST
;; and touches DEFMACRO forms ONLY — defn/fn type annotations are real and survive.
;;
;; Rides fix-text-apply + fix-text-offset-of; the EDIT-COLLECTION is the new work.

;; right-arrow? — a bare `->` SYMBOL (return-type annotation arrow, not `<-`).
(:wat::core::defn :wat::fix::right-arrow? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "symbol")
    (:wat::core::= (:wat::core::ast-name node) "->")
    false))

;; amp? — the bare `&` SYMBOL (rest-param marker in argvec).
(:wat::core::defn :wat::fix::amp? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "symbol")
    (:wat::core::= (:wat::core::ast-name node) "&")
    false))

;; defmacro? — a List whose head keyword name is ":wat::core::defmacro".
(:wat::core::defn :wat::fix::defmacro? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch)
        false
        (:wat::core::let [head (:wat::core::first ch)]
          (:wat::core::if (:wat::core::= (:wat::core::ast-kind head) "keyword")
            (:wat::core::= (:wat::core::ast-name head) ":wat::core::defmacro")
            false))))
    false))

;; argspec-type-edits-walk — position-aware left-to-right walk of an argvec's children.
;; Tracks prev-arrow? (previous token was `<-`) and after-amp? (a `&` has been seen).
;; When prev-arrow? AND kind=="keyword" → type slot: emit a replacement edit.
;; The new-text depends on after-amp?: rest param → Vector<wat::WatAST>, fixed → WatAST.
(:wat::core::defn :wat::fix::argspec-type-edits-walk
  [items       <- :wat::core::Vector<wat::WatAST>
   prev-arrow? <- :wat::core::bool
   after-amp?  <- :wat::core::bool
   lines       <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::Vector<(wat::core::i64,wat::core::i64,wat::core::String)>
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String))
    (:wat::core::let [h  (:wat::core::first items)
                      tl (:wat::core::rest items)
                      ;; is this token a type-slot?
                      is-type-slot? (:wat::core::if prev-arrow?
                                      (:wat::core::= (:wat::core::ast-kind h) "keyword")
                                      false)
                      ;; emit one edit if it's a type-slot
                      head-edits (:wat::core::if is-type-slot?
                                   (:wat::core::let [span    (:wat::core::ast-span h)
                                                     off     (:wat::fix::fix-text-offset-of span lines)
                                                     old-len (:wat::core::string::length (:wat::core::ast-name h))
                                                     new-text (:wat::core::if after-amp?
                                                                 ":wat::core::Vector<wat::WatAST>"
                                                                 ":wat::WatAST")]
                                     (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String)
                                       (:wat::core::Tuple off old-len new-text)))
                                   (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String)))
                      ;; update after-amp?: set when current token is `&`
                      next-after-amp? (:wat::core::if (:wat::fix::amp? h) true after-amp?)
                      ;; update prev-arrow?: set when current token is `<-`
                      next-prev-arrow? (:wat::core::if (:wat::core::= (:wat::core::ast-kind h) "symbol")
                                         (:wat::core::= (:wat::core::ast-name h) "<-")
                                         false)]
      (:wat::core::concat head-edits
        (:wat::fix::argspec-type-edits-walk tl next-prev-arrow? next-after-amp? lines)))))

;; rettype-edit-walk — walk top-level defmacro form children looking for the return-type
;; keyword (the keyword immediately following the `->` symbol at this level, NOT inside
;; the argvec). Emits at most one replacement edit → `:wat::WatAST`.
(:wat::core::defn :wat::fix::rettype-edit-walk
  [items            <- :wat::core::Vector<wat::WatAST>
   prev-right-arrow? <- :wat::core::bool
   lines            <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::Vector<(wat::core::i64,wat::core::i64,wat::core::String)>
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String))
    (:wat::core::let [h  (:wat::core::first items)
                      tl (:wat::core::rest items)
                      ;; is this the return-type keyword slot?
                      is-rettype? (:wat::core::if prev-right-arrow?
                                    (:wat::core::= (:wat::core::ast-kind h) "keyword")
                                    false)]
      (:wat::core::if is-rettype?
        ;; emit the single rettype replacement edit; stop (return type consumed)
        (:wat::core::let [span    (:wat::core::ast-span h)
                          off     (:wat::fix::fix-text-offset-of span lines)
                          old-len (:wat::core::string::length (:wat::core::ast-name h))]
          (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String)
            (:wat::core::Tuple off old-len ":wat::WatAST")))
        ;; not yet — recurse tracking whether current token is `->`
        (:wat::fix::rettype-edit-walk tl (:wat::fix::right-arrow? h) lines)))))

;; defmacro-edits — collect all type-replacement edits for one defmacro form.
;; Detects 6-item (no metadata) vs 7-item (with metadata map at index 2) shapes.
;; argvec: ch[2] if kind=="vector", else ch[3] (metadata-map at ch[2]).
;; return type: first keyword after `->` in the form's top-level children.
(:wat::core::defn :wat::fix::defmacro-edits
  [form  <- :wat::WatAST
   lines <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::Vector<(wat::core::i64,wat::core::i64,wat::core::String)>
  (:wat::core::let [ch     (:wat::core::ast->children form)
                    ;; ch[2]: if it's a vector, argvec is here (6-item); else ch[3] (7-item)
                    c2     (:wat::core::first (:wat::core::drop ch 2))
                    argvec (:wat::core::if (:wat::core::= (:wat::core::ast-kind c2) "vector")
                              c2
                              (:wat::core::first (:wat::core::drop ch 3)))
                    argvec-children (:wat::core::ast->children argvec)
                    ;; collect argvec type edits
                    av-edits   (:wat::fix::argspec-type-edits-walk argvec-children false false lines)
                    ;; collect rettype edit — walk form's top-level children
                    ret-edits  (:wat::fix::rettype-edit-walk ch false lines)]
    (:wat::core::concat av-edits ret-edits)))

;; collect-defmacro-edits-deep — RECURSIVE: find defmacro forms at ANY depth (incl. ones
;; nested inside quasiquote templates — a defmacro-generating-defmacro like make-deftest,
;; whose generated macro's param types are lies sitting in the template). Quasiquote /
;; unquote desugar to Lists (no distinct ast-kind), and ast->children is TOTAL (atoms → []),
;; so the recursion descends through templates and bottoms out on leaves. Mutually recursive
;; with macro-param-edits (which maps this over a vector of children).
(:wat::core::defn :wat::fix::collect-defmacro-edits-deep
  [node  <- :wat::WatAST
   lines <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::Vector<(wat::core::i64,wat::core::i64,wat::core::String)>
  (:wat::core::let [here (:wat::core::if (:wat::fix::defmacro? node)
                           (:wat::fix::defmacro-edits node lines)
                           (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String)))]
    (:wat::core::concat here
      (:wat::fix::macro-param-edits (:wat::core::ast->children node) lines))))

;; macro-param-edits — map the deep collector over a vector of forms; concat all edits.
;; Each form is walked to ALL depths, so a defmacro nested in another macro's template is
;; found and fixed, not just top-level defmacros.
(:wat::core::defn :wat::fix::macro-param-edits
  [forms <- :wat::core::Vector<wat::WatAST>
   lines <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::Vector<(wat::core::i64,wat::core::i64,wat::core::String)>
  (:wat::core::if (:wat::core::empty? forms)
    (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String))
    (:wat::core::let [form (:wat::core::first forms)
                      rest-forms (:wat::core::rest forms)]
      (:wat::core::concat (:wat::fix::collect-defmacro-edits-deep form lines)
        (:wat::fix::macro-param-edits rest-forms lines)))))

;; fix-macro-param-types — comment-faithful defmacro param/return type migrator.
;; Parses src → collects type-replacement edits for defmacro forms only →
;; reverses to right-to-left → splices the ORIGINAL text via fix-text-apply.
;; Comments, formatting, and defn/fn real types survive byte-identical.
(:wat::core::defn :wat::fix::fix-macro-param-types
  [src <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::let [lines     (:wat::core::string::split src "\n")
                    tree      (:wat::core::read-string src)
                    forms     (:wat::core::ast->children tree)
                    all-edits (:wat::fix::macro-param-edits forms lines)
                    rev-edits (:wat::core::reverse all-edits)]
    (:wat::fix::fix-text-apply src rev-edits)))

;; ─── Stone 269 vehicle — rename-keyword-prefix: comment-faithful keyword PREFIX rename ───────
;;
;; rename-keyword-prefix(old-prefix new-prefix src) → migrated-src
;;
;; Arc 283.1: boundary-aware whole-name rewrite. For every keyword LEAF in src, rewrites
;; every VALID occurrence of old-bare (colon-stripped old-prefix) → new-bare within the
;; keyword name, emitting one whole-name edit when the name changes. Comments, formatting,
;; and non-matching keywords survive byte-identical. Structural nodes recurse into children.
;;
;; A match at index i in name is VALID iff:
;;   present:     subs(name,i,i+len(old-bare)) == old-bare
;;   left-valid:  (i==1 && char-at(name,0)==":") OR char-at(name,i-1) ∈ {"<",","," "}
;;   right-valid: at-end OR char-at(name,i+len(old-bare)) ∉ [a-zA-Z0-9_-]
;;
;; This subsumes the head case (:t::Old), type-arg case (Vector<t::Old>), and accessor
;; (:t::Old/make), while excluding prefix-siblings (:t::OldExtra) and unrelated symbols
;; ending in the path (:other::t::Old, preceded by ::, not a valid left-ctx).

;; rename-ident-char? — true if the single-char string c is an identifier-continuation char.
;; [a-zA-Z0-9_-] — right-INVALID chars that signal the match bleeds into a sibling name.
(:wat::core::defn :wat::fix::rename-ident-char? [c <- :wat::core::String] -> :wat::core::bool
  (:wat::core::string::contains? "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-" c))

;; rename-strip-colon — thin alias over :wat::core::string::strip-leading-colon.
;; ":t::Old" → "t::Old"; "t::Old" → "t::Old" (idempotent on bare strings).
;; Promoted to core in Arc 260.1b Part A; kept here so call sites at lines 722/723 are untouched.
(:wat::core::defn :wat::fix::rename-strip-colon [s <- :wat::core::String] -> :wat::core::String
  (:wat::core::string::strip-leading-colon s))

;; rename-valid-match? — true iff old-bare (colon-stripped prefix) matches at index i in name
;; with a valid left and right boundary.
;;   present:     subs(name,i,i+old-len) == old-bare
;;   left-valid:  (i==1 && char-at(name,0)==":") OR char-at(name,i-1) ∈ {"<",","," "}
;;   right-valid: i+old-len==len(name) OR char-at(name,i+old-len) ∉ ident-chars
(:wat::core::defn :wat::fix::rename-valid-match?
  [name     <- :wat::core::String
   i        <- :wat::core::i64
   old-bare <- :wat::core::String
   old-len  <- :wat::core::i64
   name-len <- :wat::core::i64]
  -> :wat::core::bool
  (:wat::core::let [end (:wat::core::+ i old-len)]
    (:wat::core::if (:wat::core::> end name-len)
      ;; not enough chars to match — absent
      false
      (:wat::core::if (:wat::core::= (:wat::core::string::subs name i end) old-bare)
        ;; present — check left-valid
        (:wat::core::let [left-ok (:wat::core::if (:wat::core::= i 1)
                                    ;; head case: i==1 and name[0]==":"
                                    (:wat::core::= (:wat::core::string::subs name 0 1) ":")
                                    ;; type-arg case: preceded by "<", ",", or " "
                                    (:wat::core::if (:wat::core::< i 1)
                                      false
                                      (:wat::core::let [prev (:wat::core::string::subs name (:wat::core::- i 1) i)]
                                        (:wat::core::if (:wat::core::= prev "<") true
                                          (:wat::core::if (:wat::core::= prev ",") true
                                            (:wat::core::if (:wat::core::= prev " ") true
                                              ;; "(" — the tuple-type opener `:(A,B,C)`; its FIRST element is a
                                              ;; boundary-valid embedded name, like "<" for a parametric arg.
                                              (:wat::core::= prev "(")))))))]
          (:wat::core::if left-ok
            ;; check right-valid: at-end or not an ident char
            (:wat::core::if (:wat::core::= end name-len)
              true
              (:wat::core::not (:wat::fix::rename-ident-char? (:wat::core::string::subs name end (:wat::core::+ end 1)))))
            false))
        ;; substr doesn't match — absent
        false))))

;; rename-in-name — char-walk that rewrites every valid occurrence of old-bare → new-bare.
;; Returns the fully rewritten name string. If no occurrences are valid, returns name unchanged.
;; i is the current index; acc accumulates the output. Tail-recursive.
(:wat::core::defn :wat::fix::rename-in-name
  [name     <- :wat::core::String
   old-bare <- :wat::core::String
   new-bare <- :wat::core::String
   old-len  <- :wat::core::i64
   name-len <- :wat::core::i64
   i        <- :wat::core::i64
   acc      <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::if (:wat::core::>= i name-len)
    acc
    (:wat::core::if (:wat::fix::rename-valid-match? name i old-bare old-len name-len)
      ;; valid match — emit new-bare, advance by old-len
      (:wat::fix::rename-in-name name old-bare new-bare old-len name-len
        (:wat::core::+ i old-len)
        (:wat::core::string::concat acc new-bare))
      ;; no match — emit one char, advance by 1
      (:wat::fix::rename-in-name name old-bare new-bare old-len name-len
        (:wat::core::+ i 1)
        (:wat::core::string::concat acc (:wat::core::string::subs name i (:wat::core::+ i 1)))))))

;; rename-prefix-edits-walk — walk a vector of nodes, concating prefix-swap edits.
;; Internal helper mirroring macro-param-edits; not a public API.
(:wat::core::defn :wat::fix::rename-prefix-edits-walk
  [items      <- :wat::core::Vector<wat::WatAST>
   old-prefix <- :wat::core::String
   new-prefix <- :wat::core::String
   lines      <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::Vector<(wat::core::i64,wat::core::i64,wat::core::String)>
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String))
    (:wat::core::let [h  (:wat::core::first items)
                      tl (:wat::core::rest items)]
      (:wat::core::concat
        (:wat::fix::rename-prefix-edits h old-prefix new-prefix lines)
        (:wat::fix::rename-prefix-edits-walk tl old-prefix new-prefix lines)))))

;; rename-prefix-edits — boundary-aware whole-name rewrite: for every keyword leaf, compute
;; new-name by rewriting every valid occurrence of old-bare → new-bare within the name;
;; if new-name != name, emit (off, length(name), new-name). Structural nodes recurse.
;; structural? dispatch: (structural? node) = list/vector/map/set (fix.wat:23).
(:wat::core::defn :wat::fix::rename-prefix-edits
  [node       <- :wat::WatAST
   old-prefix <- :wat::core::String
   new-prefix <- :wat::core::String
   lines      <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::Vector<(wat::core::i64,wat::core::i64,wat::core::String)>
  (:wat::core::if (:wat::fix::structural? node)
    ;; structural: recurse into children
    (:wat::fix::rename-prefix-edits-walk (:wat::core::ast->children node) old-prefix new-prefix lines)
    ;; leaf: keyword → boundary-aware whole-name rewrite
    (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "keyword")
      (:wat::core::let [name     (:wat::core::ast-name node)
                        old-bare (:wat::fix::rename-strip-colon old-prefix)
                        new-bare (:wat::fix::rename-strip-colon new-prefix)
                        old-len  (:wat::core::string::length old-bare)
                        name-len (:wat::core::string::length name)
                        new-name (:wat::fix::rename-in-name name old-bare new-bare old-len name-len 0 "")]
        (:wat::core::if (:wat::core::= new-name name)
          ;; no change — no edit
          (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String))
          ;; changed — emit whole-token replace edit
          (:wat::core::let [off (:wat::fix::fix-text-offset-of (:wat::core::ast-span node) lines)]
            (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String)
              (:wat::core::Tuple off name-len new-name)))))
      ;; non-keyword leaf (symbol, int, float, bool, string, nil) — no edit
      (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String)))))

;; rename-keyword-prefix — comment-faithful keyword PREFIX rename rule.
;; Parses src → collects prefix-swap edits for every matching keyword leaf →
;; reverses to right-to-left → splices the ORIGINAL text via fix-text-apply.
;; Comments, formatting, and non-matching keywords survive byte-identical.
(:wat::core::defn :wat::fix::rename-keyword-prefix
  [old-prefix <- :wat::core::String
   new-prefix <- :wat::core::String
   src        <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::let [lines     (:wat::core::string::split src "\n")
                    tree      (:wat::core::read-string src)
                    forms     (:wat::core::ast->children tree)
                    all-edits (:wat::fix::rename-prefix-edits-walk forms old-prefix new-prefix lines)
                    rev-edits (:wat::core::reverse all-edits)]
    (:wat::fix::fix-text-apply src rev-edits)))

;; ── rename-keyword-exact — WHOLE-TOKEN keyword rename (the idempotent sibling of
;; rename-keyword-prefix). Renames a keyword leaf ONLY when its FULL name EQUALS `old`
;; (not a prefix/substring, no boundary walk). This is the correct tool for an APPEND rename
;; (e.g. `:t::deftest` -> `:t::deftest'`): after the rewrite the whole token is `:t::deftest'`,
;; which is != `:t::deftest`, so a re-run matches nothing — IDEMPOTENT BY CONSTRUCTION.
;; (rename-keyword-prefix treats `'` as a valid right-boundary, so it would re-match the
;; `deftest` prefix inside `deftest'` and produce `deftest''` — non-idempotent for appends.)
;; Comments/formatting survive (rides fix-text-apply's span-splice); non-matching keywords and
;; prefix-siblings (`:t::deftest-hermetic`) are untouched (exact whole-name equality). Use
;; rename-keyword-prefix for a boundary-aware prefix swap; use this for an exact whole-name rename.
(:wat::core::defn :wat::fix::rename-exact-edits-walk
  [items <- :wat::core::Vector<wat::WatAST>
   old   <- :wat::core::String
   new   <- :wat::core::String
   lines <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::Vector<(wat::core::i64,wat::core::i64,wat::core::String)>
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String))
    (:wat::core::let [h  (:wat::core::first items)
                      tl (:wat::core::rest items)]
      (:wat::core::concat
        (:wat::fix::rename-exact-edits h old new lines)
        (:wat::fix::rename-exact-edits-walk tl old new lines)))))

;; rename-exact-edits — for a keyword leaf whose full name EQUALS old, emit one whole-token
;; replace edit (off, length(old), new). Structural nodes recurse; every other leaf: no edit.
(:wat::core::defn :wat::fix::rename-exact-edits
  [node  <- :wat::WatAST
   old   <- :wat::core::String
   new   <- :wat::core::String
   lines <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::Vector<(wat::core::i64,wat::core::i64,wat::core::String)>
  (:wat::core::if (:wat::fix::structural? node)
    (:wat::fix::rename-exact-edits-walk (:wat::core::ast->children node) old new lines)
    (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "keyword")
      (:wat::core::if (:wat::core::= (:wat::core::ast-name node) old)
        (:wat::core::let [off (:wat::fix::fix-text-offset-of (:wat::core::ast-span node) lines)]
          (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String)
            (:wat::core::Tuple off (:wat::core::string::length old) new)))
        (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String)))
      (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String)))))

(:wat::core::defn :wat::fix::rename-keyword-exact
  [old <- :wat::core::String
   new <- :wat::core::String
   src <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::let [lines     (:wat::core::string::split src "\n")
                    tree      (:wat::core::read-string src)
                    forms     (:wat::core::ast->children tree)
                    all-edits (:wat::fix::rename-exact-edits-walk forms old new lines)
                    rev-edits (:wat::core::reverse all-edits)]
    (:wat::fix::fix-text-apply src rev-edits)))
