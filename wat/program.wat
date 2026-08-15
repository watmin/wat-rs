;; wat/program.wat — arc 258 A2 + arc 259 stone 5: :wat::program::Env as a flat typed record.
;;
;; Arc 293 annihilation: no longer an extensible base; it is a plain defrecord.
;; Replaces the Rust-builtin typealias (HashMap<keyword, HolonAST>) with a proper record.
;;
;; Loading order: must load AFTER wat/Record.wat (uses :wat::core::Record::def)
;; and :wat::time::Instant is a builtin already available at startup.
;; The defenum MUST precede the Record::def that names it as a field type.

;; PeerKind — nominal enum answering "what kind of peer am I":
;;   :thread  — shares the parent's address space (a thread peer)
;;   :process — owns its own address space (the root :user::main, OR a forked process peer)
;; The root :user::main owns its address space → seam stamps :process.
(:wat::core::defenum :wat::program::PeerKind :wat::enum::Pure
  :thread
  :process)

;; EmptyEnv — the 0-field nominal default for `user-data`.
;; A real record, never nil: "didn't provide one" is honest because there is no nil branch.
;; Construction: `(:wat::program::EmptyEnv)`. Extends :wat::core::Record (the root).
(:wat::core::defrecord :wat::program::EmptyEnv [])

;; Six kernel-stamped fields, plus one user-data slot (arc 259 — The Forced Hand):
;;   started-at      — the app epoch (CLI-boot instant), INHERITED unchanged down the spawn tree.
;;   peer-started-at — THIS peer's start, RE-STAMPED at each spawn (`peer-`, not `thread-`:
;;                     a peer may be :thread or :process; `thread-` would lie to a process peer).
;;   process-id      — OS process id (`std::process::id()`), stamped at the seam as i64.
;;   os-thread-id    — OS thread id (Linux `gettid()`), stamped at the seam as i64.
;;   peer-kind       — which KIND of peer (PeerKind enum); root main stamps :process (owns its
;;                     address space); thread peers stamp :thread.
;;   cpu-count       — host available parallelism (`std::thread::available_parallelism()`,
;;                     fallback 1); a host constant, INHERITED unchanged down the spawn tree
;;                     (like `started-at`). The escape-hatch home for "how many CPUs".
;;   user-data       — the user-supplied slot, typed :wat::core::Record (the root — any record
;;                     fits); default :wat::program::EmptyEnv. EC2-style user-data: arbitrary
;;                     user-specified data that crosses the spawn boundary and stays
;;                     referenceable in the spawned locus.
;;
;; Arc 296 stone H-1b: ownership used to be spelled with a `wat.`/`user.` name prefix; that prefix
;; is now dropped (renamed, not respelled) because POSITION already carries the distinction a plain
;; `defrecord` (arc 293 — no longer an extensible base, see the header above) makes structural: a
;; top-level field on `Env` IS wat-provided, kernel-stamped at the spawn seam; `user-data` is the
;; one field that is the user's — anything the user needs lives inside the record they put there.
(:wat::core::defrecord :wat::program::Env
  [started-at <- :wat::time::Instant
   peer-started-at <- :wat::time::Instant
   process-id <- :wat::core::i64
   os-thread-id <- :wat::core::i64
   peer-kind <- :wat::program::PeerKind
   cpu-count <- :wat::core::i64
   user-data <- :wat::core::Record])
