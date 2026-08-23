;; wat-scripts/fixes/reclaim-stdio-prime-names.wat — arc 170 Phase 3 Part B, the `'`-name reclamation,
;; run over real wat source files IN WAT, through the wat CLI. Self-hosted: no Rust harness, no
;; hand-edit of wat source (use-the-tool, not hand-fix).
;;
;; The hand-rolled stdio path (StdInService/StdOutService/StdErrService) is DELETED in Part A, so the
;; primed stdio defservices reclaim their plain names — drop the trailing `'` on the SERVICE and
;; SURFACE prefixes (six boundary-aware whole-name PREFIX renames):
;;   :wat::kernel::StdOut'      -> :wat::kernel::StdOut       (+ ::WriteLineRequest/Response, /write-line, ::Op/::Reply, …)
;;   :wat::kernel::StdErr'      -> :wat::kernel::StdErr
;;   :wat::kernel::StdIn'       -> :wat::kernel::StdIn        (+ ::ReadLineRequest/Response::{Line,Eof,…})
;;   :wat::kernel::stdout-svc'  -> :wat::kernel::stdout-svc   (+ /start, ::Record, ::State, ::Handle, ::init, …)
;;   :wat::kernel::stderr-svc'  -> :wat::kernel::stderr-svc
;;   :wat::kernel::stdin-svc'   -> :wat::kernel::stdin-svc
;;
;; Idempotent BY CONSTRUCTION: this DROPS a trailing `'` (a removal), so after the rewrite the old
;; `…'` prefix is gone and a re-run matches nothing. (The generated positional ctor prime `::State'`
;; keeps its OWN trailing `'` — a boundary-aware prefix rename of `…stdin-svc'` rewrites only the
;; `stdin-svc'` head, leaving `::State'` intact: `…stdin-svc::State'`.) The kept gated primitives
;; (`write-fd-raw` / `flood-stdout-raw` / `str-double` / `flood-own-stdout`) carry no `'` and match
;; none of the six prefixes — untouched. No collision with the deleted `StdInService` (different name).
;;
;; Usage (one EDN vector of EVERY path holding a `'`-name on stdin — list them ALL):
;;   printf '["wat/kernel/services/stdio.wat" "tests/services/probe_arc170_stdio_prime.wat"]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/reclaim-stdio-prime-names.wat
;;
;; The def/registration seams the codemod cannot touch (Rust doc comments, the DESIGN doc) are the
;; manual tail; the load-bearing wat CODE is this rewrite.

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::fix::rename-keyword-prefix ":wat::kernel::StdOut'" ":wat::kernel::StdOut"
    (:wat::fix::rename-keyword-prefix ":wat::kernel::StdErr'" ":wat::kernel::StdErr"
      (:wat::fix::rename-keyword-prefix ":wat::kernel::StdIn'" ":wat::kernel::StdIn"
        (:wat::fix::rename-keyword-prefix ":wat::kernel::stdout-svc'" ":wat::kernel::stdout-svc"
          (:wat::fix::rename-keyword-prefix ":wat::kernel::stderr-svc'" ":wat::kernel::stderr-svc"
            (:wat::fix::rename-keyword-prefix ":wat::kernel::stdin-svc'" ":wat::kernel::stdin-svc"
              src)))))))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::core::string::concat "[reclaimed] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
