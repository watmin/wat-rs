;; probe-mcp-stone1-read-json-gate.wat — RED gate, Stone 1 of `wat --mcp`.
;;
;; `:wat::edn::read-json` + `:wat::edn::ReadJsonOutcome`
;; (docs/arc/2026/06/278-rules-engine/BRIEF-mcp-stone-1-read-json.md).
;;
;; Three assertions; #3 is load-bearing (a bad line must not end the caller).

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    ;; 1 — decodes: a well-formed JSON object → ::Value.
    (:wat::core::match (:wat::edn::read-json "{\"edn\":\"42\"}")
      ((:wat::edn::ReadJsonOutcome::Value v)
        (:wat::kernel::println "1 decodes: OK -> ::Value"))
      ((:wat::edn::ReadJsonOutcome::Malformed cause)
        (:wat::test::assert-true false)))

    ;; 2 — CRUX-1: is a nested field readable from wat? `ReadJsonOutcome` is PARAMETRIC
    ;; (`(ReadJsonOutcome :- [T])`, corrected from an initial pass that fixed the payload at the
    ;; bare `:wat::core::Value` — the universal top, produce-only: UP is free, DOWN is
    ;; checked, so nothing could ever read a field back out of it). With `T` flowing from
    ;; the caller's use, `m` binds at a real `(HashMap :- [K V])` and the ordinary
    ;; `:wat::core::HashMap/get` accessor applies directly — no destructure sugar needed.
    (:wat::core::match (:wat::edn::read-json "{\"edn\":\"42\"}")
      ((:wat::edn::ReadJsonOutcome::Value m)
        (:wat::core::match (:wat::core::HashMap/get m "edn")
          ((:wat::core::Some s)
            (:wat::core::do
              (:wat::test::assert-eq s "42")
              (:wat::kernel::println (:wat::core::string::concat "2 CRUX-1 HashMap/get -> " s))))
          (:wat::core::None (:wat::test::assert-true false))))
      ((:wat::edn::ReadJsonOutcome::Malformed cause) (:wat::test::assert-true false)))

    ;; 3 — a malformed line leaves the caller ALIVE: ::Malformed, THEN a form
    ;; evaluated afterward still runs — its result is asserted, not just observed.
    (:wat::core::match (:wat::edn::read-json "{not json")
      ((:wat::edn::ReadJsonOutcome::Value v)
        (:wat::test::assert-true false))
      ((:wat::edn::ReadJsonOutcome::Malformed cause)
        (:wat::kernel::println "3 malformed: OK -> ::Malformed")))
    (:wat::test::assert-eq (:wat::core::i64::+ 2 2) 4)
    (:wat::kernel::println "3 survived: OK -> (+ 2 2) = 4")))
