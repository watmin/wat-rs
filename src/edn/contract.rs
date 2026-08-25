//! Arc 296 — `ToEdn` trait: the ONE serialization contract for every
//! error/diagnostic type.
//!
//! Before this arc, EDN serialization was a pile of ad-hoc free functions
//! (`runtime_error_to_edn`, `macro_error_to_edn`, `startup_error_to_edn`,
//! `payload_to_edn`, `span_to_edn`, …). Each function was wired manually;
//! a new error type shipped with no EDN form and nothing stopped it.
//!
//! Arc 296 mints ONE contract: every error/diagnostic type implements this
//! trait. The existing free functions become the impl bodies (or thin
//! wrappers around them). The serialization boundary (arc 296 slice 5)
//! is generic over `ToEdn` — a non-`ToEdn` error has no path to the
//! wire, making stringly diagnostics uncompilable.
//!
//! ## The compile fence (the wall)
//!
//! [`to_wire_edn`] is the single, named, generic conversion from an error to
//! the text that crosses the process boundary (the `ProcessDiedError`
//! payload). The `process_died_error_*_value` builders, `emit_structured_edn`
//! and the `--check-output` consumers are all **generic over `ToEdn`** — they
//! accept `impl ToEdn`, never a raw `String`. Adding a new error variant that
//! does NOT implement `ToEdn` produces a compile error at the first call site
//! that tries to reach the wire — the mistake is unrepresentable by
//! construction, not caught at runtime. A `compile_fail` doc-test on
//! [`to_wire_edn`] proves the wall is real, not aspirational.
//!
//! Genuinely message-only failures (a syscall error string, a return-type
//! name) travel through the same generic boundary as a [`FlatMessage`] —
//! they too are a `ToEdn` value, so the boundary never has to accept a bare
//! `String`.
//!
//! [`OwnedValue`] itself implements `ToEdn` as a passthrough (identity),
//! so pre-computed EDN values can be passed to the boundary without
//! unwrapping and re-wrapping.

use wat_edn::OwnedValue;

/// Re-exported from `wat-edn` (where the trait is now defined). All
/// `impl ToEdn for LocalType` sites in this crate use `crate::edn::contract::ToEdn`
/// which resolves here via this re-export — unchanged by the trait relocation.
pub use wat_edn::ToEdn;

// ─── The floor trait (arc 296 strike 2) ──────────────────────────────────────

/// A top-level substrate error that can reach the wire boundary.
///
/// `WatError` enforces the `:wat::core::Error` floor:
/// every error that crosses the wire MUST carry `:message`, `:location`,
/// and `:causes` — the three fields that tooling and the runtime always
/// expect to navigate, regardless of the specific error family.
///
/// ## Required methods
///
/// - `message()` — the human-readable error message; typically
///   `self.to_string()` via the type's `Display` impl.
/// - `location()` — the primary source location as a
///   `#wat.kernel/Location {:file :line :col}` map, or `nil` when the
///   error has no recoverable span (elide-when-unknown discipline, same as
///   `push_span_field`).
/// - `causes()` — the nested error chain as an EDN vector; `[]` for leaf
///   errors that carry no structured sub-errors.
/// - `variant()` — the variant-specific fields as a tagged map, identical
///   to the error's EXISTING `ToEdn::to_edn()` output with the raw
///   `:span` key stripped (the floor now owns `:location`; the variant
///   must not double-emit the span under a different key).
///
/// ## Provided method
///
/// `error_edn()` composes the floor. It takes `variant()` (a tagged map)
/// and inserts `:message`, `:location`, `:causes` at the front of the body
/// map, in that order. Implementors MUST NOT override this method.
///
/// ## What does NOT implement `WatError`
///
/// Sub-values embedded inside a variant's EDN — [`crate::runtime::ValueSnapshot`],
/// [`crate::value::Provenance`], [`crate::span::Span`],
/// [`crate::assertion::AssertionPayload`], [`OwnedValue`], [`FlatMessage`] —
/// are passed to `to_edn()`, never to `to_wire_edn`. They stay [`ToEdn`].
pub trait WatError {
    /// The human-readable error message. Typically `self.to_string()`.
    fn message(&self) -> String;

    /// The primary source location, or `nil` when no span is available.
    ///
    /// Build with [`location_from_span`] for Pattern-A errors (span on the
    /// outer struct); return [`OwnedValue::Nil`] for collection / wrapper
    /// errors that have no single primary span.
    fn location(&self) -> OwnedValue;

    /// The nested error chain. Return `OwnedValue::Vector(vec![])` for leaf
    /// errors; include the inner error's [`ToEdn::to_edn`] output for
    /// errors that wrap a typed cause.
    fn causes(&self) -> OwnedValue;

    /// The variant-specific fields as a tagged map.
    ///
    /// Return the existing [`ToEdn::to_edn`] output, stripped of the raw
    /// `:span` key (use [`strip_span_from_tagged`]). The floor owns
    /// `:location`; the variant MUST NOT duplicate the span under any key.
    fn variant(&self) -> OwnedValue;

    /// Compose the floor. **Do not override.**
    ///
    /// Takes `variant()` (a tagged map) and inserts `:message`,
    /// `:location`, `:causes` at the front of its body map. This is the
    /// canonical wire representation: every wire-crossing error carries
    /// exactly these three floor keys, in this order, before any
    /// variant-specific fields.
    fn error_edn(&self) -> OwnedValue {
        let variant_val = self.variant();
        match variant_val {
            OwnedValue::Tagged(tag, body) => {
                let mut fields = match *body {
                    OwnedValue::Map(f) => f,
                    other => vec![(edn_kw("body"), other)],
                };
                // Dedup: a variant map must not carry its own :message /
                // :location / :causes — the floor owns those keys. Strip any
                // pre-existing floor keys before inserting (e.g. a `FlatMessage`
                // whose `key` is literally "message" would otherwise emit a
                // duplicate-key map).
                let msg_kw = edn_kw("message");
                let loc_kw = edn_kw("location");
                let cause_kw = edn_kw("causes");
                fields.retain(|(k, _)| k != &msg_kw && k != &loc_kw && k != &cause_kw);
                // Insert floor keys at the front, in reverse order so
                // the final order is :message :location :causes <variant…>.
                fields.insert(0, (cause_kw, self.causes()));
                fields.insert(0, (loc_kw, self.location()));
                fields.insert(0, (msg_kw, edn_str(&self.message())));
                OwnedValue::Tagged(tag, Box::new(OwnedValue::Map(fields)))
            }
            other => {
                // Fallback: variant() returned a non-tagged value.
                // Wrap everything in an untagged floor map so the
                // wire payload is at least navigable.
                OwnedValue::Map(vec![
                    (edn_kw("message"), edn_str(&self.message())),
                    (edn_kw("location"), self.location()),
                    (edn_kw("causes"), self.causes()),
                    (edn_kw("variant"), other),
                ])
            }
        }
    }
}

// ─── Building blocks — primitive + container `ToEdn` impls ───────────────────
//
// These impls live in `wat-edn` (where the trait is defined) to satisfy
// the orphan rule: implementing a foreign trait for a foreign type is
// forbidden. Primitive and std impls (String, i64, Vec<T>, Option<T>, …)
// are provided by `wat-edn`. Local `wat`-crate types (FlatMessage, error
// kinds, …) are implemented in their respective modules.

// ─── Shared low-level EDN builders ───────────────────────────────────────────
//
// One canonical home for the tag/keyword/string/int/span constructors so a
// new `ToEdn` impl does not copy the helpers a sixth time (the older
// `runtime_error_edn.rs` / `macros/error_edn.rs` / `check/error_edn.rs`
// serializers each carry a private copy; new impls call these instead).

use std::borrow::Cow;
use wat_edn::{Keyword, Tag};

/// `#wat.kernel/<variant> <body>` — the kernel-namespaced tagged envelope.
pub(crate) fn edn_tag(variant: &str, body: OwnedValue) -> OwnedValue {
    OwnedValue::Tagged(Tag::ns(crate::error_ns::KERNEL, variant), Box::new(body))
}

/// A keyword EDN value (`:name`). Accepts a dynamic string.
pub(crate) fn edn_kw(name: &str) -> OwnedValue {
    OwnedValue::Keyword(Keyword::new(name))
}

/// A string EDN value.
pub(crate) fn edn_str(s: &str) -> OwnedValue {
    OwnedValue::String(Cow::Owned(s.to_owned()))
}

/// An integer EDN value.
pub(crate) fn edn_int(n: i64) -> OwnedValue {
    OwnedValue::Integer(n)
}

// Stone B (arc 296): `edn_span`, `push_span_field`, `splice_span` retired —
// `Span: ToEdn` (via derive in `wat-reader`) subsumes all three. Callers that
// previously called `splice_span(kind_edn, &self.span)` now push
// `(edn_kw("span"), self.span.to_edn())` directly.  Callers that called
// `push_span_field(&mut fields, "span", span)` now push inline.
// `location_from_span` (below) now delegates to `span.to_edn()`.
//
// The `is_span_type` special-case in `wat-to-edn-derive` is also deleted;
// `Span` fields are now handled by the normal `.to_edn()` path.

/// The first line of a (possibly multi-line) message, trimmed of trailing
/// whitespace.
///
/// The floor `:message` MUST be a single clean line: the multi-line detail
/// (hints, actual/expected, remedy sections, migration blocks) lives in the
/// structured variant fields, never re-rendered as text in `:message`. Each
/// error family's `WatError::message()` passes its **span-free** kind Display
/// through this helper so `:message` is a concise headline with no embedded
/// `\n` and no `file:line` prefix (the location lives in `:location`).
pub(crate) fn first_line(s: String) -> String {
    match s.find('\n') {
        Some(i) => s[..i].trim_end().to_string(),
        None => s,
    }
}

/// Build the `:location` value for a [`WatError`] impl.
///
/// Returns the span as `#wat.core/Span {…}` (the derive-generated tagged
/// record). Arc 298.2: every span is a real location (wat source or
/// `rust_caller_span!()`), so always emitted. Stone B: uses `span.to_edn()`
/// so the location is a proper typed record, not a bare map.
pub(crate) fn location_from_span(span: &crate::span::Span) -> OwnedValue {
    span.to_edn()
}

/// Strip the raw `:span` key from a tagged map's body.
///
/// Used by [`WatError::variant()`] impls: the floor owns `:location`; the
/// variant must not also emit a raw `:span` key or the span appears under two
/// keys on the wire. Call this on the result of the existing
/// [`ToEdn::to_edn()`] impl:
///
/// ```text
/// fn variant(&self) -> OwnedValue {
///     crate::edn::contract::strip_span_from_tagged(self.to_edn())
/// }
/// ```
///
/// If the value is not a `Tagged` or its body is not a `Map`, it is
/// returned unchanged (no-op: nothing to strip).
pub(crate) fn strip_span_from_tagged(val: OwnedValue) -> OwnedValue {
    let span_kw = edn_kw("span");
    match val {
        OwnedValue::Tagged(tag, body) => {
            let new_body = match *body {
                OwnedValue::Map(fields) => {
                    OwnedValue::Map(
                        fields.into_iter().filter(|(k, _)| k != &span_kw).collect()
                    )
                }
                other => other,
            };
            OwnedValue::Tagged(tag, Box::new(new_body))
        }
        other => other,
    }
}

/// Call `e.error_edn()` on any [`WatError`] value, returning the floor form.
///
/// Used as a `#[to_edn(via = crate::edn::contract::error_edn_of)]` target for fields
/// that embed a nested substrate error that must be serialized via the floor
/// (`:message` / `:location` / `:causes`) rather than raw `to_edn()`. The field
/// type must implement [`WatError`]; the derive default (`.to_edn()`) applies to
/// fields that implement only [`ToEdn`].
///
/// Signature matches the `via` contract: `fn(&FieldType) -> OwnedValue`.
pub(crate) fn error_edn_of(e: &impl WatError) -> OwnedValue {
    e.error_edn()
}

/// Call `cause.error_edn()` on a `Box<T: WatError>`, returning the floor form.
///
/// This is the `via` target for fields of type `Box<T>` where `T: WatError`
/// (e.g. `Box<MacroError>`, `Box<RuntimeError>`). `error_edn_of` takes
/// `&impl WatError`; a `&Box<MacroError>` does NOT coerce to that, but
/// `cause.error_edn()` auto-derefs through the Box.
///
/// Signature matches the `via` contract: `fn(&Box<T>) -> OwnedValue`.
#[expect(
    clippy::borrowed_box,
    reason = "The `&Box<T>` is REQUIRED, not incidental — see the doc comment above. This is a \
              `via` target for the ToEdn derive, which passes `&self.field` where the field is \
              `Box<T>`; that is a `&Box<T>` and does not coerce to `&T`. Taking `&T` as clippy \
              suggests makes the function uncallable from the derive it exists to serve. \
              `#[expect]` so that if the derive is ever taught to deref before calling `via`, \
              this attribute reports itself stale and the signature can be narrowed then."
)]
pub(crate) fn error_edn_of_boxed<T: WatError>(cause: &Box<T>) -> OwnedValue {
    cause.error_edn()
}

// ─── The wire boundary (the structural wall) ─────────────────────────────────

/// Convert any substrate error to its wire EDN text **through its [`WatError`]
/// impl**, enforcing the `:wat::core::Error` floor.
///
/// This is the single, named, generic conversion from an error to the text
/// that crosses the process boundary (the `ProcessDiedError` payload). It is
/// **generic over [`WatError`]**: a type that does not implement the trait is a
/// COMPILE error here, so it has no path to the wire. This is the structural
/// wall arc 296 strike 2 promises — "serialize a floor-less error" has no
/// representable form.
///
/// Calling `e.error_edn()` (rather than `e.to_edn()`) ensures every wire
/// payload carries `:message`, `:location`, and `:causes`, regardless of the
/// specific error variant. The 11-key span heresy (`:span` appearing under
/// different keys across families) is dead: the floor emits ONE `:location`
/// and the variant() strips the raw `:span`.
///
/// ## The wall: only `WatError` reaches the boundary
///
/// A type implementing only [`ToEdn`] (NOT `WatError`) is a compile error:
///
/// ```compile_fail
/// // ToEdn alone is insufficient — the floor requires WatError.
/// struct FloorlessError;
/// impl wat::edn::contract::ToEdn for FloorlessError {
///     fn to_edn(&self) -> wat_edn::OwnedValue { wat_edn::OwnedValue::Nil }
/// }
/// // ERROR[E0277]: the trait bound `FloorlessError: WatError` is not satisfied.
/// let _: String = wat::edn::contract::to_wire_edn(&FloorlessError);
/// ```
///
/// A type implementing neither `ToEdn` nor `WatError` also fails:
///
/// ```compile_fail
/// struct NotSerializable;
/// // ERROR[E0277]: `NotSerializable: WatError` is not satisfied.
/// let _: String = wat::edn::contract::to_wire_edn(&NotSerializable);
/// ```
///
/// A real substrate error (implementing `WatError`) reaches the boundary:
///
/// ```
/// use wat::value::{RuntimeError, RuntimeErrorKind};
/// let err = RuntimeError {
///     span: wat::rust_caller_span!(),
///     kind: RuntimeErrorKind::UserMainMissing,
/// };
/// let _text: String = wat::edn::contract::to_wire_edn(&err);
/// ```
pub fn to_wire_edn(e: &impl WatError) -> String {
    wat_edn::write(&e.error_edn())
}

/// The honest form for a genuinely message-only failure — a syscall
/// error string, a `:user::main` return-type name. The string IS the datum
/// (there is no span, no kind, no structured sub-fields to lose); this is NOT
/// a stringified structured error.
///
/// Serializes to `#wat.kernel/<tag> {:message "…" :location nil :causes []
/// :<key> "<message>"}`. `FlatMessage` implements [`WatError`] so it too
/// crosses the wire boundary through the floor — even a flat OS-level failure
/// carries `:message`/`:location`/`:causes`. It is NOT excluded from
/// `WatError` (unlike the embedded sub-values `ValueSnapshot`, `Provenance`,
/// `Span`, …): a `FlatMessage` IS a top-level error at the wire, not a
/// sub-value inside another error's EDN.
pub(crate) struct FlatMessage<'a> {
    pub tag: &'a str,
    pub key: &'a str,
    pub message: &'a str,
}

impl ToEdn for FlatMessage<'_> {
    fn to_edn(&self) -> OwnedValue {
        edn_tag(
            self.tag,
            OwnedValue::Map(vec![(edn_kw(self.key), edn_str(self.message))]),
        )
    }
}

impl WatError for FlatMessage<'_> {
    fn message(&self) -> String {
        first_line(self.message.to_owned())
    }
    /// A flat message has no recoverable source location.
    fn location(&self) -> OwnedValue {
        OwnedValue::Nil
    }
    fn causes(&self) -> OwnedValue {
        OwnedValue::Vector(vec![])
    }
    /// The variant carries the raw `#wat.kernel/<tag> {:<key> "…"}` envelope.
    /// When `key == "message"` the floor's own `:message` would collide;
    /// `error_edn()` dedups the floor keys, so the single floor `:message`
    /// wins and the wire map stays well-formed.
    fn variant(&self) -> OwnedValue {
        self.to_edn()
    }
}

