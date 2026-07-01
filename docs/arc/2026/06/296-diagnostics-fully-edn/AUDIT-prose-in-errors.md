# Arc 296 — Audit: structured data smuggled into prose error strings + the missing constraint

> **Produced 2026-06-30** (read-only audit agent + a missing-constraint analysis). This is the **next-move worklist**
> after the `WatError` wall (S6) lands: not ten patches — ONE constraint (`#[derive(WatErrorRecord)]`) whose absence lets
> the whole class exist.

## The heresy
A structured value (a list, a set of typed items, a coordinate, a count, a discriminated enum, a nested error) rendered
into a **prose string** — as an error's `:message`/`Display` text or a `String`-typed EDN field — where it should be a
structured EDN field (a vector, a tagged record, a keyword, an integer, a `#wat.kernel/Location`). The same double-encode
this arc exists to kill (builder #21: *"why not just an array of edn?"*), scattered across the error serializers.

## The catalog (10 findings — 9 L1, 1 L2; every one grounded)
| # | file:line · type/field | prose-encoded structure | should be | sev |
|---|---|---|---|---|
| 3 | `check/error_edn.rs:334` · `NoMatchingClauseAtCallSite.attempted_clauses` | `Vec<(usize, Vec<String>)>` **dropped entirely** (`_`) | `Vector` of `#wat.kernel/ClauseAttempt` (runtime twin already does this, `runtime_error_edn.rs:239-248`) | **L1** |
| 8 | `load.rs:369` · `LoadErrorKind::Fetch.:cause` | `LoadFetchError` (3-variant enum, defined `load.rs:191`) `.to_string()`'d; doc calls it "foreign opaque" — contradicting the type | `#wat.kernel/NotFound {:path …}` / `#wat.kernel/OutOfScope {:path … :scope …}` | **L1** |
| 4 | `check/error_edn.rs:79-83` · `ReturnTypeMismatch.:remedies` | `Vec<Remedy>` → `render_remedies()` "did you mean" blob | `Vector` of `#wat.kernel/Remedy {:form :kind :score :note}` | **L1** |
| 5 | `check/error_edn.rs:98-102` · `MalformedForm.:remedies` | same `render_remedies()` collapse | same | **L1** |
| 6 | `types/error.rs:370-375` · `MalformedVariant.:remedies` | same, in `ToEdn::to_edn()` | same | **L1** |
| 2 | `check/error_edn.rs:339` · `NoMatchingClauseAtCallSite.:called-arg-types` | `Vec<String>.join(", ")` (anchor example); runtime twin emits a `Vector` (`runtime_error_edn.rs:236`) | `Vector([:wat::core::i64 …])` | **L1** |
| 1 | `check/error_edn.rs:324` · `DefRestrictedCallerNotAllowed.:prefixes` | `Vec<String>.join(" ")` | `Vector` of prefix strings | **L1** |
| 7 | `runtime_error_edn.rs:138-139` · `EvalVerificationFailed.:error` | `crate::hash::HashError` `format!`'d ("Lazy fallback") | `#wat.kernel/HashError {…}` | **L1** |
| 9 | `load.rs:389` · `LoadErrorKind::VerificationFailed.:cause` | same `HashError.to_string()` | same | **L1** |
| 10 | `signal.rs:294` field / `runtime_error_edn.rs:219` emit · `EdnCoerceMismatch.:path` | dot-notation coordinate String (`".name"`, `".[0]"`) — consumer re-parses dots | `Vector` of segments / `#wat.kernel/FieldPath` | **L2** |
| — | (also fixed live in S6/GAP-3) · `CheckErrors.:message` | full multi-line `Display` render (`\n`, dup of `:errors`) | one-line headline; detail in structured fields | **L1** |

## What constraints are missing (the builder's question — the ROOT)
The 10 findings are 10 symptoms of ONE absent constraint. Three faces of it:

1. **The error EDN is HAND-AUTHORED, not DERIVED from the type.** Every `to_edn` hand-maps each field via `edn_str` /
   `str_val` / `.join(", ")` / `format!` / `render_remedies`. Hand-authoring is *what lets a human throw structure away*,
   field by field — a `Vec` becomes `.join`, a `Vec<Remedy>` becomes a blob, a `Vec<clause>` becomes `_`. The EDN is a
   *choice at each field*, not a function of the type, so it can lie about the structure.
2. **No requirement that an EMBEDDED type be itself structured (`ToEdn`).** `Remedy`, `LoadFetchError`, `HashError` have
   no EDN form, so they fall through to `.to_string()`. **The stringify is the escape hatch for "this embedded thing has
   no structured form," and nothing forbids it.** `Remedy`-lacking-`ToEdn` alone generates 3 findings — the prose
   renderer is the *default* the moment there is no structured one.
3. **No single canonical serialization path.** `check/error_edn.rs` and `runtime_error_edn.rs` each hand-write their own
   and drift (runtime structures the clause list; check drops it). Equality-written-twice, at the serializer layer.

**The one constraint whose absence enables all three: the EDN is not a *structural function of the type.***

## The cure — the constraint that closes the class: `#[derive(WatErrorRecord)]`
Make the EDN a total structural function of the Rust type:
- **Structure preserved by construction** — the derive emits a `Vec<T>` as a vector, an enum as a tagged value, an int as
  an int. There is no hand-authored `to_edn` to lie in, so `.join`/`format!`/`render_remedies` have *no site*. Flattening
  becomes unrepresentable.
- **Every embedded type must be `ToEdn`/`WatError`** — a field of `Remedy`/`LoadFetchError`/`HashError` without a
  structured impl is a **compile error** (the "stringify the foreign type" hatch removed, exactly as S6 removed the
  "floorless error" hatch).
- **It is *the* path** — one derived serializer; no twin to drift.

**The layering is exact: `WatError` (S6) forces the floor to be PRESENT; `#[derive(WatErrorRecord)]` forces the whole
body to be STRUCTURAL.** Together, an error whose EDN flattens any structure into prose cannot compile.

## The next move (after S6's wall lands + is weighed)
Build `#[derive(WatErrorRecord)]` as the constraint. The 10 findings die because their sites cease to exist — do NOT
patch ten serializers (they rot again). Two down-payments the derive subsumes (or that can precede it): give **`Remedy`**
a structured `ToEdn` (kills 4/5/6 + immunizes future variants), and make **`check/error_edn.rs` match its structured
`runtime_error_edn.rs` twin** (kills 2/3). `LoadFetchError`/`HashError` need `ToEdn` impls (kills 7/8/9). N5 already
crowned the name; N3 (per-phase tag namespaces) rides the same retrofit.
