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
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [h (:wat::cache::HolographicLru/new (:wat::holon::filter-accept-any) 8)
                    k (:wat::holon::leaf "a")
                    _1 (:wat::cache::HolographicLru/put h k (:wat::holon::leaf "v1"))
                    _2 (:wat::cache::HolographicLru/put h k (:wat::holon::leaf "v2"))]
    (:wat::kernel::println (:wat::cache::HolographicLru/len h))))
