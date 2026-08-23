;; wat-scripts/fixes/spawn-program-to-test-spawn-peer.wat — the arc-170 #13 IPC wall
;; migration, run over real wat source files IN WAT, through the wat CLI. The migration
;; tool, self-hosted: no Rust harness, no hand-edit of wat source (use-the-tool, not
;; hand-fix).
;;
;;   :wat::kernel::spawn-program  ->  :wat::test::spawn-peer
;;
;; WHY. `spawn-program` is now a CAPABILITY, restricted to
;; `{:restricted-to [:wat::spawn:: :wat::test::]}` (wat/spawn.wat). Spawning a locus is
;; not a verb any test fixture may reach for ad hoc. The sanctioned path for tests is
;; `:wat::test::spawn-peer` — a defclause in the capability-holding namespace that
;; dispatches on locus type EXACTLY as `spawn-program` does:
;;
;;   ([locus <- ThreadOpts   prog <- [ThreadSelfPeer<S,R> :-> nil]] -> Thread<R,S>)
;;   ([locus <- ProcessOpts  prog <- Vector<wat::WatAST>]           -> Process<I,O>)
;;
;; Same arity, same arguments, same dispatch — so this migration is a pure HEAD
;; rename and nothing else. That isomorphism is why the corpus sweep is a codemod
;; rather than a hand-sorted fleet.
;;
;; WHY NOT run-thread/run-hermetic. Those SWALLOW the peer (they spawn, await one
;; completion signal, return a RunResult). The sites this migrates need to CONVERSE
;; with what they spawned — send a request, read a reply. The harness was one verb
;; short, which is why these sites hand-rolled `spawn-program` in the first place;
;; `spawn-peer` closes that gap. A site that only asks "did the child pass?" should use
;; `run-thread`/`run-hermetic` instead — this rename does not decide that for you, and
;; deliberately leaves such sites working (they are correct, just not minimal).
;;
;; EXACT, not prefix: `rename-keyword-exact` renames a keyword leaf ONLY when its FULL
;; name equals the old one. `:wat::kernel::spawn-thread` and
;; `:wat::kernel::spawn-process` (the tier primitives beneath the defclause, separately
;; walled in Rust via #[restricted_to]) must NOT be touched — a prefix rename would
;; catch them.
;;
;; ⛔ NEVER PASS `wat/spawn.wat`. It holds the DEFINITION — `(:wat::core::defclause
;; :wat::kernel::spawn-program …)` — and the rename does not distinguish a definition
;; from a call site. Measured on a /tmp copy 2026-07-28: 3 occurrences renamed, INCLUDING
;; the defclause head, which would silently retarget the wall's own subject and leave
;; `{:restricted-to […]}` guarding a name nothing calls. Same for any file that DEFINES
;; the verb. Pass CONSUMER paths only.
;;
;; (The same run confirmed exactness in the direction that matters: `spawn-thread` and
;; `spawn-process` — the tier primitives beneath the defclause — survived untouched, 1→1
;; each. A prefix rename would have eaten both.)
;;
;; THE CONDEMNED COHORT IS INCLUDED, DELIBERATELY. Many `wat-tests/**` spawn sites sit
;; under `(:wat::test::ignore "arc-170 concurrency layer … remove before arc 170
;; closes")` — arc-170 closure item #29, which DELETES them. Migrating them anyway is
;; the right call: a condemned file still has to TYPE-CHECK for its file's live tests to
;; run at all (one bad site fails the whole file's startup, taking every test in it
;; down). Renaming costs nothing and un-reds the tree; #29 still deletes them
;; afterwards. Excluding them would hold the corpus red pending a deletion decision that
;; is not this codemod's to make.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["tests/function/wat_spawn_fn.wat" …]\n' \
;;     | cargo wat ./wat-scripts/fixes/spawn-program-to-test-spawn-peer.wat
;;
;; The rewrite is comment-faithful and idempotent (re-running yields zero changes — the
;; old name is gone), so it is safe to run over a clean tree.

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::fix::rename-keyword-exact ":wat::kernel::spawn-program" ":wat::test::spawn-peer"
    src))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::core::string::concat "[migrated] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
