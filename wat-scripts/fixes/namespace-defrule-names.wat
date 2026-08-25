;; wat-scripts/fixes/namespace-defrule-names.wat — arc 278 DESIGN-STONE-namespacing-wall, step 1:
;; namespace the `where`-expressivity corpus's 89 bare `defrule` names.
;;
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; A SIBLING of `rule-record-to-defrule.wat`, not an extension of it. That codemod's job (hand-built
;; `Rule` record → `defrule`) is DONE and its own idempotence proof (no more `-> :wat::rete::Rule`
;; rule-defns left) is unrelated to this one's gate (no more BARE `defrule` names left) — reusing its
;; body would mean threading an unrelated precondition through unrelated logic. What IS reused is the
;; shared `wat/fix.wat` machinery both ride: `wat::fix::rename-keyword-exact` (the whole-token rename
;; that did the heavy lifting there) and, new here, `wat::fix::wrap-edits` (a generic node-wrap that
;; already existed in fix.wat for a different caller — reused here verbatim for a different purpose:
;; wrapping the `Rule/name` read in a display-derivation call).
;;
;; ── THE PROBLEM (DESIGN-STONE-namespacing-wall.md) ──────────────────────────────────────────────
;; `rule-record-to-defrule.wat` minted every corpus rule's `defrule` symbol BARE (`:arith`, not
;; `:wsh::arith`) so that `(:wat::rete::Rule/name rule)` — which `defrule` derives from the DEFN
;; SYMBOL — would print the corpus's short "arith" untouched, matching the untouched Clojure oracle's
;; `(defrule arith ...)`. That resolved the printing property at the cost of 89 wat-wide naming-wall
;; violations: only args/let-bindings may be bare; every defn/defrecord/defrule/etc. top-level name
;; must be namespaced. This codemod closes the gap WITHOUT reopening the printing one.
;;
;; ── THE FIX, keeping ONE source of truth ────────────────────────────────────────────────────────
;;   1. Namespace every bare `defrule` NAME with the file's own namespace (ground it from the file's
;;      own first namespaced top-level `defn`/`defrecord` — e.g. where-shapes.wat's `:wsh::items` says
;;      this file's namespace is `wsh` — never a hand-kept path→namespace table).
;;      `:arith` -> `:wsh::arith`. `rename-keyword-exact` reaches BOTH the `defrule` symbol itself and
;;      every `build-rules` call-site keyword (`(:arith)` -> `(:wsh::arith)`) in one pass, because both
;;      spellings are the SAME keyword token, which is exactly what the whole-token exact rename is for.
;;   2. Insert one small pure helper per file, `<ns>::rule-display-name`, and wrap `run-row`'s existing
;;      `(:wat::rete::Rule/name rule)` read with it. The rule's `Rule/name` is now "wsh::arith"; the
;;      helper strips everything up to and including the LAST "::", recovering "arith" — so the printed
;;      line, and therefore the diff against the untouched Clojure oracle, is BYTE-IDENTICAL to before.
;;      No second hand-kept display-name table; the printed label is DERIVED from the one namespaced
;;      name, never carried alongside it.
;;
;;      TOTALITY, not merely "works today": a raise inside a rule fire takes the whole corpus row down
;;      (arc 278's live lesson on partial verbs — `first`/`nth`/`Option/expect` on an undefined case).
;;      `rule-display-name` never touches one. `string::split(full, "::")` ALWAYS returns >= 1 segment
;;      (the whole string, unsplit, when "::" is absent — confirmed against `eval_string_split`,
;;      src/string_ops.rs: `hay.split(sep)` on a literal never-empty pattern). Folding with
;;      `foldl fn full segments`, where `fn` unconditionally returns its second arg, walks every
;;      segment and settles on the LAST one — while the SEED (`full`) is what a `first`/`Option/expect`
;;      would have needed a raise-on-empty guard for, and here just falls out for free: an input with
;;      no "::" yields a single-element segment vector, so the fold returns that element (the input,
;;      unchanged) — and even an impossible empty split would fall back to the seed, never raise.
;;
;; ── SCOPE GUARD ──────────────────────────────────────────────────────────────────────────────────
;; The `.clj` oracle files are NEVER touched by this codemod (it is not given their paths and has no
;; verb that would reach them) — that is what makes the gate self-verifying: an untouched oracle next
;; to a byte-identical diff proves the wat-side rename was semantically inert.
;;
;; ── IDEMPOTENCE ──────────────────────────────────────────────────────────────────────────────────
;; Gated on `any-bare-defrule?`: once every `defrule` name in a file is namespaced, the predicate is
;; false and `migrate` returns `src` UNCHANGED — no renames, no re-wrap, no second helper inserted.
;;
;; Dry-run on a /tmp copy + diff, THEN apply:
;;   printf '["wat-scripts/perf/grid/where-shapes.wat" ...]' \
;;     | ./target/release/wat ./wat-scripts/fixes/namespace-defrule-names.wat

;; ── small predicates over defrule forms ─────────────────────────────────────────────────────────

;; defrule-form? — a top-level `(:wat::rete::defrule NAME ...)` list.
(:wat::core::defn :user::defrule-form? [f <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::= (:wat::fix::head-name f) ":wat::rete::defrule"))

;; defrule-name-node — the NAME keyword node (child[1]) of a defrule-form.
(:wat::core::defn :user::defrule-name-node [f <- :wat::WatAST] -> :wat::WatAST
  (:wat::core::Option/expect (:wat::core::get (:wat::core::ast->children f) 1) "defrule-name-node: name"))

;; bare-defrule? — a defrule-form whose NAME keyword has no "::" in it.
(:wat::core::defn :user::bare-defrule? [f <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:user::defrule-form? f)
    (:wat::core::not (:wat::string::contains? (:wat::core::ast-name (:user::defrule-name-node f)) "::"))
    false))

;; any-bare-defrule? — the idempotence gate: does ANY top-level form need renaming?
(:wat::core::defn :user::any-bare-defrule? [forms <- (:wat::core::Vector :- [:wat::WatAST])] -> :wat::core::bool
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::bool  f <- :wat::WatAST] -> :wat::core::bool
      (:wat::core::if acc true (:user::bare-defrule? f)))
    false
    forms))

;; ── deriving the file's namespace from its OWN first namespaced top-level defn/defrecord ───────
;; (never a hand-kept path -> namespace table)

(:wat::core::defn :user::namespaced-defn-name [f <- :wat::WatAST] -> (:wat::core::Option :wat::core::String)
  (:wat::core::let [head (:wat::fix::head-name f)]
    (:wat::core::if (:wat::core::if (:wat::core::= head ":wat::core::defn") true (:wat::core::= head ":wat::core::defrecord"))
      (:wat::core::let [namekw (:wat::core::Option/expect (:wat::core::get (:wat::core::ast->children f) 1) "namespaced-defn-name: name")]
        (:wat::core::if (:wat::core::= (:wat::core::ast-kind namekw) "keyword")
          (:wat::core::let [nm (:wat::core::ast-name namekw)]
            (:wat::core::if (:wat::string::contains? nm "::") (:wat::core::Some nm) :wat::core::None))
          :wat::core::None))
      :wat::core::None)))

;; find-ns — walk top-level forms in order; the FIRST namespaced defn/defrecord names this file's
;; namespace (e.g. ":wsh::items" -> "wsh"). STOPS if the file has none at all — never a guessed default.
(:wat::core::defn :user::find-ns [forms <- (:wat::core::Vector :- [:wat::WatAST])] -> :wat::core::String
  (:wat::core::if (:wat::core::empty? forms)
    (:wat::kernel::assertion-failed!
      "namespace-defrule-names: no namespaced top-level defn/defrecord found to derive the file namespace from"
      :wat::core::None :wat::core::None)
    (:wat::core::match (:user::namespaced-defn-name (:wat::core::first forms))
      ((:wat::core::Some nm)
        (:wat::core::let [seg0 (:wat::core::Option/expect (:wat::core::get (:wat::string::split nm "::") 0)
                                  "find-ns: split always yields >= 1 element")]
          (:wat::string::strip-leading-colon seg0)))
      (:wat::core::None (:user::find-ns (:wat::core::rest forms))))))

;; ── collecting the (old, new) rule-name rename pairs ────────────────────────────────────────────

(:wat::core::defn :user::rule-renames
  [forms <- (:wat::core::Vector :- [:wat::WatAST])  ns <- :wat::core::String]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::empty? forms)
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::String :wat::core::String]))
    (:wat::core::let [f (:wat::core::first forms) tl (:wat::core::rest forms)]
      (:wat::core::if (:user::bare-defrule? f)
        (:wat::core::let [old  (:wat::core::ast-name (:user::defrule-name-node f))
                          bare (:wat::string::strip-leading-colon old)
                          new  (:wat::core::String/concat ":"
                                 (:wat::core::String/concat ns
                                   (:wat::core::String/concat "::" bare)))]
          (:wat::core::concat
            (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::String :wat::core::String]) (:wat::core::Tuple old new))
            (:user::rule-renames tl ns)))
        (:user::rule-renames tl ns)))))

(:wat::core::defn :user::apply-renames
  [text    <- :wat::core::String
   renames <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])]
  -> :wat::core::String
  (:wat::core::if (:wat::core::empty? renames)
    text
    (:wat::core::let [p   (:wat::core::first renames)
                      old (:wat::core::first p)
                      new (:wat::core::second p)]
      (:user::apply-renames (:wat::fix::rename-keyword-exact old new text) (:wat::core::rest renames)))))

;; ── locating run-row's `(:wat::rete::Rule/name rule)` read, post-rename ────────────────────────

(:wat::core::defn :user::ends-with? [s <- :wat::core::String suf <- :wat::core::String] -> :wat::core::bool
  (:wat::core::let [ls (:wat::string::length s) lsuf (:wat::string::length suf)]
    (:wat::core::if (:wat::core::< ls lsuf)
      false
      (:wat::core::= (:wat::string::subs s (:wat::core::i64::- ls lsuf) ls) suf))))

;; run-row-defn? — a top-level `(:wat::core::defn NAME:...::run-row ...)` form.
(:wat::core::defn :user::run-row-defn? [f <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::fix::head-name f) ":wat::core::defn")
    (:wat::core::let [namekw (:wat::core::Option/expect (:wat::core::get (:wat::core::ast->children f) 1) "run-row-defn?: name")]
      (:wat::core::if (:wat::core::= (:wat::core::ast-kind namekw) "keyword")
        (:user::ends-with? (:wat::core::ast-name namekw) "::run-row")
        false))
    false))

(:wat::core::defn :user::find-run-row [forms <- (:wat::core::Vector :- [:wat::WatAST])] -> :wat::WatAST
  (:wat::core::if (:wat::core::empty? forms)
    (:wat::kernel::assertion-failed! "namespace-defrule-names: no ::run-row defn found" :wat::core::None :wat::core::None)
    (:wat::core::let [f (:wat::core::first forms)]
      (:wat::core::if (:user::run-row-defn? f) f (:user::find-run-row (:wat::core::rest forms))))))

;; find-call — deep search for the (unique, per the survey) node calling exactly `head`.
(:wat::core::defn :user::find-call [node <- :wat::WatAST  head <- :wat::core::String] -> (:wat::core::Option :wat::WatAST)
  (:wat::core::if (:wat::fix::calls-to? node head)
    (:wat::core::Some node)
    (:wat::core::if (:wat::fix::structural? node)
      (:user::find-call-seq (:wat::core::ast->children node) head)
      :wat::core::None)))

(:wat::core::defn :user::find-call-seq
  [items <- (:wat::core::Vector :- [:wat::WatAST])  head <- :wat::core::String] -> (:wat::core::Option :wat::WatAST)
  (:wat::core::if (:wat::core::empty? items)
    :wat::core::None
    (:wat::core::let [h (:wat::core::first items) tl (:wat::core::rest items)]
      (:wat::core::match (:user::find-call h head)
        ((:wat::core::Some found) (:wat::core::Some found))
        (:wat::core::None (:user::find-call-seq tl head))))))

;; ── the inserted helper's source text (only the namespace varies) ──────────────────────────────

;; concat-all — String/concat is 2-arg; fold a (Vector :- [String]) of parts left-to-right instead of
;; hand-nesting a nine-deep concat chain (error-prone and unreadable at that depth).
(:wat::core::defn :user::concat-all [parts <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String  p <- :wat::core::String] -> :wat::core::String
      (:wat::core::String/concat acc p))
    ""
    parts))

(:wat::core::defn :user::helper-defn-text [ns <- :wat::core::String] -> :wat::core::String
  (:user::concat-all
    (:wat::core::Vector :wat::core::String
      ";; rule-display-name — TOTAL derivation of the printed row label from a Rule/name that may\n"
      ";; now carry this file's namespace prefix (e.g. \"NS::arith\") after the namespacing wall.\n"
      ";; `string::split` on \"::\" always returns >= 1 segment (the whole string, unsplit, when\n"
      ";; \"::\" is absent); folding with SEED = full while always overwriting the accumulator\n"
      ";; with the current segment lands on the LAST segment without ever calling a partial\n"
      ";; verb (`first`/`nth`/`Option/expect`) — the seed also makes the no-\"::\" case return\n"
      ";; the input UNCHANGED, and even an impossible empty split falls back to the seed\n"
      ";; instead of raising.\n"
      "(:wat::core::defn :"
      ns
      "::rule-display-name\n  [full <- :wat::core::String] -> :wat::core::String\n  (:wat::core::foldl\n    (:wat::core::fn [acc <- :wat::core::String  seg <- :wat::core::String] -> :wat::core::String seg)\n    full\n    (:wat::core::string::split full \"::\")))\n\n")))

;; ── per-file migrate ─────────────────────────────────────────────────────────────────────────

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let
    [tree0  (:wat::core::match (:wat::core::read-string src) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
     forms0 (:wat::core::ast->children tree0)]
    (:wat::core::if (:wat::core::not (:user::any-bare-defrule? forms0))
      src ;; idempotent no-op — every defrule name is already namespaced
      (:wat::core::let
        [ns       (:user::find-ns forms0)
         renames  (:user::rule-renames forms0 ns)
         text1    (:user::apply-renames src renames)
         lines1   (:wat::string::split text1 "\n")
         tree1    (:wat::core::match (:wat::core::read-string text1) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
         forms1   (:wat::core::ast->children tree1)
         run-row  (:user::find-run-row forms1)
         target   (:wat::core::Option/expect (:user::find-call run-row ":wat::rete::Rule/name")
                     "migrate: no Rule/name call found in run-row")
         wrap-eds (:wat::fix::wrap-edits target
                    (:wat::core::String/concat "(:" (:wat::core::String/concat ns "::rule-display-name "))
                    ")"
                    lines1)
         ins-off  (:wat::fix::node-start-offset run-row lines1)
         ins-edit (:wat::core::Vector :wat::fix::Edit (:wat::core::Tuple ins-off "" (:user::helper-defn-text ns)))
         all-eds  (:wat::core::concat wrap-eds ins-edit)
         text2    (:wat::fix::fix-text-apply text1 (:wat::core::reverse (:wat::core::sort all-eds)))]
        text2))))

;; ── driver: rewrite each path given on stdin (a JSON array of strings) ──────────────────────────
(:wat::core::defn :user::rewrite-each [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [p (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file p (:user::migrate (:wat::io::read-file p)))
        (:wat::kernel::println (:wat::core::String/concat "[namespace-defrule-names] " p))
        (:user::rewrite-each (:wat::core::into [] (:wat::core::rest paths)))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [paths (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))]
    (:user::rewrite-each paths)))
