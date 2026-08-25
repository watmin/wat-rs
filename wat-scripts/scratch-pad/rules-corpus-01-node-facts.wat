;; rules-corpus 01 — CAN RETE CLASSIFY AST NODES AT ALL?
;;
;; THE DISCONFIRMING PROBE for the rules-ification of `wat/fix.wat`. Everything in the
;; corpus rests on one unproven claim: that a form tree can be asserted as FACTS and the
;; migration's classification decisions expressed as RULES over them. If that is false,
;; the whole plan is void and we learn it here, in twenty lines, not in a strike.
;;
;; ─── THE TWO MOVES THIS PROBE EXISTS TO PROVE ────────────────────────────────
;;
;; MOVE 1 — A MISSING FACT IS THE GUARD.
;;   Today `annotated-if?` (fix.wat:63) guards on ARITY (">= 3 children") and then calls
;;   `ast-name` on the head. Nothing forces those to agree, so a list with 3+ children whose
;;   head is NOT nameable kills the codemod. MEASURED: 43 of 1392 corpus files, this exact root.
;;   In rules there is no such gap: a node that has no name gets no `Named` fact, so a rule
;;   that joins on `Named` CANNOT see it. The precondition IS the match.
;;
;; MOVE 2 — POSITION IS A JOIN, NOT A CARRIED BIT.
;;   Today `fix-seq` (fix.wat:119) walks a child vector carrying ONE boolean, `prev-arrow?`.
;;   That is the entire memory the classifier has. Any decision needing more than "was the
;;   previous token an arrow" is inexpressible, which is why the four rules are the four
;;   rules. Here the previous sibling is a JOIN on (same parent, index - 1) — and once it is
;;   a join, next-sibling, grandparent, and head-of-parent are the same move for free.
;;
;; ─── THE FACT SHAPE (the load-bearing design choice) ─────────────────────────
;;
;;   Node  — EVERY node: identity, tree position, kind.
;;   Named — ONLY nodes that have a name. This is MOVE 1 made structural.
;;
;; Note what is NOT here: the WatAST itself. The classifier never needs the node, only its
;; attributes — every decision in fix.wat's chain (head-keyword?, arrow?, type-shaped-keyword?,
;; annotated-if?) reads kind, name, and position and nothing else.

(:wat::core::defrecord :fixr::Node
  [id     <- :wat::core::i64
   parent <- :wat::core::i64
   index  <- :wat::core::i64
   kind   <- :wat::core::String])

(:wat::core::defrecord :fixr::Named
  [id   <- :wat::core::i64
   name <- :wat::core::String])

;; ─── the classification VERDICTS (derived facts, one per decision) ───────────
(:wat::core::defrecord :fixr::IsArrow    [id <- :wat::core::i64])
(:wat::core::defrecord :fixr::IsHeadKw   [id <- :wat::core::i64])
(:wat::core::defrecord :fixr::IsTypePos  [id <- :wat::core::i64])

;; ─── LAW A (#57): a `where` admits ONLY `:wat::rete::` primitives ────────────
;; Not `:wat::core::=`, even though it is pure AND deterministic AND total. The rete query
;; language is a CLOSED vocabulary, and the fence names the exact axis that failed:
;;   "':wat::core::=' is not a rete primitive; a where admits only :wat::rete:: ops"
;; This probe was written against `:wat::core::` first and the fence taught it in one run.
;; ⚠ `wat-scripts/scratch-pad/probe-rules-rich.wat` still uses `:wat::core::>` in a `where`
;;   and is therefore DEAD ON RUN — law A fires at rule-COMPILE, which the loader gate
;;   (parse + type-check only) structurally cannot see. That is task #85's class, still live.

;; RULE A — the arrow. Joins Node x Named on id: a node with NO name never reaches this rule.
(:wat::rete::defrule :fixr::arrow
  :when [(:fixr::Node  (?id <- :id) (?k <- :kind))
         (:fixr::Named (?id <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?k "symbol"))
         (:wat::rete::where (:wat::rete::string::= ?n "<-"))]
  :then [(:fixr::IsArrow :id ?id)])

;; RULE B — the ::-namespaced call head / reference keyword.
(:wat::rete::defrule :fixr::head-kw
  :when [(:fixr::Node  (?id <- :id) (?k <- :kind))
         (:fixr::Named (?id <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::core::String/contains? ?n "::"))]
  :then [(:fixr::IsHeadKw :id ?id)])

;; RULE C — ★ THE ONE THAT REPLACES `prev-arrow?`.
;; "a node whose PREVIOUS SIBLING is an arrow is in type position" — expressed as a JOIN on
;; (same parent, index - 1), never as carried state. The arrow's own verdict (IsArrow) is the
;; join partner, so this rule stands on RULE A's conclusion — the forward chain.
(:wat::rete::defrule :fixr::type-pos
  :when [(:fixr::Node    (?id <- :id)  (?p <- :parent) (?i <- :index))
         (:fixr::Node    (?aid <- :id) (?p <- :parent) (?ai <- :index))
         (:fixr::IsArrow (?aid <- :id))
         ;; ★ TOTALITY IS STRUCTURAL: `i64::+` can overflow, so the rete spelling is 4-ary —
         ;; `(+ a b :undefined <fallback>)`. The literal keyword `:undefined` is mandatory and the
         ;; caller MUST name the value the undefined case yields. There is no jump-table opcode for
         ;; "raises", so a partial op simply has no form here (#80: every rete row must be TOTAL).
         (:wat::rete::where (:wat::rete::core::i64::= ?i (:wat::rete::core::i64::+ ?ai 1 :undefined 0)))]
  :then [(:fixr::IsTypePos :id ?id)])

(:wat::rete::defquery :fixr::q-IsArrow
  :params []
  :when [(?fact <- :fixr::IsArrow)])


(:wat::rete::defquery :fixr::q-IsHeadKw
  :params []
  :when [(?fact <- :fixr::IsHeadKw)])


(:wat::rete::defquery :fixr::q-IsTypePos
  :params []
  :when [(?fact <- :fixr::IsTypePos)])


;; ─── the driver — built as a DIFFERENTIAL, because a bare pass proves nothing ─
;;
;; Nodes 1-3 model `[body <- :wat::WatAST]`:
;;   id 1 `body`         symbol,  index 0  -> nothing
;;   id 2 `<-`           symbol,  index 1  -> IsArrow
;;   id 3 `:wat::WatAST` keyword, index 2  -> IsHeadKw AND IsTypePos (prev sibling is the arrow)
;;
;; ★ THE TWO ARMS THAT PROVE MOVE 1 — identical in every respect the rule tests, differing
;;   ONLY in whether a `Named` fact exists. A kind check alone cannot tell them apart; the
;;   join can, and must:
;;   id 4  keyword, name ":wat::core::foo"  WITHHELD from Named  -> must NOT classify
;;   id 5  keyword, name ":wat::core::foo"  PRESENT  in Named    -> MUST     classify
;;   So IsHeadKw must read exactly 2 (ids 3 and 5). A 3 means the guard leaks; a 1 means the
;;   positive arm is broken and the "no leak" reading would have been vacuous.

(:wat::core::defn :fixr::seed [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::rete::insert-all s
    (:wat::core::PersistentVector
      (:fixr::Node :id 1 :parent 0 :index 0 :kind "symbol")
      (:fixr::Node :id 2 :parent 0 :index 1 :kind "symbol")
      (:fixr::Node :id 3 :parent 0 :index 2 :kind "keyword")
      (:fixr::Node :id 4 :parent 9 :index 0 :kind "keyword")
      (:fixr::Node :id 5 :parent 9 :index 1 :kind "keyword"))))

(:wat::core::defn :fixr::seed-names [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::rete::insert-all s
    (:wat::core::PersistentVector
      (:fixr::Named :id 1 :name "body")
      (:fixr::Named :id 2 :name "<-")
      (:fixr::Named :id 3 :name ":wat::WatAST")
      ;; id 4 deliberately ABSENT — the unnameable head. This is the whole point.
      (:fixr::Named :id 5 :name ":wat::core::foo"))))

(:wat::core::defn :fixr::show [label <- :wat::core::String n <- :wat::core::i64] -> :wat::core::nil
  (:wat::kernel::println (:wat::string::concat label (:wat::core::str n))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [rules    (:wat::core::PersistentVector (:fixr::arrow) (:fixr::head-kw) (:fixr::type-pos))
     template (:wat::rete::compile-all rules (:wat::core::PersistentVector (:fixr::q-IsArrow) (:fixr::q-IsHeadKw) (:fixr::q-IsTypePos)))
     fired    (:wat::rete::fire-rules (:fixr::seed-names (:fixr::seed template)))]
    (:wat::core::do
      ;; ⚠ `query` reads accumulated PRODUCTION memory, so it can only see DERIVED facts —
      ;; querying a base type (Node/Named) returns 0 even when the seed landed. That is R18's
      ;; query-artifact, and it means the non-vacuity guard must be a DERIVED count, never a
      ;; base-fact count. IsArrow is that guard: it is 0 if the seed never landed.
      (:fixr::show "IsArrow   (want 1; 0 => seed never landed, all below vacuous): "
        (:wat::core::length (:wat::rete::query fired (:fixr::q-IsArrow))))
      (:fixr::show "IsHeadKw  (want 2 = ids 3,5; 3 => the Named guard LEAKS): "
        (:wat::core::length (:wat::rete::query fired (:fixr::q-IsHeadKw))))
      (:fixr::show "IsTypePos (want 1 = id 3, the prev-sibling JOIN): "
        (:wat::core::length (:wat::rete::query fired (:fixr::q-IsTypePos)))))))
