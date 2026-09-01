;; ★ THE IMPORT DOOR IS CHARGED FOR THE NETWORK IT BUILDS — strike-import-accounting (A7).
;;
;; `import_export` used to call neither `mark_session_origin` nor `check_session_ceiling`, so what
;; it allocated was free — and worse than uncounted: `alloc_counter::session_bytes` files an
;; unmarked session's origin at the FIRST CHECK, so the ceiling began after the network already
;; existed. Driven on the same 2 MB of allocation: marked-at-birth reads 2097268, never-marked
;; reads 0.
;;
;; This program compiles one rule, exports it, and imports it under an 8 KiB ceiling. The import
;; of this network allocates 15_172 bytes live (measured at the door, 2026-08-31), so the import
;; must REFUSE — 1.85x the ceiling, which is the margin the number 8192 was picked for.
;;
;; ⛔ WHAT THIS FIXTURE IS ACTUALLY FOR. The natural place to file the origin is after the build —
;; that is where the key (the network `PMap`'s identity) exists — and doing so reads a
;; `thread_bytes()` that ALREADY CONTAINS the build, charging the session zero. An origin would be
;; visibly filed and every probe that only asks "is one filed?" would pass. This fixture is the one
;; that cannot: it asks what the origin is WORTH. If `import_export` files
;; `thread_bytes()`-at-the-filing instead of the reading captured at the door, the ceiling sees ~0,
;; the import succeeds, and "IMPORTED" appears on stdout.
(:wat::config::rete::set-max-session-bytes! 8192)

(:wat::core::defrecord :ia::Temp [c <- :wat::core::i64])
(:wat::core::defrecord :ia::Hit [c <- :wat::core::i64])

(:wat::rete::defquery :ia::q-Hit :params [] :when [(?fact <- :ia::Hit)])

(:wat::rete::defrule :ia::cool
  :when [(:ia::Temp (?c <- :c))
         (:wat::rete::where (:wat::rete::core::i64::< ?c 20))]
  :then [(:ia::Hit ?c)])

(:wat::core::defn :ia::compiled [] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::compile-all
      (:wat::core::PersistentVector (:ia::cool))
      (:wat::core::PersistentVector (:ia::q-Hit)))
    ((:wat::rete::CompileOutcome::Compiled __session) __session)
    ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type)
      (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [e  (:wat::rete::export (:ia::compiled))
                    s1 (:wat::rete::import e)]
    (:wat::kernel::println "IMPORTED")))
