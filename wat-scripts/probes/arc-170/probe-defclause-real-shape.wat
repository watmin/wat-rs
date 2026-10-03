;; probe-defclause-real-shape.wat — NO shim. The REAL contract shape.
;; An agnostic result enum whose :Constraint variant carries a `reason <- Reason` field (exactly
;; how :wat::query::PutResult would). Construct it with a concrete SqliteReason (UP — the extend-type),
;; match it out, hand the `reason` (typed :probe::Reason by the field) to a concrete-clause defclause.
;; This settles whether the gap is REAL (the agnostic field loses the concrete type) or a shim artifact.
;; CLAIM: describe(the Reason unpacked from PutResult.Constraint's own field) == "sqlite 2067" —
;; defclause already handles a value typed by an agnostic field's declared (not literal) type.

(wat.core/defsurface probe/Reason :nature wat.type/Record :features [])
(wat.core/defrecord  probe/SqliteReason [code  :- wat.type/i64  sql :- wat.type/String])
(wat.core/defrecord  probe/RedisReason  [errno :- wat.type/i64  cmd :- wat.type/String])
(wat.core/extend-type probe/SqliteReason probe/Reason)
(wat.core/extend-type probe/RedisReason probe/Reason)

;; a client that knows sqlite + redis — CONCRETE clauses only
(wat.core/defclause probe/describe
  ([r :- probe/SqliteReason] :- wat.type/String
    (wat.string/concat "sqlite " (wat.i64/to-string (probe.SqliteReason/code r))))
  ([r :- probe/RedisReason]  :- wat.type/String
    (wat.string/concat "redis "  (wat.i64/to-string (probe.RedisReason/errno r)))))

;; the REAL agnostic result — :Constraint carries a Reason field (like :wat::query::PutResult)
(wat.core/defenum probe/PutResult wat.enum/Pure
  :Success    [ok     :- wat.type/bool]
  :Constraint [reason :- probe/Reason])

(wat.core/defn user/main [] :- wat.type/nil
  (wat.core/let
    [result (probe/PutResult.Constraint {:reason (probe/SqliteReason :code 2067 :sql "INSERT INTO users ...")})  ; concrete into a Reason field
     d      (wat.core/match result
              [probe/PutResult.Success {:ok _}   "ok"]
              [probe/PutResult.Constraint {:reason r}          ; r : :probe::Reason (the field type)
                (probe/describe r)])]                    ; concrete-clause defclause on a Reason-typed value
    (wat.core/do
      (wat.test/assert-eq d "sqlite 2067")
      (wat.kernel/println d))))
