;; wat-scripts/fixes/bare-none-keyword-to-fqdn.wat — arc 255: the bare `:None` KEYWORD dies.
;;
;; ⛔ THE HERESY. `wat is fqdn, always — anything that is not a binder is illegal.` The bare
;; `:None` keyword violates that and is LEGAL TODAY: measured 2026-09-04, a match arm spelled
;; `(:None "none")` passes `--check` with exit 0 AND runs. The comment at `src/runtime.rs:8324`
;; claims the bare form is "poisoned at type-check time" — that claim is FALSE and was measured
;; false the same day. This codemod is the corpus half of ending it.
;;
;; SCOPE — `:None` ONLY. The two obvious companions are BOTH UNSAFE and were measured so before
;; this file was written:
;;
;;   :Ok   194 corpus sites, and NONE of them is `Result::Ok`. They are VARIANT DECLARATIONS
;;         inside other enums — `(defenum :probe::Echo::EchoResponse … :Ok [reply <- String] …)`,
;;         `(defenum :wat::cache::Cache::GetResponse … :Ok [results <- …])`. Renaming them to
;;         `:wat::core::Ok` would corrupt unrelated enums.
;;   :Err  `tests/types/probe_arc241_stone9_defenum_c02.wat:2` declares `(defenum :app::Result
;;         … :Err …)` — its OWN variant. Same corruption.
;;
;;   `:None` was checked against the same test and is CLEAN: no `defenum` in the corpus declares
;;   a `:None` variant of its own.
;;
;; ⚠ WHY A FORM-TREE CODEMOD AND NOT A TEXT SUBSTITUTION — the trap is live and the orchestrator
;; fell into it while measuring this: `:None` is a SUBSTRING of `:wat::core::None`. A regex
;; rewrite produces `:wat::core::wat::core::None`, and a naive count returns 5662 where the real
;; population is 94. `rename-keyword-exact` (wat/fix.wat) walks the PARSED form tree and matches
;; on WHOLE-NAME equality (`(= (ast-name node) old)`), so it can touch neither the FQDN form nor
;; prose — a comment is not a node.
;;
;; IDEMPOTENT BY CONSTRUCTION: after one run the keyword's name is `:wat::core::None`, which does
;; not equal `:None`, so a re-run emits zero edits.
;;
;; ⛔ ADDITIVE PREDECESSOR, NOT A REPLACEMENT: this migrates `:None` -> `:wat::core::None`, the
;; spelling that WORKS TODAY. It is NOT the later migration `:wat::core::None` ->
;; `:wat::core::Option::None` (the 1542-site bridge 294 R9 named), which is gated on the runtime
;; matcher learning the qualified form. Two migrations, one dependency between them; this one is
;; ungated and safe to run now.
;;
;; Usage — list EVERY path:
;;   printf '["pathA" "pathB" …]\n' | cargo wat ./wat-scripts/fixes/bare-none-keyword-to-fqdn.wat

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::fix::rename-keyword-exact ":None" ":wat::core::None" src))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
