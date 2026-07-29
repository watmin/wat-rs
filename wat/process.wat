;; wat/process.wat — arc 170 closure #6: the two ps-visible spawned-process identities.
;;
;; SHAPE 2, cast + builder-ratified over SHAPE 1 (each caller mints its own tag):
;; a CLOSED set of two substrate-owned types, varying part as DATA. No caller defines a
;; type to become visible in `ps`; the two types below are the whole vocabulary, so an
;; operator learns the set once and matches it exhaustively — the same discipline as a
;; closed outcome enum (RecvOutcome, SendOutcome, …), applied to process identity.
;;
;; Deliberately its OWN namespace, not folded into a neighbour (weighed and rejected):
;;   :wat::spawn::   — HOW/WHERE to execute (ThreadOpts/ProcessOpts/PeerKind); not WHAT a
;;                     process IS to an operator reading `ps`.
;;   :wat::program:: — facts a running peer discovers about ITSELF (Env, PeerKind),
;;                     CHILD-computed. Conflating "child-computed self-knowledge" with
;;                     "caller-supplied, boot-fixed, outward identity" is what stopped the
;;                     first strike at this closure item — keeping them apart is deliberate.
;;   :wat::bracket:: — the pool RUNTIME (map-worker, PoolMsg, …), not a naming vocabulary.
;; `:wat::process::` owns exactly one thing: an OS process's outward, caller-supplied,
;; boot-fixed identity. Loads after wat/Record.wat (uses :wat::core::defrecord), before
;; wat/spawn.wat's two consumers (wat/bracket.wat, wat/service.wat).
;;
;; Rendered (the substrate's uniform EDN rule — record -> `#ns/Name {field-map}`, never a
;; choice made here):
;;   $ ps -o args= -C wat
;;   /usr/local/bin/wat
;;   /usr/local/bin/wat #wat.process/Bracket {:id 0}
;;   /usr/local/bin/wat #wat.process/Bracket {:id 1}
;;   /usr/local/bin/wat #wat.process/Service {:name :my::app::CounterSvc}
;; (the first line is a spawn with `ProcessOpts/label` = `:None` — unchanged from before
;; this file existed.)

;; ── the spawn ORIGIN, on both records ────────────────────────────────────────────────
;; `file`/`line` are the source position of the call that spawned this process — the
;; CALLER's position, captured by the spawner from its own body via
;; `(:wat::kernel::call-site)` (runtime.rs:20755 — "the wat equivalent of Ruby's `caller`";
;; MEASURED: wat-scripts/scratch-pad/probe-call-site-frame.wat shows two callers at
;; different lines reporting their own two lines, so it is the call's position, never the
;; callee's fixed one). Boot-fixed BY NATURE — a process is spawned exactly once, so unlike
;; a per-work-unit value this can never go stale in a `ps` line an operator trusts.
;;
;; LIFTED FLAT (`:file`/`:line`), not nested in a `:wat::kernel::Location`: an operator
;; reads this under pressure, and the flat kwargs map is what stays legible. `col` is
;; dropped deliberately — nobody reads a column out of `ps`.

;; Bracket — a pool-worker's identity: its runner index + where the pool was spawned.
;; wat/bracket.wat's map-worker sets one per runner via `:wat::spawn::with-label`.
;; The origin is what disambiguates runners: `{:id 3}` alone is ambiguous the moment two
;; pools run concurrently — three runners numbered 0,1,2 from two different call sites are
;; indistinguishable without it.
(:wat::core::defrecord :wat::process::Bracket
  [id   <- :wat::core::i64
   file <- :wat::core::String
   line <- :wat::core::i64])

;; Service — a defservice's identity: its own FQDN. `name` is a KEYWORD, not a String —
;; builder-ruled, grounded at wat/telemetry.wat's Span::IncrRequest/TimedRequest (`name <-
;; :wat::core::keyword`), which already types identity-like `name` fields this way. A
;; keyword IS the symbol carrier post the Clojure-syntax flip (`::` <-> `.`); a String would
;; need re-parsing at the boundary — a redesign wearing a swap's clothes. wat/service.wat's
;; `start`/`resume` set this via `:wat::spawn::with-label` using the service's own fqdn
;; keyword, known statically at macro-expansion time.
;; The origin here is INTENDED to be the `start`/`resume` CALL SITE, not the `defservice`
;; definition site: the name already says WHICH service this is, so the useful second fact
;; is which of possibly several starts brought THIS process up.
;;
;; ⛔ IT DOES NOT REPORT THAT TODAY — it reports `wat/core.wat:649`, for EVERY service.
;; Do not read a Service label's file/line as the start site until this is fixed.
;;
;; ROOT (measured, not inferred — wat-scripts/scratch-pad/probe-kwargs-stack-shape.wat):
;; a generated `start` is a KWARGS fn, and the kwargs lowering rewrites `(svc/start …)` into
;; `(svc/start$impl …)` at expansion. The emitted call carries the TEMPLATE's span
;; (wat/core.wat:649, the `kwargs-lower` quasiquote), so the only frame pushed names the
;; adapter at core.wat — and the author's call line is not merely buried, it is ABSENT from
;; the stack entirely. No reader-side rule can recover it (probes rule out -1, "find your
;; own name", and any selection policy): the span is DESTROYED at rewrite time, so the fix
;; must PRESERVE it there.
;;
;; This is NOT a labelling bug. The same lost span is what `assertion-failed!` reports as
;; `:location`, so an assert that fails inside ANY kwargs fn tells its author the failure is
;; in wat/core.wat:649 — a masking defect across the whole diagnostic path, surfaced by this
;; label (ALIVS ARGVIT — the consumer found the flaw). Bracket is UNAFFECTED: `map-worker`
;; is positional, so its origin is the real caller (proven by the same probes).
(:wat::core::defrecord :wat::process::Service
  [name <- :wat::core::keyword
   file <- :wat::core::String
   line <- :wat::core::i64])
