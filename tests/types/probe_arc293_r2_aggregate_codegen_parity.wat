;; 293.R2 parity gate — ONE aggregate toolkit, nature is the only variance.
;;
;; RED at HEAD: a GENERIC core-record (:r2::CR :- [T]) and a GENERIC holon-record (:r2::HR :- [T]) each
;; declare a field `v`, but their field accessors :r2::CR/v / :r2::HR/v are NEVER REGISTERED —
;; register_record_methods (runtime.rs:1315) builds the accessor key from entry.name which carries
;; the `<T>`, so the accessor lands at the mangled key `:r2::CR<T>/v` and `:r2::CR/v` resolves to
;; nothing. The GENERIC struct (:r2::ST :- [T]) works (register_struct_methods carries type_params +
;; uses the bare name) — the parity break.
;;
;; GREEN after 293.R2a: one register_aggregate_methods mints accessors for all three natures,
;; generic-aware, bare key — :r2::CR/v and :r2::HR/v resolve. (:r2::probe) => 60.
;;
;; Guard (must stay green): policy c — a holon record is accepted where a core :wat::core::Record is wanted.

(:wat::core::defstruct  :r2::ST :- [T] [v <- :T])
(:wat::core::defrecord  :r2::CR :- [T] [v <- :T])
(:wat::holon::defrecord :r2::HR :- [T] [v <- :T])

;; policy c (holon ⊂ core): a holon record passes where a core-record is wanted.
(:wat::core::defn :r2::want-core [x <- :wat::core::Record] -> :wat::core::i64 99)

(:wat::core::defn :r2::probe [] -> :wat::core::i64
  (:wat::core::let [_chk (:r2::want-core (:r2::HR :v 20))]
    (:wat::core::i64::+
      (:wat::core::i64::+ (:r2::CR/v (:r2::CR :v 10)) (:r2::HR/v (:r2::HR :v 20)))
      (:r2::ST/v (:r2::ST :v 30)))))
