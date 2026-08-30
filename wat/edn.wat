;; wat/edn.wat — user-minted values on the `:wat::edn::` surface.
;;
;; The `:wat::edn::` *verbs* are Rust intrinsics (`src/intrinsic/edn.rs`). The
;; outcome enums (`Validation`, `ReadJsonOutcome`, `ReadForeignOutcome`) are
;; Rust builtins: callers MATCH them, they never mint them. `WriteOpts` is the
;; other kind — a VALUE the caller constructs and passes, on the ProcessOpts
;; precedent (`wat/spawn.wat`: struct, zero-arg default, named single-field
;; variant). That pattern is wat `defstruct` + wat `defn`. This file is that
;; home. A historical `wat/edn.wat` (Tagged/NoTag) was deleted; this is not
;; that file resurrected.
;;
;; Loads after core.wat (defstruct/defn/i64 only). No eval-deps on later files.

(:wat::core::defstruct :wat::edn::WriteOpts
  [inst-digits <- :wat::core::i64])

;; The sane default you never need to touch: 9 fractional digits (nanos).
(:wat::core::defn :wat::edn::opts [] -> :wat::edn::WriteOpts
  (:wat::edn::WriteOpts :inst-digits 9))

;; Customize one field. Out-of-range values are CLAMPED at the JSON verb
;; (`[0, 9]`, matching `:wat::time::to-iso8601`) — not rejected here.
(:wat::core::defn :wat::edn::opts/inst-digits [n <- :wat::core::i64] -> :wat::edn::WriteOpts
  (:wat::edn::WriteOpts :inst-digits n))
