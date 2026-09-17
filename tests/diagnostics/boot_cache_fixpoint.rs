//! Excursus 001 Tier A — the boot cache's correctness gates.
//!
//! ⛔⛔ **THE OBVIOUS TEST IS WORTHLESS HERE.** `crates/wat-reader/src/span.rs:137` is
//! `impl PartialEq for Span { fn eq(&self, _: &Self) -> bool { true } }` and `Hash` is a no-op —
//! deliberately, so structurally-identical ASTs from different sources compare equal. **Therefore
//! `assert_eq!(original, decoded)` PASSES ON A DECODER THAT DROPS EVERY SPAN**, and a cache that
//! silently discarded every source location would ship green and destroy every diagnostic in the
//! language.
//!
//! So every assertion below is on **BYTES**: `encode(decode(encode(x))) == encode(x)`, plus the
//! stronger form the real payload admits — `file_bytes == encode(decode(file_bytes))`. The bytes
//! carry the spans; byte equality does not let them go missing.

use std::sync::Arc;

use wat::ast::WatAST;
use wat::freeze::boot_cache::probe;
use wat::freeze::startup_bare;
use wat::scope::{fresh_scope, Identifier};
use wat::span::Span;

/// Point every cache read/write in THIS process at a private directory. nextest runs each test in
/// its own process, so this cannot leak into a sibling.
fn isolate() -> tempfile::TempDir {
    let d = tempfile::tempdir().expect("tempdir");
    std::env::set_var("WAT_BOOT_CACHE_DIR", d.path());
    std::env::remove_var("WAT_BOOT_CACHE");
    d
}

fn cache_file(dir: &tempfile::TempDir) -> std::path::PathBuf {
    let mut found = None;
    for e in std::fs::read_dir(dir.path()).expect("read cache dir").flatten() {
        if e.path().extension().map(|x| x == "watbc").unwrap_or(false) {
            found = Some(e.path());
        }
    }
    found.expect("the first boot wrote a payload")
}

// ── row 2 ⭑⭑ — SPANS SURVIVE ────────────────────────────────────────────────────────────────

/// The real 2.8 MB stdlib payload, proven by BYTES rather than by an equality that cannot fail.
#[test]
fn the_real_payload_is_a_byte_exact_fixpoint_so_no_span_can_go_missing() {
    let dir = isolate();
    let _world = startup_bare().expect("bare world boots");
    let path = cache_file(&dir);
    let on_disk = std::fs::read(&path).expect("payload reads");
    let key = probe::key().expect("this build carries a fingerprint");

    let snap = probe::load().expect("the payload this boot wrote loads back");
    let once = probe::encode_with_key(&snap, &key);

    // ⭐ THE STRONG FORM: re-encoding what the file decoded to reproduces the file, byte for byte.
    // A decoder that dropped spans (or interned two distinct files to one, or lost a scope's
    // sharing) could not land back on these exact bytes.
    assert_eq!(on_disk.len(), once.len(), "re-encode changed the payload LENGTH");
    assert!(on_disk == once, "re-encode of the decoded payload is not the payload");

    // And the fixpoint itself, so the property is asserted in the form the DESIGN named.
    let again = probe::decode_with_key(&once, &key).expect("the payload decodes twice");
    let twice = probe::encode_with_key(&again, &key);
    assert!(once == twice, "encode(decode(encode(x))) != encode(x)");

    // The payload is not trivially small — a fixpoint over nothing proves nothing.
    let (macros, surfaces, types, builtins, fns, residue) = probe::shape_of(&snap);
    assert!(macros > 100, "only {macros} macros in the payload");
    assert!(types > 100, "only {types} types in the payload");
    // 652 as measured, not 2,177: the snapshot is taken after `register_stdlib_defines` and
    // BEFORE `6a-9 auto-method-codegen`, which mints the other ~1,525 from the COMBINED
    // (stdlib + user) `TypeEnv` and therefore cannot be cached. See the module header.
    assert!(fns > 500, "only {fns} functions in the payload");
    assert!(on_disk.len() > 1_000_000, "payload is only {} bytes", on_disk.len());
    eprintln!(
        "payload {} bytes · macros {macros} · surface-forms {surfaces} · types {types} \
         · builtin-names {builtins} · functions {fns} · runtime-def forms {residue} \
         · probe witness {} {:?}",
        on_disk.len(),
        probe::witness_of(&snap).len(),
        probe::witness_of(&snap)
    );
}

// ── row 6 ⭑ + row 7 ⭑ — NATIVES, AND SCOPES ─────────────────────────────────────────────────

/// Row 6: nothing native is in the payload — natives dispatch by STRING through a process-global
/// `OnceLock` + `inventory` registry boot never touches, so there is nothing to serialise.
#[test]
fn no_native_body_is_ever_written_into_the_payload() {
    let _dir = isolate();
    let _world = startup_bare().expect("bare world boots");
    let snap = probe::load().expect("payload loads");
    assert!(
        probe::is_representable(&snap),
        "the payload holds something the format cannot faithfully carry"
    );
}

/// Row 7: `ScopeId` is RE-MINTED, never restored by value — `Function::params`' own doc: "an
/// exec'd child restarts `fresh_scope()` at 1, so imported scopes must be REMAPPED". What must
/// survive is the SHARING structure, and that is what is asserted here.
///
/// This test also covers the five `WatAST` variants the stdlib payload never exercises
/// (`RationalLit`, `BigIntLit`, `CharLit`, `Map`, `Set`) plus the float and integer edges.
#[test]
fn the_variants_the_stdlib_never_exercises_round_trip_and_scopes_come_back_shared() {
    let key = "test-key";
    let fa = Arc::new("a.wat".to_string());
    let fb = Arc::new("b.wat".to_string());
    let s1 = Span::with_end(Arc::clone(&fa), 3, 7, 3, 19);
    let s2 = Span::new(Arc::clone(&fb), 41, 1); // point span: `end` is None
    let s3 = Span::with_end(Arc::clone(&fb), 1, 1, 9, 9);

    let shared = fresh_scope();
    let other = fresh_scope();
    let id_a = Identifier::bare("tmp").add_scope(shared);
    let id_b = Identifier::bare("tmp").add_scope(shared).add_scope(other);
    let id_c = Identifier::bare("tmp"); // no scopes at all

    let forms = vec![
        WatAST::RationalLit("-22/7".parse().expect("rational"), s1.clone()),
        WatAST::BigIntLit("170141183460469231731687303715884105727".parse().expect("bigint"), s2.clone()),
        WatAST::CharLit('😀', s3.clone()),
        WatAST::CharLit('\n', s1.clone()),
        WatAST::Map(
            vec![
                (WatAST::Keyword(":k".into(), s1.clone()), WatAST::NilLit(s2.clone())),
                (WatAST::StringLit("v".into(), s3.clone()), WatAST::BoolLit(true, s1.clone())),
            ],
            s2.clone(),
        ),
        WatAST::Set(vec![WatAST::IntLit(i64::MIN, s1.clone()), WatAST::IntLit(i64::MAX, s2.clone())], s3.clone()),
        WatAST::Vector(vec![], s1.clone()),
        WatAST::FloatLit(-0.0, s2.clone()),
        WatAST::FloatLit(f64::MIN_POSITIVE, s3.clone()),
        WatAST::FloatLit(f64::NAN, s1.clone()),
        WatAST::Symbol(id_a, s1.clone()),
        WatAST::Symbol(id_b, s2.clone()),
        WatAST::Symbol(id_c, s3.clone()),
        WatAST::List(vec![WatAST::Keyword(":wat::core::do".into(), s1.clone())], s3),
    ];

    let snap = probe::snapshot_of_forms(forms);
    let once = probe::encode_with_key(&snap, key);
    let back = probe::decode_with_key(&once, key).expect("synthetic payload decodes");
    let twice = probe::encode_with_key(&back, key);
    assert!(once == twice, "encode(decode(encode(x))) != encode(x) on the rare variants");

    // ⭐ Scopes: DIFFERENT ids (re-minted through `fresh_scope`), SAME sharing.
    let out = probe::forms_of(&back);
    let (WatAST::Symbol(a, _), WatAST::Symbol(b, _), WatAST::Symbol(c, _)) =
        (&out[10], &out[11], &out[12])
    else {
        panic!("the three symbols did not come back as symbols");
    };
    assert_eq!(a.scopes().len(), 1);
    assert_eq!(b.scopes().len(), 2);
    assert_eq!(c.scopes().len(), 0);
    let a0 = *a.scopes().iter().next().expect("one scope");
    assert!(b.scopes().contains(&a0), "the SHARED scope did not come back shared");
    assert_ne!(a0, shared, "a raw ScopeId was restored by value — it must be RE-MINTED");

    // Spans, checked by hand as well as by bytes: `end` is the field a lazy decoder drops.
    let WatAST::CharLit(ch, sp) = &out[2] else { panic!("charlit") };
    assert_eq!(*ch, '😀');
    assert_eq!(&*sp.file, "b.wat");
    assert_eq!((sp.line, sp.col), (1, 1));
    let end = sp.end.as_ref().expect("the range end survived");
    assert_eq!((end.line, end.col), (9, 9));
    let WatAST::BigIntLit(_, sp2) = &out[1] else { panic!("bigint") };
    assert!(sp2.end.is_none(), "a point span grew an end out of nowhere");
    let WatAST::FloatLit(z, _) = &out[7] else { panic!("float") };
    assert!(z.is_sign_negative() && *z == 0.0, "-0.0 lost its sign");
    let WatAST::FloatLit(n, _) = &out[9] else { panic!("float") };
    assert!(n.is_nan(), "NaN did not survive");
}

// ── row 5 ⭑⭑ — ABSENT / TRUNCATED / CORRUPT ALL FALL BACK ───────────────────────────────────

/// ⛔ A boot that fails because a cache file is bad is a worse product than a slow boot. All three
/// damage modes are DRIVEN here, and each one must both (a) be refused by the decoder and (b)
/// leave a boot that still produces the identical world.
#[test]
fn absent_truncated_and_corrupt_payloads_all_fall_back_to_deriving() {
    let dir = isolate();
    let baseline = startup_bare().expect("bare world boots").symbols().functions_iter().count();
    let path = cache_file(&dir);
    let good = std::fs::read(&path).expect("payload reads");
    assert!(probe::load().is_some(), "the payload this boot wrote must load back");

    // (1) ABSENT.
    std::fs::remove_file(&path).expect("remove");
    assert!(probe::load().is_none(), "a missing payload must not load");
    assert_eq!(
        startup_bare().expect("boot with no cache").symbols().functions_iter().count(),
        baseline
    );

    // (2) TRUNCATED — every prefix length, so the failure is not "one lucky cut".
    for cut in [0usize, 1, 7, 8, 32, good.len() / 3, good.len() / 2, good.len() - 17, good.len() - 1] {
        std::fs::write(&path, &good[..cut]).expect("write truncated");
        assert!(probe::load().is_none(), "a payload truncated to {cut} bytes LOADED");
    }
    std::fs::write(&path, &good[..good.len() / 2]).expect("write truncated");
    assert_eq!(
        startup_bare().expect("boot with a truncated cache").symbols().functions_iter().count(),
        baseline
    );

    // (3) CORRUPT ONE BYTE — at the head, the middle and the tail of the body.
    for at in [good.len() / 4, good.len() / 2, good.len() - 40] {
        let mut bad = good.clone();
        bad[at] ^= 0xff;
        std::fs::write(&path, &bad).expect("write corrupt");
        assert!(probe::load().is_none(), "a payload with byte {at} flipped LOADED");
    }
    let mut bad = good.clone();
    bad[good.len() / 2] ^= 0xff;
    std::fs::write(&path, &bad).expect("write corrupt");
    assert_eq!(
        startup_bare().expect("boot with a corrupt cache").symbols().functions_iter().count(),
        baseline
    );

    // (4) A FOREIGN PAYLOAD under the right filename — the key is inside the bytes too, so a
    // filename collision cannot pass another build's world off as this one's.
    let synthetic = probe::encode_with_key(&probe::snapshot_of_forms(vec![]), "not-this-build");
    std::fs::write(&path, &synthetic).expect("write foreign");
    assert!(probe::load().is_none(), "a payload carrying a FOREIGN key loaded");
    assert_eq!(
        startup_bare().expect("boot with a foreign cache").symbols().functions_iter().count(),
        baseline
    );
}

/// `WAT_BOOT_CACHE=off` is the A/B door every measurement in the SCORE used. It must produce the
/// same world, and must neither read nor write.
#[test]
fn the_cache_can_be_switched_off_and_the_world_is_the_same() {
    let dir = isolate();
    let with = startup_bare().expect("boots").symbols().functions_iter().count();
    assert!(cache_file(&dir).exists());
    std::env::set_var("WAT_BOOT_CACHE", "off");
    assert!(probe::load().is_none(), "the off switch still read the payload");
    let without = startup_bare().expect("boots with the cache off").symbols().functions_iter().count();
    assert_eq!(with, without);
}
