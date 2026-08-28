;; TUPLES ARE FINE IN CORE. RETE IS THE ONE UNDER-SERVING THEM.
;;
;; Arc 278 § 4.1's reachability ledger reported `:wat::rete::core::Tuple` as unrunnable, and I
;; wrote that "no rete row accesses a Tuple's elements, so even with an arm nothing could compare
;; one" — reasoning from an absence to a conclusion. That is the CORPUS FALLACY this arc already
;; refuted once, in `vocabulary.rs`'s own totality gate: absence of a caller is not evidence of
;; absence of need. The builder rejected it, and measuring says the builder was right.
;;
;; ─── What core actually gives a Tuple (all verified by running this file) ─────────────────────
;;   · `TypeExpr::Tuple` — a first-class type;  `Value::Tuple(Arc<Vec<Value>>)` — a first-class value
;;   · `(:wat::core::Tuple a b c)` — the constructor
;;   · `length` / `empty?` / `contains?` — via the generic container dispatch
;;     (NOT `:wat::core::Tuple/length`, which is an unknown function; the inner fns are reached
;;     through `StreamContainer::Tuple`)
;;   · `first` / `second` / `third` — POSITIONAL PROJECTION, the right idiom for a fixed-arity
;;     heterogeneous product. This is the door, and it works.
;;
;; ─── What does NOT work, and did not need to ──────────────────────────────────────────────────
;;   `get` / `nth` refuse at check; `Tuple/get` does not exist; `let` positional destructure
;;   refuses; and top-level `match` cannot see a Tuple at all — `MatchShape` (`check.rs:6288`)
;;   carries Option / Result / Enum / open-typed and NO Tuple, so the tuple-destructure code at
;;   `check.rs:7270` is reachable only as a NESTED sub-pattern. None of that is a flaw: a language
;;   with `first`/`second`/`third` serves tuples.
;;
;; ─── The actual gap, and it is rete's ─────────────────────────────────────────────────────────
;;   `RETE_OPS` carries `:wat::rete::core::Tuple` (the CONSTRUCTOR) and NO accessor that admits a
;;   Tuple. Its first-family rows are per-container — `PersistentVector/first`, `Vector/first`,
;;   `List/first` — and the compiled `first_of` (`expr_ir.rs`) matches PersistentVector / Vec /
;;   List and rejects everything else. There is NO `second` and NO `third` row at all, for any
;;   container. So a rete rule can BUILD a tuple in a `where` fence and never read one element of
;;   it — which is why `Tuple` is one of the three rows appearing nowhere in the 1569-file corpus.
;;   It was never usable, since genesis.

(:wat::core::defn :user::projection [] -> :wat::core::String
  (:wat::core::string::concat
    (:wat::core::i64::to-string (:wat::core::first  (:wat::core::Tuple 7 99 512)))
    (:wat::core::string::concat
      (:wat::core::string::concat "/" (:wat::core::i64::to-string (:wat::core::second (:wat::core::Tuple 7 99 512))))
      (:wat::core::string::concat "/" (:wat::core::i64::to-string (:wat::core::third  (:wat::core::Tuple 7 99 512)))))))

(:wat::core::defn :user::measurement [] -> :wat::core::String
  (:wat::core::string::concat
    (:wat::core::i64::to-string (:wat::core::length (:wat::core::Tuple 7 99 512)))
    (:wat::core::string::concat "/" (:wat::core::bool::to-string
      (:wat::core::contains? (:wat::core::Tuple 7 99 512) 99)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::PersistentMap
      "first/second/third" (:user::projection)
      "length/contains?"   (:user::measurement))))
