;; wat-scripts/fixes/defrule-then-to-vector.wat — arc 278 Stone A
;; (DESIGN-STONE-then-is-a-vector-of-singular-facts.md / BRIEF-then-vector-migration.md).
;;
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — wat rewrites wat.
;;
;; Rewrites every `(:wat::rete::defrule name :when [...] :then <insert1> <insert2> …)` call
;; form's `:then` payload from spliced varargs of `(:wat::rete::insert <fact>)` wrappers to a
;; single vector of bare fact-forms:
;;
;;   :then (:wat::rete::insert (:wmv::Hit ?k))          ->   :then [(:wmv::Hit ?k)]
;;   :then (:wat::rete::insert (:a ?x)) (:wat::rete::insert (:b ?y))
;;                                                       ->   :then [(:a ?x) (:b ?y)]
;;
;; Span-faithful: the edit covers exactly [start-of-first-then-form .. end-of-last-then-form),
;; replaced with `[` + each fact-form's OWN source text (unchanged) joined by a single space +
;; `]`. Per BRIEF-then-vector-migration.md this is the specified shape — it does not preserve
;; comments/whitespace BETWEEN then-forms (there are none in the corpus; verified by dry-run).
;;
;; defrule forms are found by a DEEP walk (not just top-level): some corpus sites pass raw
;; `(defrule …)` forms as data into a local macro (e.g. scratch-pad/probe-rule-lits.wat's
;; `:probe::mk-deduce [rule1 rule2]`), so the call sites are nested inside a Vector argument,
;; not top-level.
;;
;; STOP-2 (BRIEF-then-vector-migration.md): if a `:then` entry is not a plain
;; `(:wat::rete::insert <fact>)` list, this codemod halts with an assertion naming the rule —
;; a surprise there means the corpus disagrees with the stone's ruling that every RHS member is
;; a singular fact, and that must be reported, not silently routed around.
;;
;; Idempotent: a defrule whose :then payload is ALREADY a single Vector node (the post-migration
;; shape) is left untouched — a second run reports 0 changes.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["pathA" "pathB" …]\n' | ./target/release/wat ./wat-scripts/fixes/defrule-then-to-vector.wat

;; ── predicates ───────────────────────────────────────────────────────────────────────────────

;; defrule-form? — a List whose head keyword is EXACTLY :wat::rete::defrule.
(:wat::core::defn :user::defrule-form? [f <- :wat::WatAST] -> :wat::core::bool
  (:wat::fix::calls-to? f ":wat::rete::defrule"))

;; insert-wrapped? — a List `(:wat::rete::insert <fact>)`: exact head + exactly 2 children.
(:wat::core::defn :user::insert-wrapped? [f <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::fix::calls-to? f ":wat::rete::insert")
    (:wat::core::= (:wat::core::count (:wat::core::ast->children f)) 2)
    false))

;; all-insert-wrapped? — every then-form is insert-wrapped (STOP-2's check).
(:wat::core::defn :user::all-insert-wrapped? [forms <- (:wat::core::Vector :- [:wat::WatAST])] -> :wat::core::bool
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::bool f <- :wat::WatAST] -> :wat::core::bool
      (:wat::core::if acc (:user::insert-wrapped? f) false))
    true
    forms))

;; then-already-vector? — the post-:then payload is a single Vector node (post-migration shape).
(:wat::core::defn :user::then-already-vector?
  [then-forms <- (:wat::core::Vector :- [:wat::WatAST])] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::count then-forms) 1)
    (:wat::core::=
      (:wat::core::ast-kind (:wat::core::Option/expect (:wat::core::get then-forms 0) "then-already-vector?: unreachable"))
      "vector")
    false))

;; ── text extraction ─────────────────────────────────────────────────────────────────────────

;; fact-text — the source text of an insert-wrapped then-form's OWN fact-form (child[1]),
;; sliced verbatim from `src` by span — never re-rendered, so field order/spacing/literal
;; formatting inside the fact-form survive byte-identical.
(:wat::core::defn :user::fact-text
  [f <- :wat::WatAST src <- :wat::core::String lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::String
  (:wat::core::let
    [fact (:wat::core::Option/expect (:wat::core::get (:wat::core::ast->children f) 1) "fact-text: unreachable (insert-wrapped? already checked)")
     off  (:wat::fix::node-start-offset fact lines)
     end  (:wat::fix::node-end-offset fact lines)]
    (:wat::string::subs src off end)))

(:wat::core::defn :user::fact-texts
  [forms <- (:wat::core::Vector :- [:wat::WatAST]) src <- :wat::core::String lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::if (:wat::core::empty? forms)
    (:wat::core::Vector :- [:wat::core::String])
    (:wat::core::concat
      (:wat::core::Vector :- [:wat::core::String] (:user::fact-text (:wat::core::first forms) src lines))
      (:user::fact-texts (:wat::core::rest forms) src lines))))

;; join-with-space — left-to-right join; no trailing/leading space.
(:wat::core::defn :user::join-with-space [xs <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::String
  (:wat::core::if (:wat::core::empty? xs)
    ""
    (:wat::core::let [h (:wat::core::first xs) tl (:wat::core::rest xs)]
      (:wat::core::if (:wat::core::empty? tl)
        h
        (:wat::string::concat h (:wat::string::concat " " (:user::join-with-space tl)))))))

;; ── per-defrule edit ────────────────────────────────────────────────────────────────────────

;; defrule-edits — 0-or-1 replacement edit for one defrule form's :then payload.
;; rch layout: [0 head-sym, 1 name, 2 :when-kw, 3 when-vec, 4 :then-kw, 5.. then-forms…].
(:wat::core::defn :user::defrule-edits
  [node  <- :wat::WatAST
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:user::defrule-form? node)
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::< (:wat::core::count ch) 6)
        ;; No then-forms at all (bare `:then` with nothing after) — none observed in the
        ;; corpus (verified pre-strike); stay honest rather than silently no-op a shape that
        ;; was never checked.
        (:wat::kernel::assertion-failed!
          (:wat::string::concat "defrule-then-to-vector: :then has no forms in " (:wat::core::write-forms node))
          :wat::core::None :wat::core::None)
        (:wat::core::let [then-forms (:wat::core::into [] (:wat::core::drop ch 5))]
          (:wat::core::if (:user::then-already-vector? then-forms)
            (:wat::core::Vector :- [:wat::fix::Edit]) ;; idempotent no-op — already migrated
            (:wat::core::if (:wat::core::not (:user::all-insert-wrapped? then-forms))
              (:wat::kernel::assertion-failed!
                (:wat::string::concat
                  "defrule-then-to-vector: STOP-2 — a :then entry is not a plain (:wat::rete::insert <fact>) form in "
                  (:wat::core::write-forms node))
                :wat::core::None :wat::core::None)
              (:wat::core::let
                [fact-texts (:user::fact-texts then-forms src lines)
                 joined     (:user::join-with-space fact-texts)
                 first-fact (:wat::core::Option/expect (:wat::core::get then-forms 0) "defrule-edits: unreachable")
                 last-fact  (:wat::core::Option/expect (:wat::core::get then-forms (:wat::i64::- (:wat::core::count then-forms) 1)) "defrule-edits: unreachable")
                 first-off  (:wat::fix::node-start-offset first-fact lines)
                 ;; old-text = fix-text-span-text spanning first-fact's start to last-fact's
                 ;; end (arc 282) — sanctioned: this is a REFLOW of a known multi-form region
                 ;; (join with spaces, wrap in "[...]"), not a rename; no name-based claim
                 ;; about the inter-form whitespace exists to diverge from the span.
                 old-text   (:wat::fix::fix-text-span-text (:wat::core::ast-span first-fact) (:wat::core::ast-end-span last-fact) lines src)]
                (:wat::core::Vector :- [:wat::fix::Edit]
                  (:wat::core::Tuple first-off old-text
                    (:wat::string::concat "[" (:wat::string::concat joined "]"))))))))))
    (:wat::core::Vector :- [:wat::fix::Edit])))

;; ── deep walk (defrule forms may be nested inside a data literal, not just top-level) ─────────

(:wat::core::defn :user::walk-edits
  [node  <- :wat::WatAST
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::let [this (:user::defrule-edits node src lines)]
    (:wat::core::if (:wat::fix::structural? node)
      (:wat::core::concat this (:user::walk-seq-edits (:wat::core::ast->children node) src lines))
      this)))

(:wat::core::defn :user::walk-seq-edits
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector :- [:wat::fix::Edit])
    (:wat::core::concat
      (:user::walk-edits (:wat::core::first items) src lines)
      (:user::walk-seq-edits (:wat::core::rest items) src lines))))

;; ── per-file migrate ────────────────────────────────────────────────────────────────────────

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let
    [lines (:wat::string::split src "\n")
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
        (:wat::kernel::println (:wat::string::concat "[defrule-then-to-vector] " path))
        (:user::rewrite-each (:wat::core::into [] (:wat::core::rest paths)))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [paths (:wat::core::match (:wat::kernel::readln)
                            ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
                            (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
                            (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))]
    (:user::rewrite-each paths)))
