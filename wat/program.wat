;; wat/program.wat — arc 258 A2: :wat::program::Env as a typed extensible recordtype base.
;;
;; Replaces the Rust-builtin typealias (HashMap<keyword, HolonAST>) with a
;; proper record definition, enabling subtype extension for user programs.
;;
;; Loading order: must load AFTER wat/Record.wat (uses :wat::Record::def)
;; and :wat::time::Instant is a builtin already available at startup.

;; Four kernel-stamped fields (arc 259 — The Forced Hand):
;;   wat.started-at      — the app epoch (CLI-boot instant), INHERITED unchanged down the spawn tree.
;;   wat.peer-started-at — THIS peer's start, RE-STAMPED at each spawn (`peer-`, not `thread-`:
;;                         a peer may be :thread or :process; `thread-` would lie to a process peer).
;;   wat.process-id      — OS process id (`std::process::id()`), stamped at the seam as i64.
;;   wat.os-thread-id    — OS thread id (Linux `gettid()`), stamped at the seam as i64.
;; All `wat.*` (reserved/platform-owned). Subtypes (bracket::Env adds wat.worker-id; user slots
;; user.program / user.bracket) extend this base — see docs/arc/2026/06/259-forced-hand/DESIGN.md.
(:wat::Record::def :wat::program::Env
  [wat.started-at <- :wat::time::Instant
   wat.peer-started-at <- :wat::time::Instant
   wat.process-id <- :wat::core::i64
   wat.os-thread-id <- :wat::core::i64])
