;; wat-scripts/fixes/unignore-arc170-concurrency.wat — lift the arc-170 concurrency
;; suppression. Self-hosted fix-wat codemod: no hand-editing of .wat — wat rewrites wat.
;;
;; Deletes, span-faithfully, every top-level
;;
;;   (:wat::test::ignore "arc-170 concurrency layer (subprocess spawn / thread-on-channel)
;;                        — leaks/hangs; remove before arc 170 closes")
;;
;; leaving the `deftest` beneath it live.
;;
;; WHY. 29 of the corpus's 30 ignore markers carry that ONE reason, written when spawning
;; genuinely leaked and hung. Arc 278 has since annihilated the classes that caused it:
;; the recv'-raise-past-the-reader (R53), the send-side mask (R57), the masked-and-
;; deadlocked `:init` crash (R50), the eprintln no-stdio swallow and the harness's own
;; swallow (R55), and the stop protocol (R59). A wait on a dead peer now RETURNS
;; `Lost[cause]` instead of blocking — the shape that used to hang no longer has a form.
;;
;; The marker's own instruction is "remove before arc 170 closes", and the honest reading
;; is remove the MARKER, not the test: a suppressed test never announces that it started
;; working, so nobody re-checked. `wat/test.wat`'s vigilatum stamp already records this as
;; owed (circumspicere F4 — "the arc-170 ignore-removal gate is OWED a slow-head design
;; pass"). This codemod IS that pass; the run that follows it is the arbiter.
;;
;; ⚠ REASON-GATED, not head-gated. The 30th marker is unrelated —
;; "296-recapture-pending: lint-stdlib times out (>5s) after stone B" — and MUST survive.
;; ⊘ 2026-08-16: I briefly retired that 30th marker on an isolation measurement (1.826s) and
;;   the floor went RED — it exceeds the harness's 5000ms per-deftest limit under real floor
;;   contention, and its leaked thread took a second test down with it. The marker is BACK,
;;   with its reason restated to name the limit and to require any future unlock be measured
;;   UNDER CONTENTION. So this exclusion is still LIVE and still load-bearing: keep it.
;; So the rule fires only when the ignore's string argument contains the arc-170 phrase.
;; A head-only rule would silently lift a suppression this arc has no claim on.
;;
;; The deletion covers the form's own span ONLY. Surrounding whitespace and any adjacent
;; doc-comment survive byte-identical (the residual blank line is wat-fmt's job — never
;; eat a comment). Idempotent: once the form is gone there is nothing left to match.
;;
;; NOT A CLAIM THAT THEY PASS. This lifts the suppression so the corpus can be MEASURED.
;; Whatever reddens after it is a real finding — and per doctrine a hang is a defect to
;; locate, not an outcome to route around.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["wat-tests/test.wat" …]\n' | ./target/release/wat ./wat-scripts/fixes/unignore-arc170-concurrency.wat

;; arc170-ignore? — a List whose head keyword is `:wat::test::ignore` AND whose string
;; argument names the arc-170 concurrency suppression.
(:wat::core::defn :user::arc170-ignore? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::< (:wat::core::count ch) 2)
        false
        (:wat::core::let [head (:wat::core::first ch)
                          arg  (:wat::core::first (:wat::core::rest ch))]
          (:wat::core::if (:wat::core::= (:wat::core::ast-kind head) "keyword")
            (:wat::core::if (:wat::core::= (:wat::core::ast-name head) ":wat::test::ignore")
              (:wat::string::contains?
                (:wat::core::ast-name arg)
                "arc-170 concurrency layer")
              false)
            false))))
    false))

;; form-edits — 0-or-1 deletion edit for one top-level form: the whole ignore form.
(:wat::core::defn :user::form-edits
  [node  <- :wat::WatAST
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::if (:user::arc170-ignore? node)
    (:wat::core::let [off (:wat::fix::fix-text-offset-of (:wat::core::ast-span node) lines)
                      len (:wat::fix::fix-text-span-len
                            (:wat::core::ast-span node)
                            (:wat::core::ast-end-span node)
                            lines)]
      (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])
        (:wat::core::Tuple off len "")))
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))))

;; scan — collect edits across every top-level form (ascending offset order).
(:wat::core::defn :user::scan
  [forms <- (:wat::core::Vector :- [:wat::WatAST])
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::if (:wat::core::empty? forms)
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))
    (:wat::core::concat
      (:user::form-edits (:wat::core::first forms) lines)
      (:user::scan (:wat::core::rest forms) lines))))

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [lines     (:wat::string::split src "\n")
                    tree      (:wat::core::match (:wat::core::read-string src) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
                    forms     (:wat::core::ast->children tree)
                    all-edits (:user::scan forms lines)]
    (:wat::fix::fix-text-apply src (:wat::core::reverse all-edits))))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[unignore] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each (:wat::core::match (:wat::kernel::readln) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
