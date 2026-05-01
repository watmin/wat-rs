;; wat/std/hermetic.wat — :wat::kernel::run-sandboxed-hermetic-ast
;; reimplemented as wat stdlib on top of the arc 012 substrate.
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
;; FrozenWorld from the caller's inherited Vec<wat::WatAST> — no
;; binary reload, no tempfile, no re-parse.
;;
;; Arc 112 — fork-program-ast now returns the unified
;; :wat::kernel::Program<I,O> (same struct spawn-program-ast returns).
;; The wait mechanism is hidden inside ProgramHandle's Forked variant;
;; (:wat::kernel::Process/join-result proc) produces wat::core::Result<wat::core::unit,
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

;; Tail-recursive drain of an IOReader into a Vec<String> — one
;; String per line. Reads until read-line returns :None (EOF).
(:wat::core::define
  (:wat::kernel::drain-lines-acc
    (r   :wat::io::IOReader)
    (acc :Vec<wat::core::String>)
    -> :Vec<wat::core::String>)
  (:wat::core::match (:wat::io::IOReader/read-line r) -> :Vec<wat::core::String>
    ((Some line)
     (:wat::kernel::drain-lines-acc
       r
       (:wat::core::conj acc line)))
    (:None acc)))

(:wat::core::define
  (:wat::kernel::drain-lines (r :wat::io::IOReader) -> :Vec<wat::core::String>)
  (:wat::kernel::drain-lines-acc r (:wat::core::vec :wat::core::String)))

;; The main event. Replaces the Rust primitive bit-for-bit at the
;; user surface: same keyword path, same (forms, stdin, scope)
;; signature, same :wat::kernel::RunResult return shape. Now atop
;; the unified arc-112 Process<I,O> — no ForkedChild handle
;; threading, no exit-code prefix logic; ProcessDiedError/to-failure
;; carries the right message.
(:wat::core::define
  (:wat::kernel::run-sandboxed-hermetic-ast<I,O>
    (forms :Vec<wat::WatAST>)
    (stdin :Vec<wat::core::String>)
    (scope :wat::core::Option<wat::core::String>)
    -> :wat::kernel::RunResult)
  (:wat::core::match scope -> :wat::kernel::RunResult
    ((Some _)
     ;; Scope-forwarding through fork is a separate slice when a
     ;; caller demands. Today: :Some returns Failure.
     (:wat::core::struct-new :wat::kernel::RunResult
       (:wat::core::vec :wat::core::String)
       (:wat::core::vec :wat::core::String)
       (Some (:wat::core::struct-new :wat::kernel::Failure
               "scope not yet supported in hermetic mode (:None only for now)"
               :None
               (:wat::core::vec :wat::kernel::Frame)
               :None
               :None))))
    (:None
     (:wat::core::let*
       (((proc :wat::kernel::Program<I,O>)
         (:wat::kernel::fork-program-ast forms))
        ;; Write stdin (if any). An empty vec joins to "", which
        ;; write-all handles as a zero-byte write.
        ((_ :wat::core::i64)
         (:wat::core::let*
           (((stdin-wr :wat::io::IOWriter)
             (:wat::kernel::Process/stdin proc))
            ((joined :wat::core::String)
             (:wat::core::string::join "\n" stdin)))
           (:wat::io::IOWriter/write-string stdin-wr joined)))
        ;; Wait for the program to exit first. With small outputs
        ;; (< pipe buffer), the child's writes complete without
        ;; the parent needing to drain. This keeps the drain
        ;; code single-threaded — no spawn + join ceremony.
        ((joined-result :wat::core::Result<wat::core::unit,Vec<wat::kernel::ProcessDiedError>>)
         (:wat::kernel::Process/join-result proc))
        ((stdout-r :wat::io::IOReader)
         (:wat::kernel::Process/stdout proc))
        ((stderr-r :wat::io::IOReader)
         (:wat::kernel::Process/stderr proc))
        ((stdout-lines :Vec<wat::core::String>)
         (:wat::kernel::drain-lines stdout-r))
        ((stderr-lines :Vec<wat::core::String>)
         (:wat::kernel::drain-lines stderr-r))
        ;; Arc 113 slice 3 — same stderr-EDN preference as
         ;; drive-sandbox. The forked child renders the cascade
         ;; chain to stderr on AssertionPayload panic; we recover
         ;; it here and use it instead of the singleton from
         ;; Process/join-result. Falls back when the marker is
         ;; absent (clean exit, plain panic, runtime error). The
         ;; chain shape at the caller is identical regardless of
         ;; which transport delivered it.
         ((stderr-chain :wat::core::Option<Vec<wat::kernel::ProcessDiedError>>)
          (:wat::kernel::extract-panics stderr-lines))
         ((failure :wat::core::Option<wat::kernel::Failure>)
         (:wat::core::match joined-result -> :wat::core::Option<wat::kernel::Failure>
           ((Ok _)       :None)
           ((Err chain)
            (Some (:wat::kernel::failure-from-process-died
                    (:wat::core::match stderr-chain
                      -> :Vec<wat::kernel::ProcessDiedError>
                      ((Some sc) sc)
                      (:None     chain))))))))
       (:wat::core::struct-new :wat::kernel::RunResult
         stdout-lines
         stderr-lines
         failure)))))
