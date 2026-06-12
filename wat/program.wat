;; wat/program.wat — arc 258 A2 + arc 259 stone 5: :wat::program::Env as a typed extensible recordtype base.
;;
;; Replaces the Rust-builtin typealias (HashMap<keyword, HolonAST>) with a
;; proper record definition, enabling subtype extension for user programs.
;;
;; Loading order: must load AFTER wat/Record.wat (uses :wat::Record::def)
;; and :wat::time::Instant is a builtin already available at startup.
;; The defenum MUST precede the Record::def that names it as a field type.

;; PeerKind — nominal enum answering "what kind of peer am I":
;;   :thread  — shares the parent's address space (a thread peer)
;;   :process — owns its own address space (the root :user::main, OR a forked process peer)
;; The root :user::main owns its address space → seam stamps :process.
(:wat::core::defenum :wat::program::PeerKind
  :thread
  :process)

;; Five kernel-stamped fields (arc 259 — The Forced Hand):
;;   wat.started-at      — the app epoch (CLI-boot instant), INHERITED unchanged down the spawn tree.
;;   wat.peer-started-at — THIS peer's start, RE-STAMPED at each spawn (`peer-`, not `thread-`:
;;                         a peer may be :thread or :process; `thread-` would lie to a process peer).
;;   wat.process-id      — OS process id (`std::process::id()`), stamped at the seam as i64.
;;   wat.os-thread-id    — OS thread id (Linux `gettid()`), stamped at the seam as i64.
;;   wat.peer-kind       — which KIND of peer (PeerKind enum); root main stamps :process (owns its
;;                         address space); thread peers stamp :thread.
;; All `wat.*` (reserved/platform-owned). Subtypes (bracket::Env adds wat.worker-id; user slots
;; user.program / user.bracket) extend this base — see docs/arc/2026/06/259-forced-hand/DESIGN.md.
(:wat::Record::def :wat::program::Env
  [wat.started-at <- :wat::time::Instant
   wat.peer-started-at <- :wat::time::Instant
   wat.process-id <- :wat::core::i64
   wat.os-thread-id <- :wat::core::i64
   wat.peer-kind <- :wat::program::PeerKind])
