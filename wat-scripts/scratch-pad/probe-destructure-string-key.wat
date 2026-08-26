;; probe-destructure-string-key.wat — does `{v :k}` SILENTLY MISS a String-keyed map?
;;
;; Stone 1's rider reports that the hash-destructure match arm builds a KEYWORD key at runtime
;; (`format!(":{}", field)`) and looks it up in a map whose keys are STRINGS — so it type-checks,
;; runs, and returns `:None`. Not an error. A miss.
;;
;; If that is right it is NOT an MCP problem and NOT a JSON problem: ANY String-keyed map
;; destructured with `{v :k}` reports "absent" for a key that is present. That is a hidden
;; failure — the caller cannot tell "no such key" from "your key is the wrong TYPE" — and it is
;; the class this whole arc exists to kill.
;;
;; This isolates it from JSON entirely: the map is built by hand in wat, no decode involved.
;;
;;   MISS  => `:None` for a key that IS in the map  -> a silent failure in the destructure
;;   HIT   => `Some "v"`                            -> the rider's diagnosis is wrong

(:wat::core::defn :probe::string-keyed [] -> (:wat::core::HashMap :wat::core::String :wat::core::String)
  (:wat::hashmap::assoc
    (:wat::core::HashMap :wat::core::String :wat::core::String)
    "edn" "the-value"))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [m (:probe::string-keyed)]
    (:wat::core::do
      ;; CONTROL — the direct accessor on a concretely-typed map. This must HIT; if it does not,
      ;; the map itself is wrong and nothing below means anything.
      (:wat::kernel::println (:wat::hashmap::get m "edn"))
      ;; THE SUBJECT — the destructure sugar on the SAME map, same key.
      (:wat::kernel::println
        (:wat::core::match m
          ({s :edn} s))))))
