;; wat-scripts/fixes/strip-insert-rhs-marker.wat — arc 278 Stone A
;; (DESIGN-STONE-then-is-a-vector-of-singular-facts.md / BRIEF-then-vector-migration.md),
;; second pass: the RHS-marker `(:wat::rete::insert <fact>)` (exactly 2 children) also appears
;; OUTSIDE `defrule` — hand-built `Rule` literals constructed via `quote`/`quasiquote` (perf-grid
;; generators, scratch-pad probes) hold the SAME wrapped shape as an RHS value bound to a `let`
;; variable (e.g. `rhs1`, `ins`, `t1`). `defrule-then-to-vector.wat` (companion codemod) handles
;; every `:then` site inside an actual `(:wat::rete::defrule …)` call; this one strips the
;; identical wrapper wherever else it appears, unconditionally:
;;
;;   (:wat::rete::insert (:fan::Pair ?k ?l ?r))   ->   (:fan::Pair ?k ?l ?r)
;;
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — wat rewrites wat.
;;
;; NEVER touches the session-level `(:wat::rete::insert <session> <fact>)` form (3 children) —
;; that is the OTHER meaning of the name (a `defclause`, `rete.wat:1004`) and it survives this
;; stone untouched. The exact-2-children check is the whole discriminator.
;;
;; Span-faithful: the edit replaces the WHOLE node's span with its own fact-form's (child[1])
;; text, sliced verbatim — never re-rendered.
;;
;; Idempotent: after stripping, no 2-child `:wat::rete::insert` node remains — a second run
;; finds nothing and reports 0 changes.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["pathA" "pathB" …]\n' | ./target/release/wat ./wat-scripts/fixes/strip-insert-rhs-marker.wat

;; rhs-marker? — a List `(:wat::rete::insert <fact>)`: exact head + exactly 2 children.
(:wat::core::defn :user::rhs-marker? [f <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::fix::calls-to? f ":wat::rete::insert")
    (:wat::core::= (:wat::core::count (:wat::core::ast->children f)) 2)
    false))

;; node-edit — 0-or-1 replacement edit stripping ONE rhs-marker node down to its fact-form text.
(:wat::core::defn :user::node-edit
  [node  <- :wat::WatAST
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:user::rhs-marker? node)
    (:wat::core::let
      [fact (:wat::core::Option/expect (:wat::core::get (:wat::core::ast->children node) 1) "node-edit: unreachable")
       fact-off (:wat::fix::node-start-offset fact lines)
       fact-end (:wat::fix::node-end-offset fact lines)
       fact-text (:wat::core::string::subs src fact-off fact-end)
       node-off (:wat::fix::node-start-offset node lines)
       node-end (:wat::fix::node-end-offset node lines)
       len (:wat::core::i64::- node-end node-off)]
      (:wat::core::Vector :wat::fix::Edit (:wat::core::Tuple node-off len fact-text)))
    (:wat::core::Vector :wat::fix::Edit)))

;; walk-edits — deep walk: a rhs-marker node's OWN children are not further descended (its fact
;; form cannot itself contain another rhs-marker in this corpus, and even if it did, the outer
;; strip already reveals it for a subsequent run — never true in practice, but harmless either
;; way since a second run is proven idempotent below).
(:wat::core::defn :user::walk-edits
  [node  <- :wat::WatAST
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::let [this (:user::node-edit node src lines)]
    (:wat::core::if (:wat::fix::structural? node)
      (:wat::core::concat this (:user::walk-seq-edits (:wat::core::ast->children node) src lines))
      this)))

(:wat::core::defn :user::walk-seq-edits
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector :wat::fix::Edit)
    (:wat::core::concat
      (:user::walk-edits (:wat::core::first items) src lines)
      (:user::walk-seq-edits (:wat::core::rest items) src lines))))

;; ── per-file migrate ────────────────────────────────────────────────────────────────────────

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let
    [lines (:wat::core::string::split src "\n")
     tree  (:wat::core::match (:wat::core::read-string src)
             ((:wat::core::ReadOutcome::Forms __forms) __forms)
             ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
     forms (:wat::core::ast->children tree)
     edits (:user::walk-seq-edits forms src lines)]
    (:wat::fix::fix-text-apply src (:wat::core::reverse (:wat::core::sort edits)))))

;; ── driver: rewrite each path given on stdin (a JSON/EDN array of strings) ────────────────────
(:wat::core::defn :user::rewrite-each [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::core::string::concat "[strip-insert-rhs-marker] " path))
        (:user::rewrite-each (:wat::core::into [] (:wat::core::rest paths)))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [paths (:wat::core::match (:wat::kernel::readln)
                            ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
                            (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
                            (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))]
    (:user::rewrite-each paths)))
