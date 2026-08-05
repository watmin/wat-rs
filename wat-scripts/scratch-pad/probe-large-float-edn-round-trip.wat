;; probe-large-float-edn-round-trip.wat — disconfirming probe for
;; BRIEF-edn-float-writer-round-trips.md.
;;
;; A finite f64 literal at/above the old 1e16 boundary. Before the writer fix,
;; `write_float` falls into its `else` branch above 1e16 and emits plain
;; `Display`, which never uses a `.` or `e` for f64 — the EDN reader then reads
;; the ~200-digit run as an out-of-range integer and rejects it. This is what
;; turns `probe_arc170_edn_bridge_unspellable::c03_the_whole_corpus_crosses_the_wire`
;; RED at HEAD. After the fix, the literal must round-trip and the gate goes GREEN.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [big 1e200]
    (:wat::kernel::println (:wat::core::str big))))
