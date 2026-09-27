//! Excursus 003 step 4 (D4) — G5, a thread locus (builder correction).
//!
//! `USER_SOURCE_FILES` is thread-local; a THREAD locus
//! (`:wat::spawn::thread`/`spawn_thread_peer`, `src/kernel/spawn.rs`) runs a function of
//! the SAME already-frozen world on a NEW OS thread, with no load pipeline of its own to
//! populate it — left uninstalled, D4's derivation would be silently a no-op there. See
//! the co-located `.wat` fixture for the repro (reusing G1's own overflow).

use wat::edn::render::value_to_edn_with;
use wat::freeze::startup_beside;

#[test]
fn g5_the_overflow_locates_at_the_users_line_inside_a_thread_peer() {
    let world = startup_beside(file!()).expect("the fixture must freeze");
    let func = world
        .symbols()
        .get(":probe::overflow-in-thread")
        .expect(":probe::overflow-in-thread defined")
        .clone();

    let result = wat::runtime::apply_function(func, Vec::new(), world.symbols(), wat::rust_caller_span!())
        .expect("the fn itself returns normally — it MATCHES the thread's death into a value");

    let edn = value_to_edn_with(&result, Some(&world.types))
        .expect("the LociDiedError value must render to EDN");
    let rendered = wat_edn::write(&edn);

    wat::assert_edn_matches_file!(
        rendered,
        "probe_excursus003_step4_g5_thread_locus__died.edn"
    );
}
