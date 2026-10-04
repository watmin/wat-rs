;; wat-scripts/fixes/rename-here-to-error-site-in-fault-mints.wat — excursus 003 strike F2
;; (BRIEF-shape-strike-F2-one-location-rule.md, items 2 and 4).
;; SCOPE: wat/cache.wat wat/sqlite.wat wat/query.wat wat/telemetry/span.wat wat/telemetry/journal.wat wat/core.wat
;;
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; Strike F minted `cache::Fault`/`sqlite::Fault`/`query::Fault`'s `:location` via
;; `(:wat::kernel::here)` INSIDE the stdlib (`wat/cache.wat`, `wat/sqlite.wat`, `wat/query.wat`,
;; `wat/telemetry/span.wat`, `wat/telemetry/journal.wat`), so a user's failing call located at
;; the stdlib's own mint-site line — C-114's defect, this time in a RETURNED value. F2's cure is
;; `:wat::kernel::error-site` (src/intrinsic/kernel/source.rs) — the exact derivation
;; `RuntimeError::new` applies to every RAISED error (D4,
;; `crate::value::signal::derive_primary_location_and_frames`): the current span when it is
;; already user source, else the innermost CALL_STACK frame that is. This codemod is an EXACT
;; keyword rename, `:wat::kernel::here` -> `:wat::kernel::error-site`, over every call site in
;; the five files above — chosen over `rename-keyword-prefix` because `here` is a terminal leaf
;; keyword (no parametric tail, no suffix can follow it) and an exact match can never later
;; swallow a future sibling whose name happens to start with `here`.
;;
;; `wat/core.wat` is ALSO in scope, for a SEPARATE reason (item 4, decided by the four
;; questions, Obvious/Simple/Honest/Good-UX all YES): `:wat::core::Fault/of`
;; (`(:wat::core::defmacro :wat::core::Fault/of [msg] -> (:wat::core::Fault :message ~msg
;; :location (:wat::kernel::here)))`) splices `here` into the EXPANSION, so it fires at
;; `Fault/of`'s CALLER's coordinate — correct when the caller is user code, but the SAME C-114
;; defect when the caller is itself stdlib (e.g. `wat/spawn.wat`'s `message-only-failure`, which
;; calls `Fault/of` on the stdlib's own behalf). Renaming the ONE `here` inside `Fault/of`'s
;; quasiquote body to `error-site` fixes the stdlib case with NO change to the user-call case
;; (a span that is already user source is `error-site`'s first branch, byte-identical to what
;; `here` would have given). `core.wat`'s only OTHER `kernel::here` mention is prose (a `;;`
;; comment, outside the AST this codemod walks) — hand-edited separately, not by this tool.
;;
;; EXCLUDED, deliberately:
;;   - `wat/query/sqlite-store.wat`'s `lift-fault`, which propagates an ALREADY-located
;;     `:wat::sqlite::Fault`'s own `:location` rather than minting a fresh one — no `here` call
;;     exists there to rename.
;;
;; Idempotent by construction (`rename-keyword-exact` is exact/literal): after one application
;; no `:wat::kernel::here` keyword remains in the six files, so a re-run finds zero matches.
;;
;; Usage (one EDN vector of EVERY path on stdin):
;;   printf '["wat/cache.wat" "wat/sqlite.wat" "wat/query.wat" "wat/telemetry/span.wat" "wat/telemetry/journal.wat" "wat/core.wat"]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/rename-here-to-error-site-in-fault-mints.wat

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::fix::rename-keyword-exact ":wat::kernel::here" ":wat::kernel::error-site" src))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[here->error-site] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln)
      [:wat::kernel::ReadlnOutcome.Datum {:v __datum} __datum]
      [:wat::kernel::ReadlnOutcome.Eof {}
        (:wat::kernel::assertion-failed! :message "readln: end of input")]
      [:wat::kernel::ReadlnOutcome.Stopped {}
        (:wat::kernel::assertion-failed! :message "readln: stop requested")])))
