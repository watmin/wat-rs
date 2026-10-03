;; tests/value/probe_stone_D_join_over_seqable.wat — co-located fixture, arc 255 Stone D.
;;
;; `:wat::core::string::join`'s second parameter widens from `(Vector :- [T])` to the
;; `(Seqable :- [T])` surface (Vector · PersistentVector · List · Stream). Four rows:
;;   1. Vector    — no-regression, green before Stone D too.
;;   2. Stream    — the gap: REFUSED at check time before Stone D (`map` returns a Stream).
;;   3. List      — proves the widening reached the whole Seqable set, not just Stream.
;;   4. Rendering — a non-string element (bool here, distinct from row 2's i64) joined
;;      through the Stream path must render IDENTICALLY to the same elements joined
;;      through the Vector path — the row that catches a widening that forgot
;;      `render_str_total` (a naive Debug/to_string render would print `Bool(true)`,
;;      not `true`).

;; ── row 1: Vector, unchanged fast path ─────────────────────────────────────
(wat.core/defn probe/join-vector [] :- wat.type/String
  (wat.string/join "-" (wat.type/Vector :- [wat.type/i64] 1 2 3)))

;; ── row 2: Stream — `map` over a Vector never re-materializes; join must accept the
;;    resulting lazy Stream directly. Before Stone D this was refused at CHECK time.
(wat.core/defn probe/join-stream [] :- wat.type/String
  (wat.string/join "-"
    (wat.core/map
      (wat.core/fn [x :- wat.type/i64] :- wat.type/i64 (wat.i64/+ x 1))
      (wat.type/Vector :- [wat.type/i64] 1 2 3))))

;; ── row 3: List — the whole Seqable set, not just Stream.
(wat.core/defn probe/join-list [] :- wat.type/String
  (wat.string/join "-" (wat.type/List :- [wat.type/i64] 1 2 3)))

;; ── row 4a/4b: rendering parity — same non-string (bool) elements, one path Vector,
;;    one path Stream. Both must render identically ("true,false,true"), proving the
;;    Stream arm renders through `render_str_total`, same as the Vector arm.
(wat.core/defn probe/join-vector-bool [] :- wat.type/String
  (wat.string/join "," (wat.type/Vector :- [wat.type/bool] true false true)))

(wat.core/defn probe/join-stream-bool [] :- wat.type/String
  (wat.string/join ","
    (wat.core/map
      (wat.core/fn [x :- wat.type/bool] :- wat.type/bool x)
      (wat.type/Vector :- [wat.type/bool] true false true))))
