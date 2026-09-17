;; wat-scripts/fixes/queue-to-wat-queue.wat — excursus 001 stone "the queue matures into the
;; stdlib": the userland `:queue::` namespace becomes the stdlib `:wat::queue::` namespace.
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; ONE boundary-aware prefix rename, which reaches every shape at once:
;;   :queue::Queue           -> :wat::queue::Queue      (+ ::Op ::Reply ::SendRequest ::Wait … )
;;   :queue::queue           -> :wat::queue::queue      (the defservice, + ::Record ::Handle ::State)
;;   :queue::Envelope        -> :wat::queue::Envelope   (+ /id /body)
;;   :queue::Waiter · :queue::Counters · :queue::Stats  (+ every `/accessor`)
;;   :queue::Queue/send      -> :wat::queue::Queue/send (the generated surface client methods)
;; Blast radius measured before the run: 2035 occurrences of `:queue::` in 31 tracked `.wat`
;; files (1190 in wat-scripts/queue/sqs.wat, 194 in topic/sns-fanout.wat, 189 in fanout/circuit.wat).
;;
;; ⛔ WHY THE OLD PREFIX IS `:queue:` AND NOT `:queue::`.
;; `rename-keyword-prefix`'s right-boundary rule (wat/fix.wat:713) rejects a match whose next
;; char is an identifier char ([a-zA-Z0-9_-]). With old-bare "queue::" the next char is always
;; the first letter of the name — `:queue::Queue` → `Q` — so EVERY occurrence would be rejected
;; and this codemod would be a silent zero-edit no-op. With old-bare "queue:" the next char is
;; the second `:`, which is not an identifier char, so every namespaced occurrence matches.
;;
;; ⛔ WHY NOT `:queue` (no trailing colon). `:queue` is ALSO a live kwarg in this corpus —
;; `(:queue::Queue::SendRequest :queue name :bodies …)`. With old-bare "queue" the bare keyword
;; `:queue` matches at-end with a valid left boundary and would be silently corrupted into
;; `:wat::queue`. The single trailing colon is exactly what makes the bare kwarg ABSENT (the
;; name is one char too short to hold "queue:") while every namespaced name still matches.
;;
;; IDEMPOTENT. After the rewrite a name reads `:wat::queue::…`, where "queue:" sits at index 6
;; preceded by `:`. `rename-valid-match?` accepts a left context only at i==1-after-`:` or when
;; the previous char is one of `<` `,` ` ` `(` — `:` is none of those — so a re-run emits zero
;; edits and rewrites zero bytes.
;;
;; STRING literals and COMMENTS are untouched: the walk visits keyword LEAVES only. That is
;; deliberate. wat-scripts/fixes/{pending-to-visible,wait-ns-to-wait,declare-queue-drop-knobs,
;; add-malformed-arm,add-timedout-arm}.wat carry `:queue::…` inside string ARGUMENTS as the
;; recorded history of earlier migrations against the old name; they are listed in the run below
;; (list EVERY path) and must come back byte-identical. Prose in the migrated files is the
;; separate manual pass.
;;
;; The SPLIT itself is the manual seam a codemod cannot do — `wat/queue.wat` (the surface, the
;; defservice and its four `retry-*`/`*-after-*` impl helpers) plus its `src/load/stdlib.rs`
;; manifest row at position 50, immediately after `wat/query.wat`. Same shape as
;; wat-scripts/fixes/rename-sourcefile-to-source-file.wat, whose header says the same thing.
;;
;; Usage (ONE EDN vector of EVERY path on stdin — a missed file breaks the build):
;;   printf '["wat-scripts/queue/sqs.wat" "wat-scripts/topic/sns-fanout.wat" …]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/queue-to-wat-queue.wat

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::fix::rename-keyword-prefix ":queue:" ":wat::queue:" src))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[renamed] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
