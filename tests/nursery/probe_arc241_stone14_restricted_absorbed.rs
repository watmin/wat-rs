//! FM 2-bis probe for Stone 241.14 — `:wat::core::def-restricted` + `:wat::core::defn-restricted` ABSORB INTO METADATA-MAP.
//!
//! Stone 241.14 honors broken Stone 241.6 D10 + line-182 commitment: arc 198's
//! `def-restricted` / `defn-restricted` legacy retires; `:restricted-to` migrates
//! into binding metadata on def/defn. The `defined_value_restrictions` parallel
//! storage is DELETED; `binding_metadata` becomes sole restriction store.
//!
//! User direction 2026-05-29 very late: *"def and defn are the only ways"* —
//! both `def-restricted` and `defn-restricted` die. Restrictions live as
//! `:restricted-to` key in metadata-map.
//!
//! HEAD-disconfirmation map (5/6 DISCONFIRM at HEAD; C01 PRESERVATION):
//! - C01: `(def :name {:restricted-to [:allowed::]} expr)` allowed caller passes
//!        — PRESERVATION (passes at HEAD via no-enforcement; passes post-stone via
//!          metadata-driven enforcement; regression guard for the allowed-path)
//! - C02: same form; non-allowed caller fails with DefRestrictedCallerNotAllowed
//!        ⇒ FAILS at HEAD (no enforcement from metadata-map; non-allowed caller passes spuriously)
//! - C03: `(defn :name {:restricted-to [:allowed::]} [args] -> :Ret body)` enforces restriction
//!        ⇒ FAILS at HEAD (defn metadata-map doesn't drive enforcement)
//! - C04: `:wat::core::def-restricted` HARD-CUT-rejected at startup
//!        ⇒ FAILS at HEAD (form is still live; arc 198 mechanism active)
//! - C05: `:wat::core::defn-restricted` HARD-CUT-rejected at startup
//!        ⇒ FAILS at HEAD (form is still live; wat/core.wat macro active)
//! - C06: rejection remedies name `:wat::core::def` / `:wat::core::defn` respectively
//!        ⇒ FAILS at HEAD (no rejection fires; no error to inspect)
//!
//! Post-stone: all 6 contracts PASS.
//!
//! Run: `cargo test --release --test probe_arc241_stone14_restricted_absorbed`

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

fn try_startup(src: &str) -> Result<(), String> {
    let full = format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    );
    startup_from_source(&full, None, Arc::new(InMemoryLoader::new()))
        .map(|_| ())
        .map_err(|e| format!("{:?}", e))
}

fn try_startup_display(src: &str) -> String {
    let full = format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    );
    match startup_from_source(&full, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => String::from("<startup succeeded — no error to display>"),
        Err(e) => format!("{}", e),
    }
}

// ─── C01: def metadata-map :restricted-to — allowed caller passes ──────────────

#[test]
fn contract_01_def_metadata_restricted_allowed_caller_passes() {
    // (def :name {:restricted-to [:allowed::]} expr) registers restriction;
    // a caller whose FQDN starts with :allowed:: passes.
    //
    // At HEAD: metadata-map on def is STORED in binding_metadata but doesn't
    // drive enforcement (the walker reads from defined_value_restrictions,
    // populated only by :wat::core::def-restricted's parser). So this fixture
    // would pass startup spuriously because no enforcement fires for any caller.
    //
    // We make the test discriminating by having a NON-allowed caller invoke;
    // at HEAD it passes (no enforcement); post-stone the call fails with
    // DefRestrictedCallerNotAllowed. To distinguish C01 (allowed-passes) from
    // C02 (non-allowed-fails), we use TWO call sites with different prefixes.
    //
    // C01 specifically: allowed caller (matching the whitelist prefix) succeeds.
    let src = r#"
        (:wat::core::defn :test::restricted-target
          {:restricted-to [:test::]}
          [] -> :wat::core::i64 42)
        (:wat::core::defn :test::allowed-caller
          [] -> :wat::core::i64 (:test::restricted-target))
    "#;
    let result = try_startup(src);
    assert!(
        result.is_ok(),
        "allowed caller (matching :test:: prefix) must pass under metadata-map restriction; got: {:?}",
        result
    );
}

// ─── C02: def metadata-map :restricted-to — non-allowed caller fails ───────────

#[test]
fn contract_02_def_metadata_restricted_non_allowed_caller_fails() {
    // (def :name {:restricted-to [:allowed::]} expr) — non-allowed caller must fail.
    // At HEAD: walker reads from defined_value_restrictions (empty for metadata-map
    // declarations); non-allowed caller passes spuriously. Post-stone: walker reads
    // binding_metadata; non-allowed caller fails with DefRestrictedCallerNotAllowed.
    let src = r#"
        (:wat::core::defn :test::restricted-target
          {:restricted-to [:test::]}
          [] -> :wat::core::i64 42)
        (:wat::core::defn :other::non-allowed-caller
          [] -> :wat::core::i64 (:test::restricted-target))
    "#;
    let result = try_startup(src);
    assert!(
        result.is_err(),
        "non-allowed caller (not matching :test:: prefix) must fail metadata-map restriction post-stone; got Ok"
    );
}

// ─── C03: defn metadata-map :restricted-to — enforcement works ─────────────────

#[test]
fn contract_03_defn_metadata_restricted_enforces() {
    // defn's metadata-map clause inherits the restriction semantics (defn macro
    // expands to def + fn; metadata flows to binding). Test that defn with
    // metadata-map restriction fails for non-allowed caller post-stone.
    let src = r#"
        (:wat::core::defn :test::restricted-fn
          {:restricted-to [:test::]}
          [] -> :wat::core::i64 42)
        (:wat::core::defn :other::non-allowed-caller
          [] -> :wat::core::i64 (:test::restricted-fn))
    "#;
    let result = try_startup(src);
    assert!(
        result.is_err(),
        "defn metadata-map :restricted-to must enforce; non-allowed caller must fail post-stone; got Ok"
    );
}

// ─── C04: :wat::core::def-restricted HARD-CUT-rejected ─────────────────────────

#[test]
fn contract_04_def_restricted_hard_cut_rejected() {
    // The arc 198 substrate primitive :wat::core::def-restricted retires.
    // Post-stone: HARD-CUT rejection with structured retirement remedy.
    // At HEAD: form is live; works without rejection.
    let src = r#"
        (:wat::core::def-restricted :test::r
          :restricted-to [:test::]
          (:wat::core::fn [] -> :wat::core::i64 42))
    "#;
    let result = try_startup(src);
    assert!(
        result.is_err(),
        "`:wat::core::def-restricted` must be HARD-CUT-rejected post-stone (def + metadata-map is the only way); got Ok"
    );
}

// ─── C05: :wat::core::defn-restricted HARD-CUT-rejected ────────────────────────

#[test]
fn contract_05_defn_restricted_hard_cut_rejected() {
    // The arc 198 wat-source macro :wat::core::defn-restricted retires.
    // Per user direction: "def and defn are the only ways."
    // Post-stone: HARD-CUT rejection.
    // At HEAD: macro expands successfully.
    let src = r#"
        (:wat::core::defn-restricted :test::r
          :restricted-to [:test::]
          [] -> :wat::core::i64 42)
    "#;
    let result = try_startup(src);
    assert!(
        result.is_err(),
        "`:wat::core::defn-restricted` must be HARD-CUT-rejected post-stone (defn + metadata-map is the only way); got Ok"
    );
}

// ─── C06: rejection remedies name def / defn respectively ──────────────────────

#[test]
fn contract_06_rejection_remedies_name_replacements() {
    // The HARD-CUT rejections must include structured remedies per Stone 241.10's
    // apparatus, naming `:wat::core::def` (for def-restricted) and
    // `:wat::core::defn` (for defn-restricted) as the surviving forms,
    // via 8th + 9th RETIREMENT_TABLE entries.
    let src_def = r#"
        (:wat::core::def-restricted :test::r
          :restricted-to [:test::]
          (:wat::core::fn [] -> :wat::core::i64 42))
    "#;
    let msg_def = try_startup_display(src_def);
    assert!(
        msg_def.contains(":wat::core::def"),
        "def-restricted retirement remedy must name :wat::core::def; got:\n{}",
        msg_def
    );
    assert!(
        msg_def.contains("[replaces a retired form]"),
        "def-restricted remedy must carry '[replaces a retired form]' annotation; got:\n{}",
        msg_def
    );

    let src_defn = r#"
        (:wat::core::defn-restricted :test::r
          :restricted-to [:test::]
          [] -> :wat::core::i64 42)
    "#;
    let msg_defn = try_startup_display(src_defn);
    assert!(
        msg_defn.contains(":wat::core::defn"),
        "defn-restricted retirement remedy must name :wat::core::defn; got:\n{}",
        msg_defn
    );
    assert!(
        msg_defn.contains("[replaces a retired form]"),
        "defn-restricted remedy must carry '[replaces a retired form]' annotation; got:\n{}",
        msg_defn
    );
}
