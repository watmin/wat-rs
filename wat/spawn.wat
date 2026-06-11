;; wat/spawn.wat — the HOST opts for spawn-program (arc 259, The Forced Hand).
;;
;; The Keymaker.  (The Matrix Reloaded, 2003 — the little man in the Château who
;; cuts a different key for every door, and the right key is the only thing that
;; opens the backdoor.)  Each constructor below cuts exactly one key, for exactly
;; one hosting-door:
;;
;;   (thread)            — cuts a trivial key; the door is right here in this process.
;;   (process)           — cuts a trivial key; the door is a forked child universe.
;;   (remote url key)    — WILL NOT CUT A KEY without a url AND a signing-key. The
;;                         door to a remote host does not open without them.
;;
;; That refusal is the forced hand: "remote without a url" is not a runtime check —
;; it is an UNCUTTABLE KEY. The constructor's own arity is the lock. spawn-program
;; is a clause-set that matches on the key's TYPE (ThreadOpts / ProcessOpts /
;; RemoteOpts) and opens the matching door — and every new kind of host that ever
;; reveals itself is one new key + one new clause, the sig unmoved.  "We do not
;; need the Oracle to tell us that." We need only the right key.
;;
;; See docs/arc/2026/06/259-forced-hand/DESIGN.md § "The spawn primitive".
;; Loads AFTER wat/Record.wat (uses :wat::Record::def).

;; ── The keys (host opts records) ─────────────────────────────────────────────
;; ThreadOpts / ProcessOpts carry no config — their TYPE is the whole message
;; (where to host). RemoteOpts carries what a remote door demands; its constructor
;; cannot be called without both, so an unconfigured remote is unrepresentable.
(:wat::Record::def :wat::spawn::ThreadOpts [])
(:wat::Record::def :wat::spawn::ProcessOpts [])
(:wat::Record::def :wat::spawn::RemoteOpts
  [remote-url <- :wat::core::String
   signing-key <- :wat::core::String])

;; ── The Keymaker's friendly hand (ergonomic constructors) ────────────────────
;; (thread) / (process) / (remote url key) — the user-facing keys spawn-program reads.
(:wat::core::defn :wat::spawn::thread [] -> :wat::spawn::ThreadOpts
  (:wat::spawn::ThreadOpts))

(:wat::core::defn :wat::spawn::process [] -> :wat::spawn::ProcessOpts
  (:wat::spawn::ProcessOpts))

(:wat::core::defn :wat::spawn::remote
  [url <- :wat::core::String  signing-key <- :wat::core::String] -> :wat::spawn::RemoteOpts
  (:wat::spawn::RemoteOpts url signing-key))
