;; wat-scripts/fixes/deprime-telemetry-sqlite.wat — Stone C / STRIKE C3 de-prime,
;; run over real wat source files IN WAT, through the wat CLI. The migration tool,
;; self-hosted: no Rust harness, no hand-edit of wat source (use-the-tool, not hand-fix).
;;
;; The TRUE RECLAIM: C1 annihilated the legacy crates that squatted the bare names,
;; so the primed in-core family de-primes into its honest home:
;;   :wat::telemetry'::journal' -> :wat::telemetry::journal   (both primes, SPECIFIC-FIRST)
;;   :wat::telemetry'::span'    -> :wat::telemetry::span
;;   :wat::telemetry'          -> :wat::telemetry            (the family: Log/Metric/Scope/Level/…)
;;   :wat::query::mem-store'    -> :wat::query::mem-store     (FREE-RENAME)
;;   :wat::query::sqlite-store' -> :wat::query::sqlite-store  (FREE-RENAME)
;;   :wat::sqlite'             -> :wat::sqlite               (TRUE-RECLAIM: ::Connection/::open/bare)
;;   :rust::sqlite'           -> :rust::sqlite               (the rust dispatch, in .wat refs)
;;
;; SPECIFIC-FIRST discipline: journal'/span' carry their OWN prime, so they must fully
;; de-prime BEFORE the `:wat::telemetry'` family rename drops — else `:wat::telemetry'`
;; matches `:wat::telemetry'::journal'` first (boundary at the `::`) and leaves a dangling
;; `journal'`/`span'` prime behind.
;;
;; NOTE on rename-keyword-prefix boundaries (wat/fix.wat:632 rename-valid-match?): a match
;; is right-valid only when the char AFTER old-bare is a non-ident boundary (`::`/`/`/`<`/
;; `,`/space/end). So the family is renamed as the FULL name `:wat::telemetry'` (NOT the
;; `:wat::telemetry'::` prefix — whose trailing `::` would abut an ident char like `M` in
;; `::Metric` and be rejected). `:wat::telemetry'` catches every member/accessor/variant
;; because each is followed by `::` or end; the trailing prime keeps already-deprimed
;; `:wat::telemetry::…` names from re-matching (they have no prime), so it is idempotent.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["wat/telemetry.wat" "wat/sqlite.wat"]\n' | cargo wat ./wat-scripts/fixes/deprime-telemetry-sqlite.wat
;;
;; The rewrite is comment-faithful and idempotent (re-running yields zero changes — the
;; old primed prefixes are gone), so it is safe to run over a clean tree.

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::fix::rename-keyword-prefix ":rust::sqlite'" ":rust::sqlite"
    (:wat::fix::rename-keyword-prefix ":wat::sqlite'" ":wat::sqlite"
      (:wat::fix::rename-keyword-prefix ":wat::query::sqlite-store'" ":wat::query::sqlite-store"
        (:wat::fix::rename-keyword-prefix ":wat::query::mem-store'" ":wat::query::mem-store"
          (:wat::fix::rename-keyword-prefix ":wat::telemetry'" ":wat::telemetry"
            (:wat::fix::rename-keyword-prefix ":wat::telemetry'::span'" ":wat::telemetry::span"
              (:wat::fix::rename-keyword-prefix ":wat::telemetry'::journal'" ":wat::telemetry::journal"
                src))))))))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[deprimed] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
