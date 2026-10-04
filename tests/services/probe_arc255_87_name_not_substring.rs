//! A name is an identity. Keyword `:probe::work` and symbol `probe/work` mint the
//! same companion. The door is `canonical_identity`, then a suffix. `compose-variant`
//! names an enum variant, not `::kwargs-check`.

use wat::edn::render::canonical_identity;
use wat::freeze::startup_from_file;

fn bare(id: &str) -> &str {
    id.strip_prefix(':').unwrap_or(id)
}

#[test]
fn map_and_each_companions_agree_for_a_keyword_and_a_symbol() {
    let kw = canonical_identity(":probe::work");
    let sym = canonical_identity("probe/work");
    assert_eq!(kw, ":probe::work");
    assert_eq!(sym, kw);
    // The companion names are literals. Concatenating `::` here is the shape
    // `one_variant_separator` calls COMPOSE, and that lint has no category for
    // a companion suffix. `canonical_identity` of an already-canonical companion
    // returns the same literal: the door does not slice it.
    for companion in [
        ":probe::work::kwargs-check",
        ":probe::work::grant-worker",
        ":probe::work::revoke-worker",
        ":probe::work::Coords",
    ] {
        assert_eq!(canonical_identity(companion), companion);
    }
    startup_from_file("wat-scripts/probes/arc-170/probe-c1-clean-surface.wat")
        .expect("keyword bracket/map");
    startup_from_file("tests/services/probe_arc255_87_map_symbol.wat")
        .expect("symbol bracket/map");
    startup_from_file("tests/services/probe_arc255_87_each_keyword.wat")
        .expect("keyword bracket/each");
    startup_from_file("tests/services/probe_arc255_87_each_symbol.wat")
        .expect("symbol bracket/each");
}

#[test]
fn a_locus_head_is_compared_as_an_identity() {
    assert_eq!(canonical_identity("wat.spawn/process"), ":wat::spawn::process");
    assert_eq!(
        canonical_identity(":wat::spawn::process"),
        ":wat::spawn::process"
    );
    // `with-label` is a function member. `rekey_type_member_functions` stores
    // that as `/`, so the keyword and the symbol are that one key.
    assert_eq!(
        canonical_identity(":wat::spawn::Locus/with-label"),
        ":wat::spawn::Locus/with-label"
    );
    assert_eq!(
        canonical_identity("wat.spawn.Locus/with-label"),
        ":wat::spawn::Locus/with-label"
    );
}

#[test]
fn rule_fold_and_field_names_strip_the_sigil_after_identity() {
    assert_eq!(
        bare(&canonical_identity("weather/cold")),
        bare(&canonical_identity(":weather::cold"))
    );
    assert_eq!(bare(&canonical_identity(":weather::cold")), "weather::cold");
    assert_eq!(
        bare(&canonical_identity("user/fold")),
        bare(&canonical_identity(":user::fold"))
    );
    assert_eq!(bare(&canonical_identity(":user::fold")), "user::fold");
    assert_eq!(bare(&canonical_identity("probe/Echo")), "probe::Echo");
    assert_eq!(bare(&canonical_identity(":probe::Echo")), "probe::Echo");
    assert_eq!(bare(&canonical_identity(":echo")), "echo");
    assert_eq!(bare(&canonical_identity("echo")), "echo");
    assert_eq!(bare(&canonical_identity(":?loc")), "?loc");
    assert_eq!(
        bare(&canonical_identity("wat.rete.acc/count")),
        "wat::rete::acc::count"
    );
    assert_eq!(
        bare(&canonical_identity(":wat::rete::acc::count")),
        "wat::rete::acc::count"
    );
}
