;; wat-scripts/fixes/sweep-lint-fixes.wat — THE SWEEP. Run the self-hosted linter's auto-fixes over the
;; whole wat corpus, in wat, through the wat CLI. The toolchain cleaning its own source — proof-by-diff.
;;
;; For each path on stdin (one EDN vector — list EVERY .wat to sweep; arc 283 lesson: derive the set,
;; don't hand-guess), reads the file, runs `:wat::lint::lint-fix-file` (= lint-file → apply-fixes), and
;; writes it back ONLY if it changed (printing `[fixed] <path>`). Applies the auto-fixes that EXIST today:
;;   - nested-if-=-ladder      → (contains? (HashSet …) var)        (277.1b)
;;   - concat-abuse, bare-symbol slots → (format "…{name}…" :name …) (277.1c-fix)
;; Compound concat-abuse + everything report-only is left UNTOUCHED (fix = None) — naming there is a
;; judgment deferred to the arc-278 RETE map-consumer. So a file with only report-only findings is a no-op.
;;
;; Usage (derive the set from git, then sweep):
;;   printf '%s' "$(git ls-files 'wat/*.wat' 'wat-tests/*.wat' | sed 's/.*/"&"/' | tr '\n' ' ' | sed 's/^/[/;s/ $/]/')" \
;;     | ./target/release/wat ./wat-scripts/fixes/sweep-lint-fixes.wat
;;
;; Comment-faithful + idempotent (re-running after a clean sweep yields zero `[fixed]`).

(:wat::core::defn :user::sweep-file
  [path <- :wat::core::String] -> :wat::core::nil
  (:wat::core::let [src   (:wat::io::read-file path)
                    fixed (:wat::lint::lint-fix-file (:wat::source::File path src))]
    (:wat::core::if (:wat::core::= src fixed)
      nil
      (:wat::core::do
        (:wat::io::write-file path fixed)
        (:wat::kernel::println (:wat::core::string::concat "[fixed] " path))
        nil))))

(:wat::core::defn :user::sweep-each
  [paths <- :wat::core::Vector<wat::core::String>] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::Option/expect  
                            (:wat::core::first paths) "sweep-each: head")]
      (:wat::core::do
        (:user::sweep-file path)
        (:user::sweep-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::sweep-each
    (:wat::kernel::readln -> :wat::core::Vector<wat::core::String>)))
