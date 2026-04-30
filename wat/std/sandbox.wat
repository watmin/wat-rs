;; wat/std/sandbox.wat — :wat::kernel::run-sandboxed and -ast
;; reimplemented as wat stdlib on top of the arc 103a spawn-program
;; substrate.
;;
;; What this file replaces: the Rust primitives of the same names
;; in src/sandbox.rs, which collected stdin / stdout / stderr as
;; Vec<String> buffers — buffer-in / buffer-out, no interleaving,
;; no back-pressure. The arc 103a spawn-program substrate gives us
;; real kernel pipes; this file moves the test-convenience
;; "collect output to Vec<String>" shape to the wat layer where it
;; belongs (Vec<String> is the ASSERTION TARGET, not a substrate
;; concern). Same surface (`run-sandboxed src stdin scope` →
;; `RunResult`); same return shape; same wat-test calls — only the
;; mechanism changed.
;;
;; Limitations as shipped (same as hermetic.wat):
;; - No concurrent drain of stdout vs stderr. Parent writes stdin,
;;   closes it, drains stdout to EOF, drains stderr to EOF. Works
;;   when the child's output fits in pipe buffers (typically 64KB
;;   per direction). A child that writes more than the buffer holds
;;   to one stream while the parent is draining the other could
;;   deadlock; not seen in any shipped test, follow-up substrate
;;   work when a caller needs it.

;; Build a generic Failure payload when join-result reports the
;; child thread died. v1 doesn't discriminate Panic vs RuntimeError
;; vs ChannelDisconnected — wat-side pattern matching on
;; `:wat::kernel::ThreadDiedError` variants is on the follow-up
;; backlog (the type-checker currently mis-infers the scrutinee).
;; In practice tests check `failure: Some _ vs :None`; the message
;; contents are observed only through the captured stderr lines,
;; which carry the actual diagnostic text.
(:wat::core::define
  (:wat::kernel::generic-failure -> :wat::kernel::Failure)
  (:wat::core::struct-new :wat::kernel::Failure
    "[child thread died]"
    :None
    (:wat::core::vec :wat::kernel::Frame)
    :None
    :None))

;; Common driver — runs a Process (already spawned), pre-seeds
;; stdin, closes the writer to signal EOF, drains stdout / stderr,
;; joins. Returns the canonical RunResult shape every test author
;; matches against.
;;
;; The seeded stdin is joined with '\n' between elements (no
;; trailing newline) — same convention the deleted Rust primitive
;; used. Empty Vec<String> joins to the empty string; write-string
;; is a no-op on zero bytes; close still fires; child sees EOF on
;; first read-line.
(:wat::core::define
  (:wat::kernel::drive-sandbox
    (proc  :wat::kernel::Process)
    (stdin :Vec<String>)
    -> :wat::kernel::RunResult)
  (:wat::core::let*
    (((stdin-w :wat::io::IOWriter)   (:wat::kernel::Process/stdin proc))
     ((joined  :String)              (:wat::core::string::join "\n" stdin))
     ((_n      :i64)                 (:wat::io::IOWriter/write-string stdin-w joined))
     ((_close  :())                  (:wat::io::IOWriter/close stdin-w))
     ((stdout-r :wat::io::IOReader)  (:wat::kernel::Process/stdout proc))
     ((stderr-r :wat::io::IOReader)  (:wat::kernel::Process/stderr proc))
     ((stdout-lines :Vec<String>)    (:wat::kernel::drain-lines stdout-r))
     ((stderr-lines :Vec<String>)    (:wat::kernel::drain-lines stderr-r))
     ((join-h :wat::kernel::ProgramHandle<()>) (:wat::kernel::Process/join proc))
     ((joined-result :Result<(),wat::kernel::ThreadDiedError>)
      (:wat::kernel::join-result join-h))
     ((failure :Option<wat::kernel::Failure>)
      (:wat::core::match joined-result -> :Option<wat::kernel::Failure>
        ((Ok _)   :None)
        ((Err _)  (Some (:wat::kernel::generic-failure))))))
    (:wat::core::struct-new :wat::kernel::RunResult
      stdout-lines stderr-lines failure)))


;; --- :wat::kernel::run-sandboxed (source-string entry) ---
(:wat::core::define
  (:wat::kernel::run-sandboxed
    (src   :String)
    (stdin :Vec<String>)
    (scope :Option<String>)
    -> :wat::kernel::RunResult)
  (:wat::kernel::drive-sandbox
    (:wat::kernel::spawn-program src scope)
    stdin))


;; --- :wat::kernel::run-sandboxed-ast (AST entry) ---
(:wat::core::define
  (:wat::kernel::run-sandboxed-ast
    (forms :Vec<wat::WatAST>)
    (stdin :Vec<String>)
    (scope :Option<String>)
    -> :wat::kernel::RunResult)
  (:wat::kernel::drive-sandbox
    (:wat::kernel::spawn-program-ast forms scope)
    stdin))
