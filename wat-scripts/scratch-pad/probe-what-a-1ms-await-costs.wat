;; probe-what-a-1ms-await-costs.wat
;;
;; The publisher's rejection path does `await-timer-ms 1` 3800 times per circuit
;; run (circuit.wat:866: `:wat::kernel::after` + `recv` — a timer over the wire,
;; not a thread sleep, so it is honest in mechanism).
;;
;; The attribution charged it at 1 ms x 3800 = 3.8 s. But "1 ms" is the DELAY,
;; not the COST: each call mints a timer peer, parks, wakes, and tears it down.
;;
;; Measure the real per-call cost, and the floor it sits on (a 0-delay await is
;; illegal — NonZeroDuration — so 1 ms IS the floor for this verb).

(:wat::config::set-redef! true)

(:wat::core::defn :aw::now [] -> :wat::core::i64
  (:wat::time::epoch-nanos (:wat::time::now)))

(:wat::core::defn :aw::await-ms [ms <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::match
    (:wat::kernel::recv
      (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Milliseconds ms) :done))
    ((:wat::kernel::RecvOutcome::Message _m) nil)
    ((:wat::kernel::RecvOutcome::Lost _c) nil)
    (:wat::kernel::RecvOutcome::Stopped nil)
    (:wat::kernel::RecvOutcome::Closed nil)
    (:wat::kernel::RecvOutcome::TimedOut nil) ((:wat::kernel::RecvOutcome::Malformed _cause) (:wat::kernel::assertion-failed! "recv: malformed frame — the peer could not decode our message; this arm is an UNMIGRATED PLACEHOLDER (a-momentary-failure-is-not-fatal, stone 2 replaces it with report-final)" :wat::core::None :wat::core::None))))

(:wat::core::defn :aw::spin [n <- :wat::core::i64  ms <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::i64::<= n 0)
    0
    (:wat::core::let [_ (:aw::await-ms ms)]
      (:aw::spin (:wat::i64::- n 1) ms))))

(:wat::core::defn :aw::run [] -> :wat::core::String
  (:wat::core::let
    [n  500
     t0 (:aw::now)  _a (:aw::spin n 1)
     t1 (:aw::now)  _b (:aw::spin n 5)
     t2 (:aw::now)
     us1 (:wat::i64::/ (:wat::i64::- t1 t0) n)
     us5 (:wat::i64::/ (:wat::i64::- t2 t1) n)]
    (:wat::core::format
      "n={n};await1ms-us/call={a};await5ms-us/call={b};overhead-us={o};3800-calls-ms={t}"
      :n n
      :a (:wat::i64::/ us1 1000)
      :b (:wat::i64::/ us5 1000)
      ;; a 5ms delay costs 4ms more delay; anything beyond that is machinery
      :o (:wat::i64::/ (:wat::i64::- us1 (:wat::i64::- us5 (:wat::i64::* 4 1000000))) 1000)
      :t (:wat::i64::/ (:wat::i64::* 3800 us1) 1000000))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:aw::run)))
