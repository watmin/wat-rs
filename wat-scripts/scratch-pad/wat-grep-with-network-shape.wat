;; wat-grep-with-network-shape.wat — PROTOTYPE of the overlayed-work UX, written at the
;; CONSUMER first (builder's sequencing: build it in wat-grep, then migrate the UX to rete).
;;
;; What it proves, and why each piece is here:
;;
;;   1. ONE compile, ONE lease, held for the whole run.  `arm-session` takes the intern lease;
;;      `release-session` drops it. Gated by rete's own tests (`intern_release_drops_arm_and_
;;      next_fire_rebuilds`, `intern_overlay_is_not_a_second_lease`).
;;   2. PER-FILE ISOLATION IS FREE.  The Session is a FACT OVERLAY over circuits it does not own
;;      (`src/rete/kernel/arm.rs:572`), and it is immutable — so "reset to the known state" is
;;      NOT AN OPERATION. Each file re-seeds from `base`; `base` is never touched.
;;   3. THE OVERLAY DOES NOT RE-LEASE and does not rebuild the arm — so N files cost ONE build.
;;
;; The `with-network` shape below is the thing under evaluation. It mirrors
;; `:wat::io::with-open-file` (wat/io.wat:40), whose own doc cites Ruby's
;; `File.open(path) do |w| … end` and states the contract this is copying:
;; "managed scope, caller owns only usage."

(:wat::core::defrecord :g::Temp  [location <- :wat::core::String])
(:wat::core::defrecord :g::Wind  [location <- :wat::core::String])
(:wat::core::defrecord :g::Match [location <- :wat::core::String])

(:wat::rete::defquery :g::q-match :params [] :when [(?fact <- :g::Match)])

;; ── THE SHAPE UNDER EVALUATION ────────────────────────────────────────────────
;; with-network — hand an ARMED session to body-fn, release the lease after.
;; Promote to `:wat::rete::with-network` once the ergonomics read right at the call site.
;; ⚠ CORRECTED — the first draft called `arm-session` on the session `compile-all` returns.
;; `compile-all` ALREADY ARMS (wat/rete/compile.wat:1149, DESIGN-STONE-arm-at-compile), and
;; `arm-session`'s HIT path INCREMENTS the lease (arm.rs:709). So that draft took lease 2 and
;; released back to 1 — leaking the lease compile-all took, which is the exact thing this
;; wrapper exists to drop. `with-open-file` had the right shape all along: it OPENS the
;; resource itself. So this ACQUIRES by compiling and RELEASES at scope end; the caller never
;; holds an unreleased lease, and never has to know one exists.
(:wat::core::defn :user::with-network :- [T]
  [rules   <- (:wat::core::PersistentVector :- [:wat::rete::Rule])
   queries <- (:wat::core::PersistentVector :- [:wat::rete::Query])
   body-fn <- [:wat::rete::Session :-> T]]
  -> T
  (:wat::core::let [base   (:wat::rete::compile-all rules queries)
                    result (body-fn base)]
    (:wat::core::do
      (:wat::rete::release-session base)
      result)))

;; ── the network the user's query program would supply ─────────────────────────
(:wat::core::defn :user::the-rules [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::let
    [c1   (:wat::core::quote (:g::Temp (?loc <- :location)))
     c2   (:wat::core::quote (:g::Wind (?loc <- :location)))
     rhs  (:wat::core::quote (:g::Match ?loc))
     rule (:wat::rete::Rule :name "temp-and-wind"
            :lhs (:wat::core::PersistentVector c1 c2)
            :rhs (:wat::core::PersistentVector rhs))]
    (:wat::core::PersistentVector :- [:wat::rete::Rule] rule)))

(:wat::core::defn :user::the-queries [] -> (:wat::core::PersistentVector :- [:wat::rete::Query])
  (:wat::core::PersistentVector :- [:wat::rete::Query] (:g::q-match)))

;; kept for the base-untouched proof below
(:wat::core::defn :user::build-base [] -> :wat::rete::Session
  (:wat::rete::compile-all (:user::the-rules) (:user::the-queries)))

;; ── ONE FILE: overlay its facts on the base, fire, report the user's query ────
;; `base` is the caller's; this returns a COUNT, never the session — so nothing leaks forward.
(:wat::core::defn :user::grep-one-file
  [base <- :wat::rete::Session
   loc  <- :wat::core::String]
  -> :wat::core::i64
  (:wat::core::let
    ;; a file's facts are HETEROGENEOUS — insert-all takes the Record supertype, which is
    ;; exactly the shape a slurped file yields (Node + Named + Span facts in one vector).
    [facts  (:wat::core::PersistentVector :- [:wat::core::Record]
              (:g::Temp :location loc)
              (:g::Wind :location loc))
     fired  (:wat::rete::fire-rules (:wat::rete::insert-all base facts))]
    (:wat::core::length (:wat::rete::query fired (:g::q-match)))))

;; ── THE LOOP — what wat-grep's main becomes ───────────────────────────────────
(:wat::core::defn :user::grep-files
  [armed <- :wat::rete::Session
   files <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64  f <- :wat::core::String] -> :wat::core::i64
      (:wat::i64::+ acc (:user::grep-one-file armed f)))
    0
    files))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [files (:wat::core::Vector :- [:wat::core::String] "fileA" "fileB" "fileC")
     base  (:user::build-base)
     ;; intueri ruling: the body params are the signal. `base` is a VALUE you hold (and could
     ;; thread forward for corpus mode); `overlay` below is a VERB you call. The pair telegraphs
     ;; the FN1/FN2 difference at a doc-free use site, which the verb names alone do not.
     total (:user::with-network (:user::the-rules) (:user::the-queries)
             (:wat::core::fn [base <- :wat::rete::Session] -> :wat::core::i64
               (:user::grep-files base files)))
     _ (:wat::kernel::println total)
     ;; base must STILL be empty — proof the overlay never touched it
     _ (:wat::kernel::println
         (:wat::core::length (:wat::rete::query (:wat::rete::fire-rules base) (:g::q-match))))
     ;; variant B — must agree with A, and its body never holds the base session
     _ (:wat::kernel::println (:user::main-variant-b))]
    nil))

;; ══ VARIANT B — the body never SEES the base session ══════════════════════════
;;
;; Variant A (above) hands the body an armed Session. It works, and it mirrors
;; `with-open-file` exactly. But it leaves ONE thing to discipline: nothing stops a body
;; from threading file N's session into file N+1 and silently contaminating across files.
;; `grep-one-file` re-seeds from `base` because it was WRITTEN to, not because it must.
;;
;; Variant B closes that by construction: the body receives the OVERLAY OPERATION, not the
;; session. There is no base in scope to thread forward, so cross-file leakage has no form.
;; This is the extirpare rung above a convention — the wrong thing cannot be written down.

;; perspicere — the nesting hides a NOUN. Name it, and both signatures below read.
(:wat::core::typealias :user::Overlay
  [(:wat::core::PersistentVector :- [:wat::core::Record]) :-> :wat::rete::Session])

(:wat::core::defn :user::with-overlay :- [T]
  [rules   <- (:wat::core::PersistentVector :- [:wat::rete::Rule])
   queries <- (:wat::core::PersistentVector :- [:wat::rete::Query])
   body-fn <- [:user::Overlay :-> T]]
  -> T
  ;; built ON with-network: same acquire/release scope, one more layer of guarantee.
  (:user::with-network rules queries
    (:wat::core::fn [base <- :wat::rete::Session] -> T
      (body-fn
        (:wat::core::fn [facts <- (:wat::core::PersistentVector :- [:wat::core::Record])]
          -> :wat::rete::Session
          (:wat::rete::fire-rules (:wat::rete::insert-all base facts)))))))

(:wat::core::defn :user::main-variant-b [] -> :wat::core::i64
  (:wat::core::let [_ 0]
    (:user::with-overlay (:user::the-rules) (:user::the-queries)
      (:wat::core::fn [overlay <- :user::Overlay]
        -> :wat::core::i64
        (:wat::core::foldl
          (:wat::core::fn [acc <- :wat::core::i64  loc <- :wat::core::String] -> :wat::core::i64
            (:wat::i64::+ acc
              (:wat::core::length
                (:wat::rete::query
                  (overlay (:wat::core::PersistentVector :- [:wat::core::Record]
                             (:g::Temp :location loc)
                             (:g::Wind :location loc)))
                  (:g::q-match)))))
          0
          (:wat::core::Vector :- [:wat::core::String] "fileA" "fileB" "fileC"))))))
