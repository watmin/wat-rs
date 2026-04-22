;; wat/std/hermetic.wat — :wat::kernel::run-sandboxed-hermetic-ast
;; reimplemented as wat stdlib on top of the arc 012 substrate.
;;
;; What this file replaces: the Rust primitive of the same name in
;; src/sandbox.rs, which spawned the wat binary as a subprocess
;; (tempfile + Command::spawn + wait_with_output) to run the inner
;; program in isolation. That primitive coupled the runtime to its
;; own binary path — honest for its time, dishonest long-term.
;;
;; The fork substrate (pipe + fork-with-forms + wait-child, arc
;; 012 slices 1 + 2) makes hermetic expressible in wat. The child
;; inherits the parent's loaded runtime via COW and builds a fresh
;; FrozenWorld from the caller's inherited Vec<wat::WatAST> — no
;; binary reload, no tempfile, no re-parse.
;;
;; Limitations as shipped:
;; - No concurrent drain of stdout vs stderr. Parent waits-child
;;   first, then drains serially. Works when the child's output
;;   fits in pipe buffers (typically 64KB+). A child that writes
;;   more to one stream than the buffer holds would deadlock; no
;;   demand yet.
;; - No force-close of the parent's stdin writer. The child's main
;;   sees EOF on stdin only when the outer ForkedChild binding
;;   drops (at :user::main exit). Children that read stdin to EOF
;;   as a prerequisite to further work would deadlock; no demand
;;   yet.
;;
;; Both limitations are follow-up substrate work when a caller
;; needs them.

;; Tail-recursive drain of an IOReader into a Vec<String> — one
;; String per line. Reads until read-line returns :None (EOF).
(:wat::core::define
  (:wat::kernel::drain-lines-acc
    (r   :wat::io::IOReader)
    (acc :Vec<String>)
    -> :Vec<String>)
  (:wat::core::match (:wat::io::IOReader/read-line r) -> :Vec<String>
    ((Some line)
     (:wat::kernel::drain-lines-acc
       r
       (:wat::core::conj acc line)))
    (:None acc)))

(:wat::core::define
  (:wat::kernel::drain-lines (r :wat::io::IOReader) -> :Vec<String>)
  (:wat::kernel::drain-lines-acc r (:wat::core::vec :String)))

;; Translate an EXIT_* code into a short message prefix used in
;; Failure.message. Keep in sync with src/fork.rs's EXIT_* consts.
(:wat::core::define
  (:wat::kernel::exit-code-prefix (code :i64) -> :String)
  (:wat::core::if (:wat::core::= code 1) -> :String
    "[runtime error]"
    (:wat::core::if (:wat::core::= code 2) -> :String
      "[panic]"
      (:wat::core::if (:wat::core::= code 3) -> :String
        "[startup error]"
        (:wat::core::if (:wat::core::= code 4) -> :String
          "[:user::main signature]"
          "[nonzero exit]")))))

;; Compose a Failure.message from the prefix + the child's stderr
;; lines joined with newlines.
(:wat::core::define
  (:wat::kernel::failure-message-for-code
    (code   :i64)
    (stderr :Vec<String>)
    -> :String)
  (:wat::core::string::join "\n"
    (:wat::core::conj
      (:wat::core::vec :String (:wat::kernel::exit-code-prefix code))
      (:wat::core::string::join "\n" stderr))))

;; Build the Option<Failure> from the child's exit code + stderr.
;; Exit 0 → :None; any nonzero exit → Some with a reconstructed
;; Failure struct carrying the exit category + stderr content.
(:wat::core::define
  (:wat::kernel::failure-from-exit
    (code   :i64)
    (stderr :Vec<String>)
    -> :Option<wat::kernel::Failure>)
  (:wat::core::if (:wat::core::= code 0) -> :Option<wat::kernel::Failure>
    :None
    (Some (:wat::core::struct-new :wat::kernel::Failure
            (:wat::kernel::failure-message-for-code code stderr)
            :None
            (:wat::core::vec :wat::kernel::Frame)
            :None
            :None))))

;; The main event. Replaces the Rust primitive bit-for-bit at the
;; user surface: same keyword path, same (forms, stdin, scope)
;; signature, same :wat::kernel::RunResult return shape.
(:wat::core::define
  (:wat::kernel::run-sandboxed-hermetic-ast
    (forms :Vec<wat::WatAST>)
    (stdin :Vec<String>)
    (scope :Option<String>)
    -> :wat::kernel::RunResult)
  (:wat::core::match scope -> :wat::kernel::RunResult
    ((Some _)
     ;; Scope-forwarding through fork is a separate slice when a
     ;; caller demands. Today: :Some returns Failure.
     (:wat::core::struct-new :wat::kernel::RunResult
       (:wat::core::vec :String)
       (:wat::core::vec :String)
       (Some (:wat::core::struct-new :wat::kernel::Failure
               "scope not yet supported in hermetic mode (:None only for now)"
               :None
               (:wat::core::vec :wat::kernel::Frame)
               :None
               :None))))
    (:None
     (:wat::core::let*
       (((child :wat::kernel::ForkedChild)
         (:wat::kernel::fork-with-forms forms))
        ((handle :wat::kernel::ChildHandle)
         (:wat::kernel::ForkedChild/handle child))
        ;; Write stdin (if any). An empty vec joins to "", which
        ;; write-all handles as a zero-byte write. Inner scope
        ;; mostly for readability — the writer Arc stays alive
        ;; via the enclosing child binding either way.
        ((_ :i64)
         (:wat::core::let*
           (((stdin-wr :wat::io::IOWriter)
             (:wat::kernel::ForkedChild/stdin child))
            ((joined :String)
             (:wat::core::string::join "\n" stdin)))
           (:wat::io::IOWriter/write-string stdin-wr joined)))
        ;; Wait for the child to exit first. With small outputs
        ;; (< pipe buffer), the child's writes complete without
        ;; the parent needing to drain. This keeps the drain
        ;; code single-threaded — no spawn + join ceremony.
        ((exit-code :i64)
         (:wat::kernel::wait-child handle))
        ((stdout-r :wat::io::IOReader)
         (:wat::kernel::ForkedChild/stdout child))
        ((stderr-r :wat::io::IOReader)
         (:wat::kernel::ForkedChild/stderr child))
        ((stdout-lines :Vec<String>)
         (:wat::kernel::drain-lines stdout-r))
        ((stderr-lines :Vec<String>)
         (:wat::kernel::drain-lines stderr-r))
        ((failure :Option<wat::kernel::Failure>)
         (:wat::kernel::failure-from-exit exit-code stderr-lines)))
       (:wat::core::struct-new :wat::kernel::RunResult
         stdout-lines
         stderr-lines
         failure)))))
