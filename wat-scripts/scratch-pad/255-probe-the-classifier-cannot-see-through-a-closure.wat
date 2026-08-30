;; Scratch probe — arc 255 / W7. THE FINDING THAT STOPPED THE sort$native IMPOSITION.
;;
;; Builder ruled: impose pure ∧ deterministic ∧ total on sort's comparator.
;; This probe asks the substrate's OWN classifier (`:wat::rete::pure?`, the same
;; `classify_expr` walk `src/freeze.rs:803` uses to impose exactly this on sigma fns)
;; what it says about the THREE comparator shapes that actually reach `sort$native` (named `sort'` when this was measured).
;;
;; ── MEASURED 2026-08-30 ──
;;   1. sort/1's comparator      (fn [a b] (< a b))                  -> true
;;   2. sort-by's comparator     (fn [a b] (< (keyfn a) (keyfn b)))  -> FALSE
;;   3. an effectful comparator  (fn [a b] (do (println …) (< a b))) -> false
;;
;; ⛔ ROW 2 IS THE STOP. `wat/core.wat:1537,1546` build sort-by's comparator around
;; the FREE VARIABLE `keyfn` (and `cmp`), bound to a caller-supplied fn VALUE at
;; runtime. `classify_expr` is AST-STRUCTURAL and default-denies an unknown head
;; (`src/rete/purity.rs:920`), so it cannot see through value-level indirection.
;; Imposing `pure?` at sort$native's door would correctly refuse row 3 AND break
;; every `sort-by` / `sort/2` caller: wat/bracket.wat:783, wat/query/mem.wat:136,163,
;; tests/collection/sort.wat, tests/resolve/probe_arc251_ordering_surface.wat.
;;
;; ★ THE GENERAL FINDING, bigger than sort: the axis classifier cannot follow a
;; captured fn value. `src/freeze.rs:803` has the SAME blind spot — it never bit
;; only because sigma fns are closed arithmetic. This, not `effectful_by_prefix`,
;; is the real reason the W7 HOF family is hard.
;; Not a permanent fixture — delete when the ruling ships.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println
      (:wat::rete::pure? (:wat::core::quote
        (:wat::core::fn [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::bool
          (:wat::core::< a b)))))
    (:wat::kernel::println
      (:wat::rete::pure? (:wat::core::quote
        (:wat::core::fn [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::bool
          (:wat::core::< (keyfn a) (keyfn b))))))
    (:wat::kernel::println
      (:wat::rete::pure? (:wat::core::quote
        (:wat::core::fn [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::bool
          (:wat::core::do (:wat::kernel::println "x") (:wat::core::< a b))))))))
