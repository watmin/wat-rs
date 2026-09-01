;; DISCONFIRMING PROBE — vigilia Class A4: a SECOND session on the same thread silently
;; un-enforces the first session's memory ceiling.
;;
;; `alloc_counter::SESSION_ORIGIN` is ONE `Cell<Option<usize>>` per THREAD, and
;; `mark_session_origin` — called from `arm-session`, which every `compile-all` reaches — sets it
;; unconditionally. So compiling a second session REBASES the zero point, and everything the
;; FIRST session had already staged stops being charged to it. `session_bytes` then reads
;; `thread_bytes - origin_B`, which is a fraction of what session A actually holds.
;;
;; The module states the assumption honestly (*"one session per thread at a time"*) and names the
;; line that would move. This fixture is the measurement that the assumption is already false:
;; `arm_lease.rs` holds two live sessions on one thread in a GREEN test, and sequential
;; `compile-all` on one thread is the ordinary shape, not a corner.
;;
;; ── THE SHAPE, AND WHY THE CEILING IS 4 MB ──────────────────────────────────────────────────
;;
;; Both arms stage the SAME 16_000 facts into ONE session, in two rounds of 8_000. The only
;; difference is a single unrelated `compile-all` between the rounds. Measured at HEAD 74e7f2dd7:
;;
;;   ceiling 4_000_000 → control REFUSES at staged 8477 · probe NO-BREACH   ← the differential
;;   ceiling 2_500_000 → control 4898 · probe 4896                          ← both refuse
;;   ceiling 1_500_000 → control 2512 · probe 2510                          ← both refuse
;;
;; The lower ceilings confirm the mechanism rather than weakening it: there the breach lands in
;; ROUND ONE, before the rebase can forgive anything, so the two arms agree. The differential
;; appears exactly when the rebase falls between the rounds — which is what it is supposed to do.
;;
;; ── THE THIRD ARM, AND WHY IT IS NOT DECORATION ─────────────────────────────────────────────
;;
;; `rearm` hands the SAME session back to `arm-session` after it has already staged — legal
;; (`arm-session`'s intern HIT path exists and increments the lease; `syntax.wat` discourages it
;; only because the lease then leaks), and the one shape where the SECOND mark carries the FIRST
;; session's own key. Keying the origin is not enough there: `mark_session_origin` must also refuse
;; to overwrite an origin it already holds. Added 2026-08-30 because the strike's prescribed
;; mutation — "make `mark_session_origin` clobber regardless of id" — turns out to be INERT against
;; the `control`/`probe` pair: with distinct keys, `insert` and `or_insert` behave identically, and
;; every arm stayed green. Measured with `or_insert` replaced by `insert`:
;;
;;   control REFUSED · probe REFUSED · rearm NO-BREACH   ← only this arm can see it
;;
;; ⛔⛔ THAT LAST LINE IS FALSE, AND WAS FALSE THE DAY IT SHIPPED. Re-driven 2026-08-31 during
;; A7: with `or_insert` replaced by `insert`, this whole test goes **GREEN** — the `rearm` arm
;; sees nothing. The mask is `LAST_ORIGIN`, the one-entry cache in front of `SESSION_ORIGINS`,
;; which is never invalidated on a write: the first staging round caches `(key, origin0)`, the
;; re-arm's clobber rewrites the map but not the cache, and every later `session_bytes` takes the
;; fast path and returns the OLD origin. The code is still correct — the cache is sound GIVEN
;; `or_insert` — but the cache and this arm landed in the SAME commit (`42704d57b`), so the
;; measurement above was taken before the cache existed beside it and was never re-taken. A
;; self-certifying measurement is one nobody re-checks; that is the shape `wat-rs/CLAUDE.md`
;; opens with, committed here by the hand that wrote the warning.
;;
;; ✅ THE LIVE GATE for the non-clobber rule is
;; `probe_arc278_import_accounting::an_origin_already_filed_is_never_re_based` (arc 278 A7),
;; which is a unit-level probe and DOES go red under that mutation — driven both ways.
;; This arm still earns its place for the KEYING half, which is what its `#[test]` doc claims.
(:wat::config::rete::set-max-session-bytes! 4000000)

(:wat::core::defrecord :sc::Edge [a <- :wat::core::i64  b <- :wat::core::i64])
(:wat::rete::defrule :sc::noop :when [(:sc::Edge (?a <- :a))] :then [])

(:wat::core::defn :sc::compile [] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :sc) (:wat::core::PersistentVector))
    ((:wat::rete::CompileOutcome::Compiled __s) __s)
    ((:wat::rete::CompileOutcome::MayNotTerminate __r __f)
      (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None))))

;; Hand an ALREADY-STAGED session back to `arm-session` — the intern HIT path, and the one door
;; through which a second `mark_session_origin` arrives carrying the first session's own key.
(:wat::core::defn :sc::rearm [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::arm-session s)
    ((:wat::rete::CompileOutcome::Compiled __s) __s)
    ((:wat::rete::CompileOutcome::MayNotTerminate __r __f)
      (:wat::kernel::assertion-failed! "rearm: the rule set may not terminate" :wat::core::None :wat::core::None))))

;; Stage `n` facts, SHORT-CIRCUITING on a ceiling so the first breach is carried back intact.
(:wat::core::defn :sc::stage [s <- :wat::rete::Session  n <- :wat::core::i64] -> :wat::rete::InsertOutcome
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::rete::InsertOutcome  i <- :wat::core::i64] -> :wat::rete::InsertOutcome
      (:wat::core::match acc
        ((:wat::rete::InsertOutcome::Inserted session)
          (:wat::rete::insert session (:sc::Edge :a i :b (:wat::core::i64::+ i 1))))
        ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __l __u __s) acc)))
    (:wat::rete::InsertOutcome::Inserted s)
    (:wat::core::range 0 n)))

(:wat::core::defn :sc::stage-more [o <- :wat::rete::InsertOutcome  n <- :wat::core::i64] -> :wat::rete::InsertOutcome
  (:wat::core::match o
    ((:wat::rete::InsertOutcome::Inserted s) (:sc::stage s n))
    ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __l __u __st) o)))

(:wat::core::defn :sc::rearm-more [o <- :wat::rete::InsertOutcome  n <- :wat::core::i64] -> :wat::rete::InsertOutcome
  (:wat::core::match o
    ((:wat::rete::InsertOutcome::Inserted s) (:sc::stage (:sc::rearm s) n))
    ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __l __u __st) o)))

(:wat::core::defn :sc::report [o <- :wat::rete::InsertOutcome  tag <- :wat::core::String] -> :wat::core::nil
  (:wat::core::match o
    ((:wat::rete::InsertOutcome::Inserted staged)
      (:wat::core::do (:wat::kernel::println tag) (:wat::kernel::println "NO-BREACH")))
    ((:wat::rete::InsertOutcome::MemoryCeilingExceeded limit used staged)
      (:wat::core::do (:wat::kernel::println tag) (:wat::kernel::println "REFUSED")))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    ;; CONTROL — one session, two staging rounds, nothing in between.
    (:sc::report (:sc::stage-more (:sc::stage (:sc::compile) 8000) 8000) "control")
    ;; PROBE — identical workload; one unrelated `compile-all` BETWEEN the rounds.
    (:wat::core::let [a  (:sc::compile)
                      o1 (:sc::stage a 8000)
                      b  (:sc::compile)]
      (:sc::report (:sc::stage-more o1 8000) "probe"))
    ;; REARM — identical workload; the SAME session re-armed between the rounds, so the second
    ;; `mark_session_origin` arrives under the first session's own key. Keyed-but-clobbering is
    ;; indistinguishable from keyed-and-refusing above; only here does it show.
    (:sc::report (:sc::rearm-more (:sc::stage (:sc::compile) 8000) 8000) "rearm")))
