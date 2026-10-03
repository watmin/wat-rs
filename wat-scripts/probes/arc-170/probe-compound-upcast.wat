;; probe-compound-upcast.wat — POSITIVE gate for the generalized expected-type-directed
;; up-cast rule (this strike): Tuple / Map constructors/literals up-cast their
;; components against a known expected type, same as fbc60b94 did for `[...]` Vector.
;;
;; eh (echo'::Handle) IS-A :wat::capability::Capability via the defservice-auto-emitted
;; extend-type (same subtype pair fbc60b94's vector fix + probe-c1-capability-upcast.wat
;; already used for the scalar case).
;;
;; GATE: `wat --check` on this file must exit 0 (both forms type-check: Tuple via
;; ann-form, Map via call-arg — each up-casts eh: Handle -> Capability at construction).
;; Both also RUN to completion, driven by
;; tests/process/probe_arc255_74_compound_upcast_runs.rs.
;;
;; Stone 255.74 RETIRED a third claim this probe used to make: a Set case up-casting eh
;; into `(HashSet :- [Capability])`. A service handle is a RustOpaque at runtime and was
;; never key-eligible — pre-255.74 that was a checker-BLIND runtime panic
;; ("Value::RustOpaque is not atomizable"); Stone 255.74 made the checker refuse it
;; statically instead. The refusal is now its own negative fixture:
;; wat-scripts/probes/arc-255/probe-255.74-set-of-capability.wat.bad (driven by
;; tests/types/probe_arc255_74_key_must_be_data.rs).
;;
;; CLAIM (exit 0): both up-casts (Tuple via ann-form, Map via call-arg) type-check AND
;; construct successfully — there is no further computed value past "did construction
;; raise" (pr/mp are never inspected beyond being built), so exit 0 — plus the dedicated
;; `--check` assertion in tests/process/probe_arc255_74_compound_upcast_runs.rs — is the
;; whole proof.
(wat.core/defsurface probe/Echo :nature wat.kernel/Peer
  :messages [(wat.core/defrecord probe.Echo/EchoRequest  [msg   :- wat.type/String])
             (wat.core/defenum probe.Echo/EchoResponse wat.enum/Pure :Ok [reply :- wat.type/String] :RequestTooLarge [bytes :- wat.type/i64  cap :- wat.type/i64]
                                                                                                                :RequestMalformed [path :- (wat.type/Vector :- [wat.type/String])  expected :- wat.type/String  got :- wat.type/String])]
  :features [(echo [self :- probe/Echo  req :- probe.Echo/EchoRequest] :- probe.Echo/EchoResponse :max-request-bytes 524288)])

(wat.service/defservice probe/echo :satisfies probe/Echo :durable [] :ephemeral []
  :impls [(echo [s ctx req] (wat.service/Outcome.Reply {:state s
            :reply (probe.Echo/EchoResponse.Ok {:reply (wat.string/concat "echo:" (probe.Echo.EchoRequest/msg req))})}))])

(wat.core/defn probe/as-map [m :- (wat.type/HashMap :- [wat.type/keyword wat.capability/Capability])]
  :- (wat.type/HashMap :- [wat.type/keyword wat.capability/Capability])
  m)

(wat.core/defn user/main [] :- wat.type/nil
  (wat.core/let
    [eh (probe.echo/start :locus (wat.spawn/process) :record (probe.echo/Record))
     ;; Tuple — ann-form site: (:wat::core::Tuple :echo eh) ascribed to
     ;; :(wat::core::keyword,wat::capability::Capability); eh up-casts Handle -> Capability.
     pr (wat.core/ann-form (wat.type/Tuple :- [wat.type/keyword wat.capability/Capability] :echo eh)
          (wat.type/Tuple :- [wat.type/keyword wat.capability/Capability]))
     ;; Map — call-arg site: {:echo eh} against as-map's (HashMap :- [keyword Capability]) param.
     mp (probe/as-map {:echo eh})]
    (wat.kernel/println "compound-upcast: ok")))
