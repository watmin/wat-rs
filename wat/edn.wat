;; wat/edn.wat — arc 296 J: `:wat::edn::*` outcome enums, declared in wat.
;;
;; Read/validate sum types formerly registered as hand-written `EnumDef`
;; literals in `src/types.rs`. wat is the source of truth; Rust consumes
;; them via `wat_enum_register_from!`.
;;
;; Load order: after `wat/core.wat` (`Error`, parametric `Option`/`Result`).

;; Arc 278 Stone 1 (`wat --mcp`) — (:wat::edn::ReadJsonOutcome :- [T]) — what
;; `:wat::edn::read-json` returns.
;;
;; `:wat::edn::read-json`'s input arrives from a REMOTE, UNTRUSTED harness over stdio (the
;; MCP JSON-RPC transport), so a malformed byte must not be able to raise: exactly the failure
;; `:wat::core::read-string` was converted to fix above — one bad byte unwinding a raise
;; THROUGH the loop and killing the session. Mirrors `:wat::core::ReadOutcome` variant-for-
;; variant, including WHY the cause is the structural `:wat::core::Error` and not a
;; JSON-specific enum: `wat_edn::JsonError`'s variants in every caller's exhaustive match would
;; be arms nobody branches on. Discrimination lives in the navigable causes tree.
;;
;; PARAMETRIC over `T` — CORRECTED from a first pass that declared `:Value`'s payload as the
;; bare `:wat::core::Value` (the universal top, arc 278 R7). That was wrong: UP is free, DOWN is
;; CHECKED, so a `:wat::core::Value` payload can be PRODUCED but never CONSUMED — no accessor
;; (`HashMap/get`, a Struct/Record field, …) type-checks against an opaque `Value` receiver, by
;; design (measured: `HashMap/get` on a decoded JSON object refused with "expected
;; HashMap<?,?>; got :wat::core::Value"). `T` is generic exactly as `(ReadlnOutcome :- [T])`'s `T` is
;; (immediately above) and for the same reason — the payload's type is the CALLER's, driven by
;; the annotated binding, not ours to fix in advance.
;;
;;   :Value     [value <- T] — the decoded value, at the caller's chosen type
;;   :Malformed [cause]      — the JSON text did not parse (or did not decode)
;;
;; Pure, and for `ReadOutcome`'s stated reason: the payload holds no fd and no peer (T here is
;; ordinary decoded data — a String/HashMap/record — never a live resource, unlike
;; `(ReadlnOutcome :- [T])`'s T which can be), and `:wat::core::Error` is Record-natured. Marking it
;; Impure would bar it from pure aggregates and the wire for nothing.
(:wat::core::defenum :wat::edn::ReadJsonOutcome :- [T] :wat::enum::Pure
  :Value [value <- :T]
  :Malformed [cause <- :wat::core::Error])

;; `:wat::edn::ReadForeignOutcome<T>` — what `:wat::edn::read-foreign` returns.
;; Twin of `ReadJsonOutcome<T>`: the verb's input is an untrusted String (a journal
;; Log/message from another universe, a scratch payload), so a malformed byte must
;; not raise. Same two variants, same parametric T (the caller's binding pins it;
;; a ForeignRecord consumer unifies T with `:wat::edn::ForeignRecord`).
;;
;;   :Value     [value <- T] — the decoded value (ForeignRecord / ForeignVariant / typed)
;;   :Malformed [cause]      — the EDN text did not parse, or did not decode
(:wat::core::defenum :wat::edn::ReadForeignOutcome :- [T] :wat::enum::Pure
  :Value [value <- :T]
  :Malformed [cause <- :wat::core::Error])

;; :wat::edn::Validation — Arc 278 the REQUEST-MALFORMED wall (Stone 1,
;; DESIGN-request-malformed-input-sanitization.md). The outcome of
;; `(:wat::edn::validate <value> :DeclaredType)` — the DEEP shape check at a
;; trust boundary. `:wat::core::conforms?` cannot answer this question: for an
;; Aggregate it is a NOMINAL identity check only (runtime.rs `conforms_check`,
;; the `TypeDef::Aggregate` arm → `concrete_type_name_matches`) — it never
;; recurses into a record's FIELDS, so `#dos.Bag/PutRequest {:items [1 2 3]}`
;; against `items <- (Vector :- [String])` passes it. `edn_to_typed_value`
;; (edn/render.rs) IS the deep walker (per-field, per-element, with the offending
;; path); `validate` is its thin wat-facing wrapper.
;;   :Valid   []                       — the value matches the declared shape.
;;   :Invalid [path expected got]      — it does not, at `path` (segments, e.g.
;;                                       ["items" "[0]"]), where the declaration
;;                                       says `expected` and the wire carried `got`.
;; `path` is STRUCTURED ((Vector :- [String]) — segments the caller can index/walk);
;; `expected`/`got` are STRINGS, ruled by the four questions (see
;; DESIGN-request-malformed-input-sanitization.md): `expected` is
;; `check::format_type`'s rendering (the ONE authoritative type renderer), and
;; `got` is the EDN SHAPE that arrived ("Integer", "Vector", "Map") — an
;; untyped wire value has NO declared type, so structuring `got` as a type form
;; would FABRICATE information. An asymmetric pair (structured expected, string
;; got) would imply a comparison the substrate cannot make. This is a 400-class
;; diagnostic, not data anyone computes on.
;; PURE — three Strings/String-vectors and a nullary variant; fully
;; EDN-reconstructable. Registered as a builtin because the defservice-generated
;; serve loop matches on it, before any wat defenum would load.
(:wat::core::defenum :wat::edn::Validation :wat::enum::Pure
  :Valid
  :Invalid [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])

;; ─── EdnReadErrorKind, as declared wat records (excursus 003 strike T2) ──────────────────
;;
;; `src/edn/render.rs`'s `EdnReadErrorKind` — every reason `decode_trusted_wire` can refuse a
;; wire frame — mirrored here one record per variant, exactly `wat/runtime-errors.wat`'s step-3a
;; pattern for `RuntimeErrorKind`: the `:wat::core::Error` floor (`message`/`location`) first, so
;; every one of these structurally satisfies `:wat::core::Error`, then the kind's own data
;; fields, named exactly as the Rust variant's fields (never hand-retyped on the Rust side —
;; `EdnReadError::to_record`, `src/edn/render.rs`, sources every field NAME from these forms via
;; `wat_field_names_from!`). `message` is the kind's own `Display` rendering (span-free);
;; `location` is the raising site's `:wat::core::Span` (`EdnReadError::span`,
;; `crate::rust_caller_span!()` at every raise site — never a client-attributable position, since
;; EDN decode walks an already-parsed tree with no source span of its own).
;;
;; T2's target: a decode failure at a service's recv'/poll' boundary (`src/kernel/message.rs`)
;; carries one of these as `Failure.error` (via `EdnReadError::to_record`), replacing the
;; `message_only_failure(format!(...))` prose T2 found flattening the read error there. The
;; shape-class kinds (`FieldTypeMismatch`/`UnknownField`/`UndeclaredFieldType`/
;; `UnknownStructField`) ALSO get a one-shot lenient re-decode (`decode_trusted_wire_lenient`) so
;; the per-op `:wat::edn::validate`/`:RequestMalformed` guard (`wat/service.wat`) still gets its
;; contractual say; everything else (an unparseable frame, an unknown/unsupported tag, no type
;; registry) stays the generic `ServiceEvent::Malformed` → `Reply::Failed[cause]`, now structured.
;;
;; `NoTypeRegistry` is measured UNREACHABLE at the one call site this strike wires (a running
;; service's `SymbolTable` always carries a `TypeEnv` — see the strike's report) but is declared
;; anyway: `EdnReadError::to_record`'s match is exhaustive, no `_` arm, so a future caller that
;; CAN reach it (hitting `decode_trusted_wire`/`read_edn_caps` with `types=None`) still gets a
;; structured record, not a compile-time gap papered over with a wildcard.

(:wat::core::defrecord :wat::edn::UnknownTag
  [message <- :wat::core::String
   location <- :wat::core::Span
;; The unresolved `#ns/Name` tag's two halves, plus what the wire body's own shape was
;; ("Map"/"Vector"/"Nil"/…) — lets a caller tell "no such type" from "wrong body for this type".
   ns <- :wat::core::String
   name <- :wat::core::String
   body-shape <- :wat::core::String])

(:wat::core::defrecord :wat::edn::UnsupportedTag
  [message <- :wat::core::String
   location <- :wat::core::Span
;; The whole offending tag text (reserved-substrate-tag family the bridge doesn't understand).
   tag <- :wat::core::String])

(:wat::core::defrecord :wat::edn::NoTypeRegistry
  [message <- :wat::core::String
   location <- :wat::core::Span])

(:wat::core::defrecord :wat::edn::UnknownStructField
  [message <- :wat::core::String
   location <- :wat::core::Span
;; The declared struct that was missing one of ITS OWN declared fields on the wire map.
   type-path <- :wat::core::String
   key <- :wat::core::String])

(:wat::core::defrecord :wat::edn::EnumVariantNotFound
  [message <- :wat::core::String
   location <- :wat::core::Span
;; The declared enum, and the variant name the wire named that it does not carry.
   type-path <- :wat::core::String
   variant <- :wat::core::String])

;; Strike T's own `FieldTypeMismatch`/`UnknownField`/`UndeclaredFieldType` — the shape-class
;; trio `value_conforms`/`refuse_unknown_fields` raise (B1's hole). `field`/`key` are dot-paths
;; (`"items.[0]"`), the SAME convention `EdnCoerceError.path` and `Mismatch.path` already use.
(:wat::core::defrecord :wat::edn::FieldTypeMismatch
  [message <- :wat::core::String
   location <- :wat::core::Span
   type-path <- :wat::core::String
   field <- :wat::core::String
   expected <- :wat::core::String
   got <- :wat::core::String])

(:wat::core::defrecord :wat::edn::UnknownField
  [message <- :wat::core::String
   location <- :wat::core::Span
   type-path <- :wat::core::String
   key <- :wat::core::String])

(:wat::core::defrecord :wat::edn::UndeclaredFieldType
  [message <- :wat::core::String
   location <- :wat::core::Span
   type-path <- :wat::core::String
   field <- :wat::core::String
   ty <- :wat::core::String])

;; The `Other(String)` catch-all (an unparseable frame, an unsupported `wat_edn::Value` variant
;; like `Symbol`/`BigInt`, a non-hashable map key, …) — `detail` carries the SAME string the
;; `Other` variant already wraps, so nothing is lost going from prose to structure; `message` is
;; the identical text (there is no further structure to split it into).
(:wat::core::defrecord :wat::edn::ReadError
  [message <- :wat::core::String
   location <- :wat::core::Span
   detail <- :wat::core::String])
