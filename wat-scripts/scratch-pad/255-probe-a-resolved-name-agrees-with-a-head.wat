;; Scratch probe — arc 255 Stone A-2-ii-a: A RESOLVED NAME GETS THE SAME DOORS AS A HEAD.
;;
;; BRIEF:  docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-A-2-ii-a-a-resolved-name-gets-the-same-doors-as-a-head.md
;; DESIGN: docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-A-2-ii-a-a-resolved-name-gets-the-same-doors-as-a-head.md
;;
;; The stone patches `classify_closure`'s `FunctionBody::Native` arm (`src/rete/purity.rs`) so a
;; resolved fn's OWN name, when it has one, is routed through `head_ok`'s full door ladder
;; (constructor_meta -> accessor_meta -> sym.has_function/classify_fn -> intrinsic_meta -> deny)
;; instead of consulting `intrinsic_meta` alone — carrying both recursion guards (`seen`,
;; `closure_seen`) across the delegation. An anonymous native (`name: None`) keeps default-deny.
;;
;; ⛔ WHAT THIS PROBE CANNOT SHOW, established by reading the source, not merely by running it:
;; `FunctionBody::Native` has ZERO live constructors anywhere in this codebase today. Grepped
;; exhaustively for `body: FunctionBody::Native` / `FunctionBody::Native,` as a struct-literal
;; field across `src/` (including the `wat-macros` proc-macro crate) — nothing builds one. The
;; variant's own doc (`src/value/environment.rs`) says so outright: "Used starting in arc 255.1b;
;; nothing constructs this in slice 255.1a." Every generated accessor / constructor / `is-T?`
;; predicate / newtype accessor (`register_aggregate_methods`, `register_type_predicates`,
;; `runtime.rs`) is `FunctionBody::Wat`; every core-spelled builtin (`:wat::core::+`,
;; `Record/field-at`, `struct-field`, …) is dispatched through the separate `NativeHandler` table
;; (`src/intrinsic/mod.rs`) and never wrapped in a `Function` at all — a bare keyword naming one
;; evaluates to `Value::wat__core__keyword`, not `Value::wat__core__fn` (`runtime.rs:~5078`).
;;
;; So no wat program can construct a NAMED `Value::wat__core__fn` with a Native body: the patched
;; arm is unreachable from THIS surface today, and neither "the accessor agrees true/true through
;; a binding" nor an anonymous-native negative row can be built to exercise it — the brief's own
;; hedge, "an anonymous native through a binding, IF YOU CAN CONSTRUCT ONE", anticipates exactly
;; this. See the rider's report for the STOP-3 finding this triggers; nothing here was widened to
;; force a row to pass.
;;
;; What CAN be measured on THIS runtime, and is measured below — the record accessor is WAT-
;; bodied (`register_aggregate_methods`), so it exercises `classify_closure`'s Wat arm, which this
;; stone does NOT touch, in both rows:
;;
;;   row 1 — a record field accessor, as a HEAD            -> true   (head_ok's accessor_meta
;;                                                                     door — untouched, unaffected)
;;   row 2 — the SAME accessor, through an env binding      -> false (UNCHANGED by this stone — the
;;           Wat arm walks the accessor's synthesized body and denies on an internal unclassified
;;           verb, `:wat::core::Record/field-at`, hit mid-walk; a DIFFERENT mechanism than the one
;;           this stone's patch addresses, and the accessor asymmetry this stone does NOT close)
;;   row 3 — an effectful fn, through an env binding        -> false (no widening — same Wat-arm
;;           mechanism A-2-i's own probes already cover, re-asserted here as the negative row this
;;           stone's brief asks for)

(:wat::core::defrecord :probe::R [sk <- :wat::core::i64])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    ;; row 1 — accessor as a HEAD -> true
    (:wat::kernel::println
      (:wat::rete::pure? (:wat::core::quote
        (:wat::core::fn [a <- :probe::R] -> :wat::core::i64
          (:probe::R/sk a)))))
    ;; row 2 — the SAME accessor, resolved through an env binding -> false (unchanged)
    (:wat::kernel::println
      (:wat::core::let [k :probe::R/sk]
        (:wat::rete::pure? (:wat::core::quote
          (:wat::core::fn [a <- :probe::R] -> :wat::core::i64
            (k a))))))
    ;; row 3 — an EFFECTFUL fn, resolved through an env binding -> false (no widening)
    (:wat::kernel::println
      (:wat::core::let [k (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
                             (:wat::core::do (:wat::kernel::println "!") x))]
        (:wat::rete::pure? (:wat::core::quote
          (:wat::core::fn [a <- :wat::core::i64] -> :wat::core::i64
            (k a))))))))
