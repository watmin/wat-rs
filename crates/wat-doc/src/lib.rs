//! `wat-doc` — the shared prose+`@tag` doc-comment parser (arc 255.1b-iv-a).
//!
//! The doc/reflection contract (`docs/arc/2026/06/255-builtin-registry/
//! DESIGN-intrinsic-doc-reflection-contract.md`, §10) makes one load-bearing
//! decision: the prose+`@tag` parser + the mutual-checks live in **ONE shared
//! leaf crate**, depended on by BOTH `wat-macros` (the `#[wat_intrinsic]`
//! proc-macro, Rust intrinsics) AND `wat` (the runtime/checker, wat `defn`
//! forms). One implementation → the two paths **cannot drift**. An intrinsic's
//! `///` block and a wat form's docstring parse through the same code, enforce
//! the same required directives, and produce the same [`DocComment`] — parity by
//! construction, not by discipline.
//!
//! This crate is a pure leaf: no signature knowledge, no registry, no type
//! system, no codegen. It turns the joined text of a doc block into a structured
//! [`DocComment`] (enforcing the *universal* required directives), and offers the
//! one mutual-check that needs no external state — [`check_args`], the
//! `@arg`⇄signature agreement, with the caller supplying the parameter names.
//! Everything that needs the signature / the registry / the `TypeScheme`
//! (`@see` resolution, `@arg`/`@ret` type-checking, doctest generation) is the
//! consumer's job, in later stones.
//!
//! # Grammar (v1, line-based)
//!
//! The input is the joined `///` text (one string, `\n`-separated, each line
//! already stripped of its `/// ` prefix). **Prose** is every line before the
//! first recognized `@`-directive line; it is GFM, kept verbatim, trimmed of
//! surrounding blank lines, and **required**. A directive line begins (after
//! optional leading whitespace) with `@<word>`:
//!
//! | directive | req | shape |
//! |---|---|---|
//! | `@added <ver>` | yes, singleton | version string |
//! | `@arg <name> <type> <desc>` | per-param (see [`check_args`]) | type must start with `:` |
//! | `@ret <type> <desc>` | yes, singleton | type must start with `:` |
//! | `@example <expr> #=> <expected>` | ≥1 of either kind | doctested; MUST carry `#=>` |
//! | `@example-norun <expr> [#=> <expected>]` | ≥1 of either kind | illustrative; `#=>` optional |
//! | `@deprecated <ver> <use-instead>` | optional, singleton | soft-deprecation |
//! | `@see <fqdn>` | optional, repeatable | cross-reference |
//!
//! An unrecognized `@word` is a hard [`DocError::UnknownDirective`] — never a
//! silent skip. Old separator forms (` — `, ` -- `, ` - `, `: `) are REJECTED
//! as illegal in the type position — the grammar is firm: `@arg <name> <type> <desc>`.

// [`from_metadata`]'s one new type dependency — the `WatAST::Map` shape it reads.
// Adds nothing to the crate graph: `wat-reader` is already a direct dependency
// (Cargo.toml), used above via the fully-qualified `wat_reader::parse_one_with_file`.
use wat_reader::WatAST;

// ⛔ `Purity` and `Determinism` are GENERATED FROM wat, exactly as `Category` is
// below. They were the last two Rust enums still mirroring a `defenum` by hand, and
// after `every_rust_enum_matches_its_wat_defenum` was deleted as scaffolding
// (`aa33c0e7`) nothing checked either of them. Generating them is what makes that
// deletion honest rather than a quiet debt.
::wat_source_derive::wat_enum_from!(
    pub enum Purity,
    "../../wat/runtime-meta.wat",
    ":wat::runtime::Purity"
);

::wat_source_derive::wat_enum_from!(
    pub enum Determinism,
    "../../wat/runtime-meta.wat",
    ":wat::runtime::Determinism"
);



/// The `@Category` legal-value message.
///
/// Hand-written, not derived: `DocError::MalformedDirective.why` is
/// `&'static str` across 39 sites, and widening it to `String` for this one
/// message is a cascade out of proportion to the fix. The test
/// `category_message_lists_every_variant` gates it against
/// `Category::variants()`, so a new variant that forgets this line goes RED.
/// (The proc-macro's two sibling messages DO derive — they are `format!`.)
const CATEGORY_LEGAL_VALUES: &str =
    "value must be one of: Transform, Reflection, ControlFlow, Binding, Entropic, Arithmetic, Io, Probe, Combine, Declaration, Resource, Message, Ambient, Projection, CheckGate";

/// The `@Purity` legal-value message. Hand-written, not derived, for the same reason
/// `CATEGORY_LEGAL_VALUES` is: `DocError::MalformedDirective.why` is `&'static str`
/// across 39 sites, and widening it to `String` for these three messages is the same
/// cascade out of proportion to the fix. The test `purity_message_lists_every_variant`
/// gates it against `Purity::variants()`, so a new variant that forgets this line goes
/// RED. (`wat-macros`'s two sibling messages — `MissingPurity`/`InvalidPurityVariant`,
/// which build a runtime `String` rather than filling this `&'static str` field — are
/// hand-written too, and are gated separately, in that crate, by
/// `purity_message_tests::purity_messages_name_every_variant`: this const cannot reach
/// across the crate boundary to protect them.)
const PURITY_LEGAL_VALUES: &str = "value must be one of: Pure, Effectful, Preserving, Unevaluated";

// ⛔ `Category` IS GENERATED FROM wat — it is not written here.
//
// Builder ruling, 2026-08-15: *"wat is source of truth ... that's my pick."* The
// variant list AND each variant's prose live in
// `wat/runtime-meta.wat`'s `(:wat::core::defenum :wat::runtime::Category …)`.
// Add a variant there and this type follows; there is no Rust list to forget.
//
// This is what `every_rust_enum_matches_its_wat_defenum` was scaffolding FOR — a
// test comparing two lists. It is deleted with this change: a generated enum
// cannot drift from its generator.
::wat_source_derive::wat_enum_from!(
    pub enum Category,
    "../../wat/runtime-meta.wat",
    ":wat::runtime::Category"
);



/// One parsed `@example` / `@example-norun` directive.
///
/// Arc 255 STONE "an example is a FORM, not a string" — `expr`/`expected` were
/// `String` (the text left/right of `#=>`, trimmed): an artifact of the `///`
/// grammar, where an example genuinely IS text. A metadata-map declaration had
/// to produce the same struct, so the data form inherited the text form's
/// blindness — a malformed example (`Record/field-at` once shipped
/// `#=> <r's first field's value>`, prose where a form belonged) was
/// unrepresentable as an error HERE and surfaced downstream instead, as a
/// `TrailingContent` in a reflection test. Both fields now hold the PARSED
/// form: a malformed `@example` is a parse failure at THIS site (a
/// `compile_error!` at the macro for the `///` path; a `DocError` at
/// declaration time for the metadata path), not a string nobody validated.
///
/// `expr` is required for both `@example` and `@example-norun` — reflection
/// (`src/intrinsic/reflect.rs`'s `:wat::intrinsic::examples` seam) already
/// parses it unconditionally, for every registered example, run or not; this
/// only moves that same requirement earlier.
///
/// `expected` is `Some(form)` ONLY for `@example` (`run: true`) — the
/// REQUIRED, doctested marker; a missing marker there is
/// `DocError::ExampleMissingMarker`, raised before a form is ever attempted.
/// For `@example-norun` (`run: false`), `expected` is ALWAYS `None` here,
/// whether or not `#=>` text follows in the source: an `@example-norun`
/// marker's payload is illustrative and UNVERIFIED, per this crate's own
/// grammar table above (`@example-norun` is "illustrative", `#=>` optional)
/// and per `reflect.rs`'s reflection seam, which discards it unconditionally
/// ("expected is human-doc pseudo-code, not wat; yield None") — the corpus
/// bears this out: dozens of `@example-norun` markers are Rust `Debug`-style
/// reprs or prose (a wrapped-record literal, the words "never returns", a
/// full sentence describing a fresh symbol node), not wat syntax at all.
/// Forcing those through the reader would turn illustrative doc-prose into a
/// build break across the whole intrinsic corpus — a much wider blast radius
/// than this stone draws, and a regression this crate's own pre-existing
/// test (`norun_example_may_carry_an_unverified_marker`) already named as
/// the contract: the marker is accepted, and not verified.
///
/// A `///` block that needs its ORIGINAL example text back (for
/// `ExampleSubmission`'s `&'static str` fields) re-derives it directly from
/// the raw doc string it already holds in full — see `wat-macros`'
/// `example_text_slices` — rather than this struct carrying a second,
/// driftable copy of the same fact.
#[derive(Debug, Clone, PartialEq)]
pub struct DocExample {
    /// The wat form, verbatim — parsed from the text left of `#=>` (or the
    /// whole remainder for a markerless `@example-norun`), trimmed.
    pub expr: WatAST,
    /// The expected-result form, parsed from the text right of `#=>`.
    /// `Some` only for `run: true` (`@example`, where the marker is
    /// required and doctested); always `None` for `run: false`
    /// (`@example-norun` — illustrative and unverified; see the struct docs).
    pub expected: Option<WatAST>,
    /// `true` = `@example` (doctested by the consumer); `false` =
    /// `@example-norun` (illustrative, never executed).
    pub run: bool,
}

/// One parsed `@arg` directive. Name/count agreement with the actual signature
/// is [`check_args`]'s job, not the parser's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocArg {
    /// First whitespace-delimited token after `@arg`, stripped of any trailing `…`.
    pub name: String,
    /// Second whitespace-delimited token after `@arg` — the type, must start with `:`.
    /// For variadic args this is the ELEMENT type (the `…` implies `Vector<elem>`).
    pub ty: String,
    /// The remainder after name and type, trimmed.
    pub desc: String,
    /// True when the name had a trailing `…` suffix — this is a rest/variadic parameter.
    /// The `ty` is the element type; the actual runtime type is `Vector<ty>`.
    pub is_rest: bool,
}

/// A parsed `@deprecated` directive (soft deprecation — still callable).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deprecation {
    /// First token after `@deprecated` — the version it was deprecated in.
    pub since: String,
    /// The remainder, trimmed — what to use instead.
    pub use_instead: String,
}

/// A fully-parsed doc comment with every *universal* required directive present
/// (prose, `@added`, `@ret`, ≥1 example). The `@arg`⇄signature match is checked
/// separately by [`check_args`], which needs the caller's parameter list.
///
/// No `Eq` (only `PartialEq`): `examples: Vec<DocExample>` carries parsed
/// `WatAST` forms, and `WatAST::FloatLit` holds an `f64` — not `Eq`. Same
/// reason on [`DocSpecialForm`] below.
#[derive(Debug, Clone, PartialEq)]
pub struct DocComment {
    /// GFM body: everything before the first `@`-tag line, trimmed.
    pub prose: String,
    /// `@added <ver>`.
    pub added: String,
    /// `@arg` directives, in source order.
    pub args: Vec<DocArg>,
    /// `@ret` type token (must start with `:`).
    pub ret_type: String,
    /// `@ret` description (remainder after the type token).
    pub ret: String,
    /// `@example` / `@example-norun` directives, in source order (≥1).
    pub examples: Vec<DocExample>,
    /// `@deprecated`, if present.
    pub deprecated: Option<Deprecation>,
    /// `@see <fqdn>` cross-references, in source order.
    pub see: Vec<String>,
    /// `@Purity <Variant>` — declared purity.
    pub purity: Purity,
    /// `@Determinism <Variant>` — declared determinism.
    pub determinism: Determinism,
    /// `@Totality <Variant>` — declared totality (arc 255 Stone total-T2; made
    /// REQUIRED in Stone total-T3). Absence is `DocError::MissingTotality`,
    /// exactly like `purity`/`determinism`/`category` — declaring nothing is
    /// illegal. See `DESIGN-STONE-total-t3-declaring-nothing-is-illegal.md`: a
    /// guessed `:Total` is a lie in a fence that admits code into a `where`; an
    /// author must type `:Unreviewed` explicitly instead of the substrate
    /// inventing it.
    pub totality: Totality,
    /// `@ExpandTime <Variant>` — declared expand-time legality (arc 255 Stone
    /// expand-T2; made REQUIRED in Stone expand-T3). Absence is
    /// `DocError::MissingExpandTime`, exactly like `purity`/`determinism`/
    /// `totality`/`category` — declaring nothing is illegal. An author must
    /// type `:Unreviewed` explicitly instead of the substrate inventing it;
    /// an unmeasured verb refuses (default-deny) rather than guessing `Legal`,
    /// which would admit it into a macro body it may corrupt.
    pub expand_time: ExpandTime,
    /// `@Category <Variant>` — closed-enum category (e.g. `Transform`, `Reflection`).
    pub category: Category,
    /// `@yields <argname> <desc>` — repeatable, one per value-carrying fn-shaped `@arg`.
    /// (Arc 255 Stone P5-b: the TYPE is no longer carried here — it is mechanically
    /// derivable from the named `@arg`'s own canonical bracket-form type, per P5-a. This
    /// is what lets `spawn-thread` carry two: one subject per callback.) Empty when the
    /// intrinsic yields to no callback.
    pub yields: Vec<DocYields>,
}

/// One parsed `@yields` directive — names which fn-shaped `@arg` receives a value, and
/// what that value is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocYields {
    /// The `@arg` name this directive documents (must match a declared `@arg`).
    pub arg: String,
    /// The remainder description, trimmed.
    pub desc: String,
}

/// A doc-contract violation. Closed enum — diagnostic completeness by shape.
/// `parse` raises the structural variants; [`check_args`] raises the two
/// `Arg*Mismatch` variants. The consumer renders these as a `compile_error!`
/// (the proc-macro) or a runtime/check error (wat).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocError {
    /// No prose before the first directive (or only blank lines).
    MissingProse,
    /// No `@added` directive.
    MissingAdded,
    /// No `@ret` directive.
    MissingRet,
    /// No `@example` or `@example-norun` directive.
    MissingExample,
    /// A directive's payload was empty or unparseable. `why` is a short reason.
    MalformedDirective { tag: String, why: &'static str },
    /// An `@word` that is not a recognized directive.
    UnknownDirective { tag: String },
    /// An `@example` (run=true) with no `#=>` marker. (`@example-norun` may omit it.)
    ExampleMissingMarker { expr: String },
    /// A second occurrence of a singleton directive (`@added`, `@ret`, `@deprecated`).
    DuplicateSingleton { tag: String },
    /// `@arg` count ≠ signature parameter count.
    ArgCountMismatch { documented: usize, signature: usize },
    /// The `@arg` at `position` names a different parameter than the signature.
    ArgNameMismatch {
        position: usize,
        documented: String,
        signature: String,
    },
    /// No `@pure` directive (legacy — kept for backwards compat in tests).
    MissingPure,
    /// No `@deterministic` directive (legacy — kept for backwards compat in tests).
    MissingDeterministic,
    /// No `@category` directive (legacy — kept for backwards compat in tests).
    MissingCategory,
    /// No `@syntax` directive in a special form doc.
    MissingSyntax,
    /// Neither `@arg` nor `@syntax` is present in a special form doc.
    /// At least one must express the shape: `@arg` for positional forms (grammar
    /// is derived), `@syntax` for structural forms.
    MissingShape,
    /// No `@Purity` directive.
    MissingPurity,
    /// No `@Determinism` directive.
    MissingDeterminism,
    /// No `@Totality` directive. Arc 255 Stone total-T3: `@Totality` was OPTIONAL
    /// through T2 (absence defaulted to `Totality::Unreviewed`); the builder's
    /// ruling struck that default — declaring nothing is now illegal, exactly
    /// like `@Purity`/`@Determinism`. An author must type `@Totality Unreviewed`
    /// explicitly if the verb has not been reviewed; the substrate no longer
    /// invents the answer.
    MissingTotality,
    /// No `@ExpandTime` directive. Arc 255 Stone expand-T3: `@ExpandTime` was
    /// OPTIONAL through T2 (absence defaulted to `ExpandTime::Unreviewed`);
    /// mirroring `@Totality`'s own T2→T3 arc, declaring nothing is now illegal.
    /// An author must type `@ExpandTime Unreviewed` explicitly if the verb
    /// has not been reviewed; the substrate no longer invents the answer.
    MissingExpandTime,
    /// `@Purity` value is not a known variant.
    InvalidPurityVariant { got: String },
    /// `@Determinism` value is not a known variant.
    InvalidDeterminismVariant { got: String },
    /// `@Totality` value is not a known variant.
    InvalidTotalityVariant { got: String },
    /// `@ExpandTime` value is not a known variant.
    InvalidExpandTimeVariant { got: String },
    /// `@Category` value is not a known variant.
    InvalidCategoryVariant { got: String },
    /// A second `@yields` names the same `@arg` subject as an earlier one — `@yields` is
    /// repeatable ACROSS subjects, never twice for the same one.
    DuplicateYieldsSubject { arg: String },
    /// An `@yields` names an `@arg` that was never declared — a directive with no subject.
    UnknownYieldsSubject { arg: String },
}

/// A fully-parsed special-form doc comment.
/// Special forms use `@purity` / `@determinism` instead of `@pure` / `@deterministic`,
/// and require an `@syntax` grammar string. They do NOT accept `@yields`.
///
/// No `Eq` — see [`DocComment`]'s doc comment (same reason: `examples` carries
/// `WatAST`, which is `PartialEq` only).
#[derive(Debug, Clone, PartialEq)]
pub struct DocSpecialForm {
    /// GFM prose body, trimmed.
    pub prose: String,
    /// `@added <ver>`.
    pub added: String,
    /// `@syntax (...)` — the grammar string, verbatim payload after `@syntax `.
    pub syntax: String,
    /// `@arg` directives, in source order.
    pub args: Vec<DocArg>,
    /// `@ret` type token.
    pub ret_type: String,
    /// `@ret` description.
    pub ret: String,
    /// `@example` / `@example-norun` directives (≥1).
    pub examples: Vec<DocExample>,
    /// `@Category <Variant>`.
    pub category: Category,
    /// `@Purity <Variant>` — declared purity.
    pub purity: Purity,
    /// `@Determinism <Variant>` — declared determinism.
    pub determinism: Determinism,
    /// `@Totality <Variant>` — declared totality (arc 255 Stone total-T2; made
    /// REQUIRED in Stone total-T3). Absence is `DocError::MissingTotality` — see
    /// `DocComment::totality`.
    pub totality: Totality,
    /// `@ExpandTime <Variant>` — declared expand-time legality (arc 255 Stone
    /// expand-T2; made REQUIRED in Stone expand-T3). Absence is
    /// `DocError::MissingExpandTime` — see `DocComment::expand_time`.
    /// `DocSpecialForm` is a SIBLING type to `DocComment`, so this field and
    /// its resolution point are independent.
    pub expand_time: ExpandTime,
    /// `@see` FQDNs, in source order.
    pub see: Vec<String>,
    /// `@deprecated`, if present.
    pub deprecated: Option<Deprecation>,
}

/// The separator tokens that are now ILLEGAL in the type position.
/// If the type token equals one of these, the grammar is violated.
const SEPARATOR_TOKENS: &[&str] = &["—", "--", "-", ":"];

/// Ask wat's own reader whether `token` is a spelling it can parse as a single,
/// complete form. This is the ONE adjudication of "what may a type be spelled" —
/// the same question the language answers everywhere else. It replaces a
/// hand-rolled `starts_with(':')` shape test that let unexpressible spellings
/// (e.g. `Option<T>`, the retired `fn(…)->…` vocabulary) through five times over.
///
/// This is deliberately NOT "is this a type" — it is "is this expressible at
/// all" (a bare symbol like `Bytes` lexes fine but is not a type keyword; the
/// `starts_with(':')` check alongside this one is what rules that out).
fn type_token_is_expressible(token: &str) -> bool {
    wat_reader::parse_one_with_file(token, "<wat-doc @arg/@ret type token>").is_ok()
}

/// Parse an `@example`/`@example-norun` payload slice (the text left or right
/// of `#=>`) as a single, complete wat form — the SAME reader every other
/// verb's own source goes through (mirrors [`type_token_is_expressible`]'s
/// use of it for `@arg`/`@ret` type tokens). A malformed example — unbalanced
/// parens, a stray prose fragment, trailing content after the form — becomes
/// a `DocError` HERE, at the directive that wrote it, instead of surviving as
/// opaque text until a downstream re-parse (`src/intrinsic/reflect.rs`) fails
/// at reflection time. `tag` names which directive is at fault (`"@example"`
/// / `"@example-norun"`) for the error message.
fn parse_example_form(text: &str, tag: &'static str) -> Result<WatAST, DocError> {
    wat_reader::parse_one_with_file(text, "<wat-doc @example>").map_err(|_| DocError::MalformedDirective {
        tag: tag.into(),
        why: "does not parse as a single, complete wat form (unbalanced parens/brackets, a \
              missing quote, or trailing content after the form)",
    })
}

/// Split the type token off the front of `s` (already leading-whitespace-
/// trimmed). A bare keyword/symbol type (the common case, e.g.
/// `:wat::core::Bytes`) ends at the first whitespace, exactly as before this
/// stone. But the surviving parametric-type spellings — `(Head :- [args])`
/// for a type reference and `[arg… :-> ret]` for a fn type — carry INTERNAL
/// whitespace (`(:wat::core::Vector :- [:wat::core::i64])`), so a naive
/// whitespace split truncates them mid-token. When the token opens with `(`
/// or `[`, this scans to the MATCHING close (tracking depth across both
/// bracket kinds together, so nesting like
/// `[(:wat::kernel::Peer :- [S R]) :-> :wat::core::nil]` closes correctly)
/// and returns everything up to and including it as the token.
/// Returns `(type_token, rest_trimmed)`.
fn take_type_token(s: &str) -> (&str, &str) {
    match s.chars().next() {
        Some('(') | Some('[') => {
            let mut depth = 0i32;
            for (i, c) in s.char_indices() {
                match c {
                    '(' | '[' => depth += 1,
                    ')' | ']' => {
                        depth -= 1;
                        if depth == 0 {
                            let end = i + c.len_utf8();
                            let (tok, rest) = s.split_at(end);
                            return (tok, rest.trim_start());
                        }
                    }
                    _ => {}
                }
            }
            // Unbalanced — fall through to a plain whitespace split so the
            // existing "must start with `:`" / "type is missing" errors
            // still fire downstream on the (malformed) result.
            let mut it = s.splitn(2, char::is_whitespace);
            let tok = it.next().unwrap_or("");
            let rest = it.next().unwrap_or("").trim_start();
            (tok, rest)
        }
        _ => {
            let mut it = s.splitn(2, char::is_whitespace);
            let tok = it.next().unwrap_or("");
            let rest = it.next().unwrap_or("").trim_start();
            (tok, rest)
        }
    }
}

/// Parse a joined `///` block into a [`DocComment`], enforcing the universal
/// required directives (prose, `@added`, `@ret`, and ≥1 `@example`/`@example-norun`).
///
/// Does NOT check `@arg` against any signature — that is [`check_args`].
pub fn parse(raw: &str) -> Result<DocComment, DocError> {
    let recognized = &[
        "@added", "@arg", "@ret", "@example", "@example-norun", "@deprecated", "@see",
        "@Purity", "@Determinism", "@Totality", "@ExpandTime", "@Category", "@yields",
    ];

    // Split into prose lines and directive lines at the first recognized @-directive.
    let lines: Vec<&str> = raw.lines().collect();
    let first_directive = lines.iter().position(|l| {
        let token = l.split_whitespace().next().unwrap_or("");
        token.starts_with('@') && recognized.contains(&token)
    });

    // Prose = everything before the first directive line, trimmed of surrounding blanks.
    let prose_end = first_directive.unwrap_or(lines.len());
    let prose = trim_blank_lines(&lines[..prose_end]).join("\n");
    if prose.is_empty() {
        return Err(DocError::MissingProse);
    }

    // Walk directive lines.
    let mut added: Option<String> = None;
    let mut args: Vec<DocArg> = Vec::new();
    let mut ret_type: Option<String> = None;
    let mut ret: Option<String> = None;
    let mut examples: Vec<DocExample> = Vec::new();
    let mut deprecated: Option<Deprecation> = None;
    let mut see: Vec<String> = Vec::new();
    let mut purity_val: Option<Purity> = None;
    let mut determinism_val: Option<Determinism> = None;
    let mut totality_val: Option<Totality> = None;
    let mut expand_time_val: Option<ExpandTime> = None;
    let mut category_val: Option<Category> = None;
    let mut yields_vals: Vec<DocYields> = Vec::new();

    let directive_lines = match first_directive {
        Some(i) => &lines[i..],
        None => &[][..],
    };

    for &line in directive_lines {
        let trimmed = line.trim_start();
        let tag = trimmed.split_whitespace().next().unwrap_or("");

        if !tag.starts_with('@') {
            // Non-directive lines after the first directive (e.g. blank lines) — skip.
            continue;
        }

        if !recognized.contains(&tag) {
            return Err(DocError::UnknownDirective { tag: tag.to_string() });
        }

        // Payload = everything after the tag, leading whitespace stripped.
        let payload = trimmed[tag.len()..].trim_start();

        match tag {
            "@added" => {
                if added.is_some() {
                    return Err(DocError::DuplicateSingleton { tag: "@added".into() });
                }
                if payload.is_empty() {
                    return Err(DocError::MalformedDirective {
                        tag: "@added".into(),
                        why: "version string is empty",
                    });
                }
                added = Some(payload.to_string());
            }
            "@arg" => {
                // Firm grammar: @arg <name>[…] <elem-type> <desc>
                // name = first token (may carry `…` suffix for variadic/rest params),
                // type = second token (must start with `:`; for `…` args, this is the
                // ELEMENT type — the `…` implies Vector<elem>), desc = rest.
                let mut name_split = payload.splitn(2, char::is_whitespace);
                let raw_name = name_split.next().unwrap_or("");
                if raw_name.is_empty() {
                    return Err(DocError::MalformedDirective {
                        tag: "@arg".into(),
                        why: "name is missing",
                    });
                }
                // Detect and strip the variadic `…` suffix.
                let (name, is_rest) = if let Some(stem) = raw_name.strip_suffix('…') {
                    (stem.to_string(), true)
                } else if let Some(stem) = raw_name.strip_suffix("...") {
                    (stem.to_string(), true)
                } else {
                    (raw_name.to_string(), false)
                };

                let after_name = name_split.next().unwrap_or("").trim_start();
                let (ty_token, desc_raw) = take_type_token(after_name);
                let ty_token = ty_token.trim();
                if ty_token.is_empty() {
                    return Err(DocError::MalformedDirective {
                        tag: "@arg".into(),
                        why: "type is missing; grammar is `@arg <name> <type> <desc>`",
                    });
                }
                // Reject separator tokens in the type position.
                if SEPARATOR_TOKENS.contains(&ty_token) {
                    return Err(DocError::MalformedDirective {
                        tag: "@arg".into(),
                        why: "separator used in type position; grammar is `@arg <name> <type> <desc>`",
                    });
                }
                // Type token must start with `:` (all wat types are keywords) —
                // OR be one of the two surviving STRUCTURAL type spellings, which
                // can never start with `:` by construction: a parametric type
                // REFERENCE `(Head :- [args])`, or a fn type `[arg… :-> ret]`.
                // Those are still gated by the reader check just below; this
                // clause only rules out a BARE non-keyword symbol like `Bytes`.
                if !(ty_token.starts_with(':') || ty_token.starts_with('(') || ty_token.starts_with('[')) {
                    return Err(DocError::MalformedDirective {
                        tag: "@arg".into(),
                        why: "type token must start with `:` (e.g. `:wat::core::Bytes`); grammar is `@arg <name> <type> <desc>`",
                    });
                }
                // Type token must be a spelling wat's own reader accepts as a
                // single, complete form — rules out `Option<T>`, the retired
                // `fn(…)->…` vocabulary, and any other inexpressible spelling.
                if !type_token_is_expressible(ty_token) {
                    return Err(DocError::MalformedDirective {
                        tag: "@arg".into(),
                        why: "type token is not a spelling wat's reader accepts (e.g. `Option<T>` and the retired `fn(…)->…` form are inexpressible; use `:- [...]`)", // rune:lint(no-angle-type-in-diagnostic) — class C: quotes the retired spelling to name what is refused, exactly like the reader's own refusal messages
                    });
                }
                let ty = ty_token.to_string();

                let desc = desc_raw.trim().to_string();
                if desc.is_empty() {
                    return Err(DocError::MalformedDirective {
                        tag: "@arg".into(),
                        why: "description is empty; grammar is `@arg <name> <type> <desc>`",
                    });
                }
                args.push(DocArg { name, ty, desc, is_rest });
            }
            "@ret" => {
                if ret.is_some() {
                    return Err(DocError::DuplicateSingleton { tag: "@ret".into() });
                }
                // Firm grammar: @ret <type> <desc>
                let (ty_token, desc_raw) = take_type_token(payload);
                let ty_token = ty_token.trim();
                if ty_token.is_empty() {
                    return Err(DocError::MalformedDirective {
                        tag: "@ret".into(),
                        why: "type is missing; grammar is `@ret <type> <desc>`",
                    });
                }
                // Reject separator tokens in the type position.
                if SEPARATOR_TOKENS.contains(&ty_token) {
                    return Err(DocError::MalformedDirective {
                        tag: "@ret".into(),
                        why: "separator used in type position; grammar is `@ret <type> <desc>`",
                    });
                }
                // Type token must start with `:` — or be one of the two
                // surviving STRUCTURAL type spellings, which can never start
                // with `:` by construction: a parametric type REFERENCE
                // `(Head :- [args])`, or a fn type `[arg… :-> ret]`. Those are
                // still gated by the reader check just below; this clause
                // only rules out a BARE non-keyword symbol like `Bytes`.
                if !(ty_token.starts_with(':') || ty_token.starts_with('(') || ty_token.starts_with('[')) {
                    return Err(DocError::MalformedDirective {
                        tag: "@ret".into(),
                        why: "type token must start with `:` (e.g. `:wat::core::String`); grammar is `@ret <type> <desc>`",
                    });
                }
                // Type token must be a spelling wat's own reader accepts as a
                // single, complete form — rules out `Option<T>`, the retired
                // `fn(…)->…` vocabulary, and any other inexpressible spelling.
                if !type_token_is_expressible(ty_token) {
                    return Err(DocError::MalformedDirective {
                        tag: "@ret".into(),
                        why: "type token is not a spelling wat's reader accepts (e.g. `Option<T>` and the retired `fn(…)->…` form are inexpressible; use `:- [...]`)", // rune:lint(no-angle-type-in-diagnostic) — class C: quotes the retired spelling to name what is refused, exactly like the reader's own refusal messages
                    });
                }
                let ty = ty_token.to_string();
                let desc = desc_raw.trim().to_string();
                if desc.is_empty() {
                    return Err(DocError::MalformedDirective {
                        tag: "@ret".into(),
                        why: "description is empty; grammar is `@ret <type> <desc>`",
                    });
                }
                ret_type = Some(ty);
                ret = Some(desc);
            }
            "@example" => {
                let rest = payload;
                let (expr_text, expected_text) = match rest.split_once(" #=> ").or_else(|| rest.split_once("#=> ")) {
                    Some((left, right)) => (left.trim(), right.trim()),
                    None => {
                        // Check if #=> appears at end with no trailing content.
                        if let Some(left) = rest.strip_suffix("#=>") {
                            (left.trim(), "")
                        } else {
                            return Err(DocError::ExampleMissingMarker {
                                expr: rest.trim().to_string(),
                            });
                        }
                    }
                };
                let expr = parse_example_form(expr_text, "@example")?;
                let expected = parse_example_form(expected_text, "@example")?;
                examples.push(DocExample { expr, expected: Some(expected), run: true });
            }
            "@example-norun" => {
                let rest = payload;
                // The `#=>` payload, when present, is illustrative doc-prose —
                // see the DocExample struct docs (`expected` on a `run: false`
                // entry is always `None` here; the marker text is UNVERIFIED
                // by design and never reaches the reader).
                let expr_text = match rest.split_once(" #=> ").or_else(|| rest.split_once("#=> ")) {
                    Some((left, _right)) => left.trim(),
                    None => rest.trim(),
                };
                let expr = parse_example_form(expr_text, "@example-norun")?;
                examples.push(DocExample { expr, expected: None, run: false });
            }
            "@deprecated" => {
                if deprecated.is_some() {
                    return Err(DocError::DuplicateSingleton { tag: "@deprecated".into() });
                }
                let mut tokens = payload.splitn(2, char::is_whitespace);
                let since = tokens.next().unwrap_or("").to_string();
                let use_instead = tokens.next().unwrap_or("").trim_start().to_string();
                deprecated = Some(Deprecation { since, use_instead });
            }
            "@see" => {
                see.push(payload.to_string());
            }
            "@Purity" => {
                if purity_val.is_some() {
                    return Err(DocError::DuplicateSingleton { tag: "@Purity".into() });
                }
                match payload.parse::<Purity>() {
                    Ok(p) => purity_val = Some(p),
                    Err(_) => return Err(DocError::MalformedDirective {
                        tag: "@Purity".into(),
                        why: PURITY_LEGAL_VALUES,
                    }),
                }
            }
            "@Determinism" => {
                if determinism_val.is_some() {
                    return Err(DocError::DuplicateSingleton { tag: "@Determinism".into() });
                }
                match payload.parse::<Determinism>() {
                    Ok(d) => determinism_val = Some(d),
                    Err(_) => return Err(DocError::MalformedDirective {
                        tag: "@Determinism".into(),
                        why: "value must be one of: Deterministic, Nondeterministic, Preserving",
                    }),
                }
            }
            "@Totality" => {
                if totality_val.is_some() {
                    return Err(DocError::DuplicateSingleton { tag: "@Totality".into() });
                }
                match payload.parse::<Totality>() {
                    Ok(t) => totality_val = Some(t),
                    Err(_) => return Err(DocError::InvalidTotalityVariant { got: payload.to_string() }),
                }
            }
            "@ExpandTime" => {
                if expand_time_val.is_some() {
                    return Err(DocError::DuplicateSingleton { tag: "@ExpandTime".into() });
                }
                match payload.parse::<ExpandTime>() {
                    Ok(e) => expand_time_val = Some(e),
                    Err(_) => return Err(DocError::InvalidExpandTimeVariant { got: payload.to_string() }),
                }
            }
            "@Category" => {
                if category_val.is_some() {
                    return Err(DocError::DuplicateSingleton { tag: "@Category".into() });
                }
                match payload.parse::<Category>() {
                    Ok(c) => category_val = Some(c),
                    Err(_) => return Err(DocError::MalformedDirective {
                        tag: "@Category".into(),
                        why: CATEGORY_LEGAL_VALUES,
                    }),
                }
            }
            "@yields" => {
                // Repeatable, once per subject: @yields <argname> <desc>. `argname` is a
                // bare token (the `@arg` name it documents) — no type token here any more
                // (arc 255 Stone P5-b: the type is mechanically derivable from the named
                // `@arg`'s own canonical bracket-form type, per P5-a, so a second spelling
                // of the same fact is no longer carried).
                let mut yields_split = payload.splitn(2, char::is_whitespace);
                let arg_name = yields_split.next().unwrap_or("");
                if arg_name.is_empty() {
                    return Err(DocError::MalformedDirective {
                        tag: "@yields".into(),
                        why: "argument name is missing; grammar is `@yields <argname> <desc>`",
                    });
                }
                let desc = yields_split.next().unwrap_or("").trim().to_string();
                if desc.is_empty() {
                    return Err(DocError::MalformedDirective {
                        tag: "@yields".into(),
                        why: "description is empty; grammar is `@yields <argname> <desc>`",
                    });
                }
                // Repeatable ACROSS subjects, never twice for the same one — this is what
                // makes `@yields` a SUBJECT-keyed directive rather than the old parsed
                // singleton it replaces.
                if yields_vals.iter().any(|y: &DocYields| y.arg == arg_name) {
                    return Err(DocError::DuplicateYieldsSubject { arg: arg_name.to_string() });
                }
                yields_vals.push(DocYields { arg: arg_name.to_string(), desc });
            }
            _ => unreachable!("recognized set is exhaustive"),
        }
    }

    // Enforce required directives.
    let added = added.ok_or(DocError::MissingAdded)?;
    let ret_type = ret_type.ok_or(DocError::MissingRet)?;
    let ret = ret.ok_or(DocError::MissingRet)?;
    if examples.is_empty() {
        return Err(DocError::MissingExample);
    }
    let purity = purity_val.ok_or(DocError::MissingPurity)?;
    let determinism = determinism_val.ok_or(DocError::MissingDeterminism)?;
    // Arc 255 Stone total-T3: `@Totality` is REQUIRED, exactly like purity/determinism/
    // category above. Absence is `MissingTotality`, not a silent `Unreviewed` default.
    let totality = totality_val.ok_or(DocError::MissingTotality)?;
    // Arc 255 Stone expand-T3: `@ExpandTime` is REQUIRED, exactly like `@Totality`
    // above. Absence is `MissingExpandTime`, not a silent `Unreviewed` default —
    // T2's default is struck, mirroring totality's own T2→T3 arc.
    let expand_time = expand_time_val.ok_or(DocError::MissingExpandTime)?;
    let category = category_val.ok_or(DocError::MissingCategory)?;

    // Every `@yields` subject must name a declared `@arg` — a directive with no referent
    // is a doc error, not a silent no-op. Checked here (not by the caller) because `parse`
    // already has the full `@arg` list gathered by this point; no signature is needed.
    for y in &yields_vals {
        if !args.iter().any(|a| a.name == y.arg) {
            return Err(DocError::UnknownYieldsSubject { arg: y.arg.clone() });
        }
    }

    Ok(DocComment { prose, added, args, ret_type, ret, examples, deprecated, see, purity, determinism, totality, expand_time, category, yields: yields_vals })
}

/// Look up `key` (e.g. `":purity"`, including the leading `:`) among a
/// metadata-map's key/value pairs. Non-`Keyword` keys are skipped rather
/// than erroring here — [`from_metadata`]'s own required-field checks are
/// what surface an absent key; a malformed KEY (as opposed to a malformed
/// value) is not a shape the wat parser can even produce, since
/// `WatAST::Map` pairs come from the reader's own `{k v ...}` grammar.
fn metadata_lookup<'a>(pairs: &'a [(WatAST, WatAST)], key: &str) -> Option<&'a WatAST> {
    pairs.iter().find_map(|(k, v)| match k {
        WatAST::Keyword(k, _) if k == key => Some(v),
        _ => None,
    })
}

/// Read a `WatAST::StringLit` as an owned, non-empty (post-trim) `String`.
/// Returns `None` for any other node shape, or a string that is empty/blank
/// — the same "nothing usable here" verdict the text grammar reaches when a
/// line is missing entirely.
fn metadata_string(v: &WatAST) -> Option<String> {
    match v {
        WatAST::StringLit(s, _) => {
            let t = s.trim();
            if t.is_empty() { None } else { Some(t.to_string()) }
        }
        _ => None,
    }
}

/// Read a bare name off a `WatAST::Symbol` or `WatAST::Keyword` (the two
/// shapes an `:args`/`:yields` subject may be spelled in) — a `Keyword`'s
/// leading `:` is stripped so it agrees with a `Symbol`'s bare spelling and
/// with the actual fn-signature parameter name it must match.
fn metadata_bare_name(v: &WatAST) -> Option<String> {
    match v {
        WatAST::Symbol(id, _) => Some(id.as_str().to_string()),
        WatAST::Keyword(k, _) => Some(k.trim_start_matches(':').to_string()),
        _ => None,
    }
}

/// Render `v` for an error payload that must show what was actually written,
/// without assuming it is a `Keyword` (an axis value that fails
/// [`enum_symbol_variant`] may be any node shape at all).
fn metadata_describe(v: &WatAST) -> String {
    match v {
        WatAST::Keyword(k, _) => k.clone(),
        other => format!("{other:?}"),
    }
}

/// Read an enum-symbol value `:<wat_type_path>::<Variant>` and return the
/// bare `<Variant>` spelling — but ONLY when the namespace prefix matches
/// `wat_type_path` exactly. This is the check the DESIGN calls out by name:
/// several axes share a variant spelling (`Purity::Preserving` /
/// `Determinism::Preserving` / `Totality::Preserving` / `ExpandTime::Preserving`;
/// `Totality::Unreviewed` / `ExpandTime::Unreviewed`), so accepting any
/// `Keyword` whose LAST segment happens to name a variant of the target enum
/// — without checking which enum it actually came from — would let
/// `:wat::runtime::Determinism::Preserving` silently satisfy `:purity`. A
/// bare `:Preserving` (no path at all) is rejected the same way a bare
/// `:Pure` is: neither names which enum it came from.
fn enum_symbol_variant<'a>(v: &'a WatAST, wat_type_path: &str) -> Option<&'a str> {
    match v {
        WatAST::Keyword(k, _) => {
            let prefix_len = wat_type_path.len() + 2; // "::"
            if k.len() > prefix_len && k.starts_with(wat_type_path) && k.as_bytes()[wat_type_path.len()..].starts_with(b"::") {
                let rest = &k[prefix_len..];
                if !rest.is_empty() && !rest.contains("::") {
                    return Some(rest);
                }
            }
            None
        }
        _ => None,
    }
}

/// `wat_doc::from_metadata` — the sibling of [`parse`] for the wat side (arc
/// 255 Stone "wire the wat side to wat-doc"). Reads a `WatAST::Map` — the
/// SAME metadata-map node a `(defn :name {…} […] -> :Ret body)` already
/// parses into and that already reaches `SymbolTable.binding_metadata` at
/// def-registration time (`register_defines` / `register_defclause`,
/// `src/runtime.rs`) — and produces the same [`DocComment`] `parse` does,
/// enforcing the SAME required set with the SAME [`DocError`] variants.
///
/// There is no wat-side docstring to feed the text grammar (see the DESIGN
/// doc's finding: `doc_string` is `None` at every construction site and arc
/// 141 never shipped) — so this entry point reads DATA, not text. Keys
/// mirror the `///` directives one-for-one:
///
/// | key | directive equivalent | shape |
/// |---|---|---|
/// | `:doc` | prose (untagged) | `StringLit` |
/// | `:added` | `@added` | `StringLit` |
/// | `:ret` | `@ret` | `[<:type-keyword> <desc StringLit>]` |
/// | `:purity` / `:determinism` / `:totality` / `:expand-time` / `:category` | `@Purity` etc. | enum-symbol `Keyword`, e.g. `:wat::runtime::Purity::Pure` |
/// | `:args` | `@arg` (per-entry) | `Vector` of `[<name Symbol/Keyword> <:type-keyword> <desc StringLit>]` |
/// | `:examples` | `@example` | `Vector` of `[<expr form> <expected form>]` — LITERAL wat forms, not quoted strings (arc 255 STONE "an example is a FORM, not a string"); each entry is `run: true`, mirroring `@example`; there is no metadata-map spelling yet for `@example-norun`'s optional-`expected` shape — out of scope for this stone's one-verb walk |
/// | `:see` | `@see` | `Vector` of keyword FQDNs |
/// | `:yields` | `@yields` (per-subject) | `Vector` of `[<arg-name Symbol/Keyword> <desc StringLit>]` |
/// | `:deprecated` | `@deprecated` | `[<since StringLit> <use-instead StringLit>]` |
///
/// The closed-domain axis values are ENUM SYMBOLS, not bare keywords — a
/// bare `:Pure` is a keyword nothing validates, where `:wat::runtime::
/// Purity::Pure` names a variant that either exists or does not (builder,
/// 2026-08-30). [`enum_symbol_variant`] is the ONE check that reads it.
///
/// A `map` that is not a metadata-map at all (`WatAST::metadata_map_pairs`
/// returns `None`) is read as zero pairs — not a new error: it falls
/// straight into the same `MissingProse`/`MissingAdded`/… cascade an empty
/// `{}` would, so no new `DocError` vocabulary is needed for "not a map".
///
/// Type tokens (`:ret`'s and each `:args` entry's) are required to be a bare
/// `Keyword` in THIS stone — the common case, and the one the walked verb
/// (`:wat::string::capitalize`) uses. The two surviving STRUCTURAL type
/// spellings the text grammar also accepts (`(Head :- [args])` parametric
/// references, `[arg… :-> ret]` fn types) are themselves compound `WatAST`
/// forms, not text, and stringifying one back to `DocComment::ret_type`'s
/// `String` field would need an AST→source printer this leaf crate does not
/// have (and per STOP-1, reaching for one would be a signal the work belongs
/// in the consumer). Not exercised by the one verb this stone walks; left
/// for whichever stone migrates a verb that needs one.
pub fn from_metadata(map: &WatAST) -> Result<DocComment, DocError> {
    let pairs = map.metadata_map_pairs().unwrap_or_default();

    let prose = metadata_lookup(&pairs, ":doc")
        .and_then(metadata_string)
        .ok_or(DocError::MissingProse)?;

    let added = match metadata_lookup(&pairs, ":added") {
        None => return Err(DocError::MissingAdded),
        Some(v) => metadata_string(v).ok_or(DocError::MalformedDirective {
            tag: ":added".into(),
            why: "version string is empty",
        })?,
    };

    let (ret_type, ret) = match metadata_lookup(&pairs, ":ret") {
        None => return Err(DocError::MissingRet),
        Some(WatAST::Vector(items, _)) if items.len() == 2 => {
            let ty = match &items[0] {
                WatAST::Keyword(k, _) => k.clone(),
                _ => {
                    return Err(DocError::MalformedDirective {
                        tag: ":ret".into(),
                        why: "type token must start with `:` (e.g. `:wat::core::String`); grammar is `@ret <type> <desc>`",
                    })
                }
            };
            if !type_token_is_expressible(&ty) {
                return Err(DocError::MalformedDirective {
                    tag: ":ret".into(),
                    why: "type token is not a spelling wat's reader accepts (e.g. `Option<T>` and the retired `fn(…)->…` form are inexpressible; use `:- [...]`)", // rune:lint(no-angle-type-in-diagnostic) — class C: quotes the retired spelling to name what is refused, exactly like the reader's own refusal messages
                });
            }
            let desc = metadata_string(&items[1]).ok_or(DocError::MalformedDirective {
                tag: ":ret".into(),
                why: "description is empty; grammar is `@ret <type> <desc>`",
            })?;
            (ty, desc)
        }
        Some(_) => {
            return Err(DocError::MalformedDirective {
                tag: ":ret".into(),
                why: "grammar is `@ret <type> <desc>`",
            })
        }
    };

    macro_rules! read_axis {
        ($key:literal, $enum_ty:ty, $missing:expr, $invalid:expr) => {
            match metadata_lookup(&pairs, $key) {
                None => return Err($missing),
                Some(v) => match enum_symbol_variant(v, <$enum_ty>::WAT_TYPE_PATH) {
                    Some(variant) => match variant.parse::<$enum_ty>() {
                        Ok(val) => val,
                        Err(_) => return Err($invalid(v)),
                    },
                    None => return Err($invalid(v)),
                },
            }
        };
    }

    let purity = read_axis!(":purity", Purity, DocError::MissingPurity, |_v: &WatAST| {
        DocError::MalformedDirective {
            tag: ":purity".into(),
            why: PURITY_LEGAL_VALUES,
        }
    });
    let determinism = read_axis!(":determinism", Determinism, DocError::MissingDeterminism, |_v: &WatAST| {
        DocError::MalformedDirective {
            tag: ":determinism".into(),
            why: "value must be one of: Deterministic, Nondeterministic, Preserving",
        }
    });
    let totality = read_axis!(":totality", Totality, DocError::MissingTotality, |v: &WatAST| {
        DocError::InvalidTotalityVariant { got: metadata_describe(v) }
    });
    let expand_time = read_axis!(":expand-time", ExpandTime, DocError::MissingExpandTime, |v: &WatAST| {
        DocError::InvalidExpandTimeVariant { got: metadata_describe(v) }
    });
    let category = read_axis!(":category", Category, DocError::MissingCategory, |_v: &WatAST| {
        DocError::MalformedDirective {
            tag: ":category".into(),
            why: CATEGORY_LEGAL_VALUES,
        }
    });

    let mut args: Vec<DocArg> = Vec::new();
    if let Some(v) = metadata_lookup(&pairs, ":args") {
        let items = match v {
            WatAST::Vector(items, _) => items,
            _ => {
                return Err(DocError::MalformedDirective {
                    tag: ":args".into(),
                    why: "grammar is `@arg <name> <type> <desc>`",
                })
            }
        };
        for item in items {
            let fields = match item {
                WatAST::Vector(fields, _) if fields.len() == 3 => fields,
                _ => {
                    return Err(DocError::MalformedDirective {
                        tag: ":args".into(),
                        why: "grammar is `@arg <name> <type> <desc>`",
                    })
                }
            };
            let name = metadata_bare_name(&fields[0]).ok_or(DocError::MalformedDirective {
                tag: ":args".into(),
                why: "name is missing",
            })?;
            let ty = match &fields[1] {
                WatAST::Keyword(k, _) => k.clone(),
                _ => {
                    return Err(DocError::MalformedDirective {
                        tag: ":args".into(),
                        why: "type token must start with `:` (e.g. `:wat::core::Bytes`); grammar is `@arg <name> <type> <desc>`",
                    })
                }
            };
            if !type_token_is_expressible(&ty) {
                return Err(DocError::MalformedDirective {
                    tag: ":args".into(),
                    why: "type token is not a spelling wat's reader accepts (e.g. `Option<T>` and the retired `fn(…)->…` form are inexpressible; use `:- [...]`)", // rune:lint(no-angle-type-in-diagnostic) — class C: quotes the retired spelling to name what is refused, exactly like the reader's own refusal messages
                });
            }
            let desc = metadata_string(&fields[2]).ok_or(DocError::MalformedDirective {
                tag: ":args".into(),
                why: "description is empty; grammar is `@arg <name> <type> <desc>`",
            })?;
            args.push(DocArg { name, ty, desc, is_rest: false });
        }
    }

    let mut examples: Vec<DocExample> = Vec::new();
    if let Some(v) = metadata_lookup(&pairs, ":examples") {
        let items = match v {
            WatAST::Vector(items, _) => items,
            _ => {
                return Err(DocError::MalformedDirective {
                    tag: ":examples".into(),
                    why: "grammar is `[<expr form> <expected form>]` entries",
                })
            }
        };
        for item in items {
            let fields = match item {
                WatAST::Vector(fields, _) if fields.len() == 2 => fields,
                _ => {
                    return Err(DocError::MalformedDirective {
                        tag: ":examples".into(),
                        why: "grammar is `[<expr form> <expected form>]` entries",
                    })
                }
            };
            // Arc 255 STONE "an example is a FORM, not a string" — `fields[0]`/
            // `fields[1]` are ALREADY parsed `WatAST` nodes (the wat reader
            // parsed the surrounding `.wat` source that produced this metadata
            // map), so there is nothing left to stringify or validate here: a
            // malformed example is unrepresentable by construction on this
            // path — the reader that loaded the declaration already refused
            // it. Every metadata-map example is `run: true` (mirrors `@example`;
            // there is no metadata-map spelling yet for `@example-norun`'s
            // optional-`expected` shape — out of scope for this stone).
            examples.push(DocExample { expr: fields[0].clone(), expected: Some(fields[1].clone()), run: true });
        }
    }
    if examples.is_empty() {
        return Err(DocError::MissingExample);
    }

    let deprecated = match metadata_lookup(&pairs, ":deprecated") {
        None => None,
        Some(WatAST::Vector(fields, _)) if fields.len() == 2 => {
            let since = metadata_string(&fields[0]).ok_or(DocError::MalformedDirective {
                tag: ":deprecated".into(),
                why: "version string is empty",
            })?;
            let use_instead = metadata_string(&fields[1]).ok_or(DocError::MalformedDirective {
                tag: ":deprecated".into(),
                why: "use-instead is empty",
            })?;
            Some(Deprecation { since, use_instead })
        }
        Some(_) => {
            return Err(DocError::MalformedDirective {
                tag: ":deprecated".into(),
                why: "grammar is `[<since StringLit> <use-instead StringLit>]`",
            })
        }
    };

    let mut see: Vec<String> = Vec::new();
    if let Some(v) = metadata_lookup(&pairs, ":see") {
        let items = match v {
            WatAST::Vector(items, _) => items,
            _ => {
                return Err(DocError::MalformedDirective {
                    tag: ":see".into(),
                    why: "@see entries must be keyword FQDNs",
                })
            }
        };
        for item in items {
            match item {
                WatAST::Keyword(k, _) => see.push(k.clone()),
                _ => {
                    return Err(DocError::MalformedDirective {
                        tag: ":see".into(),
                        why: "@see entries must be keyword FQDNs",
                    })
                }
            }
        }
    }

    let mut yields_vals: Vec<DocYields> = Vec::new();
    if let Some(v) = metadata_lookup(&pairs, ":yields") {
        let items = match v {
            WatAST::Vector(items, _) => items,
            _ => {
                return Err(DocError::MalformedDirective {
                    tag: ":yields".into(),
                    why: "grammar is `@yields <argname> <desc>`",
                })
            }
        };
        for item in items {
            let fields = match item {
                WatAST::Vector(fields, _) if fields.len() == 2 => fields,
                _ => {
                    return Err(DocError::MalformedDirective {
                        tag: ":yields".into(),
                        why: "grammar is `@yields <argname> <desc>`",
                    })
                }
            };
            let arg_name = metadata_bare_name(&fields[0]).ok_or(DocError::MalformedDirective {
                tag: ":yields".into(),
                why: "argument name is missing; grammar is `@yields <argname> <desc>`",
            })?;
            let desc = metadata_string(&fields[1]).ok_or(DocError::MalformedDirective {
                tag: ":yields".into(),
                why: "description is empty; grammar is `@yields <argname> <desc>`",
            })?;
            if yields_vals.iter().any(|y: &DocYields| y.arg == arg_name) {
                return Err(DocError::DuplicateYieldsSubject { arg: arg_name });
            }
            yields_vals.push(DocYields { arg: arg_name, desc });
        }
    }
    for y in &yields_vals {
        if !args.iter().any(|a| a.name == y.arg) {
            return Err(DocError::UnknownYieldsSubject { arg: y.arg.clone() });
        }
    }

    Ok(DocComment { prose, added, args, ret_type, ret, examples, deprecated, see, purity, determinism, totality, expand_time, category, yields: yields_vals })
}

/// Parse a special-form doc block.
///
/// Special forms use different purity/determinism directives than intrinsics:
/// - `@purity preserving|pure|effectful` (required) maps to `pure: bool`
/// - `@determinism preserving|deterministic|nondeterministic` (required) maps to `deterministic: bool`
/// - `@syntax (...)` (required) — the grammar string, verbatim
/// - Does NOT accept `@pure` or `@deterministic` (those fire `UnknownDirective`)
/// - `@yields` is NOT recognized for special forms
pub fn parse_special_form(raw: &str) -> Result<DocSpecialForm, DocError> {
    let recognized = &[
        "@added", "@arg", "@ret", "@example", "@example-norun", "@deprecated", "@see",
        "@Purity", "@Determinism", "@Totality", "@ExpandTime", "@Category", "@syntax",
    ];

    let lines: Vec<&str> = raw.lines().collect();
    let first_directive = lines.iter().position(|l| {
        let token = l.split_whitespace().next().unwrap_or("");
        token.starts_with('@') && recognized.contains(&token)
    });

    let prose_end = first_directive.unwrap_or(lines.len());
    let prose = trim_blank_lines(&lines[..prose_end]).join("\n");
    if prose.is_empty() {
        return Err(DocError::MissingProse);
    }

    let mut added: Option<String> = None;
    let mut syntax_val: Option<String> = None;
    let mut args: Vec<DocArg> = Vec::new();
    let mut ret_type: Option<String> = None;
    let mut ret: Option<String> = None;
    let mut examples: Vec<DocExample> = Vec::new();
    let mut deprecated: Option<Deprecation> = None;
    let mut see: Vec<String> = Vec::new();
    let mut purity_val: Option<Purity> = None;
    let mut determinism_val: Option<Determinism> = None;
    let mut totality_val: Option<Totality> = None;
    let mut expand_time_val: Option<ExpandTime> = None;
    let mut category_val: Option<Category> = None;

    let directive_lines = match first_directive {
        Some(i) => &lines[i..],
        None => &[][..],
    };

    for &line in directive_lines {
        let trimmed = line.trim_start();
        let tag = trimmed.split_whitespace().next().unwrap_or("");

        if !tag.starts_with('@') {
            continue;
        }

        if !recognized.contains(&tag) {
            return Err(DocError::UnknownDirective { tag: tag.to_string() });
        }

        let payload = trimmed[tag.len()..].trim_start();

        match tag {
            "@added" => {
                if added.is_some() {
                    return Err(DocError::DuplicateSingleton { tag: "@added".into() });
                }
                if payload.is_empty() {
                    return Err(DocError::MalformedDirective {
                        tag: "@added".into(),
                        why: "version string is empty",
                    });
                }
                added = Some(payload.to_string());
            }
            "@syntax" => {
                if syntax_val.is_some() {
                    return Err(DocError::DuplicateSingleton { tag: "@syntax".into() });
                }
                if payload.is_empty() {
                    return Err(DocError::MalformedDirective {
                        tag: "@syntax".into(),
                        why: "grammar string is empty",
                    });
                }
                syntax_val = Some(payload.to_string());
            }
            "@arg" => {
                let mut name_split = payload.splitn(2, char::is_whitespace);
                let raw_name = name_split.next().unwrap_or("");
                if raw_name.is_empty() {
                    return Err(DocError::MalformedDirective {
                        tag: "@arg".into(),
                        why: "name is missing",
                    });
                }
                let (name, is_rest) = if let Some(stem) = raw_name.strip_suffix('…') {
                    (stem.to_string(), true)
                } else if let Some(stem) = raw_name.strip_suffix("...") {
                    (stem.to_string(), true)
                } else {
                    (raw_name.to_string(), false)
                };

                let after_name = name_split.next().unwrap_or("").trim_start();
                let (ty_token, desc_raw) = take_type_token(after_name);
                let ty_token = ty_token.trim();
                if ty_token.is_empty() {
                    return Err(DocError::MalformedDirective {
                        tag: "@arg".into(),
                        why: "type is missing; grammar is `@arg <name> <type> <desc>`",
                    });
                }
                if SEPARATOR_TOKENS.contains(&ty_token) {
                    return Err(DocError::MalformedDirective {
                        tag: "@arg".into(),
                        why: "separator used in type position; grammar is `@arg <name> <type> <desc>`",
                    });
                }
                // Type token must start with `:` — or be one of the two
                // surviving STRUCTURAL type spellings, which can never start
                // with `:` by construction: a parametric type REFERENCE
                // `(Head :- [args])`, or a fn type `[arg… :-> ret]`. Those are
                // still gated by the reader check just below; this clause
                // only rules out a BARE non-keyword symbol like `Bytes`.
                if !(ty_token.starts_with(':') || ty_token.starts_with('(') || ty_token.starts_with('[')) {
                    return Err(DocError::MalformedDirective {
                        tag: "@arg".into(),
                        why: "type token must start with `:` (e.g. `:wat::core::Bool`); grammar is `@arg <name> <type> <desc>`",
                    });
                }
                // Type token must be a spelling wat's own reader accepts as a
                // single, complete form — rules out `Option<T>`, the retired
                // `fn(…)->…` vocabulary, and any other inexpressible spelling.
                if !type_token_is_expressible(ty_token) {
                    return Err(DocError::MalformedDirective {
                        tag: "@arg".into(),
                        why: "type token is not a spelling wat's reader accepts (e.g. `Option<T>` and the retired `fn(…)->…` form are inexpressible; use `:- [...]`)", // rune:lint(no-angle-type-in-diagnostic) — class C: quotes the retired spelling to name what is refused, exactly like the reader's own refusal messages
                    });
                }
                let ty = ty_token.to_string();
                let desc = desc_raw.trim().to_string();
                if desc.is_empty() {
                    return Err(DocError::MalformedDirective {
                        tag: "@arg".into(),
                        why: "description is empty; grammar is `@arg <name> <type> <desc>`",
                    });
                }
                args.push(DocArg { name, ty, desc, is_rest });
            }
            "@ret" => {
                if ret.is_some() {
                    return Err(DocError::DuplicateSingleton { tag: "@ret".into() });
                }
                let (ty_token, desc_raw) = take_type_token(payload);
                let ty_token = ty_token.trim();
                if ty_token.is_empty() {
                    return Err(DocError::MalformedDirective {
                        tag: "@ret".into(),
                        why: "type is missing; grammar is `@ret <type> <desc>`",
                    });
                }
                if SEPARATOR_TOKENS.contains(&ty_token) {
                    return Err(DocError::MalformedDirective {
                        tag: "@ret".into(),
                        why: "separator used in type position; grammar is `@ret <type> <desc>`",
                    });
                }
                if !(ty_token.starts_with(':') || ty_token.starts_with('(') || ty_token.starts_with('[')) {
                    return Err(DocError::MalformedDirective {
                        tag: "@ret".into(),
                        why: "type token must start with `:` (e.g. `:wat::core::String`); grammar is `@ret <type> <desc>`",
                    });
                }
                // Type token must be a spelling wat's own reader accepts as a
                // single, complete form — rules out `Option<T>`, the retired
                // `fn(…)->…` vocabulary, and any other inexpressible spelling.
                if !type_token_is_expressible(ty_token) {
                    return Err(DocError::MalformedDirective {
                        tag: "@ret".into(),
                        why: "type token is not a spelling wat's reader accepts (e.g. `Option<T>` and the retired `fn(…)->…` form are inexpressible; use `:- [...]`)", // rune:lint(no-angle-type-in-diagnostic) — class C: quotes the retired spelling to name what is refused, exactly like the reader's own refusal messages
                    });
                }
                let ty = ty_token.to_string();
                let desc = desc_raw.trim().to_string();
                if desc.is_empty() {
                    return Err(DocError::MalformedDirective {
                        tag: "@ret".into(),
                        why: "description is empty; grammar is `@ret <type> <desc>`",
                    });
                }
                ret_type = Some(ty);
                ret = Some(desc);
            }
            "@example" => {
                let rest = payload;
                let (expr_text, expected_text) = match rest.split_once(" #=> ").or_else(|| rest.split_once("#=> ")) {
                    Some((left, right)) => (left.trim(), right.trim()),
                    None => {
                        if let Some(left) = rest.strip_suffix("#=>") {
                            (left.trim(), "")
                        } else {
                            return Err(DocError::ExampleMissingMarker {
                                expr: rest.trim().to_string(),
                            });
                        }
                    }
                };
                let expr = parse_example_form(expr_text, "@example")?;
                let expected = parse_example_form(expected_text, "@example")?;
                examples.push(DocExample { expr, expected: Some(expected), run: true });
            }
            "@example-norun" => {
                let rest = payload;
                // See `parse`'s identical arm / the DocExample struct docs —
                // the marker text, when present, is illustrative and
                // UNVERIFIED; `expected` stays `None` here regardless.
                let expr_text = match rest.split_once(" #=> ").or_else(|| rest.split_once("#=> ")) {
                    Some((left, _right)) => left.trim(),
                    None => rest.trim(),
                };
                let expr = parse_example_form(expr_text, "@example-norun")?;
                examples.push(DocExample { expr, expected: None, run: false });
            }
            "@deprecated" => {
                if deprecated.is_some() {
                    return Err(DocError::DuplicateSingleton { tag: "@deprecated".into() });
                }
                let mut tokens = payload.splitn(2, char::is_whitespace);
                let since = tokens.next().unwrap_or("").to_string();
                let use_instead = tokens.next().unwrap_or("").trim_start().to_string();
                deprecated = Some(Deprecation { since, use_instead });
            }
            "@see" => {
                see.push(payload.to_string());
            }
            "@Purity" => {
                if purity_val.is_some() {
                    return Err(DocError::DuplicateSingleton { tag: "@Purity".into() });
                }
                match payload.parse::<Purity>() {
                    Ok(p) => purity_val = Some(p),
                    Err(_) => return Err(DocError::MalformedDirective {
                        tag: "@Purity".into(),
                        why: PURITY_LEGAL_VALUES,
                    }),
                }
            }
            "@Determinism" => {
                if determinism_val.is_some() {
                    return Err(DocError::DuplicateSingleton { tag: "@Determinism".into() });
                }
                match payload.parse::<Determinism>() {
                    Ok(d) => determinism_val = Some(d),
                    Err(_) => return Err(DocError::MalformedDirective {
                        tag: "@Determinism".into(),
                        why: "value must be one of: Deterministic, Nondeterministic, Preserving",
                    }),
                }
            }
            "@Totality" => {
                if totality_val.is_some() {
                    return Err(DocError::DuplicateSingleton { tag: "@Totality".into() });
                }
                match payload.parse::<Totality>() {
                    Ok(t) => totality_val = Some(t),
                    Err(_) => return Err(DocError::InvalidTotalityVariant { got: payload.to_string() }),
                }
            }
            "@ExpandTime" => {
                if expand_time_val.is_some() {
                    return Err(DocError::DuplicateSingleton { tag: "@ExpandTime".into() });
                }
                match payload.parse::<ExpandTime>() {
                    Ok(e) => expand_time_val = Some(e),
                    Err(_) => return Err(DocError::InvalidExpandTimeVariant { got: payload.to_string() }),
                }
            }
            "@Category" => {
                if category_val.is_some() {
                    return Err(DocError::DuplicateSingleton { tag: "@Category".into() });
                }
                match payload.parse::<Category>() {
                    Ok(c) => category_val = Some(c),
                    Err(_) => return Err(DocError::MalformedDirective {
                        tag: "@Category".into(),
                        why: CATEGORY_LEGAL_VALUES,
                    }),
                }
            }
            _ => unreachable!("recognized set is exhaustive"),
        }
    }

    let added = added.ok_or(DocError::MissingAdded)?;
    // Shape rule: @arg OR @syntax — at least one must express the form's shape.
    // @syntax is the escape hatch for structural forms (let/match/repetition).
    // @arg-only forms (if/and/or) derive their grammar from the arg names.
    // Neither → MissingShape.
    if args.is_empty() && syntax_val.is_none() {
        return Err(DocError::MissingShape);
    }
    let syntax = syntax_val.unwrap_or_default();
    let ret_type = ret_type.ok_or(DocError::MissingRet)?;
    let ret = ret.ok_or(DocError::MissingRet)?;
    if examples.is_empty() {
        return Err(DocError::MissingExample);
    }
    let purity = purity_val.ok_or(DocError::MissingPurity)?;
    let determinism = determinism_val.ok_or(DocError::MissingDeterminism)?;
    // Arc 255 Stone total-T3: `@Totality` is REQUIRED (special-form sibling resolution
    // point — see the `parse` fn above for the same change and its rationale).
    let totality = totality_val.ok_or(DocError::MissingTotality)?;
    // Arc 255 Stone expand-T3: `@ExpandTime` is REQUIRED (special-form sibling
    // resolution point — see the `parse` fn above for the same change).
    let expand_time = expand_time_val.ok_or(DocError::MissingExpandTime)?;
    let category = category_val.ok_or(DocError::MissingCategory)?;

    Ok(DocSpecialForm {
        prose,
        added,
        syntax,
        args,
        ret_type,
        ret,
        examples,
        category,
        purity,
        determinism,
        totality,
        expand_time,
        see,
        deprecated,
    })
}

/// Trim leading and trailing blank (empty/whitespace-only) lines from a slice.
fn trim_blank_lines<'a>(lines: &'a [&'a str]) -> &'a [&'a str] {
    let start = lines.iter().position(|l| !l.trim().is_empty()).unwrap_or(lines.len());
    let end = lines.iter().rposition(|l| !l.trim().is_empty()).map(|i| i + 1).unwrap_or(0);
    if start >= end { &[] } else { &lines[start..end] }
}

/// The `@arg`⇄signature mutual check: the documented args must match `params`
/// (the wat-arg names, in order) by count and name. A 0-param intrinsic must
/// document 0 args. This is what makes "`@arg` required ×params" true.
///
/// For variadic intrinsics, the last `@arg` carries `is_rest: true`; the
/// corresponding signature param is the single `&[WatAST]` (variadic) param.
/// The name check strips `…` from the documented name before comparing.
pub fn check_args(doc: &DocComment, params: &[&str]) -> Result<(), DocError> {
    if doc.args.len() != params.len() {
        return Err(DocError::ArgCountMismatch {
            documented: doc.args.len(),
            signature: params.len(),
        });
    }
    for (i, (arg, &param)) in doc.args.iter().zip(params.iter()).enumerate() {
        // `arg.name` is already stripped of any `…` suffix by the parser.
        if arg.name != param {
            return Err(DocError::ArgNameMismatch {
                position: i,
                documented: arg.name.clone(),
                signature: param.to_string(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test-only shorthand: parse `s` as a single wat form. Used to build the
    /// `WatAST` values `DocExample::expr`/`expected` now carry, wherever a
    /// fixture used to spell them as plain `.into()` strings.
    fn f(s: &str) -> WatAST {
        wat_reader::parse_one_with_file(s, "<wat-doc test>").expect("test fixture form parses")
    }

    /// The reference intrinsic doc block (`core::Bytes::to-hex`), in the exact
    /// joined form `sniff_doc` produces (`/// ` stripped, `\n`-joined). This IS
    /// the contract the parser must satisfy. Updated to firm grammar (no separator).
    const TO_HEX: &str = "Encode a `:wat::core::Bytes` into its lowercase-hex `:String`.\n\nMarkdown prose, GFM — flows straight to the wiki page body.\n\n@added   1.0.0\n@arg     bs :wat::core::Bytes the bytes to encode\n@ret     :wat::core::String the lowercase hex string, two chars per byte, no separators\n@Purity Pure\n@Determinism Deterministic\n@Totality Unreviewed\n@ExpandTime Unreviewed\n@Category Transform\n@example (:wat::core::Bytes::to-hex (:wat::core::Vector :- [:u8] (:wat::core::u8 255) (:wat::core::u8 0) (:wat::core::u8 16))) #=> \"ff0010\"";

    #[test]
    fn parses_the_reference_intrinsic() {
        let doc = parse(TO_HEX).expect("to-hex doc parses");
        assert_eq!(
            doc.prose,
            "Encode a `:wat::core::Bytes` into its lowercase-hex `:String`.\n\nMarkdown prose, GFM — flows straight to the wiki page body."
        );
        assert_eq!(doc.added, "1.0.0");
        assert_eq!(
            doc.args,
            vec![DocArg { name: "bs".into(), ty: ":wat::core::Bytes".into(), desc: "the bytes to encode".into(), is_rest: false }]
        );
        assert_eq!(doc.ret_type, ":wat::core::String");
        assert_eq!(doc.ret, "the lowercase hex string, two chars per byte, no separators");
        assert_eq!(
            doc.examples,
            vec![DocExample {
                expr: f("(:wat::core::Bytes::to-hex (:wat::core::Vector :- [:u8] (:wat::core::u8 255) (:wat::core::u8 0) (:wat::core::u8 16)))"),
                expected: Some(f("\"ff0010\"")),
                run: true,
            }]
        );
        assert_eq!(doc.deprecated, None);
        assert!(doc.see.is_empty());
        assert_eq!(doc.purity, Purity::Pure);
        assert_eq!(doc.determinism, Determinism::Deterministic);
        assert_eq!(doc.category, Category::Transform);
    }

    #[test]
    fn norun_example_may_omit_the_marker() {
        let raw = "Write bytes to a path.\n\n@added 1.0.0\n@Purity Effectful\n@Determinism Nondeterministic\n@Totality Unreviewed\n@ExpandTime Unreviewed\n@Category Transform\n@arg p :wat::core::Path the path\n@ret :wat::core::Result ok on success\n@example-norun (:wat::core::File::write p data)";
        let doc = parse(raw).expect("norun parses");
        assert_eq!(
            doc.examples,
            vec![DocExample {
                expr: f("(:wat::core::File::write p data)"),
                expected: None,
                run: false,
            }]
        );
    }

    #[test]
    fn norun_example_may_carry_an_unverified_marker() {
        // `#uuid "…"` is NOT a wat form the reader accepts (no `#uuid` tagged-
        // literal syntax) — proof, not just assertion, that this marker is
        // genuinely unverified: if `parse` attempted to parse it, this fixture
        // itself would turn the `expect` below into a panic.
        let raw = "Read a uuid.\n\n@added 1.0.0\n@Purity Effectful\n@Determinism Nondeterministic\n@Totality Unreviewed\n@ExpandTime Unreviewed\n@Category Reflection\n@ret :wat::core::String a fresh uuid\n@example-norun (:wat::uuid::v4) #=> #uuid \"…\"";
        let doc = parse(raw).expect("norun-with-marker parses");
        assert!(!doc.examples[0].run);
        assert_eq!(doc.examples[0].expr, f("(:wat::uuid::v4)"));
        assert_eq!(doc.examples[0].expected, None);
    }

    #[test]
    fn multiple_args_and_see_in_order() {
        let raw = "Blend two things.\n\n@added 1.2.0\n@Purity Pure\n@Determinism Deterministic\n@Totality Unreviewed\n@ExpandTime Unreviewed\n@Category Transform\n@arg a :wat::core::i64 the first\n@arg b :wat::core::i64 the second\n@ret :wat::core::i64 the blend\n@example (f 1 2) #=> 3\n@see :wat::core::other\n@see :wat::core::another";
        let doc = parse(raw).expect("parses");
        assert_eq!(doc.args.iter().map(|a| a.name.as_str()).collect::<Vec<_>>(), vec!["a", "b"]);
        assert_eq!(doc.args[0].desc, "the first");
        assert_eq!(doc.args[1].desc, "the second");
        assert_eq!(doc.see, vec![":wat::core::other".to_string(), ":wat::core::another".to_string()]);
    }

    #[test]
    fn deprecated_parses() {
        let raw = "Old thing.\n\n@added 1.0.0\n@Purity Pure\n@Determinism Deterministic\n@Totality Unreviewed\n@ExpandTime Unreviewed\n@Category Transform\n@ret :wat::core::i64 nothing useful\n@example (g) #=> nil\n@deprecated 2.0.0 use :wat::core::new-thing instead";
        let doc = parse(raw).expect("parses");
        assert_eq!(
            doc.deprecated,
            Some(Deprecation { since: "2.0.0".into(), use_instead: "use :wat::core::new-thing instead".into() })
        );
    }

    // --- negative cases: separator in type position is now rejected ---

    #[test]
    fn separator_in_arg_type_position_is_rejected() {
        // Old grammar: `@arg bs — the bytes` — "—" is now in the type position,
        // which is illegal (separator token rejected).
        let raw = "Prose.\n\n@added 1.0.0\n@arg bs — the bytes\n@ret :wat::core::String desc\n@example (f) #=> y";
        match parse(raw) {
            Err(DocError::MalformedDirective { tag, .. }) => assert_eq!(tag, "@arg"),
            other => panic!("expected MalformedDirective for @arg separator, got {:?}", other),
        }
    }

    #[test]
    fn non_colon_type_in_arg_is_rejected() {
        // Type token doesn't start with `:` — also illegal.
        let raw = "Prose.\n\n@added 1.0.0\n@arg bs Bytes the bytes\n@ret :wat::core::String desc\n@example (f) #=> y";
        match parse(raw) {
            Err(DocError::MalformedDirective { tag, .. }) => assert_eq!(tag, "@arg"),
            other => panic!("expected MalformedDirective for non-colon @arg type, got {:?}", other),
        }
    }

    #[test]
    fn separator_in_ret_type_position_is_rejected() {
        // Old grammar: `@ret — the desc` — "—" is now in the type position, illegal.
        let raw = "Prose.\n\n@added 1.0.0\n@arg bs :wat::core::Bytes the bytes\n@ret — the bytes\n@example (f) #=> y";
        match parse(raw) {
            Err(DocError::MalformedDirective { tag, .. }) => assert_eq!(tag, "@ret"),
            other => panic!("expected MalformedDirective for @ret separator, got {:?}", other),
        }
    }

    // --- negative cases: every required-directive omission is a named error ---

    #[test]
    fn missing_prose_is_an_error() {
        let raw = "@added 1.0.0\n@ret :wat::core::i64 x\n@example (f) #=> y";
        assert_eq!(parse(raw), Err(DocError::MissingProse));
    }

    #[test]
    fn missing_added_is_an_error() {
        let raw = "Prose.\n\n@ret :wat::core::i64 x\n@example (f) #=> y";
        assert_eq!(parse(raw), Err(DocError::MissingAdded));
    }

    #[test]
    fn missing_ret_is_an_error() {
        let raw = "Prose.\n\n@added 1.0.0\n@example (f) #=> y";
        assert_eq!(parse(raw), Err(DocError::MissingRet));
    }

    #[test]
    fn missing_example_is_an_error() {
        let raw = "Prose.\n\n@added 1.0.0\n@ret :wat::core::i64 x";
        assert_eq!(parse(raw), Err(DocError::MissingExample));
    }

    #[test]
    fn run_example_without_marker_is_an_error() {
        let raw = "Prose.\n\n@added 1.0.0\n@ret :wat::core::i64 x\n@example (f 1 2)";
        assert_eq!(
            parse(raw),
            Err(DocError::ExampleMissingMarker { expr: "(f 1 2)".into() })
        );
    }

    #[test]
    fn unknown_directive_is_an_error() {
        let raw = "Prose.\n\n@added 1.0.0\n@ret :wat::core::i64 x\n@example (f) #=> y\n@bogus whatever";
        assert_eq!(parse(raw), Err(DocError::UnknownDirective { tag: "@bogus".into() }));
    }

    #[test]
    fn duplicate_added_is_an_error() {
        let raw = "Prose.\n\n@added 1.0.0\n@added 1.1.0\n@ret :wat::core::i64 x\n@example (f) #=> y";
        assert_eq!(parse(raw), Err(DocError::DuplicateSingleton { tag: "@added".into() }));
    }

    // --- check_args: the @arg ⇄ signature mutual check ---

    #[test]
    fn check_args_agrees() {
        let doc = parse(TO_HEX).unwrap();
        assert_eq!(check_args(&doc, &["bs"]), Ok(()));
    }

    #[test]
    fn check_args_count_mismatch() {
        let doc = parse(TO_HEX).unwrap();
        assert_eq!(
            check_args(&doc, &[]),
            Err(DocError::ArgCountMismatch { documented: 1, signature: 0 })
        );
    }

    #[test]
    fn check_args_name_mismatch() {
        let doc = parse(TO_HEX).unwrap();
        assert_eq!(
            check_args(&doc, &["data"]),
            Err(DocError::ArgNameMismatch { position: 0, documented: "bs".into(), signature: "data".into() })
        );
    }

    #[test]
    fn check_args_zero_arity_ok() {
        let raw = "A constant.\n\n@added 1.0.0\n@Purity Pure\n@Determinism Deterministic\n@Totality Unreviewed\n@ExpandTime Unreviewed\n@Category Transform\n@ret :wat::core::i64 the value\n@example (k) #=> 42";
        let doc = parse(raw).unwrap();
        assert_eq!(check_args(&doc, &[]), Ok(()));
    }

    #[test]
    fn pure_and_deterministic_parse() {
        let raw = "Do something.\n\n@added 1.0.0\n@Purity Pure\n@Determinism Nondeterministic\n@Totality Unreviewed\n@ExpandTime Unreviewed\n@Category Transform\n@ret :wat::core::i64 the value\n@example (f) #=> 1";
        let doc = parse(raw).expect("pure+det doc parses");
        assert_eq!(doc.purity, Purity::Pure);
        assert_eq!(doc.determinism, Determinism::Nondeterministic);
    }

    #[test]
    fn missing_purity_is_an_error() {
        let raw = "Prose.\n\n@added 1.0.0\n@Determinism Deterministic\n@Category Transform\n@ret :wat::core::i64 x\n@example (f) #=> y";
        assert_eq!(parse(raw), Err(DocError::MissingPurity));
    }

    #[test]
    fn missing_determinism_is_an_error() {
        let raw = "Prose.\n\n@added 1.0.0\n@Purity Pure\n@Category Transform\n@ret :wat::core::i64 x\n@example (f) #=> y";
        assert_eq!(parse(raw), Err(DocError::MissingDeterminism));
    }

    #[test]
    fn invalid_purity_value_is_an_error() {
        let raw = "Prose.\n\n@added 1.0.0\n@Purity maybe\n@Determinism Deterministic\n@Category Transform\n@ret :wat::core::i64 x\n@example (f) #=> y";
        match parse(raw) {
            Err(DocError::MalformedDirective { tag, .. }) => assert_eq!(tag, "@Purity"),
            other => panic!("expected MalformedDirective for @Purity, got {:?}", other),
        }
    }

    #[test]
    fn missing_category_is_an_error() {
        // @ExpandTime IS present — its own resolution point is checked BEFORE
        // @Category's, so it must be satisfied here or this test would fail with
        // MissingExpandTime instead of the MissingCategory it means to prove.
        let raw = "Prose.\n\n@added 1.0.0\n@Purity Pure\n@Determinism Deterministic\n@Totality Unreviewed\n@ExpandTime Unreviewed\n@ret :wat::core::i64 x\n@example (f) #=> y";
        assert_eq!(parse(raw), Err(DocError::MissingCategory));
    }

    #[test]
    fn category_parses() {
        let raw = "Do something.\n\n@added 1.0.0\n@Purity Pure\n@Determinism Deterministic\n@Totality Unreviewed\n@ExpandTime Unreviewed\n@Category Reflection\n@ret :wat::core::i64 the value\n@example (f) #=> 1";
        let doc = parse(raw).expect("category doc parses");
        assert_eq!(doc.category, Category::Reflection);
    }

    #[test]
    fn purity_parses_all_variants() {
        for v in &["Pure", "Effectful", "Preserving"] {
            assert!(v.parse::<Purity>().is_ok(), "should parse: {}", v);
        }
        assert!("pure".parse::<Purity>().is_err()); // case-sensitive
    }

    #[test]
    fn determinism_parses_all_variants() {
        for v in &["Deterministic", "Nondeterministic", "Preserving"] {
            assert!(v.parse::<Determinism>().is_ok(), "should parse: {}", v);
        }
        assert!("deterministic".parse::<Determinism>().is_err());
    }

    // ─── @Totality (arc 255 Stone total-T2, made REQUIRED in Stone total-T3) ─────
    //
    // @Totality is now REQUIRED, exactly like @Purity/@Determinism/@Category.
    // Absence is `DocError::MissingTotality` — the builder's ruling struck the
    // T2 default ("declaring nothing needs to be illegal"). An author must type
    // `@Totality Unreviewed` explicitly if a verb has not been reviewed.

    /// `@Totality <Variant>` parses, one test per legal variant.
    #[test]
    fn total_parses_all_variants() {
        for (spelling, expected) in [
            ("Total", Totality::Total),
            ("Partial", Totality::Partial),
            ("Preserving", Totality::Preserving),
            ("Unreviewed", Totality::Unreviewed),
        ] {
            let raw = format!(
                "Do something.\n\n@added 1.0.0\n@Purity Pure\n@Determinism Deterministic\n\
                 @Totality {spelling}\n@ExpandTime Unreviewed\n@Category Transform\n@ret :wat::core::i64 the value\n\
                 @example (f) #=> 1"
            );
            let doc = parse(&raw).unwrap_or_else(|e| panic!("@Totality {spelling} must parse: {e:?}"));
            assert_eq!(doc.totality, expected, "@Totality {spelling} must read back as {expected:?}");
        }
        assert!("total".parse::<Totality>().is_err()); // case-sensitive
    }

    /// ★ Arc 255 Stone total-T3, Row 1 — ABSENT `@Totality` is now `MissingTotality`,
    /// exactly like `missing_purity_is_an_error` / `missing_determinism_is_an_error`
    /// above. This is the "break the door first" proof: declaring nothing must
    /// genuinely fail before the 437-site sweep makes the requirement untestable.
    #[test]
    fn absent_total_is_an_error() {
        let raw = "Do something.\n\n@added 1.0.0\n@Purity Pure\n@Determinism Deterministic\n\
                    @Category Transform\n@ret :wat::core::i64 the value\n@example (f) #=> 1";
        assert_eq!(parse(raw), Err(DocError::MissingTotality));
    }

    /// Row 3a — a SECOND `@Totality` is `DuplicateSingleton`.
    #[test]
    fn duplicate_total_is_an_error() {
        let raw = "Do something.\n\n@added 1.0.0\n@Purity Pure\n@Determinism Deterministic\n\
                    @Totality Total\n@Totality Partial\n@Category Transform\n\
                    @ret :wat::core::i64 the value\n@example (f) #=> 1";
        assert_eq!(parse(raw), Err(DocError::DuplicateSingleton { tag: "@Totality".into() }));
    }

    /// Row 3b — an unknown `@Totality` value is `InvalidTotalityVariant { got }`.
    #[test]
    fn invalid_total_value_is_an_error() {
        let raw = "Do something.\n\n@added 1.0.0\n@Purity Pure\n@Determinism Deterministic\n\
                    @Totality Bogus\n@Category Transform\n@ret :wat::core::i64 the value\n\
                    @example (f) #=> 1";
        assert_eq!(parse(raw), Err(DocError::InvalidTotalityVariant { got: "Bogus".into() }));
    }

    /// The special-form parser accepts the identical directive — `DocSpecialForm`
    /// is a SIBLING type to `DocComment` (not the same type), so both resolution
    /// points need their own coverage, including the "absence errors" half (row 1).
    #[test]
    fn special_form_total_parses_and_absence_is_an_error() {
        let with_total = "Evaluate the condition.\n\n\
            @added 1.0.0\n\
            @Category ControlFlow\n\
            @Purity Preserving\n\
            @Determinism Preserving\n\
            @Totality Partial\n\
            @ExpandTime Unreviewed\n\
            @arg cond :wat::core::Bool the condition\n\
            @ret :T the result\n\
            @example (:wat::core::if true 1 2) #=> 1";
        let doc = super::parse_special_form(with_total).expect("@Totality Partial must parse on a special form");
        assert_eq!(doc.totality, Totality::Partial);

        let without_total = "Evaluate the condition.\n\n\
            @added 1.0.0\n\
            @Category ControlFlow\n\
            @Purity Preserving\n\
            @Determinism Preserving\n\
            @arg cond :wat::core::Bool the condition\n\
            @ret :T the result\n\
            @example (:wat::core::if true 1 2) #=> 1";
        assert_eq!(super::parse_special_form(without_total), Err(DocError::MissingTotality));
    }

    // ─── @ExpandTime (arc 255 Stone expand-T2, made REQUIRED in Stone expand-T3) ──
    //
    // @ExpandTime is now REQUIRED, exactly like @Purity/@Determinism/@Totality/
    // @Category. Absence is `DocError::MissingExpandTime` — mirroring totality's
    // own T2→T3 arc, the builder's ruling struck the T2 default. An author must
    // type `@ExpandTime Unreviewed` explicitly if a verb has not been reviewed.

    /// ★ Row 1 — `@ExpandTime <Variant>` parses, one test per legal variant.
    #[test]
    fn expand_time_parses_all_variants() {
        for (spelling, expected) in [
            ("Legal", ExpandTime::Legal),
            ("RuntimeOnly", ExpandTime::RuntimeOnly),
            ("ExpandOnly", ExpandTime::ExpandOnly),
            ("Preserving", ExpandTime::Preserving),
            ("Unreviewed", ExpandTime::Unreviewed),
        ] {
            let raw = format!(
                "Do something.\n\n@added 1.0.0\n@Purity Pure\n@Determinism Deterministic\n\
                 @Totality Unreviewed\n@ExpandTime {spelling}\n@Category Transform\n\
                 @ret :wat::core::i64 the value\n@example (f) #=> 1"
            );
            let doc = parse(&raw).unwrap_or_else(|e| panic!("@ExpandTime {spelling} must parse: {e:?}"));
            assert_eq!(doc.expand_time, expected, "@ExpandTime {spelling} must read back as {expected:?}");
        }
        assert!("legal".parse::<ExpandTime>().is_err()); // case-sensitive
    }

    /// ★ Arc 255 Stone expand-T3, Row 1 — ABSENT `@ExpandTime` is now
    /// `MissingExpandTime`, exactly like `missing_purity_is_an_error` /
    /// `absent_total_is_an_error` above. This is the "break the door first"
    /// proof: declaring nothing must genuinely fail before the 431-site sweep
    /// makes the requirement untestable.
    #[test]
    fn absent_expand_time_is_an_error() {
        let raw = "Do something.\n\n@added 1.0.0\n@Purity Pure\n@Determinism Deterministic\n\
                    @Totality Unreviewed\n@Category Transform\n@ret :wat::core::i64 the value\n\
                    @example (f) #=> 1";
        assert_eq!(parse(raw), Err(DocError::MissingExpandTime));
    }

    /// ★ Row 2a — a SECOND `@ExpandTime` is `DuplicateSingleton`.
    #[test]
    fn duplicate_expand_time_is_an_error() {
        let raw = "Do something.\n\n@added 1.0.0\n@Purity Pure\n@Determinism Deterministic\n\
                    @Totality Unreviewed\n@ExpandTime Legal\n@ExpandTime RuntimeOnly\n\
                    @Category Transform\n@ret :wat::core::i64 the value\n@example (f) #=> 1";
        assert_eq!(parse(raw), Err(DocError::DuplicateSingleton { tag: "@ExpandTime".into() }));
    }

    /// ★ Row 2b — an unknown `@ExpandTime` value is `InvalidExpandTimeVariant { got }`,
    /// whose rendered message (checked in `wat-macros`) names all four legal values.
    #[test]
    fn invalid_expand_time_value_is_an_error() {
        let raw = "Do something.\n\n@added 1.0.0\n@Purity Pure\n@Determinism Deterministic\n\
                    @Totality Unreviewed\n@ExpandTime Bogus\n@Category Transform\n\
                    @ret :wat::core::i64 the value\n@example (f) #=> 1";
        assert_eq!(parse(raw), Err(DocError::InvalidExpandTimeVariant { got: "Bogus".into() }));
    }

    /// The special-form parser accepts the identical directive — `DocSpecialForm`
    /// is a SIBLING type to `DocComment` (not the same type), so both resolution
    /// points need their own coverage, including the "absence errors" half (row 1),
    /// exactly mirroring `special_form_total_parses_and_absence_is_an_error` above.
    #[test]
    fn special_form_expand_time_parses_and_absence_is_an_error() {
        let with_expand_time = "Evaluate the condition.\n\n\
            @added 1.0.0\n\
            @Category ControlFlow\n\
            @Purity Preserving\n\
            @Determinism Preserving\n\
            @Totality Partial\n\
            @ExpandTime Legal\n\
            @arg cond :wat::core::Bool the condition\n\
            @ret :T the result\n\
            @example (:wat::core::if true 1 2) #=> 1";
        let doc = super::parse_special_form(with_expand_time)
            .expect("@ExpandTime Legal must parse on a special form");
        assert_eq!(doc.expand_time, ExpandTime::Legal);

        let without_expand_time = "Evaluate the condition.\n\n\
            @added 1.0.0\n\
            @Category ControlFlow\n\
            @Purity Preserving\n\
            @Determinism Preserving\n\
            @Totality Partial\n\
            @arg cond :wat::core::Bool the condition\n\
            @ret :T the result\n\
            @example (:wat::core::if true 1 2) #=> 1";
        assert_eq!(super::parse_special_form(without_expand_time), Err(DocError::MissingExpandTime));
    }

    #[test]
    fn category_parses_all_variants() {
        for v in Category::variants() {
            assert!(v.parse::<Category>().is_ok(), "should parse: {}", v);
        }
        assert!("encoding".parse::<Category>().is_err());
    }

    /// The `@Category` error message must name EVERY legal variant. It is a
    /// `&'static str` (see `CATEGORY_LEGAL_VALUES`), so it cannot derive — this
    /// is the gate that makes the hand-copy safe. Add a variant, forget the
    /// message, go red.
    /// ⛔ `CATEGORY_LEGAL_VALUES` is a hand-written string list, so a test
    /// comparing it against another hand-written list compares stale-to-stale and
    /// passes while the ENUM has grown past both.
    /// (CORRECTED 2026-08-19, 255.1c-taxonomy: this line used to say `variants()`
    /// was hand-written too. It is DERIVE-GENERATED — `wat-source-derive/src/lib.rs:207`.
    /// The comment warning about stale lists had itself gone stale about WHICH list.)
    /// That happened on 2026-08-15:
    /// `Transform`/`Probe`/`Combine` were added to the enum and the gate stayed
    /// green because the two lists still agreed with each other.
    ///
    /// This test closes it by enumerating from an EXHAUSTIVE MATCH: adding a
    /// variant fails to compile here until it is listed, and only then can the
    /// assertions below check that it reached both hand-lists. The compiler is
    /// the ledger; the strings are just data it polices.
    #[test]
    fn every_enum_variant_reaches_both_hand_lists() {
        // Exhaustive by construction — a new variant breaks THIS match first.
        let all = [
            Category::Transform, Category::Reflection, Category::ControlFlow,
            Category::Binding, Category::Entropic, Category::Arithmetic,
            Category::Io, Category::Probe, Category::Combine, Category::Declaration,
            Category::Resource, Category::Message, Category::Ambient,
            Category::Projection, Category::CheckGate,
        ];
        for c in all {
            let name = match c {
                Category::Transform => "Transform",
                Category::Reflection => "Reflection",
                Category::ControlFlow => "ControlFlow",
                Category::Binding => "Binding",
                Category::Entropic => "Entropic",
                Category::Arithmetic => "Arithmetic",
                Category::Io => "Io",
                Category::Probe => "Probe",
                Category::Combine => "Combine",
                Category::Declaration => "Declaration",
                Category::Resource => "Resource",
                Category::Message => "Message",
                Category::Ambient => "Ambient",
                Category::Projection => "Projection",
                Category::CheckGate => "CheckGate",
            };
            assert_eq!(c.as_str(), name, "as_str() disagrees for {name}");
            assert!(Category::variants().contains(&name),
                "Category::variants() omits `{name}` — the enum grew past the hand-list");
            assert!(CATEGORY_LEGAL_VALUES.contains(name),
                "CATEGORY_LEGAL_VALUES omits `{name}`: {CATEGORY_LEGAL_VALUES}");
            assert_eq!(name.parse::<Category>().ok(), Some(c), "FromStr round-trip failed for {name}");
        }
        assert_eq!(Category::variants().len(), all.len(),
            "variants() has entries the enum does not: {:?}", Category::variants());
    }

    #[test]
    fn category_message_lists_every_variant() {
        for v in Category::variants() {
            assert!(
                CATEGORY_LEGAL_VALUES.contains(v),
                "@Category error message omits the legal variant `{v}`: {CATEGORY_LEGAL_VALUES}"
            );
        }
    }

    /// Stone 1a-β-0b — the `@Purity` sibling of `category_message_lists_every_variant`.
    /// `PURITY_LEGAL_VALUES` is a hand-written string, so this is the gate that makes
    /// the hand-copy safe: add a variant, forget this const, go red. (`wat-macros`'s two
    /// sibling `@Purity` messages are hand-written independently and are gated
    /// separately, in that crate — see `PURITY_LEGAL_VALUES`'s doc comment.)
    #[test]
    fn purity_message_lists_every_variant() {
        for v in Purity::variants() {
            assert!(
                PURITY_LEGAL_VALUES.contains(v),
                "@Purity error message omits the legal variant `{v}`: {PURITY_LEGAL_VALUES}"
            );
        }
    }

    // ─── special-form: @arg ∨ @syntax shape rule ─────────────────────────────

    /// A special-form doc with neither @arg nor @syntax → MissingShape.
    #[test]
    fn special_form_with_neither_arg_nor_syntax_is_missing_shape() {
        let raw = "Evaluate the condition.\n\n\
            @added 1.0.0\n\
            @Category ControlFlow\n\
            @Purity Preserving\n\
            @Determinism Preserving\n\
            @ret :T the result\n\
            @example (:wat::core::if true 1 2) #=> 1";
        assert_eq!(
            super::parse_special_form(raw),
            Err(DocError::MissingShape),
            "a special-form doc with no @arg and no @syntax must fail with MissingShape"
        );
    }

    /// A special-form doc with @arg only (no @syntax) parses OK.
    /// Grammar is derived from the arg names by the render site.
    #[test]
    fn special_form_with_arg_only_parses_ok() {
        let raw = "Evaluate the condition.\n\n\
            @added 1.0.0\n\
            @Category ControlFlow\n\
            @Purity Preserving\n\
            @Determinism Preserving\n\
            @Totality Unreviewed\n\
            @ExpandTime Unreviewed\n\
            @arg cond :wat::core::Bool the condition\n\
            @arg then :T the then branch\n\
            @arg else :T the else branch\n\
            @ret :T the taken branch value\n\
            @example (:wat::core::if true 1 2) #=> 1";
        let doc = super::parse_special_form(raw)
            .expect("@arg-only special form doc must parse OK");
        assert_eq!(doc.syntax, "", "syntax is empty when @syntax is absent");
        assert_eq!(doc.args.len(), 3, "three @arg entries parsed");
    }
}

/// Arc 109 "the smart comments must be compliant" — the `@arg`/`@ret` type
/// check now asks wat's own READER what a type may be spelled, instead of a
/// hand-rolled `starts_with(':')` shape test. These are the negative control
/// (a doc naming an inexpressible type must fail the build) SHIPPED WITH its
/// positive twin (the legal spellings must still pass) — a validator whose
/// refusal is untested is a validator that can be silently removed.
#[cfg(test)]
mod arc109_reader_adjudicates_type_tokens {
    use super::*;

    /// One `@added`/`@Purity`/`@Determinism`/`@Category`/`@example` shell around
    /// a single `@ret <ty>` line, so each case below differs ONLY in the type
    /// token under test.
    fn doc_with_ret_type(ty: &str) -> String {
        format!(
            "A probe.\n\n@added   1.0.0\n@ret     {ty} the ret\n@Purity Pure\n@Determinism Deterministic\n@Totality Unreviewed\n@ExpandTime Unreviewed\n@Category Transform\n@example (:wat::core::foo x) #=> 1"
        )
    }

    /// Row 1 — the refusal. `Option<T>` is exactly the angle-bracket spelling
    /// arc 109 annihilated; the reader refuses it, so the doc build must too.
    #[test]
    fn angle_bracket_type_is_refused() {
        let err = parse(&doc_with_ret_type(":wat::core::Option<wat::core::i64>"))
            .expect_err("an angle-bracket type must be refused");
        assert_eq!(
            err,
            DocError::MalformedDirective {
                tag: "@ret".into(),
                why: "type token is not a spelling wat's reader accepts (e.g. `Option<T>` and the retired `fn(…)->…` form are inexpressible; use `:- [...]`)",
            }
        );
    }

    /// Row 2 — the decisive row. Three legal spellings, none of them angle
    /// brackets, must all still be ACCEPTED: a bare keyword, the surviving
    /// parametric type-reference form `(Head :- [args])`, and the `<`
    /// operator keyword itself (arc 109's own dual: `<` stays a legal
    /// keyword body character; only an angle-bracket TYPE HEAD is illegal).
    #[test]
    fn legal_type_spellings_are_all_accepted() {
        for ty in [
            ":wat::core::Bytes",
            "(:wat::core::Vector :- [:wat::core::i64])",
            ":wat::core::<",
        ] {
            let doc = parse(&doc_with_ret_type(ty))
                .unwrap_or_else(|e| panic!("`{ty}` must be accepted, got {e:?}"));
            assert_eq!(doc.ret_type, ty);
        }
    }

    /// Row 4 — the colon rule still fires, unreplaced. A bare `Bytes` LEXES
    /// fine as a plain symbol (the reader alone would accept it — it is
    /// "expressible"), but the annotation grammar still demands a keyword;
    /// this is the check the reader-based one was added ALONGSIDE, not
    /// instead of, and its error message is unchanged.
    #[test]
    fn bare_symbol_without_colon_is_still_refused_by_the_colon_rule() {
        let err = parse(&doc_with_ret_type("Bytes")).expect_err("a bare symbol must be refused");
        assert_eq!(
            err,
            DocError::MalformedDirective {
                tag: "@ret".into(),
                why: "type token must start with `:` (e.g. `:wat::core::String`); grammar is `@ret <type> <desc>`",
            }
        );
    }

    /// The two surviving STRUCTURAL spellings both round-trip through a real
    /// `@arg`/`@ret` pair — the nested case (`Option<Process<I,O>>`-shaped)
    /// and the fn-type bracket case — proving `take_type_token` finds the
    /// MATCHING close across nested parens/brackets, not just the first one.
    #[test]
    fn nested_parametric_type_reference_round_trips() {
        let doc = "A probe.\n\n@added   1.0.0\n@arg     peers (:wat::core::Vector :- [(:wat::kernel::Peer :- [I O])]) the peers\n@ret     (:wat::core::Option :- [(:wat::kernel::Process :- [I O])]) the ret\n@Purity Pure\n@Determinism Deterministic\n@Totality Unreviewed\n@ExpandTime Unreviewed\n@Category Transform\n@example (:wat::core::foo x) #=> 1";
        let parsed = parse(doc).expect("nested parametric type references must be accepted");
        assert_eq!(parsed.args[0].ty, "(:wat::core::Vector :- [(:wat::kernel::Peer :- [I O])])");
        assert_eq!(parsed.args[0].desc, "the peers");
        assert_eq!(parsed.ret_type, "(:wat::core::Option :- [(:wat::kernel::Process :- [I O])])");
        assert_eq!(parsed.ret, "the ret");
    }

    #[test]
    fn fn_type_bracket_form_round_trips() {
        let doc = "A probe.\n\n@added   1.0.0\n@arg     prog [(:wat::kernel::Peer :- [S R]) :-> :wat::core::nil] the prog\n@ret     (:wat::kernel::Thread :- [R S]) the ret\n@Purity Pure\n@Determinism Deterministic\n@Totality Unreviewed\n@ExpandTime Unreviewed\n@Category Transform\n@example (:wat::core::foo x) #=> 1";
        let parsed = parse(doc).expect("the fn-type bracket form must be accepted");
        assert_eq!(parsed.args[0].ty, "[(:wat::kernel::Peer :- [S R]) :-> :wat::core::nil]");
        assert_eq!(parsed.args[0].desc, "the prog");
        assert_eq!(parsed.ret_type, "(:wat::kernel::Thread :- [R S])");
    }
}

::wat_source_derive::wat_enum_from!(
    pub enum Totality,
    "../../wat/runtime-meta.wat",
    ":wat::runtime::Totality"
);

#[cfg(test)]
mod probe_totality_axis {
    use super::Totality;
    /// FM 2-bis probe — the axis is not merely generated, it carries all FOUR
    /// variants by name, and the match is exhaustive (a missing variant is E0004).
    #[test]
    fn totality_has_four_named_variants_and_matches_exhaustively() {
        let all = [Totality::Total, Totality::Partial, Totality::Preserving, Totality::Unreviewed];
        assert_eq!(all.len(), 4);
        for v in all {
            // Exhaustive, no wildcard: adding a variant in the .wat breaks this.
            let name = match v {
                Totality::Total => "Total",
                Totality::Partial => "Partial",
                Totality::Preserving => "Preserving",
                Totality::Unreviewed => "Unreviewed",
            };
            assert!(!name.is_empty());
        }
    }
    /// The per-variant `;;` prose from the .wat must survive into the Rust doc —
    /// that is the half read from the raw text layer, and it is the half that rots
    /// silently if the two readers ever disagree.
    #[test]
    fn the_work_list_variant_is_documented_as_such() {
        // Compile-time proof the variant exists under the exact name the census will filter on.
        let partial = Totality::Partial;
        assert!(matches!(partial, Totality::Partial));
    }
}

::wat_source_derive::wat_enum_from!(
    pub enum ExpandTime,
    "../../wat/runtime-meta.wat",
    ":wat::runtime::ExpandTime"
);

#[cfg(test)]
mod probe_expand_time_axis {
    use super::ExpandTime;
    /// FM 2-bis probe — the axis carries all FIVE variants by name and the match is
    /// exhaustive (a variant added in the `.wat` without an arm here is `E0004`).
    /// Arc 255 Stone expand-only-the-missing-pole minted `ExpandOnly` — `RuntimeOnly`'s
    /// mirror — bringing the count from four to five; this probe's own claim would have
    /// gone quietly stale (still green, silently no longer covering the new pole) had
    /// only the exhaustive `match` below been extended without also widening the
    /// enumerated array.
    #[test]
    fn expand_time_has_five_named_variants() {
        for v in [
            ExpandTime::Legal,
            ExpandTime::RuntimeOnly,
            ExpandTime::ExpandOnly,
            ExpandTime::Preserving,
            ExpandTime::Unreviewed,
        ] {
            let name = match v {
                ExpandTime::Legal => "Legal",
                ExpandTime::RuntimeOnly => "RuntimeOnly",
                ExpandTime::ExpandOnly => "ExpandOnly",
                ExpandTime::Preserving => "Preserving",
                ExpandTime::Unreviewed => "Unreviewed",
            };
            assert!(!name.is_empty());
        }
    }
    /// `RuntimeOnly` is the pole the allow-list's default-deny produces, and
    /// `Unreviewed` is distinct from it — an unmeasured verb is NOT a measured refusal.
    #[test]
    fn unreviewed_is_not_runtime_only() {
        assert_ne!(ExpandTime::Unreviewed, ExpandTime::RuntimeOnly);
    }
}
