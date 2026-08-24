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
(:wat::core::defn :user::with-network :- [T]
  [base    <- :wat::rete::Session
   body-fn <- [:wat::rete::Session :-> T]]
  -> T
  (:wat::core::let [armed  (:wat::rete::arm-session base)
                    result (body-fn armed)]
    (:wat::core::do
      (:wat::rete::release-session armed)
      result)))

;; ── the network the user's query program would supply ─────────────────────────
(:wat::core::defn :user::build-base [] -> :wat::rete::Session
  (:wat::core::let
    [c1   (:wat::core::quote (:g::Temp (?loc <- :location)))
     c2   (:wat::core::quote (:g::Wind (?loc <- :location)))
     rhs  (:wat::core::quote (:g::Match ?loc))
     rule (:wat::rete::Rule :name "temp-and-wind"
            :lhs (:wat::core::PersistentVector c1 c2)
            :rhs (:wat::core::PersistentVector rhs))]
    (:wat::rete::compile-all (:wat::core::PersistentVector rule)
                             (:wat::core::PersistentVector (:g::q-match)))))

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
      (:wat::core::i64::+ acc (:user::grep-one-file armed f)))
    0
    files))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [base  (:user::build-base)
     files (:wat::core::Vector :- [:wat::core::String] "fileA" "fileB" "fileC")
     total (:user::with-network base
             (:wat::core::fn [r <- :wat::rete::Session] -> :wat::core::i64
               (:user::grep-files r files)))
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
  [base    <- :wat::rete::Session
   body-fn <- [:user::Overlay :-> T]]
  -> T
  (:wat::core::let
    [armed  (:wat::rete::arm-session base)
     ;; the ONLY handle the body gets: facts in, a FIRED session out, always re-seeded
     overlay (:wat::core::fn [facts <- (:wat::core::PersistentVector :- [:wat::core::Record])]
               -> :wat::rete::Session
               (:wat::rete::fire-rules (:wat::rete::insert-all armed facts)))
     result (body-fn overlay)]
    (:wat::core::do
      (:wat::rete::release-session armed)
      result)))

(:wat::core::defn :user::main-variant-b [] -> :wat::core::i64
  (:wat::core::let [base (:user::build-base)]
    (:user::with-overlay base
      (:wat::core::fn [overlay <- :user::Overlay]
        -> :wat::core::i64
        (:wat::core::foldl
          (:wat::core::fn [acc <- :wat::core::i64  loc <- :wat::core::String] -> :wat::core::i64
            (:wat::core::i64::+ acc
              (:wat::core::length
                (:wat::rete::query
                  (overlay (:wat::core::PersistentVector :- [:wat::core::Record]
                             (:g::Temp :location loc)
                             (:g::Wind :location loc)))
                  (:g::q-match)))))
          0
          (:wat::core::Vector :- [:wat::core::String] "fileA" "fileB" "fileC"))))))
