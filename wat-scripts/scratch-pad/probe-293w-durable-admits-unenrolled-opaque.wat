;; probe-293w-durable-admits-unenrolled-opaque.wat — arc 278, STOP-3 of the connection-scoped-world
;; stone. THE INSTRUMENT THAT PRODUCED THE STOP-3 ANSWER, kept so the number is reproducible
;; ([[feedback_an_instrument_must_outlive_the_number_it_produced]]).
;;
;; ⛔ THIS FILE LOADING GREEN *IS* THE DEFECT. It is not a passing test; it is a standing
;;    demonstration that the 293.W containment wall does not reach an unenrolled Rust opaque.
;;    When `293/NOTE-containment-wall-blind-to-rust-opaques.md`'s enrollment fix lands, THIS FILE
;;    MUST GO RED — that redness is the fix's acceptance gate, pre-written. Do not "fix" the file
;;    by moving the field to `:ephemeral`; delete it, and note the wall closed.
;;
;; ─── the question STOP-3 asked ───────────────────────────────────────────────────────────────
;; "Is 293.W's 'an impure field can only live in :ephemeral' a COMPILER-ENFORCED WALL, or a
;;  convention?" Answered 2026-08-08, and the answer is two-sided.
;;
;; (a) THE WALL IS REAL AND REACHES `:durable`. A defservice's `:durable` slot synthesizes
;;     `<svc>::Record`, a PURE aggregate, so `validate_aggregate_containment` governs it. Run
;;     against `target/release/wat --check`, exit codes read by hand:
;;
;;       :durable [count <- :wat::core::i64]     -> exit 0  accepted   (NON-VACUITY control)
;;       :durable [w <- :wat::io::IOWriter]      -> exit 1  REFUSED    (POSITIVE control)
;;                #wat.type/ImpureFieldInPureAggregate, naming ":probe::ctr::Record"
;;
;;     So it is not a convention. Both controls fired; neither arm is assumed.
;;
;; (b) BUT ITS ENROLLMENT HAS A HOLE, AND THAT IS WHAT THIS FILE STANDS ON. `is_pure_type`
;;     decides a Rust opaque's purity from hand-written lists. A PARAMETRIC opaque never reaches
;;     the `TypeExpr::Path` arm at all — it lands in `TypeExpr::Parametric`, whose head match lists
;;     only the kernel channel/peer heads and then falls through to
;;
;;         _ => args.iter().all(|a| is_pure_type(a, types))    // "pure iff its TYPE ARGS are pure"
;;
;;     so the CONTAINER is presumed pure and only its type ARGUMENTS are checked. The
;;     discriminator that proves the args ARE walked (i.e. the miss is the container, not the
;;     recursion) — run, and it cannot live in this file because it would correctly go RED:
;;
;;       (defrecord :probe::R [c <- :wat::cache::Lru<wat::io::IOWriter,wat::core::i64>])
;;         -> exit 1  REFUSED
;;
;;     THE SAME ARM WAS PATCHED BEFORE: `src/check.rs:12868-12882` records `Peer<i64,String>`
;;     judged pure by this exact fallthrough and admitted into a pure Record, fixed 2026-08-03 by
;;     adding four names to the hardcoded head list. `Lru` is the next type standing in the
;;     identical hole — four hand-patches to one stem.
;;
;; ─── why this matters beyond the cache ───────────────────────────────────────────────────────
;; `DESIGN-STONE-the-connection-scoped-world.md` places a `HashMap<ConnId, World>` in `:ephemeral`
;; and needs to know whether that placement is ENFORCED or CHOSEN. `World` will be a
;; `#[wat_dispatch]` opaque, so — today — writing it into `:durable` would compile, exactly as the
;; `Lru` below does. The placement is correct and it is chosen; it is not, yet, a guarantee.
;;
;; Blast radius of enrolling the opaques, MEASURED 2026-08-08: three live families
;; (`cache::Lru`, `sqlite::Connection`, `sqlite::ReadConnection`), 18 corpus sites, and NOT ONE is
;; an illegal aggregate field — all are fn params, correct `:ephemeral` slots, or
;; `:wat::cache::HolographicLru`, which is already (correctly) a `defstruct`. So the fix goes RED
;; on nothing. The NOTE's original "it is a cascade, not a one-liner" warning is, measured, wrong.

(:wat::core::defsurface :probe::w293::Ctr :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::w293::Ctr::GetRequest [])
   (:wat::core::defenum :probe::w293::Ctr::GetResponse :wat::enum::Pure
     :Ok               [value <- :wat::core::i64]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- :wat::core::Vector<wat::core::String>
                        expected <- :wat::core::String
                        got <- :wat::core::String])]
  :features
  [(get [self <- :probe::w293::Ctr  req <- :probe::w293::Ctr::GetRequest]
     -> :probe::w293::Ctr::GetResponse :max-request-bytes 524288)])

;; ⛔ THE DEMONSTRATION. A live, thread-owned `Lru` handle declared in `:durable` — the slot whose
;;    whole contract is "plain EDN that survives a wire and a hibernation." An `Lru` can do
;;    neither. This must not compile. It does.
(:wat::service::defservice :probe::w293::ctr
  :satisfies :probe::w293::Ctr
  :durable   [cache <- :wat::cache::Lru<wat::core::String,wat::core::i64>]
  :ephemeral []
  :impls
  [(get [s req]
     (:wat::service::Outcome::Reply s (:probe::w293::Ctr::GetResponse::Ok 1)))])
