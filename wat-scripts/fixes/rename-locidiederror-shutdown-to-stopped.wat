;; wat-scripts/fixes/rename-locidiederror-shutdown-to-stopped.wat — arc 170 closure item #3.
;;
;; Self-hosted, comment-faithful fix-wat codemod — NO hand-editing of .wat files, use the tool.
;;
;; THE MIGRATION (arc-170 intueri cast, RULING A, BRIEF-stopped-not-shutdown-rename.md; tracked as
;; CLOSURE-BACKLOG.md item 3): the wat-visible unit variant `:wat::kernel::LociDiedError::Shutdown`
;; becomes `::Stopped`. wat already owns the word `stopped` — `(:wat::kernel::stopped?)` means exactly
;; "has a stop been requested?" — and nothing is shutting down when this variant fires; a stop was
;; *requested* and the program decides. This is the LAST wat-visible holdout still wearing Rust's
;; `shutdown` vocabulary. The Rust side (`RecvError::Shutdown`, `trigger_shutdown`,
;; `SHUTDOWN_BROADCAST_READ_FD`, …) is UNTOUCHED — this codemod only ever reaches `.wat` text, and the
;; keyword rename below is exact-FQDN so it cannot cross into a Rust identifier regardless.
;;
;; Three disjoint text passes, in order:
;;   (1) `rename-keyword-exact` on the exact FQDN keyword `:wat::kernel::LociDiedError::Shutdown` ->
;;       `:wat::kernel::LociDiedError::Stopped` — the AST-based, span-faithful rewrite of every CODE
;;       occurrence (match-arm heads). `LociDiedError` has no type params and `Shutdown` is a terminal
;;       Unit variant (no fields, so it never carries a `<...>` parametric tail) — an EXACT match
;;       reaches every real occurrence and, unlike a prefix rename, can never later swallow some future
;;       sibling variant whose name happens to start with "Shutdown" (e.g. a hypothetical
;;       `ShutdownReason`). Chosen deliberately over `rename-keyword-prefix`: the prefix primitive
;;       earns its keep only when a parametric tail (`(Peer' :- [S R])`) or an appended suffix can follow the
;;       matched token, and neither is possible here.
;;   (2) a literal `WRONG:Shutdown` -> `WRONG:Stopped` text pass (split+join substring replace — no
;;       AST primitive touches STRING LITERALS). Four test fixtures carry a `WRONG:<variant>` sentinel
;;       per LociDiedError arm so a wrong-arm RED names exactly which death arrived; the sentinel must
;;       track the variant it names or it becomes exactly the kind of stale text this migration exists
;;       to remove.
;;   (3) a literal `LociDiedError::Shutdown` -> `LociDiedError::Stopped` text pass (split+join) for
;;       COMMENT prose that names the fact in words rather than in a keyword token (fix-text-apply's
;;       AST walk cannot see inside a `;;` comment). Safe to run globally: after pass (1) no CODE
;;       keyword contains this substring any more, so pass (3) can only ever touch prose.
;;
;; All three passes are literal/exact — never a prefix — so idempotent by construction: after one
;; application none of `:wat::kernel::LociDiedError::Shutdown` / `WRONG:Shutdown` /
;; `LociDiedError::Shutdown` remain in the text, so a re-run's three calls each find zero matches and
;; return the input unchanged.
;;
;; NOT touched, deliberately: `:wat::spawn::ServiceEvent::Shutdown` (a DIFFERENT wat-visible enum,
;; unrelated to LociDiedError and not named by the ruling) survives unrenamed by construction — this
;; codemod's old-value is the full LociDiedError FQDN, which a ServiceEvent keyword never matches.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["pathA" "pathB" …]\n' \
;;     | cargo wat ./wat-scripts/fixes/rename-locidiederror-shutdown-to-stopped.wat

;; literal-replace — substring replace via split+join (no dedicated string::replace primitive exists
;; in wat core). `old` must be non-empty (string::split rejects an empty separator).
(:wat::core::defn :user::literal-replace
  [src <- :wat::core::String  old <- :wat::core::String  new <- :wat::core::String] -> :wat::core::String
  (:wat::string::join new (:wat::string::split src old)))

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let
    [src1 (:wat::fix::rename-keyword-exact
            ":wat::kernel::LociDiedError::Shutdown" ":wat::kernel::LociDiedError::Stopped" src)
     src2 (:user::literal-replace src1 "WRONG:Shutdown" "WRONG:Stopped")
     src3 (:user::literal-replace src2 "LociDiedError::Shutdown" "LociDiedError::Stopped")]
    src3))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[shutdown->stopped] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
