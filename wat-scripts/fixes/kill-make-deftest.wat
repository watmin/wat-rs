;; wat-scripts/fixes/kill-make-deftest.wat — arc 278, the make-deftest annihilation (class-1).
;; Self-hosted fix-wat codemod: no hand-editing of .wat — wat rewrites wat.
;;
;; After the prelude slot was annihilated (drop-deftest-prelude.wat), the
;; `make-deftest` / `make-deftest-hermetic` factory macros became pure alias
;; shells: `(:wat::test::make-deftest :X)` just makes `:X` a synonym for
;; `:wat::test::deftest` — it bakes nothing. This codemod kills the factory:
;;
;;   (a) DROP every top-level registration form whose head keyword is
;;       :wat::test::make-deftest OR :wat::test::make-deftest-hermetic
;;       (span-delete the WHOLE form, ast-span → ast-end-span). Done on the
;;       ORIGINAL parsed tree so the spans are valid.
;;   (b) THEN, on the drop-result string, rewrite each file-local alias
;;       call-head to the prime directly:
;;         :deftest :deftest-hcs :deftest-lru :my-deftest
;;         :wat-tests::std::test::cfg-deftest   ->  :wat::test::deftest'
;;       via rename-keyword-exact (whole-token, idempotent by construction).
;;
;; ORDER (a)-before-(b) is REQUIRED: the `:deftest` inside the registration
;; `(make-deftest :deftest)` is dropped in (a), so only real `(:deftest …)`
;; call-heads survive to be renamed in (b). A file lacking a given alias
;; yields 0 changes for that rename — safe to chain them all unconditionally.
;;
;; Aliases → prime (grounded from the corpus, NOT the brief):
;;   :deftest       (service-template, ReciprocalLog, roundtrip, HologramCache)
;;   :deftest-hcs   (HologramCacheService)
;;   :deftest-lru   (CacheService)
;;   :my-deftest    (tests/macros/make_deftest.wat)
;;   :wat-tests::std::test::cfg-deftest  (wat-tests/test.wat — 2 LIVE callsites;
;;     the brief mislabelled this as :cfg-deftest / 0 uses. Confirmed 279,283.)
;;
;; DO NOT run on wat/test.wat — it DEFINES the two factory macros; those are
;; deleted by hand (they cannot be span-dropped as call forms). Pass only
;; the caller files.
;;
;; IDEMPOTENT: after a run, no make-deftest head remains to drop, and each
;; rename fired to `:wat::test::deftest'` (!= the old alias) so a re-run
;; matches nothing. Re-run == 0 changes.
;;
;; Usage (one EDN vector of caller paths on stdin):
;;   printf '["wat-tests/..." …]\n' | ./target/release/wat ./wat-scripts/fixes/kill-make-deftest.wat

;; make-deftest-head? — a List whose head keyword is one of the two factories.
(:wat::core::defn :user::make-deftest-head? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch)
        false
        (:wat::core::let [head (:wat::core::first ch)]
          (:wat::core::if (:wat::core::= (:wat::core::ast-kind head) "keyword")
            (:wat::fix::str-in? (:wat::core::ast-name head)
              (:wat::core::Vector :wat::core::String
                ":wat::test::make-deftest"
                ":wat::test::make-deftest-hermetic"))
            false))))
    false))

;; form-edits — 0-or-1 whole-form deletion edit for one top-level form:
;; fires only when the head is a make-deftest factory. Deletes the entire
;; form span [ast-span .. ast-end-span); surrounding whitespace survives.
(:wat::core::defn :user::form-edits
  [node  <- :wat::WatAST
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::if (:user::make-deftest-head? node)
    (:wat::core::let [off (:wat::fix::fix-text-offset-of (:wat::core::ast-span node) lines)
                      len (:wat::fix::fix-text-span-len
                            (:wat::core::ast-span node)
                            (:wat::core::ast-end-span node)
                            lines)]
      (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])
        (:wat::core::Tuple off len "")))
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))))

;; scan — collect drop edits across every top-level form (ascending offset).
(:wat::core::defn :user::scan
  [forms <- (:wat::core::Vector :- [:wat::WatAST])
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::if (:wat::core::empty? forms)
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))
    (:wat::core::concat
      (:user::form-edits (:wat::core::first forms) lines)
      (:user::scan (:wat::core::rest forms) lines))))

;; drop-registrations — (a): span-delete every make-deftest factory form.
(:wat::core::defn :user::drop-registrations [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [lines     (:wat::core::string::split src "\n")
                    tree      (:wat::core::match (:wat::core::read-string src) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
                    forms     (:wat::core::ast->children tree)
                    all-edits (:user::scan forms lines)]
    (:wat::fix::fix-text-apply src (:wat::core::reverse all-edits))))

;; rename-aliases — (b): each file-local alias call-head → the prime.
(:wat::core::defn :user::rename-aliases [src <- :wat::core::String] -> :wat::core::String
  (:wat::fix::rename-keyword-exact ":wat-tests::std::test::cfg-deftest" ":wat::test::deftest'"
    (:wat::fix::rename-keyword-exact ":my-deftest" ":wat::test::deftest'"
      (:wat::fix::rename-keyword-exact ":deftest-lru" ":wat::test::deftest'"
        (:wat::fix::rename-keyword-exact ":deftest-hcs" ":wat::test::deftest'"
          (:wat::fix::rename-keyword-exact ":deftest" ":wat::test::deftest'" src))))))

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:user::rename-aliases (:user::drop-registrations src)))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::core::string::concat "[kill-make-deftest] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each (:wat::core::match (:wat::kernel::readln) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
