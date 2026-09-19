;; wat-scripts/fixes/drop-sqs-load-file.wat — excursus 001 stone "the queue matures into the
;; stdlib", second half of the migration: delete the now-pointless
;; `(:wat::load-file! "../queue/sqs.wat")` from every program that only ever wanted the LIBRARY.
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; WHY THE LINE MUST GO, AND WHY DEFINITIONS DO NOT ARRIVE TWICE EITHER WAY.
;; wat-scripts/queue/sqs.wat used to hold the `:queue::Queue` surface and the `:queue::queue`
;; service; 18 programs reached them with a relative `load-file!`. Both now ship in the binary
;; as wat/queue.wat (`:wat::queue::`, manifest position 50), so the load is no longer how the
;; library arrives — and because sqs.wat no longer DEFINES the library, a stale load cannot
;; double-define it. What a stale load WOULD do is worse and quieter: it injects a foreign
;; program's TEST DRIVERS — sqs.wat's `main`, `compute`, `depth`, `long-poll`, `lifecycle` and
;; the five `lp-*` rows — into the loading program's user rendezvous namespace, where today
;; they are only masked by `set-redef!` plus the accident that the loader's own `main` is
;; defined later in the file. Deleting the form removes the shadowing, not a dependency.
;;
;; WHAT IT DOES NOT TOUCH. Two probes call sqs.wat's `dial-queue` panic-gate helper, which
;; stayed in the driver half on purpose, so they genuinely still need the load and are NOT in
;; the path list below:
;;   wat-scripts/scratch-pad/probe-a-caller-chunks-to-the-declared-limit.wat
;;   wat-scripts/scratch-pad/probe-a-read-declares-its-page.wat
;;
;; PRECISION. The predicate matches a top-level list of EXACTLY two children whose head is the
;; keyword `:wat::load-file!` and whose argument is a STRING whose value ends with
;; `queue/sqs.wat`. Every other `load-file!` in these files survives — three of the sixteen
;; carry a second one (`../query/faulting-store.wat` in probe-accepted-is-a-count.wat and
;; probe-store-can-fail.wat, `../topic/sns-fanout.wat` in fanout/circuit.wat), and a
;; head-only predicate would have eaten those. `ast-name` on a StringLit returns the unquoted
;; content (eprintln-recv-arm-to-assertion-failed.wat:58 records the same fact).
;;
;; IDEMPOTENT: once the form is gone the predicate matches nothing and zero bytes are rewritten.
;; Comment-faithful: `wat-grep-strip` deletes only the matched forms' own character spans, so a
;; comment ABOVE the deleted line survives — the prose pass is separate, as always.
;;
;; Usage (ONE EDN vector of EVERY path on stdin):
;;   printf '["wat-scripts/topic/sns-fanout.wat" …]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/drop-sqs-load-file.wat

(:wat::load-file! "../lib/wat-grep.wat")

;; sqs-load-arg? — ARG is a string literal whose value ends with "queue/sqs.wat".
(:wat::core::defn :user::sqs-load-arg? [arg <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind arg) "string")
    (:wat::string::ends-with? (:wat::core::ast-name arg) "queue/sqs.wat")
    false))

;; sqs-load-file? — a top-level list `(:wat::load-file! "…queue/sqs.wat")`, exactly 2 children.
(:wat::core::defn :user::sqs-load-file? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::= (:wat::core::length ch) 2)
        (:wat::core::let [head (:wat::core::first ch)]
          (:wat::core::if (:wat::core::= (:wat::core::ast-kind head) "keyword")
            (:wat::core::if (:wat::core::= (:wat::core::ast-name head) ":wat::load-file!")
              (:user::sqs-load-arg?
                (:wat::core::Option/expect (:wat::core::get ch 1) "sqs-load-file?: arg"))
              false)
            false))
        false))
    false))

(:wat::core::defn :user::process-file [path <- :wat::core::String] -> :wat::core::nil
  (:wat::core::let [src      (:wat::io::read-file path)
                    n        (:wat::core::length (:user::wat-grep src :user::sqs-load-file?))
                    stripped (:user::wat-grep-strip src :user::sqs-load-file?)]
    (:wat::core::if (:wat::core::= src stripped)
      (:wat::kernel::println (:wat::string::concat "[unchanged] " path))
      (:wat::core::do
        (:wat::io::write-file path stripped)
        (:wat::kernel::println
          (:wat::string::concat "[dropped " (:wat::i64::to-string n) "] " path))
        nil))))

(:wat::core::defn :user::process-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::do
      (:user::process-file (:wat::core::first paths))
      (:user::process-each (:wat::core::rest paths)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::process-each
    (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
