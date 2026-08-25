;; wat-scripts/fixes/strip-useless-mains.wat — codemod: find + remove useless placeholder
;; :main defns from co-located .wat test fixtures. Driven by an EDN vector of .wat paths
;; on stdin (the established codemod shape — see sweep-lint-fixes.wat).
;;
;; A "useless main" is a TOP-LEVEL form matching ALL of:
;;   1. head keyword == :wat::core::defn
;;   2. function name is a keyword whose last segment is `main`
;;      (e.g. :user::main, :t::main, :my::main — any namespace)
;;   3. the param vector is EMPTY ([])
;;   4. return type is :wat::core::nil and body is exactly the literal nil
;;
;; I.e., the exact form: (:wat::core::defn <kw-ending-::main> [] -> :wat::core::nil nil)
;;
;; TOP-LEVEL forms only — embedded mains inside (:wat::core::forms …) blocks (spawned-child
;; entrypoints) are NOT touched; this property comes for free from the grep primitive's
;; top-level-only walk.
;;
;; Two behaviours in one pass:
;;   REPORT  [useless-main] <path>:<line> <name>  — for every match found
;;   STRIP   [stripped] <path>                     — only if the file was modified
;;
;; Uses the general :wat::fix::wat-grep + :wat::fix::wat-grep-strip primitives from
;; wat-scripts/lib/wat-grep.wat. Future predicates (calls-to-X, defns-named-Y, …)
;; and actions plug in as one-liners over the same library.
;;
;; Usage:
;;   printf '["/tmp/test.wat"]' \
;;     | cargo wat ./wat-scripts/fixes/strip-useless-mains.wat
;;
;;   # Derive full corpus from git and sweep:
;;   printf '%s' "$(git ls-files 'tests/**/*.wat' | sed 's/.*/"&"/' | tr '\n' ' ' \
;;     | sed 's/^/[/;s/ $/]/')" \
;;     | cargo wat ./wat-scripts/fixes/strip-useless-mains.wat

(:wat::load-file! "../lib/wat-grep.wat")

;; ── Predicate: exactly the trivial-nil :main defn ────────────────────────────────────
;;
;; useless-main? — true iff node is a list of exactly 6 children:
;;   [0] keyword :wat::core::defn
;;   [1] keyword ending with ::main   (the function name)
;;   [2] empty vector                 (the param list)
;;   [3] symbol ->                    (return-type arrow)
;;   [4] keyword :wat::core::nil      (return type)
;;   [5] nil literal                  (the body)
;; No other forms are matched — real bodies, non-nil return types, non-empty params, all safe.
(:wat::core::defn :user::useless-main? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch)
        false
        (:wat::core::let [c0 (:wat::core::first ch)
                          r1 (:wat::core::rest ch)]
          (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::ast-kind c0) "keyword")
                            (:wat::core::= (:wat::core::ast-name c0) ":wat::core::defn")
                            false)
            (:wat::core::if (:wat::core::empty? r1)
              false
              (:wat::core::let [c1 (:wat::core::first r1)
                                r2 (:wat::core::rest r1)]
                (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::ast-kind c1) "keyword")
                                  (:wat::string::ends-with? (:wat::core::ast-name c1) "::main")
                                  false)
                  (:wat::core::if (:wat::core::empty? r2)
                    false
                    (:wat::core::let [c2 (:wat::core::first r2)
                                      r3 (:wat::core::rest r2)]
                      (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::ast-kind c2) "vector")
                                        (:wat::core::empty? (:wat::core::ast->children c2))
                                        false)
                        (:wat::core::if (:wat::core::empty? r3)
                          false
                          (:wat::core::let [c3 (:wat::core::first r3)
                                            r4 (:wat::core::rest r3)]
                            (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::ast-kind c3) "symbol")
                                              (:wat::core::= (:wat::core::ast-name c3) "->")
                                              false)
                              (:wat::core::if (:wat::core::empty? r4)
                                false
                                (:wat::core::let [c4 (:wat::core::first r4)
                                                  r5 (:wat::core::rest r4)]
                                  (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::ast-kind c4) "keyword")
                                                    (:wat::core::= (:wat::core::ast-name c4) ":wat::core::nil")
                                                    false)
                                    (:wat::core::if (:wat::core::empty? r5)
                                      false
                                      (:wat::core::let [c5 (:wat::core::first r5)
                                                        r6 (:wat::core::rest r5)]
                                        (:wat::core::if (:wat::core::= (:wat::core::ast-kind c5) "nil")
                                          (:wat::core::empty? r6)
                                          false)))
                                    false)))
                              false)))
                        false)))
                  false)))
            false))))
    false))

;; ── Predicate: any top-level defn (for the sole-defn guard) ──────────────────────────
;;
;; top-level-defn? — true iff node is a list whose head keyword is :wat::core::defn.
;; Used to count total top-level defns so a useless main that is the SOLE defn (the
;; arc-170 "main-AS-SUBJECT" fixture) is never stripped.
(:wat::core::defn :user::top-level-defn? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch)
        false
        (:wat::core::let [c0 (:wat::core::first ch)]
          (:wat::core::if (:wat::core::= (:wat::core::ast-kind c0) "keyword")
            (:wat::core::= (:wat::core::ast-name c0) ":wat::core::defn")
            false))))
    false))

;; ── Report one match: [useless-main] path:line name ──────────────────────────────────
(:wat::core::defn :user::report-match
  [path  <- :wat::core::String
   match <- :wat::WatAST] -> :wat::core::nil
  (:wat::core::let [span (:wat::core::ast-span match)
                    line (:wat::core::Option/expect
                           (:wat::core::HashMap/get span :line)
                           "report-match: :line")
                    ch   (:wat::core::ast->children match)
                    name (:wat::core::ast-name (:wat::core::first (:wat::core::rest ch)))]
    (:wat::kernel::println
      (:wat::string::concat "[useless-main] " path ":" (:wat::core::i64::to-string line) " " name))))

;; ── Report all matches for a file ────────────────────────────────────────────────────
(:wat::core::defn :user::report-matches
  [path    <- :wat::core::String
   matches <- (:wat::core::Vector :- [:wat::WatAST])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? matches)
    nil
    (:wat::core::do
      (:user::report-match path (:wat::core::first matches))
      (:user::report-matches path (:wat::core::rest matches)))))

;; ── Skip-set: multi-defn fixtures whose :main is nonetheless load-bearing ────────────
;;
;; The sole-defn guard below catches the arc-170 main-AS-SUBJECT case generically. This
;; explicit set is the escape hatch for a MULTI-defn fixture whose main is still the
;; test's subject (the guard alone would strip it). Empty today — no such fixture found;
;; kept as the documented extension point per the disposition doctrine.
(:wat::core::defn :user::skip-file? [path <- :wat::core::String] -> :wat::core::bool
  false)

;; ── Per-file: report matches + GUARDED strip + write back if changed ─────────────────
;;
;; GUARD (sole-defn): never strip a useless main that is the SOLE top-level defn — such a
;; file is a main-AS-SUBJECT fixture (arc-170 main-contract tests), not dead weight. Only
;; strip when at least one OTHER (non-useless-main) top-level defn remains:
;;     keep = (total top-level defns) − (useless-main matches);  strip only if keep >= 1.
(:wat::core::defn :user::process-file [path <- :wat::core::String] -> :wat::core::nil
  (:wat::core::if (:user::skip-file? path)
    nil
    (:wat::core::let [src         (:wat::io::read-file path)
                      matches     (:user::wat-grep src :user::useless-main?)
                      total-defns (:wat::core::length
                                    (:user::wat-grep src :user::top-level-defn?))
                      n-useless   (:wat::core::length matches)
                      keep        (:wat::core::- total-defns n-useless)]
      (:wat::core::if (:wat::core::< keep 1)
        ;; sole-defn (or all-defns-are-useless-mains) → main is the subject; strip NOTHING
        nil
        (:wat::core::do
          (:user::report-matches path matches)
          (:wat::core::let [stripped (:user::wat-grep-strip src :user::useless-main?)]
            (:wat::core::if (:wat::core::= src stripped)
              nil
              (:wat::core::do
                (:wat::io::write-file path stripped)
                (:wat::kernel::println (:wat::string::concat "[stripped] " path))
                nil))))))))

;; ── Walk all paths from the stdin EDN vector ─────────────────────────────────────────
(:wat::core::defn :user::process-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::do
      (:user::process-file (:wat::core::first paths))
      (:user::process-each (:wat::core::rest paths)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::process-each
    (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
