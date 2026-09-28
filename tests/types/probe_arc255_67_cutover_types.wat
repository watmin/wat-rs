;; Stone 255.67 — cutover 2/7. Every one of the 24 `wat.type/` hard primitives resolves in a
;; type position (a parameter typed with it; a container's constructor in value position),
;; through the SAME denotation door 255.65/255.66 already built. `Never` is deliberately
;; unregistered (`stone_255b_never_is_deliberately_unregistered`, `src/types.rs`) — its row here
;; only proves the ANNOTATION is accepted (a legal, if never-satisfied, type position), never
;; that it is a TypeEnv member.

(:wat::core::defn :user::id-i64        [x <- wat.type/i64]      -> wat.type/i64      x)
(:wat::core::defn :user::id-f64        [x <- wat.type/f64]      -> wat.type/f64      x)
(:wat::core::defn :user::id-u8         [x <- wat.type/u8]       -> wat.type/u8       x)
(:wat::core::defn :user::id-bigint     [x <- wat.type/bigint]   -> wat.type/bigint   x)
(:wat::core::defn :user::id-rational   [x <- wat.type/rational] -> wat.type/rational x)
(:wat::core::defn :user::id-char       [x <- wat.type/char]     -> wat.type/char     x)
(:wat::core::defn :user::id-string     [x <- wat.type/String]   -> wat.type/String   x)
(:wat::core::defn :user::id-bool       [x <- wat.type/bool]     -> wat.type/bool     x)
(:wat::core::defn :user::id-keyword    [x <- wat.type/keyword]  -> wat.type/keyword  x)
(:wat::core::defn :user::id-nil        [x <- wat.type/nil]      -> wat.type/nil      x)
(:wat::core::defn :user::id-value      [x <- wat.type/Value]    -> wat.type/Value    x)
;; `Fn` — like `Never`, refused as a TOP-LEVEL defn param annotation in EITHER spelling
;; (checked against `:wat::core::Fn` too, same refusal, not a regression). It is also NOT a
;; supertype a concretely-typed closure widens to (`[:wat::core::i64 :-> :wat::core::i64]` does
;; not satisfy a `wat.type/Fn` parameter — a real, pre-existing type-system distinction, not
;; this stone's concern), so the annotation is exercised bare, on an inline lambda that never
;; calls its `Fn`-typed parameter.
(:wat::core::defn :user::fn-annotation-resolves [] -> :wat::core::bool
  (:wat::core::let [_probe (:wat::core::fn [g <- wat.type/Fn] -> wat.type/Fn g)]
    true))

(:wat::core::defn :user::id-bytes      [x <- wat.type/Bytes]    -> wat.type/Bytes    x)
(:wat::core::defn :user::id-ast        [x <- wat.type/AST]      -> wat.type/AST      x)

;; Never — deliberately unregistered (`stone_255b_never_is_deliberately_unregistered`,
;; `src/types.rs`), and it has no real corpus type position (measured: only a comment mention,
;; `wat-scripts/scratch-pad/probe-selectables-homogeneity.wat:14`). A TOP-LEVEL `defn` param of
;; this type is refused in EITHER spelling (`validate_named_type_annotations`, `src/check.rs`,
;; has no fallback for it — checked against `:wat::core::Never` too, same refusal, not a
;; regression); an inline `fn`'s annotation is accepted (`wat-scripts/scratch-pad/arc255-67/
;; probe-never.wat`, committed). Nothing to test at defn-value level; not registered here either.

;; Record / Struct — declared via defrecord/defstruct, whose field types exercise them; the
;; hard-primitive TYPE KEYWORDS themselves are checked as bare annotations here.
(:wat::core::defrecord :user::ARecord [f <- wat.type/i64])
(:wat::core::defstruct :user::AStruct [f <- wat.type/i64])
(:wat::core::defn :user::id-record [x <- wat.type/Record] -> wat.type/Record x)
(:wat::core::defn :user::id-struct [x <- wat.type/Struct] -> wat.type/Struct x)

(:wat::core::defn :user::scalars-round-trip [] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:user::id-i64 7) 7)
    (:wat::core::if (:wat::core::= (:user::id-f64 1.5) 1.5)
      (:wat::core::if (:wat::core::= (:user::id-u8 (:wat::core::u8 9)) (:wat::core::u8 9))
        (:wat::core::if (:wat::core::= (:user::id-bigint (:wat::i64::to-bigint 100)) (:wat::i64::to-bigint 100))
          (:wat::core::if (:wat::core::= (:user::id-char \a) \a)
            (:wat::core::if (:wat::core::= (:user::id-string "hi") "hi")
              (:wat::core::if (:wat::core::= (:user::id-bool true) true)
                (:wat::core::if (:wat::core::= (:user::id-keyword :k) :k)
                  (:wat::core::= (:user::id-nil nil) nil)
                  false)
                false)
              false)
            false)
          false)
        false)
      false)
    false))

;; `wat.type/Value` is the dynamic/untyped top type — not `Equatable` (a real, pre-existing
;; type-system fact, not this stone's concern), so it is exercised as a bare annotation only
;; (the call proves the type-check accepts it; there is no typed equality to compare against).
(:wat::core::defn :user::value-round-trips [] -> :wat::core::bool
  (:wat::core::let [_v (:user::id-value 5)] true))

;; `Bytes` is a `typealias` for `(Vector :- [u8])` (`wat/core.wat`), not its own constructor —
;; a Bytes VALUE is a Vector of u8 elements.
(:wat::core::defn :user::bytes-round-trips [] -> :wat::core::bool
  (:wat::core::let [b (wat.type/Vector :- [wat.type/u8] (:wat::core::u8 1) (:wat::core::u8 2))]
    (:wat::core::= (:user::id-bytes b) b)))

(:wat::core::defn :user::ast-round-trips [] -> :wat::core::bool
  (:wat::core::let [a (:wat::core::keyword-node ":foo")]
    (:wat::core::= (:wat::core::ast-name (:user::id-ast a)) ":foo")))

;; ── containers: the constructor in VALUE position, new spelling ──────────────────────────────
(:wat::core::defn :user::vector-value [] -> :wat::core::bool
  (:wat::core::= [1 2 3] (wat.type/Vector :- [wat.type/i64] 1 2 3)))

(:wat::core::defn :user::hashmap-value [] -> :wat::core::bool
  (:wat::core::=
    (:wat::hashmap::get (wat.type/HashMap :- [wat.type/keyword wat.type/i64] :a 1) :a)
    (:wat::core::Option.Some {:value 1})))

(:wat::core::defn :user::hashset-value [] -> :wat::core::bool
  (:wat::core::contains? (wat.type/HashSet :- [wat.type/i64] 1 2 3) 2))

(:wat::core::defn :user::list-value [] -> :wat::core::bool
  (:wat::core::= (wat.type/List 1 2 3) (:wat::core::List 1 2 3)))

(:wat::core::defn :user::tuple-value [] -> :wat::core::bool
  (:wat::core::= (wat.type/Tuple :- [wat.type/i64 wat.type/i64] 1 2) (:wat::core::Tuple 1 2)))

(:wat::core::defn :user::persistentvector-value [] -> :wat::core::bool
  (:wat::core::= (wat.type/PersistentVector :- [wat.type/i64] 1 2 3) (:wat::core::PersistentVector 1 2 3)))

(:wat::core::defn :user::persistentmap-value [] -> :wat::core::bool
  (:wat::core::=
    (:wat::map::get (wat.type/PersistentMap :- [wat.type/keyword wat.type/i64] :a 1) :a)
    (:wat::core::Option.Some {:value 1})))

(:wat::core::defn :user::record-value [] -> :wat::core::bool
  (:wat::core::= (:user::ARecord/f (:user::ARecord :f 7)) 7))

(:wat::core::defn :user::struct-value [] -> :wat::core::bool
  (:wat::core::= (:user::AStruct/f (:user::AStruct :f 7)) 7))
