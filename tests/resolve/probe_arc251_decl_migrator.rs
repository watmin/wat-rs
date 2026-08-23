//! Forward-proof probe — arc 251 Stone 251.5 / Slice 4.1: the declaration migrator.
//!
//! Verifies `fix-form` (`:migrate::fix-form`) closes the two gaps left by
//! `fix.wat`'s `fix-source` in DECLARATION and DEFN forms.
//!
//! The migrator code was baked verbatim into the co-located fixture
//! `probe_arc251_decl_migrator.wat` at migration time (wat-migrate/fix-decl.wat
//! is non-blessed and retires with the hard-cut; cannot be auto-loaded).
//!
//! Cases:
//!   C01 (gap A)  — typealias bare type-slot → `keyword/to-type-form`, not symbol.
//!   C02 (gap B)  — defn with `<T>` name → PLAIN symbol (`<T>` dropped).
//!   C03          — generic decl name + parametric target.
//!   C04          — preservation: user-type targets already correct under fix-source.
//!   C05          — newtype: bare type-slot handled; name-fix on plain name.
//!   C06          — typeunion: core member vector uses type form.
//!   C07          — typeunion: user-type member vector preserved.
//!   C08          — defenum: variant tags stay keywords, field types converted.
//!
//! Run: `cargo test --release --test probe_arc251_decl_migrator`

use wat::freeze::call_beside_value;
use wat::runtime::Value;

// just-eval (rubric): each `:user::cNN` zero-arg fn lives in the co-located fixture;
// drive it via `call_beside_value` and inspect the returned typed String.
fn eval_string(fn_name: &str) -> Result<String, String> {
    let full = format!(":user::{}", fn_name);
    match call_beside_value(file!(), &full).map_err(|e| format!("eval {fn_name}: {e:?}"))? {
        Value::String(s) => Ok((*s).clone()),
        other => Err(format!("non-string from {fn_name}: {other:?}")),
    }
}

fn normalize(s: String) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn c01_typealias_bare_type_slot_uses_type_form() {
    let got = normalize(eval_string("c01").expect("C01 fix-form"));
    assert_eq!(
        got,
        include_str!("probe_arc251_decl_migrator__c01-typealias-type-slot.wat"),
        "C01 (gap A): typealias core-scalar type-slot must render as wat.type/i64"
    );
}

#[test]
fn c02_defn_generic_name_drops_type_params() {
    // Arc 109 (wave 2, "annihilate the angle bracket") — ANNIHILATED, class 3 (c).
    // C02's original subject was `:migrate::name-fix` stripping a `<T>` suffix off a
    // DEFN NAME (`:my::ns::map<T>` → plain symbol `map`, `<T>` dropped rather than
    // migrated). The LEXER wall (this stone) refuses `<` in a name at the READER —
    // before `:migrate::fix-form` (or anything else) ever sees the AST — so
    // `:migrate::name-fix`'s `split(kw, "<")` branch can no longer be reached by any
    // legal `read-string` input. It is not merely untested; it is UNREACHABLE. Fixture
    // migrated to a plain `::`-qualified, non-generic name so the test keeps exercising
    // the surviving half of `name-fix` (keyword → symbol conversion) — this is a
    // WEAKER assertion than the original ("drops <T>"), reported honestly per STOP-3
    // rather than disguised as a like-for-like fix. The `<`-stripping branch itself is
    // a purge candidate for the sibling stone.
    let got = normalize(eval_string("c02").expect("C02 fix-form"));
    assert_eq!(
        got,
        include_str!("probe_arc251_decl_migrator__c02-defn-drop-type-params.wat"),
        "C02 (surviving half): defn name with only `::` (no `<T>`, which is now \
         unreachable input) must still become a plain symbol"
    );
}

#[test]
fn c03_generic_decl_name_and_parametric_target() {
    // Arc 109 ③ (a PRIOR stone) walled the TYPE PARSER, so C03 used to prove that a
    // parametric TARGET keyword (`Vector<i64>`) got refused by `keyword/to-type-form`
    // with "angle-bracket parametric types are illegal". Arc 109 wave 2 (THIS stone)
    // walls the LEXER, one door earlier: `:user::topform`'s `read-string` on a source
    // string containing `Foo<T>` / `Vector<wat::core::i64>` now fails at the READER,
    // before `keyword/to-type-form` is ever reached — and `:user::topform`'s
    // `ReadOutcome::Malformed` arm calls `(:wat::core::Error/message __cause)` on a
    // `:wat::edn::ForeignRecord` that does not implement a `message` surface method, so
    // the read-string path now crashes with an unrelated `UnknownFunction` before it
    // can even report the lex refusal. That crash is a SEPARATE, already-documented
    // defect (DESIGN-STONE-annihilate-the-angle-bracket.md's sequencing section) — out
    // of this stone's boundary (test module only, not `read-string`'s Rust or
    // `wat/edn.wat`'s `ForeignRecord`).
    //
    // Class 3 (b) — re-pointed as a refusal control, NOT (a): I tried migrating the
    // fixture's target to an already-`:-`-form type reference (`(:wat::core::Vector :-
    // [:wat::core::i64])`) to keep testing "parametric target survives"; that fails a
    // DIFFERENT way — `:migrate::type-slot-2?`'s branch calls `keyword/to-type-form`
    // UNCONDITIONALLY on the typealias target (unlike `:migrate::fix-types`'s ast-kind
    // check for typeunion members), so it cannot accept a List there at all:
    // `MalformedForm { reason: "keyword/to-type-form requires a Keyword node" }`. So
    // there is no legal input — parametric or not, keyword or already-`:-`-form — for
    // which C03 exercises anything beyond what C01 (plain type-slot) and C04
    // (user-type preservation) already cover. STOP-3: making it green with a plain
    // keyword target would be a materially weaker assertion wearing this test's name.
    // Left pointed at the ORIGINAL angle-bracket source instead, re-purposed as a
    // refusal control on THAT: `read-string` now refuses `Foo<T>` / `Vector<wat::core::
    // i64>` at the lexer wall, but `:user::topform`'s `ReadOutcome::Malformed` arm
    // calls `(:wat::core::Error/message __cause)` on a `:wat::edn::ForeignRecord` that
    // has no `message` surface method — a SEPARATE, already-documented defect
    // (DESIGN-STONE-annihilate-the-angle-bracket.md's sequencing section) that fires
    // before the lex refusal can be reported cleanly. Asserting the crash's mechanism
    // (not a made-up clean message) so this goes red again, honestly, if that separate
    // bug is ever fixed and the message changes shape.
    //
    // Class 3 (c) ALSO applies, more broadly than C02: `keyword/to-type-form`'s whole
    // reason to exist — parsing a `<...>`-embedded parametric type OUT OF a keyword
    // string — is dead code. No legal `read-string` input can ever contain such a
    // keyword again. Purge candidate for the sibling stone.
    let err = eval_string("c03").expect_err("angle-bracket source must fail to read");
    assert!( // rune:lint(loose-assert) — targeted substring: the read-string crash's mechanism, not the whole located error's structure
        err.contains("ForeignRecord") && err.contains("message"),
        "expected the read-string/ForeignRecord crash (see comment above); got: {err}"
    );
}

#[test]
fn c04_user_type_target_preserved() {
    let got = normalize(eval_string("c04").expect("C04 fix-form"));
    assert_eq!(
        got,
        include_str!("probe_arc251_decl_migrator__c04-user-type-preserved.wat"),
        "C04 (preservation): user-type target must preserve namespace"
    );
}

#[test]
fn c05_newtype_type_slot_handled() {
    let got = normalize(eval_string("c05").expect("C05 fix-form"));
    assert_eq!(
        got,
        include_str!("probe_arc251_decl_migrator__c05-newtype-type-slot.wat"),
        "C05 (newtype): bare type-slot in newtype must use keyword/to-type-form"
    );
}

#[test]
fn c06_typeunion_core_member_vector_uses_type_form() {
    let got = normalize(eval_string("c06").expect("C06 fix-form"));
    assert_eq!(
        got,
        include_str!("probe_arc251_decl_migrator__c06-typeunion-core-members.wat"),
        "C06: typeunion core-scalar members must render as wat.type/ in the member vector"
    );
}

#[test]
fn c07_typeunion_user_member_vector_preserved() {
    let got = normalize(eval_string("c07").expect("C07 fix-form"));
    assert_eq!(
        got,
        include_str!("probe_arc251_decl_migrator__c07-typeunion-user-members.wat"),
        "C07: typeunion user-type members must preserve their namespace"
    );
}

#[test]
fn c08_defenum_variant_tags_stay_fields_converted() {
    let got = normalize(eval_string("c08").expect("C08 fix-form"));
    assert_eq!(
        got,
        include_str!("probe_arc251_decl_migrator__c08-defenum-variant-tags.wat"),
        "C08 (defenum): variant tags stay keywords; field types convert via the arrow rule"
    );
}
