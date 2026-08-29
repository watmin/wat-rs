;; wat-tests/test.wat — self-tests for wat/test.wat.
;;
;; The test harness tests itself. Every assertion primitive gets both
;; a pass-case deftest (the assertion succeeds → deftest returns a
;; clean RunResult) and a fail-case deftest (run an inner program
;; that invokes the assertion with mismatched args, then inspect the
;; inner RunResult's Failure slot to verify the right diagnostic
;; surfaced).
;;
;; Inner programs use run-thread wrapping pass/fail-case assertions.
;; All active (non-ignored) tests use body-AST entry via deftest.
;; The arc-170-ignored tests preserve the hermetic-capture pattern
;; (run-hermetic + assert-stdout-is) for when concurrency is re-enabled.


;; ─── assert-eq — pass cases ───────────────────────────────────────────

(:wat::test::deftest :wat-tests::test::test-assert-eq-on-i64
  
  (:wat::test::assert-eq 42 42))

(:wat::test::deftest :wat-tests::test::test-assert-eq-on-strings
  
  (:wat::test::assert-eq "hello" "hello"))

(:wat::test::deftest :wat-tests::test::test-assert-eq-on-bools
  
  (:wat::test::assert-eq true true))

(:wat::test::deftest :wat-tests::test::test-assert-eq-on-vec
  
  (:wat::core::let
    [a (:wat::core::Vector :- [:wat::core::String] "x" "y")
     b (:wat::core::Vector :- [:wat::core::String] "x" "y")]
    (:wat::test::assert-eq a b)))

;; ─── assert-eq — fail case surfaces message ───────────────────────────


(:wat::test::deftest :wat-tests::test::test-assert-eq-fail-populates-message
  
  ;; arc 170 #13 — the IPC wall. This test observes a FAILING child, which is why it
  ;; used to hand-roll the harness (spawn-program + the self-peer closure + the
  ;; send'/recv' outcome matching) instead of using it. That duplication is no longer
  ;; needed: since the recv'-outcome wall (arc 278 R53/R55) `run-thread` RETURNS a
  ;; RunResult rather than crashing on a failing child, so the harness itself hands
  ;; back exactly what this test wants. `spawn-program` is now a capability restricted
  ;; to [:wat::spawn:: :wat::test::]; corpus tests reach it THROUGH the harness.
  (:wat::core::let
    [fail (:wat::core::match (:wat::test::run-thread (:wat::test::assert-eq 42 43))
            (:wat::kernel::RunResult::Passed :wat::core::None)
            ((:wat::kernel::RunResult::Failed f) (:wat::core::Some f)))]
    (:wat::core::match fail
      ((:wat::core::Some f) (:wat::test::assert-eq
                  (:wat::kernel::Failure/message f)
                  "assert-eq failed"))
      (:wat::core::None (:wat::kernel::assertion-failed!
               "expected Failure, got :None"
               :wat::core::None :wat::core::None)))))

;; ─── assert-contains — pass + fail ────────────────────────────────────

(:wat::test::deftest :wat-tests::test::test-assert-contains-hit
  
  (:wat::test::assert-contains "the quick brown fox" "quick"))


(:wat::test::deftest :wat-tests::test::test-assert-contains-fail-populates-actual
  
  ;; rune:complectens(embedded-program) — outer let has 2 bindings (p, fail); bulk is embedded-program AST literal (test fixture, not composition)
  ;; arc 278 IPC de-prime: run-thread → primed peer wire (spawn-program' :thread + recv').
  ;; The failing assert-contains crashes the self-peer → recv' Lost[cause];
  ;; LociDiedError/to-failure rebuilds the (Option :- [Failure]) (preserving actual/expected),
  ;; so the downstream match on `fail` is unchanged.
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::thread)
         (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
           (:wat::core::do
             (:wat::test::assert-contains "hello" "xyz")
             (:wat::core::match (:wat::kernel::send self 0)
               (:wat::kernel::SendOutcome::Sent   nil)
               (:wat::kernel::SendOutcome::Closed nil)
               ;; arc 278 #73 — same body as Sent/Closed: this send-outcome wall just
               ;; needs to proceed regardless; the failing assertion above already
               ;; panicked before this line could even run.
               (:wat::kernel::SendOutcome::Stopped nil)
               ((:wat::kernel::SendOutcome::Lost _c) nil)))))
     fail (:wat::core::match (:wat::kernel::recv p)
            ((:wat::kernel::RecvOutcome::Message _m) :wat::core::None)
            ((:wat::kernel::RecvOutcome::Lost cause) (:wat::core::Some (:wat::kernel::LociDiedError/to-failure cause)))
            ;; arc 278 #73 — a stop is neither the failure this file exists to verify
            ;; nor a clean pass; assert it distinctly rather than fold it into either
            ;; :None (Closed's meaning here) or :Some (Lost's meaning here).
            (:wat::kernel::RecvOutcome::Stopped
              (:wat::kernel::assertion-failed!
                "stopped — the substrate was asked to stop; the thread was ALIVE and the channel open"
                :wat::core::None :wat::core::None))
            (:wat::kernel::RecvOutcome::Closed :wat::core::None))]
    (:wat::core::match fail  
      ((:wat::core::Some f)
        (:wat::core::let
          [actual (:wat::kernel::Failure/actual f)
           expected (:wat::kernel::Failure/expected f)
           _
            (:wat::core::match actual  
              ((:wat::core::Some a) (:wat::test::assert-eq a "hello"))
              (:wat::core::None (:wat::kernel::assertion-failed!
                       "actual slot empty" :wat::core::None :wat::core::None)))]
          (:wat::core::match expected  
            ((:wat::core::Some e) (:wat::test::assert-eq e "xyz"))
            (:wat::core::None (:wat::kernel::assertion-failed!
                     "expected slot empty" :wat::core::None :wat::core::None)))))
      (:wat::core::None (:wat::kernel::assertion-failed!
               "expected Failure, got :None" :wat::core::None :wat::core::None)))))

;; ─── assert-coincident — pass + fail-renders-explanation ─────────────

(:wat::test::deftest :wat-tests::test::test-assert-coincident-pass
  
  (:wat::test::assert-coincident
    (:wat::holon::to-holon "alice")
    (:wat::holon::to-holon "alice")))

;; The fail-side test exercises arc 069's wiring: when the assertion
;; fails, the rendered CoincidentExplanation lands in the failure
;; payload's `actual` slot. We grep for each named field; their
;; presence is what matters, not exact numeric values (those depend
;; on the encoder's d at run time).

(:wat::test::deftest :wat-tests::test::test-assert-coincident-fail-renders-explanation
  
  ;; rune:complectens(embedded-program) — outer let has 2 bindings (p, fail); bulk is embedded-program AST literal (test fixture, not composition)
  ;; arc 278 IPC de-prime: run-thread → primed peer wire (spawn-program' :thread + recv').
  ;; The failing assert-coincident crashes the self-peer → recv' Lost[cause];
  ;; LociDiedError/to-failure rebuilds the (Option :- [Failure]) (preserving the rendered
  ;; explanation in `actual`), so the downstream match on `fail` is unchanged.
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::thread)
         (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
           (:wat::core::do
             (:wat::test::assert-coincident
               (:wat::holon::to-holon "alice")
               (:wat::holon::to-holon "charlie"))
             (:wat::core::match (:wat::kernel::send self 0)
               (:wat::kernel::SendOutcome::Sent   nil)
               (:wat::kernel::SendOutcome::Closed nil)
               ;; arc 278 #73 — same body as Sent/Closed: this send-outcome wall just
               ;; needs to proceed regardless; the failing assertion above already
               ;; panicked before this line could even run.
               (:wat::kernel::SendOutcome::Stopped nil)
               ((:wat::kernel::SendOutcome::Lost _c) nil)))))
     fail (:wat::core::match (:wat::kernel::recv p)
            ((:wat::kernel::RecvOutcome::Message _m) :wat::core::None)
            ((:wat::kernel::RecvOutcome::Lost cause) (:wat::core::Some (:wat::kernel::LociDiedError/to-failure cause)))
            ;; arc 278 #73 — a stop is neither the failure this file exists to verify
            ;; nor a clean pass; assert it distinctly rather than fold it into either
            ;; :None (Closed's meaning here) or :Some (Lost's meaning here).
            (:wat::kernel::RecvOutcome::Stopped
              (:wat::kernel::assertion-failed!
                "stopped — the substrate was asked to stop; the thread was ALIVE and the channel open"
                :wat::core::None :wat::core::None))
            (:wat::kernel::RecvOutcome::Closed :wat::core::None))]
    (:wat::core::match fail  
      ((:wat::core::Some f)
        (:wat::core::let
          [actual (:wat::kernel::Failure/actual f)]
          (:wat::core::match actual  
            ((:wat::core::Some a)
              (:wat::core::do
                (:wat::test::assert-contains a "cosine")
                (:wat::test::assert-contains a "floor")
                (:wat::test::assert-contains a "dim")
                (:wat::test::assert-contains a "sigma")
                (:wat::test::assert-contains
                            a "min-sigma-to-pass")
                nil))
            (:wat::core::None (:wat::kernel::assertion-failed!
                     "actual slot empty — explanation should populate it"
                     :wat::core::None :wat::core::None)))))
      (:wat::core::None (:wat::kernel::assertion-failed!
               "expected Failure, got :None" :wat::core::None :wat::core::None)))))

;; ─── assert-stdout-is — pass case ─────────────────────────────────────


(:wat::test::deftest-hermetic :wat-tests::test::test-assert-stdout-is-matches
  
  ;; arc 278 IPC de-prime: run-hermetic → primed peer wire (spawn-program' :process + recv').
  ;; On the wire each printed value crosses DECODED (native String "alpha"/"beta"), not a
  ;; scraped EDN stdout line ("\"alpha\""); the old assert-stdout-is over captured lines
  ;; becomes assert-eq over the two received Messages. The trailing nil returns nil → Closed.
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::core::do
               (:wat::kernel::println "alpha")
               (:wat::kernel::println "beta")
               nil))))
     m1 (:wat::core::match (:wat::kernel::recv p)
          ((:wat::kernel::RecvOutcome::Message m) m)
          ((:wat::kernel::RecvOutcome::Lost cause)
            (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Stopped
            (:wat::kernel::assertion-failed! "assert-stdout-is-matches: stopped before first line — the child was ALIVE" :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Closed
            (:wat::kernel::assertion-failed! "assert-stdout-is-matches: child closed before first line" :wat::core::None :wat::core::None)))
     m2 (:wat::core::match (:wat::kernel::recv p)
          ((:wat::kernel::RecvOutcome::Message m) m)
          ((:wat::kernel::RecvOutcome::Lost cause)
            (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Stopped
            (:wat::kernel::assertion-failed! "assert-stdout-is-matches: stopped before second line — the child was ALIVE" :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Closed
            (:wat::kernel::assertion-failed! "assert-stdout-is-matches: child closed before second line" :wat::core::None :wat::core::None)))]
    (:wat::core::do
      (:wat::test::assert-eq m1 "alpha")
      (:wat::test::assert-eq m2 "beta"))))

;; ─── run-hermetic-with-prelude — proof of capability (arc 170 slice 6) ──
;;
;; Arc 170 slice 6 minted the new `:wat::test::run-hermetic-with-prelude`
;; macro that exposes the program's prelude slot (the substrate's new
;; spawn-process program shape lets the caller construct any wat program
;; — config setters, type declarations, helper defines — preceding the
;; entry-point `(:user::main -> :nil)` define).
;;
;; (arc 278 — the prelude-proof deftest is RETIRED with the prelude feature it proved:
;; `run-hermetic-with-prelude` and the prelude slot are annihilated. Preludes existed to
;; hoist declarations into a test's world (originally shared vocab via load-file!, later
;; local type-decls); that need is gone — thread decls live at file top-level, hermetic
;; check-cases ride inline in the child's opaque forms. Nothing left to prove.)

;; ─── assert-stderr-matches — pass + fail-reports-pattern ──────────────
;;
;; Arc 278 no-hidden-failures — eprintln is now a TERMINATING form: the inner
;; child writes its line to stderr, then crashes (the #wat.kernel/ProcessPanics
;; envelope follows). assert-stderr-matches is unanchored per-line, so a pattern
;; chosen to match (or, in the fail case, NOT match) the emitted line behaves
;; identically whether or not the trailing crash envelope is present.


(:wat::test::deftest-hermetic :wat-tests::test::test-assert-stderr-matches-pass
  
  ;; arc 278 IPC de-prime: run-hermetic → primed peer wire (spawn-program' :process + recv').
  ;; eprintln is a TERMINAL (dying) form — the child crashes; recv' → Lost[Panic] whose
  ;; message carries the emitted value's EDN. assert-stderr-matches (a regex OR-fold over
  ;; captured lines) becomes a single regex match against that one crossed line.
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::kernel::eprintln "error: code 42"))))
     msg (:wat::core::match (:wat::kernel::recv p)
           ((:wat::kernel::RecvOutcome::Message _m)
             (:wat::kernel::assertion-failed! "assert-stderr-matches-pass: expected Lost[Panic], got Message" :wat::core::None :wat::core::None))
           ((:wat::kernel::RecvOutcome::Lost cause)
             (:wat::core::match cause
               ((:wat::kernel::LociDiedError::Panic message _failure) message)
               (_ (:wat::kernel::assertion-failed! "assert-stderr-matches-pass: expected Lost[Panic], got other Lost" :wat::core::None :wat::core::None))))
           (:wat::kernel::RecvOutcome::Stopped
             (:wat::kernel::assertion-failed! "assert-stderr-matches-pass: expected Lost[Panic], got Stopped" :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Closed
             (:wat::kernel::assertion-failed! "assert-stderr-matches-pass: expected Lost[Panic], got Closed" :wat::core::None :wat::core::None)))]
    (:wat::test::assert-true (:wat::regex::matches? "code [0-9]+" msg))))

;; :wat-tests::test::test-assert-stderr-matches-fail-reports-pattern
;; DELETED (arc 278 wave 2d) — it existed solely to verify
;; :wat::test::assert-stderr-matches's failure-reporting shape (rebuilding a
;; RunResult with a stderr Vec, then asserting the pattern lands in the
;; Failure `expected` slot). assert-stderr-matches is gone with the
;; stdout/stderr capture fields; there is no stderr to match. Concern retired.

;; ─── hermetic-capture pattern (arc-170-ignored; original intent retired) ──
;;
;; The tests below originally exercised the legacy string-entry path
;; (:wat::test::run) and the AST-entry path (:wat::test::run-ast via
;; :wat::test::program). Both paths were swept to canonical macros
;; during arc 170 slice 4a-β; these tests now verify the simpler
;; hermetic-child-prints-parent-captures pattern and are preserved
;; per accumulate-tests-defer-cleanup policy (cleanup is post-109).

;; Duplicate of :wat-tests::test::test-assert-stdout-is-matches at line 132 —
;; same hermetic-print-and-capture pattern with different fixture string. Preserved
;; per accumulate-tests-defer-cleanup policy (test cleanup is post-109; coverage
;; tooling needed to verify safe deletion). Original test purpose
;; ("test the legacy STRING-entry path") retired during arc 170 slice 4a-β
;; when the legacy :wat::test::run path was swept to canonical macros.

(:wat::test::deftest-hermetic :wat-tests::test::test-run-string-entry-path
  
  ;; Arc 170 slice 4a-β: this test originally exercised the legacy
  ;; :wat::test::run STRING-parsing path; the inner source carried a
  ;; (:wat::config::set-capacity-mode! :error) form that the legacy
  ;; substrate config-collected. The modern body-AST shape has no
  ;; analogue — config-setters are file-level, not body-runtime forms.
  ;; The test now verifies the simpler post-migration shape: hermetic
  ;; child prints, parent captures stdout. The original "STRING-path
  ;; tested" intent retires with the legacy :wat::test::run define.
  ;; arc 278 IPC de-prime: run-hermetic → primed peer wire. The printed value crosses
  ;; DECODED (native String "from-string"), so the old assert-stdout-is over a captured
  ;; EDN line becomes assert-eq over the received Message.
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::kernel::println "from-string"))))
     msg (:wat::core::match (:wat::kernel::recv p)
           ((:wat::kernel::RecvOutcome::Message m) m)
           ((:wat::kernel::RecvOutcome::Lost cause)
             (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Stopped
             (:wat::kernel::assertion-failed! "run-string-entry-path: stopped before the child sent its value — the child was ALIVE" :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Closed
             (:wat::kernel::assertion-failed! "run-string-entry-path: child closed before sending its value" :wat::core::None :wat::core::None)))]
    (:wat::test::assert-eq msg "from-string")))

;; Duplicate of :wat-tests::test::test-assert-stdout-is-matches at line 132 —
;; same hermetic-print-and-capture pattern with different fixture string. Preserved
;; per accumulate-tests-defer-cleanup policy (test cleanup is post-109; coverage
;; tooling needed to verify safe deletion). Original test purpose
;; ("test the legacy AST-via-program path") retired during arc 170 slice 4a-β
;; when the legacy :wat::test::run-ast path was swept to canonical macros.

(:wat::test::deftest-hermetic :wat-tests::test::test-run-ast-via-program
  
  ;; arc 278 IPC de-prime: run-hermetic → primed peer wire. The printed value crosses
  ;; DECODED (native String "from-ast"), so the old assert-stdout-is over a captured EDN
  ;; line becomes assert-eq over the received Message.
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::kernel::println "from-ast"))))
     msg (:wat::core::match (:wat::kernel::recv p)
           ((:wat::kernel::RecvOutcome::Message m) m)
           ((:wat::kernel::RecvOutcome::Lost cause)
             (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Stopped
             (:wat::kernel::assertion-failed! "run-ast-via-program: stopped before the child sent its value — the child was ALIVE" :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Closed
             (:wat::kernel::assertion-failed! "run-ast-via-program: child closed before sending its value" :wat::core::None :wat::core::None)))]
    (:wat::test::assert-eq msg "from-ast")))

;; deftest's self-test is redundant here — every other passing deftest
;; in this file IS proof that deftest registered a callable zero-arg
;; :wat::kernel::RunResult-returning function, because `wat test`
;; discovered them by exactly that signature and invoked them
;; (signature-only discovery; the legacy `test-` last-segment filter
;; was dropped 2026-04-25). If deftest were broken, this whole file
;; would fail at discovery / startup, not one test.

;; ─── :wat::test::make-deftest — arc 029 slice 2 ──────────────────────
;;
;; Configured-deftest factory. The preamble registers an ambient
;; name; subsequent callsites are just name + body. Proves the
;; macro-generating-macro path end-to-end: outer make-deftest
;; expands to a defmacro registration, the generated defmacro
;; expands to a deftest call, the deftest expands to the full
;; run-sandboxed-ast scaffolding, and the test runs.



(:wat::test::deftest
  :wat-tests::test::test-make-deftest-runs
  (:wat::test::assert-eq (:wat::i64::+ 2 2) 4))

(:wat::test::deftest
  :wat-tests::test::test-make-deftest-second-test
  (:wat::test::assert-eq 10 (:wat::i64::* 5 2)))

;; ─── :wat::core::macroexpand / macroexpand-1 — arc 030 ────────────────
;;
;; The standard Lisp macro-debugging tool. Quote a form, hand it to
;; macroexpand(-1), inspect the returned AST. Lets users see what a
;; macro call produces without evaluating it.

(:wat::test::deftest :wat-tests::test::test-macroexpand-1-non-macro
  
  ;; A plain expression (no macro head) expands to itself. Verify by
  ;; evaluating the expanded AST and checking it produces Ok.
  (:wat::core::match
    (:wat::eval-ast!
      (:wat::core::macroexpand-1
        (:wat::core::quote (:wat::i64::+ 2 2))))
     
    ((:wat::core::Ok _) (:wat::test::assert-eq true true))
    ((:wat::core::Err _) (:wat::test::assert-eq true false))))

(:wat::test::deftest :wat-tests::test::test-macroexpand-fixpoint-evaluates
  
  ;; macroexpand returns a :wat::WatAST; hand it to eval-ast!
  ;; to prove the expansion is evaluable.
  (:wat::core::match
    (:wat::eval-ast!
      (:wat::core::macroexpand
        (:wat::core::quote (:wat::i64::* 3 4))))
     
    ((:wat::core::Ok _) (:wat::test::assert-eq true true))
    ((:wat::core::Err _) (:wat::test::assert-eq true false))))

;; ─── Substrate primitives — public sandbox-entry verbs ANNIHILATED ───
;;
;; Arc 170 CULMINATION (arc 278 IPC de-prime): the :wat::test::run /
;; run-in-scope / run-ast wrappers over the :wat::kernel::run-sandboxed
;; family were the manual "drive the sandbox by hand" surface. That
;; family is annihilated (subsumed by spawn-program' + recv'), so the
;; test-run-ast-direct witness demoing run-ast is deleted with it — its
;; "run a forms-program + check outcome" coverage is fully carried by
;; the primed-peer tests (tests/kernel/wat_run_sandboxed*.wat et al.).
