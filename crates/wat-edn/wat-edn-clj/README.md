# wat-edn-clj

Clojure-side bridge to the [wat-edn](../) EDN format.

Lives inside the `wat-edn` Rust crate for now; will move to a
standalone Clojars artifact once the API stabilizes.

## What it provides

```clojure
(require '[wat-edn.core :as wat])

;; ─── Schema-driven generators ──────────────────────────────
;; .wat files are header files: same artifact wat-rs's type
;; checker reads, exposed to Clojure as schema.

(wat/load-types! "shared.wat")
(wat/list-types)
;; => [enterprise.config/SizeAdjust
;;     enterprise.observer.market/TradeSignal ...]

(wat/gen 'enterprise.config/SizeAdjust
         {:asset :BTC :factor 1.5 :reason "vol spike"})
;; => #enterprise.config/SizeAdjust {:asset :BTC, :factor 1.5,
;;                                    :reason "vol spike"}
;; (validation throws ex-info on field type mismatch or missing
;;  fields, BEFORE the EDN ever leaves Clojure)

(wat/emit 'enterprise.config/SizeAdjust
          {:asset :BTC :factor 1.5 :reason "vol spike"})
;; => "#enterprise.config/SizeAdjust {:asset :BTC, :factor 1.5,
;;                                     :reason \"vol spike\"}"

(wat/read-typed 'enterprise.config/SizeAdjust edn-string)
;; => {:asset :BTC, :factor 1.5, :reason "vol spike"}
;;    on success; throws on wrong tag or invalid fields

;; ─── Untyped read (for any wat.* tag) ──────────────────────

(wat/read-str "#wat.core/Vec<i64> [1 2 3]")
;; => [1 2 3]   (default reader strips collection wrappers)

(wat/read-str "#wat.core/Some<f64> 3.14")
;; => [::wat-edn.core/some 3.14]   (variant tuple)

(wat/some-variant? *1)  ;; => true
(wat/unwrap-some  *2)  ;; => 3.14

;; ─── Variant constructors ──────────────────────────────────

(wat/some-of #inst "2026-04-28T16:00:00Z")
;; → emits as #wat.core/Some #inst "2026-04-28T16:00:00.000-00:00"
(wat/none-of)
(wat/ok-of   42)
(wat/err-of  "boom")
```

## How load-types! works

`load-types!` reads `.wat` source text and extracts every
`(:wat::core::struct :ns::path::Name ...)` form into the registry.
Function declarations, macros, imports — everything else — is
silently skipped. The scanner is hand-rolled (~150 LOC) because
wat's `::` namespace separator collides with Clojure's reader.

The same `.wat` file is consumed by:
- **wat-rs's type checker** as code (struct registration in the
  SymbolTable)
- **wat-edn-clj's load-types!** as schema (Clojure registry of
  field types per tag)

One file. Two readers. The schema is shared.

## v0.1 scope

- Reads `(:wat::core::struct ...)` forms; ignores `:wat::core::enum`
  (will land when wat-rs's enum surface stabilizes)
- Validates primitive field types strictly (`:Keyword`, `:String`,
  `:i64`, `:f64`, `:bool`, `:Bytes`); collections and user-defined
  types are accepted as opaque (no recursive validation yet)
- Round-trips through `clojure.edn/read` — no helper required for
  the standard EDN spec types

## Running the tests

```sh
cd crates/wat-edn/wat-edn-clj
clojure -M:test
```
