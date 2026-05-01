;; wat/std/sandbox.wat — :wat::kernel::run-sandboxed and -ast.
;;
;; Reimplements the test-convenience `run-sandboxed` family on top
;; of the arc 103a spawn-program substrate + arc 105a's Result-
;; returning failure-as-data shape + arc 105b's
;; ThreadDiedError/message accessor.
;;
;; What this file replaces (arc 105c): the Rust primitives of the
;; same names in src/sandbox.rs, which collected stdin / stdout /
;; stderr as wat::core::Vector<String> buffers — buffer-in / buffer-out, no
;; interleaving, no back-pressure. The arc 103a spawn-program
;; substrate gives real kernel pipes; this file moves the test-
;; convenience "collect output to wat::core::Vector<String>" shape to the wat
;; layer where it belongs. wat::core::Vector<String> is the ASSERTION TARGET
;; for tests, not a substrate concern. Same surface (run-sandboxed
;; src stdin scope → RunResult); same return shape; same wat-test
;; calls — only the mechanism changed.
;;
;; Failure path:
;; - spawn-program returns :wat::core::Result<:Process, :StartupError>.
;;   On (Err startup-err), drive-sandbox synthesizes a RunResult
;;   with empty stdout/stderr + Some(Failure) carrying the error
;;   message. Same shape the deleted substrate produced.
;; - join-result returns :wat::core::Result<wat::core::unit, :ThreadDiedError> after a
;;   successful spawn. On (Err thread-died), drive-sandbox builds
;;   a Failure with ThreadDiedError/message extracting the panic
;;   or runtime-error message regardless of variant. Captured
;;   stdout/stderr (whatever the child wrote before dying) are
;;   preserved.
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
;; here.
(:wat::core::define
  (:wat::kernel::failure-from-startup
    (err :wat::kernel::StartupError)
    -> :wat::kernel::Failure)
  (:wat::core::struct-new :wat::kernel::Failure
    (:wat::core::string::concat
      "startup: " (:wat::kernel::StartupError/message err))
    :None
    (:wat::core::Vector :wat::kernel::Frame)
    :None
    :None))

;; Build a Failure payload from a ProcessDiedError chain (arc 112 +
;; 113). Arc 113 widened the Err arm to wat::core::Vector<PDE> so cascading
;; failures carry the full chain; this helper takes the chain and
;; renders the HEAD's structured Failure (the immediate peer that
;; died). Future-arc consumers can walk the tail via
;; (:wat::core::rest chain) — :wat::core::first gives the head.
;; The substrate accessor :wat::kernel::ProcessDiedError/to-failure
;; preserves arc 064's structured actual / expected / location /
;; frames when the panic carried an AssertionPayload; falls back
;; to a message-only Failure for plain panics, runtime errors, and
;; the unit ChannelDisconnected variant.
(:wat::core::define
  (:wat::kernel::failure-from-process-died
    (chain :wat::core::Vector<wat::kernel::ProcessDiedError>)
    -> :wat::kernel::Failure)
  (:wat::core::match (:wat::core::first chain)
    -> :wat::kernel::Failure
    ((Some err) (:wat::kernel::ProcessDiedError/to-failure err))
    (:None
     ;; Empty chain — should not occur; substrate always emits at
     ;; least the immediate-peer death. Defensive default.
     (:wat::core::struct-new :wat::kernel::Failure
       "empty died-chain (substrate bug)"
       :None
       (:wat::core::Vector :wat::kernel::Frame)
       :None
       :None))))

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
(:wat::core::define
  (:wat::kernel::drive-sandbox<I,O>
    (proc  :wat::kernel::Program<I,O>)
    (stdin :wat::core::Vector<wat::core::String>)
    -> :wat::kernel::RunResult)
  (:wat::core::let*
    (((stdin-w :wat::io::IOWriter)   (:wat::kernel::Process/stdin proc))
     ((joined  :wat::core::String)              (:wat::core::string::join "\n" stdin))
     ((_n      :wat::core::i64)                 (:wat::io::IOWriter/write-string stdin-w joined))
     ((_close  :wat::core::unit)                  (:wat::io::IOWriter/close stdin-w))
     ((stdout-r :wat::io::IOReader)  (:wat::kernel::Process/stdout proc))
     ((stderr-r :wat::io::IOReader)  (:wat::kernel::Process/stderr proc))
     ((stdout-lines :wat::core::Vector<wat::core::String>)    (:wat::kernel::drain-lines stdout-r))
     ((stderr-lines :wat::core::Vector<wat::core::String>)    (:wat::kernel::drain-lines stderr-r))
     ((joined-result :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ProcessDiedError>>)
      (:wat::kernel::Process/join-result proc))
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
     ((stderr-chain :wat::core::Option<wat::core::Vector<wat::kernel::ProcessDiedError>>)
      (:wat::kernel::extract-panics stderr-lines))
     ((failure :wat::core::Option<wat::kernel::Failure>)
      (:wat::core::match joined-result -> :wat::core::Option<wat::kernel::Failure>
        ((Ok _)    :None)
        ((Err err)
         (Some (:wat::kernel::failure-from-process-died
                 (:wat::core::match stderr-chain
                   -> :wat::core::Vector<wat::kernel::ProcessDiedError>
                   ((Some chain) chain)
                   (:None         err))))))))
    (:wat::core::struct-new :wat::kernel::RunResult
      stdout-lines stderr-lines failure)))

;; Build a RunResult that captures a startup-time spawn failure.
;; Empty stdout/stderr (the child never ran); failure carries the
;; StartupError message. Mirrors what the deleted substrate
;; eval_kernel_run_sandboxed did when startup_from_source returned
;; Err.
(:wat::core::define
  (:wat::kernel::startup-failure-result
    (err :wat::kernel::StartupError)
    -> :wat::kernel::RunResult)
  (:wat::core::struct-new :wat::kernel::RunResult
    (:wat::core::Vector :wat::core::String)
    (:wat::core::Vector :wat::core::String)
    (Some (:wat::kernel::failure-from-startup err))))


;; --- :wat::kernel::run-sandboxed (source-string entry) ---
(:wat::core::define
  (:wat::kernel::run-sandboxed
    (src   :wat::core::String)
    (stdin :wat::core::Vector<wat::core::String>)
    (scope :wat::core::Option<wat::core::String>)
    -> :wat::kernel::RunResult)
  (:wat::core::match (:wat::kernel::spawn-program src scope)
    -> :wat::kernel::RunResult
    ((Ok proc)  (:wat::kernel::drive-sandbox proc stdin))
    ((Err err)  (:wat::kernel::startup-failure-result err))))


;; --- :wat::kernel::run-sandboxed-ast (AST entry) ---
(:wat::core::define
  (:wat::kernel::run-sandboxed-ast
    (forms :wat::core::Vector<wat::WatAST>)
    (stdin :wat::core::Vector<wat::core::String>)
    (scope :wat::core::Option<wat::core::String>)
    -> :wat::kernel::RunResult)
  (:wat::core::match (:wat::kernel::spawn-program-ast forms scope)
    -> :wat::kernel::RunResult
    ((Ok proc)  (:wat::kernel::drive-sandbox proc stdin))
    ((Err err)  (:wat::kernel::startup-failure-result err))))
