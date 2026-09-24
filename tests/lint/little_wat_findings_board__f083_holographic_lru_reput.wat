;; Board specimen — the-little-wat F-083: re-putting a key into a `HolographicLru` DELETES it.
;;
;; SHAPE: WRONG-ANSWER — the checker accepts AND the runtime exits 0. Exit codes cannot see
;; this one at all: the only evidence is the VALUE printed, `0` where `1` is correct. That is
;; why this row pins stdout, and why the board refuses a (0, 0) row that pins nothing.
;;
;; ⚠ SPELLING: `the-little-wat`'s own repro says `HolographicLru::new`, which this tree has
;; RETIRED in favour of `HolographicLru/new`. Written their way the file dies at CHECK and the
;; row reads a plausible, WRONG (1, 3). The retirement is exactly what blinded their instrument.
;; ⛔ Do NOT "fix" this file. The finding is that `len` says 0 after two puts of one key.
;;
;; ⚠ THE `Result/expect` IS NOT A FIX AND DID NOT MOVE THE ROW. Excursus 003 stone A made
;; `HolographicLru/new` return `(Result :- [HolographicLru :wat::cache::Fault])` (the-little-wat
;; F-084 — a non-positive capacity was a Rust panic), so this call site had to be rewrapped to
;; keep compiling; the wrap was applied by `wat-scripts/fixes/wrap-cache-new-in-result-expect.wat`,
;; not by hand. Capacity `8` is positive, so the expect never fires and the row was re-measured
;; UNCHANGED at (check 0, run 0, stdout "0"). F-083 is a different defect and stone A did not
;; touch it.
;;
;; ⚠ THE SAME, AGAIN, FOR THE TWO PUTS. Excursus 003 stone C made `HolographicLru/put` return
;; `(Result :- [nil :wat::cache::Fault])` (an unhashable key was a Rust panic), so `_1`/`_2` were
;; rewrapped by `wat-scripts/fixes/wrap-cache-put-in-result-expect.wat`. A `HolonAST` key is always
;; hashable, so neither expect fires; stone C also moved the LRU push AHEAD of the Hologram write
;; inside `put`, but the Hologram still sees put-then-remove of the same key, so a re-put still
;; empties it. Row re-measured UNCHANGED at (check 0, run 0, stdout "0").
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [h (:wat::core::Result/expect (:wat::cache::HolographicLru/new (:wat::holon::filter-accept-any) 8) ":wat::cache::Lru/new refused the capacity: it must be positive")
                    k (:wat::holon::leaf "a")
                    _1 (:wat::core::Result/expect (:wat::cache::HolographicLru/put h k (:wat::holon::leaf "v1")) ":wat::cache::Lru/put refused the key: it must be a hashable value")
                    _2 (:wat::core::Result/expect (:wat::cache::HolographicLru/put h k (:wat::holon::leaf "v2")) ":wat::cache::Lru/put refused the key: it must be a hashable value")]
    (:wat::kernel::println (:wat::cache::HolographicLru/len h))))
