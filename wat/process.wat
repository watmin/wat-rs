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

;; Bracket — a pool-worker's identity: its runner index. wat/bracket.wat's map-worker sets
;; one per runner via `:wat::spawn::with-label` before spawning it.
(:wat::core::defrecord :wat::process::Bracket [id <- :wat::core::i64])

;; Service — a defservice's identity: its own FQDN. `name` is a KEYWORD, not a String —
;; builder-ruled, grounded at wat/telemetry.wat's Span::IncrRequest/TimedRequest (`name <-
;; :wat::core::keyword`), which already types identity-like `name` fields this way. A
;; keyword IS the symbol carrier post the Clojure-syntax flip (`::` <-> `.`); a String would
;; need re-parsing at the boundary — a redesign wearing a swap's clothes. wat/service.wat's
;; `start`/`resume` set this via `:wat::spawn::with-label` using the service's own fqdn
;; keyword, known statically at macro-expansion time.
(:wat::core::defrecord :wat::process::Service [name <- :wat::core::keyword])
