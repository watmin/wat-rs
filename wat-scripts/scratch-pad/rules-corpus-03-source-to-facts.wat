;; rules-corpus 03 — REAL SOURCE BECOMES FACTS
;;
;; Corpus 01 proved AST nodes CAN be facts. Corpus 02 proved the gate/unlock chain reasons over
;; them. Both seeded facts BY HAND. This is the load-bearing unknown between those probes and an
;; actual migration: can a real `.wat` file on disk be turned into the fact base the rules read?
;;
;; If this fails, the whole rules-ification is a toy. So it is measured here, on real files,
;; before anything else is built on top of it.
;;
;; ─── THE ONE DESIGN DECISION ─────────────────────────────────────────────────
;; Node identity is assigned by a PRE-ORDER walk with a threaded counter. Nothing about the
;; source carries an id, and the rules need one to join on (corpus 01's `Node`/`Named` split and
;; corpus 02's whole chain are joins on `?id`). Pre-order is chosen because it makes `parent`
;; always ALREADY ASSIGNED when a child is numbered — a post-order walk would force a second
;; pass to back-fill parents.
;;
;; ─── AND THE GUARD THAT CORPUS 01 PREDICTED, NOW LOAD-BEARING FOR REAL ───────
;; `ast-name` is PARTIAL — it raises on any node that is not Symbol/Keyword/StringLit. That is
;; the exact defect that killed the hand-rolled codemod on 43 of 1392 corpus files (it guarded on
;; ARITY and then called `ast-name` anyway). Here the guard is structural instead: a `Named` fact
;; is emitted ONLY for a nameable kind, so an unnameable node simply HAS NO NAME FACT and every
;; downstream rule that joins `Named` cannot see it. The absence IS the guard (corpus 01 L1).
;;
;; ★ NON-VACUITY: `Node` must exceed `Named` on any real file. Every node gets a `Node`; only
;; nameable ones get a `Named`. If the two counts were EQUAL the guard would be doing nothing and
;; every reading below would be meaningless.

(:wat::core::defrecord :fx::Node
  [id     <- :wat::core::i64
   parent <- :wat::core::i64
   index  <- :wat::core::i64
   kind   <- :wat::core::String])

(:wat::core::defrecord :fx::Named
  [id   <- :wat::core::i64
   name <- :wat::core::String])

;; the walk's accumulator — threaded, never mutated
(:wat::core::defrecord :fx::Acc
  [next-id <- :wat::core::i64
   nodes   <- (:wat::core::PersistentVector :- [:fx::Node])
   named   <- (:wat::core::PersistentVector :- [:fx::Named])])

;; per-level child accumulator: the walk's Acc plus this level's running index
(:wat::core::defrecord :fx::ChildAcc
  [acc <- :fx::Acc
   idx <- :wat::core::i64])

;; nameable? — the TOTAL guard in front of the partial `ast-name`.
(:wat::core::defn :fx::nameable? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::let [k (:wat::core::ast-kind node)]
    (:wat::core::if (:wat::core::= k "symbol") true
      (:wat::core::if (:wat::core::= k "keyword") true
        (:wat::core::= k "string")))))

;; structural? — does this node HAVE children to descend into?
(:wat::core::defn :fx::structural? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::let [k (:wat::core::ast-kind node)]
    (:wat::core::contains?
      (:wat::core::HashSet :wat::type::Infer "list" "vector" "map" "set") k)))

;; walk — assign this node an id, emit its facts, then descend. Pre-order, so `parent` is always
;; already numbered when a child is reached.
(:wat::core::defn :fx::walk
  [acc    <- :fx::Acc
   node   <- :wat::WatAST
   parent <- :wat::core::i64
   index  <- :wat::core::i64]
  -> :fx::Acc
  (:wat::core::let
    [id    (:fx::Acc/next-id acc)
     kind  (:wat::core::ast-kind node)
     nodes (:wat::core::PersistentVector/conj (:fx::Acc/nodes acc)
             (:fx::Node :id id :parent parent :index index :kind kind))
     ;; THE GUARD: no name fact for an unnameable node. `ast-name` is never reached for one.
     named (:wat::core::if (:fx::nameable? node)
             (:wat::core::PersistentVector/conj (:fx::Acc/named acc)
               (:fx::Named :id id :name (:wat::core::ast-name node)))
             (:fx::Acc/named acc))
     acc'  (:fx::Acc :next-id (:wat::core::i64::+ id 1) :nodes nodes :named named)]
    (:wat::core::if (:fx::structural? node)
      (:fx::ChildAcc/acc
        (:wat::core::foldl
          (:wat::core::fn [ca <- :fx::ChildAcc  child <- :wat::WatAST] -> :fx::ChildAcc
            (:fx::ChildAcc
              :acc (:fx::walk (:fx::ChildAcc/acc ca) child id (:fx::ChildAcc/idx ca))
              :idx (:wat::core::i64::+ (:fx::ChildAcc/idx ca) 1)))
          (:fx::ChildAcc :acc acc' :idx 0)
          (:wat::core::ast->children node)))
      acc')))

;; extract — every top-level form of one source file, walked into one fact base.
;;
;; `read-string` returns a FACED OUTCOME, not a bare vector — the no-hidden-failures law. A file
;; that will not parse is a RESULT the extractor carries, never a crash: in the one-shot that is
;; a `.wat.bad` negative fixture or a genuinely broken file, and the migration needs to know
;; WHICH file rather than die on it. Malformed yields an EMPTY fact base, which every downstream
;; rule reads as "nothing to say about this file" — the honest answer.
(:wat::core::defn :fx::empty-acc [] -> :fx::Acc
  (:fx::Acc :next-id 1
            :nodes (:wat::core::PersistentVector)
            :named (:wat::core::PersistentVector)))

(:wat::core::defn :fx::extract [src <- :wat::core::String] -> :fx::Acc
  (:wat::core::match (:wat::core::read-string src)
    ((:wat::core::ReadOutcome::Forms forms)
      (:fx::ChildAcc/acc
        (:wat::core::foldl
          (:wat::core::fn [ca <- :fx::ChildAcc  form <- :wat::WatAST] -> :fx::ChildAcc
            (:fx::ChildAcc
              :acc (:fx::walk (:fx::ChildAcc/acc ca) form 0 (:fx::ChildAcc/idx ca))
              :idx (:wat::core::i64::+ (:fx::ChildAcc/idx ca) 1)))
          (:fx::ChildAcc :acc (:fx::empty-acc) :idx 0)
          (:wat::core::ast->children forms))))
    ((:wat::core::ReadOutcome::Malformed __cause) (:fx::empty-acc))))

(:wat::core::defn :fx::report [path <- :wat::core::String] -> :wat::core::nil
  (:wat::core::let [acc (:fx::extract (:wat::io::read-file path))
                    n   (:wat::core::length (:fx::Acc/nodes acc))
                    m   (:wat::core::length (:fx::Acc/named acc))]
    (:wat::kernel::println
      (:wat::core::string::concat path
        (:wat::core::string::concat "  Node=" (:wat::core::str n)
          (:wat::core::string::concat "  Named=" (:wat::core::str m)))))))


;; ─── ★ THE JOIN: the extractor's facts FEED THE RULES, on real source ────────
;; Corpus 01's three verdicts, now over a file from disk instead of five hand-written facts.
;; Nothing about the rules changed to accept real input — that is the point. The rules never
;; knew the facts were hand-made, and they do not know now that they are not.
(:wat::core::defrecord :fx::IsArrow   [id <- :wat::core::i64])
(:wat::core::defrecord :fx::IsHeadKw  [id <- :wat::core::i64])
(:wat::core::defrecord :fx::IsTypePos [id <- :wat::core::i64])

(:wat::rete::defrule :fx::arrow
  :when [(:fx::Node  (?id <- :id) (?k <- :kind))
         (:fx::Named (?id <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::core::string::= ?k "symbol"))
         (:wat::rete::where (:wat::rete::core::string::= ?n "<-"))]
  :then [(:fx::IsArrow :id ?id)])

(:wat::rete::defrule :fx::head-kw
  :when [(:fx::Node  (?id <- :id) (?k <- :kind))
         (:fx::Named (?id <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::core::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::core::String/contains? ?n "::"))]
  :then [(:fx::IsHeadKw :id ?id)])

;; the prev-sibling JOIN that replaces fix-seq's single carried boolean — over real source now
(:wat::rete::defrule :fx::type-pos
  :when [(:fx::Node    (?id <- :id)  (?p <- :parent) (?i <- :index))
         (:fx::Node    (?aid <- :id) (?p <- :parent) (?ai <- :index))
         (:fx::IsArrow (?aid <- :id))
         (:wat::rete::where
           (:wat::rete::core::i64::= ?i (:wat::rete::core::i64::+ ?ai 1 :undefined 0)))]
  :then [(:fx::IsTypePos :id ?id)])

(:wat::rete::defquery :fx::q-IsArrow
  :params []
  :when [(?fact <- :fx::IsArrow)])


(:wat::rete::defquery :fx::q-IsHeadKw
  :params []
  :when [(?fact <- :fx::IsHeadKw)])


(:wat::rete::defquery :fx::q-IsTypePos
  :params []
  :when [(?fact <- :fx::IsTypePos)])


(:wat::core::defn :fx::classify [path <- :wat::core::String] -> :wat::core::nil
  (:wat::core::let
    [acc   (:fx::extract (:wat::io::read-file path))
     rules (:wat::core::PersistentVector (:fx::arrow) (:fx::head-kw) (:fx::type-pos))
     s0    (:wat::rete::insert-all (:wat::rete::compile-all rules (:wat::core::PersistentVector (:fx::q-IsArrow) (:fx::q-IsHeadKw) (:fx::q-IsTypePos))) (:fx::Acc/nodes acc))
     fired (:wat::rete::fire-rules (:wat::rete::insert-all s0 (:fx::Acc/named acc)))]
    (:wat::kernel::println
      (:wat::core::string::concat path
        (:wat::core::string::concat "  arrows=" (:wat::core::str (:wat::core::length (:wat::rete::query fired (:fx::q-IsArrow))))
          (:wat::core::string::concat "  head-kw=" (:wat::core::str (:wat::core::length (:wat::rete::query fired (:fx::q-IsHeadKw))))
            (:wat::core::string::concat "  type-pos=" (:wat::core::str (:wat::core::length (:wat::rete::query fired (:fx::q-IsTypePos)))))))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    ;; Real files, deliberately spanning shapes: the codemod itself, a rules file, and a probe
    ;; whose macro forms carry the reader-macro sigils that CORRUPTED the text-edit engine.
    (:fx::report "wat/fix.wat")
    (:fx::report "wat-scripts/perf/grid/neg-consumer.wat")
    (:fx::report "tests/macros/probe_do_splice_define_via_macro.wat")
    ;; ★ and the rules, reading those same files
    (:fx::classify "wat-scripts/perf/grid/neg-consumer.wat")
    (:fx::classify "tests/macros/probe_do_splice_define_via_macro.wat")))
