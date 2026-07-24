;; tests/diagnostics/probe_plain_panic_produces_structured_edn.wat — co-located fixture for the
;; sibling probe (.rs), slurped via startup_beside(file!()).
;;
;; IPC de-prime (arc 278): migrated off the non-prime `:wat::test::run-hermetic`
;; (fork + OS-pipe scrape → :wat::kernel::RunResult) onto the PRIMED peer wire — a
;; direct `(:wat::kernel::spawn-program' (:wat::spawn::process) (:wat::core::forms …))`
;; child + `(:wat::kernel::recv' p)`. `RunResult` is GONE from this file.
;;
;; Body: dim_count=1 → budget=floor(sqrt(1))=1; a Bundle with 2 atoms exceeds capacity
;; and triggers panic!("...: capacity exceeded ...") — a bare Rust String panic, NOT an
;; AssertionPayload. This is the only reliably reachable non-AssertionPayload panic path
;; from a wat body. The child crash surfaces over the wire as recv' → Lost[cause] with
;; cause = LociDiedError::Panic; the panic's String rides Panic.message. We return that
;; message as a plain String for the Rust driver to assert on.
;;
;; The child runs in a SEPARATE process with its own runtime, so `set-dim-count!` /
;; `set-capacity-mode!` are private to the child — the hermetic property the retired
;; run-hermetic provided is preserved by spawn-program' :process.
(:wat::core::defn :probe::plain-panic [] -> :wat::core::String
  (:wat::core::let
    [p (:wat::kernel::spawn-program' (:wat::spawn::process)
         (:wat::core::forms
           ;; Config setters are load-time DIRECTIVES collected by the child's
           ;; entry-file pass — they MUST sit at the top level of the forms,
           ;; preceding all other forms (they are not runtime functions callable
           ;; from inside :user::main's body — that surfaces UnknownFunction).
           (:wat::config::set-dim-count! 1)
           (:wat::config::set-capacity-mode! :panic)
           (:wat::core::defn :user::main [] -> :wat::core::nil
             ;; Two Atom children exceed floor(sqrt(1))=1 budget
             ;; → panic!("capacity exceeded under :panic") fires inside eval_algebra_bundle.
             (:wat::core::let
               [_bundle
                 (:wat::holon::Bundle
                   (:wat::core::Vector :wat::holon::HolonAST
                     (:wat::holon::to-holon "key1")
                     (:wat::holon::to-holon "key2")))]
               nil))))]
    (:wat::core::match (:wat::kernel::recv' p)
      ((:wat::kernel::RecvOutcome::Message _m) "UNEXPECTED-MESSAGE")
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::core::match cause
          ;; TRUE variant: a bare Rust panic (capacity exceeded, NOT an AssertionPayload)
          ;; → LociDiedError::Panic; the panic String rides Panic.message. Return it.
          ((:wat::kernel::LociDiedError::Panic message _failure) message)
          ;; LociDiedError is the no-hidden-failures enum — every OTHER death is named
          ;; EXPLICITLY (no `_` lump; verbosity is the shield). A distinct WRONG:<variant>
          ;; sentinel makes a RED name exactly which non-Panic death surfaced instead.
          ((:wat::kernel::LociDiedError::RuntimeError _m) "WRONG:RuntimeError")
          (:wat::kernel::LociDiedError::Disconnected "WRONG:Disconnected")
          (:wat::kernel::LociDiedError::Shutdown "WRONG:Shutdown")
          ((:wat::kernel::LociDiedError::StartupError _m) "WRONG:StartupError")
          ((:wat::kernel::LociDiedError::EntryFormFailure _m) "WRONG:EntryFormFailure")
          ((:wat::kernel::LociDiedError::MainSignature _m) "WRONG:MainSignature")
          ((:wat::kernel::LociDiedError::BadReturn _m) "WRONG:BadReturn")))
      (:wat::kernel::RecvOutcome::Closed "UNEXPECTED-CLOSED"))))
