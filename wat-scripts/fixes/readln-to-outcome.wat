;; wat-scripts/fixes/readln-to-outcome.wat — arc 170 closure #24, the readln totality flip.
;;
;; Self-hosted, comment-faithful fix-wat codemod — NO hand-editing of .wat files, use the tool.
;;
;; THE CHANGE. `:wat::kernel::readln` RAISED on end-of-input and on a process-wide stop. It was the
;; LAST IPC verb still raising; every sibling (recv'/send'/close'/accept'/connect'/poll') already
;; hands its failure back as a matchable value, because a raise in a language with no try/catch
;; unwinds PAST the reader (R53 `VERBO MEO CAPTVS`). `readln` now returns
;; `(:wat::kernel::ReadlnOutcome :- [T])::{Datum [value <- :T], Eof, Stopped}` and every call site faces it.
;;
;; The capability was BANKED, not missing: `StdIn` has always returned a matchable `::Eof`, and
;; `stdio-read` raised on it "to preserve the old fd-0 behavior for the 72 readln callers". That bank
;; is what made a REPL loop unable to stop cleanly. This spends it.
;;
;; THE REWRITE — each `(readln …)` becomes the match that unwraps it:
;;
;;   (:wat::kernel::readln )
;;     ->  (:wat::core::match (:wat::kernel::readln )
;;           ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
;;           (:wat::kernel::ReadlnOutcome::Eof     (:wat::kernel::assertion-failed! "readln: end of input" …))
;;           (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" …)))
;;
;; WHY THE DEATH ARMS ARE BLANKET, AND WHY THAT IS NOT THE COLLAPSE WE JUST SPENT.
;; `readln` is generic in `T`, so the only arm body expressible at EVERY site without knowing the
;; consumer's type is a terminating form (`assertion-failed!` types as `∀T. T`). This mirrors
;; `read-string-to-outcome.wat` exactly, and the distinction it draws is the load-bearing one --
;; `DESIGN-no-hidden-failures.md`, verbatim: "An AUTHOR-written recv' arm MAY assertion-failed! (a
;; visible chosen death)… a GENERATED method may NOT." `stdio-read`'s collapse lived INSIDE the verb
;; where no caller could opt out; this one lives at the CALL SITE as three visible edges any caller
;; can refine. The two outcomes are kept DISTINCT (separate arms, separate messages) -- an Eof is not
;; a Stop, and the site now says which one killed it.
;;
;; A caller that must SURVIVE a stop refines its own arms afterwards -- that is the whole point of
;; the flip, and the serve loops (`repl-daemon.wat`, the stdio-service demo) are the first that do.
;; Their `Eof`/`Stopped` arms become a clean exit, which is what `repl-daemon.wat`'s own comment
;; already claims ("the honest stop") and does not yet do.
;;
;; MECHANISM: `:wat::fix::wrap-calls-in-match` (wat/fix.wat, THE WRAP FAMILY). The matcher is an
;; EXACT head comparison, so the prime `:wat::kernel::readln'` is never touched; the kwargs form
;; `(readln :max-buffer-bytes N)` is wrapped identically (the whole call is the scrutinee, whatever
;; its args); and it is idempotent by construction (a match whose arm heads already name
;; `ReadlnOutcome::` keeps its scrutinee, while its ARMS are still walked, so a nested readln is
;; still reached). Re-run = 0 edits, proven by a dry-run diff before the corpus was touched.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["pathA" …]\n' | ./target/release/wat ./wat-scripts/fixes/readln-to-outcome.wat
;;
;; NOTE ON SELF-APPLICATION: this file's own `:user::main` is written in the NEW form already -- it
;; has to be, because the substrate flip is live and the old form no longer type-checks. Running the
;; codemod over the corpus INCLUDING this file is a no-op on it.

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::fix::wrap-calls-in-match src
    ":wat::kernel::readln"
    "ReadlnOutcome::"
    "(:wat::core::match "
    " ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! \"readln: end of input\" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! \"readln: stop requested\" :wat::core::None :wat::core::None)))"))

;; ── driver ───────────────────────────────────────────────────────────────────
(:wat::core::defn :user::apply-each [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let
      [path (:wat::core::first paths)
       src  (:wat::io::read-file path)
       out  (:user::migrate src)]
      (:wat::core::do
        (:wat::core::if (:wat::core::= src out)
          nil
          (:wat::io::write-file path out))
        (:user::apply-each (:wat::core::into [] (:wat::core::rest paths)))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln )
      ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
      (:wat::kernel::ReadlnOutcome::Eof     (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
      (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
