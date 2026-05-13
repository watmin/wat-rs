;; wat/kernel/hermetic.wat — :wat::kernel::run-sandboxed-hermetic-ast
;; restored from git history (eb655d1^:wat/std/hermetic.wat) in
;; arc 170 slice 1f-δ. File location updated to wat/kernel/ per
;; arc 109 K-namespace doctrine. Content is literal restore; the
;; TIERS.md migration to spawn-process remains a separate future arc.
;;
;; What this file replaces: the Rust primitive of the same name in
;; src/sandbox.rs, which spawned the wat binary as a subprocess
;; (tempfile + Command::spawn + wait_with_output) to run the inner
;; program in isolation. That primitive coupled the runtime to its
;; own binary path — honest for its time, dishonest long-term.
;;
;; The fork substrate (pipe + fork-program-ast + Process/join-result,
;; arc 012 + arc 112) makes hermetic expressible in wat. The child
;; inherits the parent's loaded runtime via COW and builds a fresh
;; FrozenWorld from the caller's inherited wat::core::Vector<wat::WatAST> — no
;; binary reload, no tempfile, no re-parse.
;;
;; Arc 112 — fork-program-ast now returns the unified
;; :wat::kernel::Program<I,O> (same struct spawn-program-ast returns).
;; The wait mechanism is hidden inside ProgramHandle's Forked variant;
;; (:wat::kernel::Process/join-result proc) produces wat::core::Result<wat::core::nil,
;; ProcessDiedError> uniformly. The pre-arc-112 ForkedChild +
;; wait-child + per-exit-code prefix logic collapsed: the substrate
;; renders exit-code interpretation directly in the ProcessDiedError
;; payload.
;;
;; Limitations as shipped:
;; - No concurrent drain of stdout vs stderr. Parent waits-program
;;   first, then drains serially. Works when the child's output
;;   fits in pipe buffers (typically 64KB+). A child that writes
;;   more to one stream than the buffer holds would deadlock; no
;;   demand yet.
;; - No force-close of the parent's stdin writer. The child's main
;;   sees EOF on stdin only when the outer Process binding drops
;;   (at :user::main exit). Children that read stdin to EOF as a
;;   prerequisite to further work would deadlock; no demand yet.
;;
;; Both limitations are follow-up substrate work when a caller
;; needs them.

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
;;
;; Folded in from git history (eb655d1^:wat/std/sandbox.wat) —
;; hermetic.wat needs this helper and sandbox.wat is the canonical
;; source. Duplicate definition intentionally avoided: only hermetic.wat
;; defines it (sandbox.wat is not loaded in the kernel path).
(:wat::core::define
  (:wat::kernel::failure-from-process-died
    (chain :wat::core::Vector<wat::kernel::ProcessDiedError>)
    -> :wat::kernel::Failure)
  (:wat::core::match (:wat::core::first chain)
    -> :wat::kernel::Failure
    ((:wat::core::Some err) (:wat::kernel::ProcessDiedError/to-failure err))
    (:wat::core::None
     ;; Empty chain — should not occur; substrate always emits at
     ;; least the immediate-peer death. Defensive default.
     (:wat::core::struct-new :wat::kernel::Failure
       "empty died-chain (substrate bug)"
       :wat::core::None
       (:wat::core::Vector :wat::kernel::Frame)
       :wat::core::None
       :wat::core::None))))

;; Tail-recursive drain of an IOReader into a wat::core::Vector<String> — one
;; String per line. Reads until read-line returns :None (EOF).
(:wat::core::define
  (:wat::kernel::drain-lines-acc
    (r   :wat::io::IOReader)
    (acc :wat::core::Vector<wat::core::String>)
    -> :wat::core::Vector<wat::core::String>)
  (:wat::core::match (:wat::io::IOReader/read-line r) -> :wat::core::Vector<wat::core::String>
    ((:wat::core::Some line)
     (:wat::kernel::drain-lines-acc
       r
       (:wat::core::conj acc line)))
    (:wat::core::None acc)))

(:wat::core::define
  (:wat::kernel::drain-lines (r :wat::io::IOReader) -> :wat::core::Vector<wat::core::String>)
  (:wat::kernel::drain-lines-acc r (:wat::core::Vector :wat::core::String)))

;; The main event. Replaces the Rust primitive bit-for-bit at the
;; user surface: same keyword path, same (forms, stdin, scope)
;; signature, same :wat::kernel::RunResult return shape. Now atop
;; the unified arc-112 Process<I,O> — no ForkedChild handle
;; threading, no exit-code prefix logic; ProcessDiedError/to-failure
;; carries the right message.
(:wat::core::define
  (:wat::kernel::run-sandboxed-hermetic-ast<I,O>
    (forms :wat::core::Vector<wat::WatAST>)
    (stdin :wat::core::Vector<wat::core::String>)
    (scope :wat::core::Option<wat::core::String>)
    -> :wat::kernel::RunResult)
  (:wat::core::match scope -> :wat::kernel::RunResult
    ((:wat::core::Some _)
     ;; Scope-forwarding through fork is a separate slice when a
     ;; caller demands. Today: :Some returns Failure.
     (:wat::core::struct-new :wat::kernel::RunResult
       (:wat::core::Vector :wat::core::String)
       (:wat::core::Vector :wat::core::String)
       (:wat::core::Some (:wat::core::struct-new :wat::kernel::Failure
               "scope not yet supported in hermetic mode (:None only for now)"
               :wat::core::None
               (:wat::core::Vector :wat::kernel::Frame)
               :wat::core::None
               :wat::core::None))))
    (:wat::core::None
     (:wat::core::let
       [proc
         (:wat::kernel::fork-program-ast forms)
        ;; Write stdin (if any). An empty vec joins to "", which
        ;; write-all handles as a zero-byte write.
        _
         (:wat::core::let
           [stdin-wr
             (:wat::kernel::Process/stdin proc)
            joined
             (:wat::core::string::join "\n" stdin)]
           (:wat::io::IOWriter/write-string stdin-wr joined))
        ;; Inner scope: output Receivers + drained lines.
        ;; SERVICE-PROGRAMS.md § "The lockstep": stdout-r and stderr-r
        ;; drop when this inner let exits; drain threads see EOF; child
        ;; can exit; outer join-result unblocks cleanly.
        drain-pair
         (:wat::core::let
           [stdout-r      (:wat::kernel::Process/stdout proc)
            stderr-r      (:wat::kernel::Process/stderr proc)
            stdout-lines  (:wat::kernel::drain-lines stdout-r)
            stderr-lines  (:wat::kernel::drain-lines stderr-r)]
           (:wat::core::Tuple stdout-lines stderr-lines))
        stdout-lines  (:wat::core::first drain-pair)
        stderr-lines  (:wat::core::second drain-pair)
        ;; Inner scope has exited; Receivers dropped; child can exit.
        joined-result
         (:wat::kernel::Process/join-result proc)
        ;; Arc 113 slice 3 — same stderr-EDN preference as
        ;; drive-sandbox. The forked child renders the cascade
        ;; chain to stderr on AssertionPayload panic; we recover
        ;; it here and use it instead of the singleton from
        ;; Process/join-result. Falls back when the marker is
        ;; absent (clean exit, plain panic, runtime error). The
        ;; chain shape at the caller is identical regardless of
        ;; which transport delivered it.
        stderr-chain
         (:wat::kernel::extract-panics stderr-lines)
        failure
         (:wat::core::match joined-result -> :wat::core::Option<wat::kernel::Failure>
           ((:wat::core::Ok _)       :wat::core::None)
           ((:wat::core::Err chain)
            (:wat::core::Some (:wat::kernel::failure-from-process-died
                    (:wat::core::match stderr-chain
                      -> :wat::core::Vector<wat::kernel::ProcessDiedError>
                      ((:wat::core::Some sc) sc)
                      ;; Arc 170 slice 1i — substrate contract: every child error
                      ;; MUST emit structured #wat.kernel/ProcessPanics EDN.
                      ;; Concat actual stderr-lines into the panic message so
                      ;; the substrate's contract violation is self-diagnosing
                      ;; (mirrors wat/test.wat run-hermetic-driver).
                      (:wat::core::None
                       (:wat::kernel::assertion-failed!
                         (:wat::core::string::concat
                           "structured-stderr-only contract violation: child error but no parseable ProcessPanics found on stderr.\nActual stderr content:\n"
                           (:wat::core::string::join "\n" stderr-lines))
                         :wat::core::None :wat::core::None)))))))]
       (:wat::core::struct-new :wat::kernel::RunResult
         stdout-lines
         stderr-lines
         failure)))))
