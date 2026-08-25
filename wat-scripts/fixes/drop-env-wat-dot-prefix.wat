;; wat-scripts/fixes/drop-env-wat-dot-prefix.wat — arc 296 stone H-1b, run over real wat source
;; files IN WAT, through the wat CLI. The migration tool, self-hosted: no Rust harness, no
;; hand-edit of wat source (use-the-tool, not hand-fix).
;;
;; THE MIGRATION (BRIEF-296-H1b-env-members-lose-the-dots.md): `:wat::program::Env` declares six
;; kernel-stamped fields plus one user-data slot, all carrying a dead `wat.`/`user.` marker prefix
;; — the distinction it recorded (wat-provided vs. user-provided) is now carried by POSITION
;; (top-level field == wat-provided), so the prefix is dropped, not respelled. The seventh field
;; is also RESPELLED, not just de-prefixed — `program` was rejected as a mumble (it would read as
;; an eighth kernel-stamped fact, and `Env/program` stutters the namespace), so builder's ruling
;; lands on `user-data` (EC2-style: arbitrary user-supplied data crossing the spawn boundary):
;;
;;   wat.started-at      -> started-at
;;   wat.peer-started-at -> peer-started-at
;;   wat.process-id      -> process-id
;;   wat.os-thread-id    -> os-thread-id
;;   wat.peer-kind       -> peer-kind
;;   wat.cpu-count       -> cpu-count
;;   user.program        -> user-data
;;
;; Each field name shows up in FOUR distinct shapes, so each gets four passes:
;;   (a) the bare SYMBOL in the defrecord binder list — `wat.started-at <- :wat::time::Instant`
;;       (rename-symbol-exact; ast-kind "symbol", no leading colon)
;;   (b) the standalone KEYWORD used as a kwarg key at construction — `:wat.started-at`
;;       (rename-keyword-exact, exact whole-name)
;;   (c) the ACCESSOR keyword, which the reader lexes as ONE token including the class path and
;;       the `/` separator — `:wat::program::Env/wat.started-at` (rename-keyword-exact, exact
;;       whole-name on the FULL accessor keyword; the boundary-aware `rename-keyword-prefix` does
;;       NOT reach this shape, since its left-boundary set is `{"<","," ,"(",":"}` and does not
;;       include `/`).
;;   (d) COMMENT PROSE naming the field in words (e.g. "the injected user.program record") —
;;       unreachable by the AST passes (fix-text-apply's walk never sees inside a `;;` comment,
;;       by design — code stays span-faithful). A literal split+join substring pass (mirrors
;;       rename-locidiederror-shutdown-to-stopped.wat's pass 3) catches these; run LAST, after
;;       (a)-(c) have eliminated every CODE occurrence of the bare dotted token, so it can only
;;       ever touch prose.
;;
;; 28 passes total (7 fields x 4 shapes), all exact-literal (never prefix/regex), so idempotent by
;; construction: after one pass none of the 7 bare dotted tokens remain anywhere in the text
;; (code or comment), so a re-run's 28 calls each find zero matches. Deliberately does NOT touch
;; `tests/types/probe_arc258_dotted_record_field.wat` — that fixture declares its OWN dotted field
;; `wat.started-at` on an unrelated `:user::Probe` record to de-risk the (now-retired) general
;; dotted-field mechanism; it is a second dotted-binder record the brief's census missed (STOP-1)
;; and is reported separately, not renamed here.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["wat/program.wat" "wat/service.wat" …]\n' \
;;     | cargo wat ./wat-scripts/fixes/drop-env-wat-dot-prefix.wat

;; literal-replace — substring replace via split+join (no dedicated string::replace primitive
;; exists in wat core). `old` must be non-empty (string::split rejects an empty separator).
(:wat::core::defn :user::literal-replace
  [src <- :wat::core::String  old <- :wat::core::String  new <- :wat::core::String] -> :wat::core::String
  (:wat::string::join new (:wat::string::split src old)))

(:wat::core::defn :user::migrate-field
  [src <- :wat::core::String  bare-old <- :wat::core::String  bare-new <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::let
    [kw-old  (:wat::string::concat ":" bare-old)
     kw-new  (:wat::string::concat ":" bare-new)
     acc-old (:wat::string::concat ":wat::program::Env/" bare-old)
     acc-new (:wat::string::concat ":wat::program::Env/" bare-new)
     s1      (:wat::fix::rename-symbol-exact bare-old bare-new src)
     s2      (:wat::fix::rename-keyword-exact kw-old kw-new s1)
     s3      (:wat::fix::rename-keyword-exact acc-old acc-new s2)
     s4      (:user::literal-replace s3 bare-old bare-new)]
    s4))

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let
    [s1 (:user::migrate-field src "wat.started-at" "started-at")
     s2 (:user::migrate-field s1  "wat.peer-started-at" "peer-started-at")
     s3 (:user::migrate-field s2  "wat.process-id" "process-id")
     s4 (:user::migrate-field s3  "wat.os-thread-id" "os-thread-id")
     s5 (:user::migrate-field s4  "wat.peer-kind" "peer-kind")
     s6 (:user::migrate-field s5  "wat.cpu-count" "cpu-count")
     s7 (:user::migrate-field s6  "user.program" "user-data")]
    s7))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[env-dot-drop] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
