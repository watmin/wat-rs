;; wat-scripts/fixes/bare-symbol-shorthand-to-fqdn.wat — arc 255 STONE: the bare-symbol
;; shorthand dies (DESIGN-STONE-the-bare-symbol-shorthand-dies.md).
;;
;; Arc 109 slice 1h/1i retired bare `Some`/`Ok`/`Err` at CONSTRUCTOR sites only
;; (`(Some x)` -> `(:wat::core::Some x)`); the MATCH-PATTERN half of the same remedy
;; (`((Some v) ...)` -> `((:wat::core::Some v) ...)`) was never closed by a checker arm
;; and never migrated in the corpus. This codemod is that migration — the corpus half of
;; closing the pattern door (src/check.rs) that ships alongside it.
;;
;; ⚠ RENAME, NOT ALIAS — the bare spelling stops parsing as a recognized variant-
;; constructor pattern head entirely once the checker change lands; there is no dual-
;; spelling transition period in the corpus.
;;
;; `rename-symbol-exact` (wat/fix.wat) is the right tool, used AS-IS, three times: it
;; renames a bare-SYMBOL leaf ONLY on WHOLE-NAME equality (never a prefix/substring), so
;; `Some`/`Ok`/`Err` are matched exactly and nothing else. It is not position-restricted to
;; "list head" — but the population this migrates is validated (against the checker, not
;; grep — the design's own earlier count of 20 CONSTRUCTOR sites was 14 comment lines plus
;; prose) to be pattern-head occurrences with ZERO live bare-symbol CONSTRUCTOR sites in
;; the corpus, and `Some`/`Ok`/`Err` are otherwise unbound names (nothing in the corpus
;; legitimately closes over a variable literally named `Some`) — so a whole-tree rename is
;; exactly as precise as a pattern-head-only walk would be, without inventing new
;; position-aware machinery this migration doesn't need.
;;
;; Idempotent by construction: after one run, the token at that source position is a
;; KEYWORD (`:wat::core::Some`), not a Symbol — `rename-symbol-exact`'s leaf check is
;; `ast-kind == "symbol"`, so a re-run's parse no longer sees a Symbol there and emits no
;; edit for it.
;;
;; Usage:
;;   printf '["wat-scripts/perf/grid/where-control.wat" "tests/cli/wat_cli__programs_are_atoms.wat" "tests/cli/wat_cli__presence_proof.wat"]\n' \
;;     | cargo wat ./wat-scripts/fixes/bare-symbol-shorthand-to-fqdn.wat

;; The migration as DATA — one row per retired bare spelling, for symmetry with the
;; multi-row recorded migrations (rename-sort-prime-to-native.wat is the shape this mirrors).
(:wat::core::defn :user::renames [] -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])
  (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])]
    (:wat::core::Tuple "Some" ":wat::core::Some")
    (:wat::core::Tuple "Ok"   ":wat::core::Ok")
    (:wat::core::Tuple "Err"  ":wat::core::Err")))

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String
                     pr  <- (:wat::core::Tuple :- [:wat::core::String :wat::core::String])] -> :wat::core::String
      (:wat::fix::rename-symbol-exact (:wat::core::first pr) (:wat::core::second pr) acc))
    src
    (:user::renames)))

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
