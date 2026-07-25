;; wat/kernel/sandbox.wat — :wat::kernel::run-sandboxed and -ast.
;;
;; Restored from git history (eb655d1^:wat/std/sandbox.wat) in
;; arc 170 slice 1f-δ′. File location updated to wat/kernel/ per
;; arc 109 K-namespace doctrine.
;;
;; This file MUST be loaded AFTER wat/kernel/hermetic.wat. The
;; drain-lines-acc / drain-lines / failure-from-process-died helpers
;; are defined there (slice 1f-δ). Sandbox.wat reuses them without
;; redefinition.
;;
;; Reimplements the test-convenience `run-sandboxed` family on top
;; of the spawn-program (source-string) and spawn-process (AST forms)
;; substrates. run-sandboxed (source-string) still uses spawn-program;
;; run-sandboxed-ast (AST) uses spawn-process (arc 214 1b-ii-ζ.1).
;;
;; What this file replaces (arc 105c): the Rust primitives of the
;; same names in src/sandbox.rs, which collected stdin / stdout /
;; stderr as wat::core::Vector<String> buffers — buffer-in / buffer-out, no
;; interleaving, no back-pressure. The spawn substrates give real
;; kernel pipes; this file moves the test-convenience "collect output
;; to wat::core::Vector<String>" shape to the wat layer where it belongs.
;; wat::core::Vector<String> is the ASSERTION TARGET for tests, not a substrate
;; concern. Same surface (run-sandboxed src stdin scope → RunResult);
;; same return shape; same wat-test calls — only the mechanism changed.
;;
;; Failure path:
;; - spawn-program returns :wat::core::Result<:Process, :StartupError>.
;;   On (Err startup-err), startup-failure-result synthesizes a
;;   RunResult with empty stdout/stderr + Some(Failure) carrying the
;;   error message. Same shape the deleted substrate produced.
;; - spawn-process returns :Process directly; join-result returns
;;   :wat::core::Result<wat::core::nil, wat::core::Vector<ProcessDiedError>>.
;;   On (Err chain), drive-sandbox builds a Failure with
;;   failure-from-process-died extracting the panic or runtime-error
;;   message. Captured stdout/stderr (whatever the child wrote before
;;   dying) are preserved.
;;
;; Limitations as shipped (same as hermetic.wat):
;; - No concurrent drain of stdout vs stderr. Parent writes stdin,
;;   closes it, drains stdout to EOF, drains stderr to EOF. Works
;;   when the child's output fits in pipe buffers (typically 64KB
;;   per direction). A child that writes more than the buffer holds
;;   to one stream while the parent is draining the other could
;;   deadlock; not seen in any shipped test, follow-up substrate
;;   work when a caller needs it.

;; Build a Failure payload from a StartupError. Spawn-program
;; failures (parse, type-check, signature mismatch) flow through
;; here. :wat::kernel::StartupError is a struct with one field
;; (message :wat::core::String); the auto-generated accessor
;; StartupError/message extracts it.
(:wat::core::defn :wat::kernel::failure-from-startup [err <- :wat::kernel::StartupError] -> :wat::kernel::Failure
  (:wat::kernel::message-only-failure (:wat::core::string::concat
        "startup: " (:wat::kernel::StartupError/message err))))

;; Common driver — runs a Process (already spawned successfully),
;; pre-seeds stdin, closes the writer to signal EOF, drains
;; stdout / stderr, joins via Process/join-result. Returns the
;; canonical RunResult shape every test author matches against.
;; Always returns Ok-from-spawn shape; spawn-time failures are
;; handled before drive-sandbox runs (in the run-sandboxed
;; wrappers below).
;;
;; The seeded stdin is joined with '\n' between elements (no
;; trailing newline) — same convention the deleted Rust primitive
;; used. Empty wat::core::Vector<String> joins to the empty string; write-string
;; is a no-op on zero bytes; close still fires; child sees EOF on
;; first read-line.
(:wat::core::defn :wat::kernel::drive-sandbox<I,O> [proc <- :wat::kernel::Program<I,O> stdin <- :wat::core::Vector<wat::core::String>] -> :wat::kernel::RunResult
  ;; Outer scope: proc handle + join-result.  SERVICE-PROGRAMS.md § "The
    ;; lockstep": inner-let owns every output Receiver; when inner body
    ;; returns, stdout-r and stderr-r drop; drain threads see EOF; child
    ;; can exit; outer join-result unblocks cleanly.
    (:wat::core::let
      [stdin-w        (:wat::kernel::Process/stdin proc)
       joined         (:wat::core::string::join "\n" stdin)
       _n             (:wat::io::IOWriter/write-string stdin-w joined)
       _close         (:wat::io::IOWriter/close stdin-w)
       ;; Inner scope: output Receivers + drained lines.
       ;; Dropping stdout-r and stderr-r lets the child's OS pipes
       ;; drain to EOF before join.
       drain-pair
        (:wat::core::let
          [stdout-r      (:wat::kernel::Process/stdout proc)
           stderr-r      (:wat::kernel::Process/stderr proc)
           stdout-lines  (:wat::kernel::drain-lines stdout-r)
           stderr-lines  (:wat::kernel::drain-lines stderr-r)]
          (:wat::core::Tuple stdout-lines stderr-lines))
       ;; Arc 278 wave 2d — stdout is still DRAINED inside the inner let
       ;; (deadlock-avoidance: the child must not block on a full pipe),
       ;; but no longer bound out or stored: RunResult holds only failure.
       ;; stderr-lines stays — extract-panics recovers the panic chain from it.
       stderr-lines   (:wat::core::second drain-pair)
       ;; Inner scope has exited; Receivers dropped; child can exit.
       joined-result
        (:wat::kernel::Process/join-result proc)
       ;; Arc 113 slice 3 — symmetry with the thread cascade. When
       ;; the forked child panicked with an upstream-chain-bearing
       ;; AssertionPayload, fork.rs's emit_panics_to_stderr
       ;; rendered the chain as a tagged EDN line on stderr. The
       ;; substrate verb extract-panics walks stderr lines and
       ;; recovers the typed wat::core::Vector<ProcessDiedError>; we prefer it over
       ;; Process/join-result's singleton "exited N" shape.
       ;; Threads pass DiedError values directly through crossbeam;
       ;; processes pass them as EDN over kernel pipes; the chain at
       ;; the caller is identical regardless. Only the wire differs.
       stderr-chain
        (:wat::kernel::extract-panics stderr-lines)
       failure
        (:wat::core::match joined-result 
          ((:wat::core::Ok _)    :wat::core::None)
          ((:wat::core::Err err)
           (:wat::core::Some (:wat::kernel::failure-from-process-died
                   ;; spawn-process forks a real OS process; the panic chain
                   ;; comes through the subprocess stderr pipe as EDN.
                   ;; extract-panics recovers it; if absent, fall through to
                   ;; the join-result chain (err) which carries the full
                   ;; ProcessDiedError from the wait path.
                   (:wat::core::match stderr-chain
                     
                     ((:wat::core::Some chain) chain)
                     (:wat::core::None         err))))))]
      (:wat::core::struct-new :wat::kernel::RunResult
        failure)))

;; Build a RunResult that captures a startup-time spawn failure.
;; Empty stdout/stderr (the child never ran); failure carries the
;; StartupError message. Mirrors what the deleted substrate
;; eval_kernel_run_sandboxed did when startup_from_source returned
;; Err.
(:wat::core::defn :wat::kernel::startup-failure-result [err <- :wat::kernel::StartupError] -> :wat::kernel::RunResult
  (:wat::core::struct-new :wat::kernel::RunResult
      (:wat::core::Some (:wat::kernel::failure-from-startup err))))


;; --- :wat::kernel::run-sandboxed (source-string entry) ---
(:wat::core::defn :wat::kernel::run-sandboxed [src <- :wat::core::String stdin <- :wat::core::Vector<wat::core::String> scope <- :wat::core::Option<wat::core::String>] -> :wat::kernel::RunResult
  (:wat::core::match (:wat::kernel::spawn-program src scope)
      
      ((:wat::core::Ok proc)  (:wat::kernel::drive-sandbox proc stdin))
      ((:wat::core::Err err)  (:wat::kernel::startup-failure-result err))))


;; --- :wat::kernel::run-sandboxed-ast (AST entry) ---
;; Arc 214 1b-ii-ζ.1 — migrated to spawn-process (retired in-thread AST spawn).
;; spawn-process returns Process<I,O> directly (no Result wrapper); startup
;; errors flow through drive-sandbox's join-result Err arm. The scope
;; parameter is kept for API compatibility but is not forwarded (spawn-process
;; takes only the forms; scope-through-fork is a separate future arc).
(:wat::core::defn :wat::kernel::run-sandboxed-ast [forms <- :wat::core::Vector<wat::WatAST> stdin <- :wat::core::Vector<wat::core::String> scope <- :wat::core::Option<wat::core::String>] -> :wat::kernel::RunResult
  (:wat::kernel::drive-sandbox (:wat::kernel::spawn-process forms) stdin))
