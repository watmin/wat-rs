;; wat-scripts/fixes/vend-only-wat-repl-names.wat — "the stdlib vends only `:wat::`".
;;
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; DESIGN: docs/excursus/2026/08/001-sns-sqs/the-stdlib-vends-only-wat/DESIGN.md
;; ROOT:   docs/excursus/2026/08/001-sns-sqs/can-a-user-def-change-a-stdlib-verdict/FINDING.md
;;
;; Builder's ruling, verbatim: *"it must be `:wat::repl::*` — we must only vend `:wat::*`"*.
;;
;;   :repl::turn           ->  :wat::repl::turn
;;   :repl::eval-form      ->  :wat::repl::eval-form
;;   :repl::eval-and-loop  ->  :wat::repl::eval-and-loop
;;
;; WHY. `wat/repl.wat` is a stdlib MODULE, so those three `defn` heads are registered into
;; the frozen world every wat program inherits. Only `:wat::` / `:rust::` / `:$bound::` are
;; reserved, so — unlike every other one of the 3028 names the stdlib vends — these three are
;; names a USER program can legally declare. Proven, not inferred: a plain program calling
;; `(:repl::turn "x")` gets a `TypeMismatch` (the name RESOLVES, to the stdlib's signature),
;; while `(:wat::repl::turn "x")` got `UnknownFunction` before this rename.
;;
;; ⛔ And they are the whole hole. `(:wat::core::defclause :repl::turn ([s <- String] -> String s))`
;; plus a trivial `:user::main` turns FIVE green bodies inside `wat/repl.wat` red
;; (`NoMatchingClauseAtCallSite` at lines 75/91/97/104/111) — the user's clause table wins over
;; the stdlib's own registered scheme at `src/check.rs:6042`. The spike census of the stdlib body
;; sweep measured 30,964 name-probes over 3,060 distinct names, of which exactly THREE were
;; user-declarable: these. This rename takes that surface 3 -> 0.
;;
;; ── EXACT, NOT PREFIX, and this is the load-bearing choice ───────────────────────────────
;;
;; `rename-keyword-exact` fires only when a keyword leaf's FULL name equals the old name.
;; A PREFIX rename of `:repl::` would be WRONG here, because `:repl::` is a namespace two
;; USER programs in this corpus legitimately own:
;;
;;   wat-scripts/demos/stdio-service/stdio-service.wat  — `:repl::serve` / `:repl::Cmd` / `:repl::Reply`
;;   crates/wat-edn/demo/repl-daemon.wat                — `:repl::serve`
;;
;; Those are USER definitions. Renaming them into `:wat::repl::` would be refused by
;; `ReservedPrefix` (user source may not define under `:wat::`) and would break both demos.
;; They are deliberately NOT migrated: a user program owning `:repl::serve` is exactly the
;; freedom the ruling restores — the namespace belongs to users once the stdlib vacates it.
;; (⚠ The DESIGN listed both files as codemod targets. They hold ZERO occurrences of the three
;; names; the census below is the authority, and it reports 0 for each.)
;;
;; IDEMPOTENT BY CONSTRUCTION: after the rewrite a leaf spells `:wat::repl::turn`, which is
;; != `:repl::turn`, so a re-run emits 0 edits and the finder returns 0 matches.
;;
;; COMMENT-FAITHFUL, and therefore INCOMPLETE BY DESIGN: the rewrite walks the form tree, so
;; prose is untouched. `wat/repl.wat`'s header, `wat-scripts/scratch-pad/
;; probe-repl-declaration-refusal.wat`'s header, `src/distribution/mod.rs` and
;; `src/load/stdlib.rs`'s doc comments, and `src/check.rs:848`'s note are the manual tail —
;; plus `src/distribution/mod.rs`'s `REPL_SOURCE`, which is a generated PROGRAM in a Rust
;; string literal and reaches no `.wat` file at all.
;;
;; TWO ENTRY POINTS, one set of names:
;;   wat --grep <this file>  -> :user::grep   (the finder: prints every match, writes nothing)
;;   wat <this file>         -> :user::main   (the applier: rewrites files in place)
;;
;; Usage — census (count BEFORE writing anything):
;;   git ls-files '*.wat' | sed 's/.*/"&"/' | tr '\n' ' ' | sed 's/^/[/;s/ $/]/' \
;;     | ./target/release/wat --grep ./wat-scripts/fixes/vend-only-wat-repl-names.wat
;;
;; Usage — dry-run (on a /tmp COPY, then diff):
;;   cp wat/repl.wat /tmp/pilot.wat && printf '["/tmp/pilot.wat"]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/vend-only-wat-repl-names.wat
;;   diff wat/repl.wat /tmp/pilot.wat
;;
;; Usage — apply (list EVERY path):
;;   printf '["wat/repl.wat"]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/vend-only-wat-repl-names.wat

;; ── the finder — three rules over wat/grep.wat's stdlib fact base ────────────────────────
;;
;; ⚠ KEYWORD ONLY. `wat/grep.wat`'s `Named` fact also fires for a "string" kind, and this
;; corpus carries the three names as STRING LITERALS (this file's own `:user::migrate` below,
;; and the usage comments above are not nodes at all). A string literal's span covers its
;; quotes while its `name` does not, so a string hit is both a false positive for the census
;; and uncorrectable by a span splice. The applier's char-walk excludes them structurally
;; (`rename-exact-edits` tests `ast-kind == "keyword"`); a fact-based finder must say so.

(:wat::rete::defrule :vw::repl-turn
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":repl::turn"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "repl-turn-to-wat-repl-turn"
           :captures (:wat::rete::core::PersistentVector))])

(:wat::rete::defrule :vw::repl-eval-form
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":repl::eval-form"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "repl-eval-form-to-wat-repl-eval-form"
           :captures (:wat::rete::core::PersistentVector))])

(:wat::rete::defrule :vw::repl-eval-and-loop
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":repl::eval-and-loop"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "repl-eval-and-loop-to-wat-repl-eval-and-loop"
           :captures (:wat::rete::core::PersistentVector))])

(:wat::core::defn :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::rete::collect-rules :vw))

;; ── the applier ─────────────────────────────────────────────────────────────────────────
;;
;; Order is irrelevant: the three old names are distinct whole tokens, and no new name
;; contains an old one as its full name.

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::fix::rename-keyword-exact ":repl::turn" ":wat::repl::turn"
    (:wat::fix::rename-keyword-exact ":repl::eval-form" ":wat::repl::eval-form"
      (:wat::fix::rename-keyword-exact ":repl::eval-and-loop" ":wat::repl::eval-and-loop"
        src))))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[vend-only-wat] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln )
      ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
      (:wat::kernel::ReadlnOutcome::Eof
        (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
      (:wat::kernel::ReadlnOutcome::Stopped
        (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
