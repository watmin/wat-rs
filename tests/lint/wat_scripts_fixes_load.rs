//! Gate — EVERY `.wat` under `wat-scripts/` must LOAD (parse + type-check) on the CURRENT runtime.
//!
//! `wat-scripts/` holds the kept refactor-tooling (`fixes/`), the showpiece demos, and library
//! helpers. None of it is frozen into the binary (unlike `wat/*.wat`, which fails the build on
//! drift) and none of it is loaded by any other test — so a substrate contract change (e.g. `first`
//! going Option->element under arc-047) rotted these scripts silently while the measured stdlib
//! usages got updated. A stale exemplar that no longer runs is a graveyard that reads like live
//! code (it trapped a prior session). This gate closes that blind spot: ALL wat must remain correct,
//! always — a `wat-scripts/` file that no longer type-checks goes RED here, so the rot cannot hide.
//!
//! `startup_from_source` + `FsLoader` (the disk loader cargo-wat uses) parses + type-checks the
//! whole world (all defns, including `:user::main`'s body, and any disk `load-file!` dependencies)
//! without running `main` — exactly the check that catches a dead idiom (`Option/expect`-over-`first`)
//! or a broken declaration form (the retired `:wat::core::Record::def`). It must use the SAME loader the
//! scripts run under (NOT `InMemoryLoader`), or it lies about scripts with relative `load-file!`.
//!
//! ## SHARDED — stride, not directory (excursus 003, stone D, 2026-09-23)
//!
//! This was ONE `#[test]` over the whole corpus: 425.736s isolated at 769 files, its budget raised
//! three times (45/90 -> 120/240 -> 300/600 -> 1500/3000), and 98% of the floor's wall-clock. The
//! per-file cost is FLAT (median 985 ms under `wat --check`, top 10 files = 1.4% of the total), so
//! a STRIDE split balances near-perfectly; a per-directory split does not (`scratch-pad/` alone is
//! 55% of the corpus). Shape copied from `every_wat_bad_fixture_actually_fails.rs`.
//!
//! ⛔ Every shard's name keeps `every_wat_scripts_file_loads_on_the_current_runtime` as a PREFIX.
//! Seventeen files cite the gate by that name, and nextest `test(...)` filters are SUBSTRING
//! matches, so each citation — `.config/nextest.toml`'s overrides included — stays true unedited.
//!
//! ⛔ IF A SHARD EVER NEEDS ITS BUDGET RAISED, RAISE `N_SHARDS` INSTEAD. The sizing arithmetic is at
//! [`N_SHARDS`]; the budget it is sized against is the override in `.config/nextest.toml`.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::loader::FsLoader;

/// How many stride shards the corpus is split across. DERIVED, not copied from the sibling's 16 —
/// the invariant (`.config/nextest.toml`'s own flake line): a shard's LOADED cost, isolated x 4.4
/// (the top of the recorded contention band), sits under 69% of its kill.
///
/// MEASURED 2026-09-23 (box load ~3.5, 8 cores — not quiet, so an over-estimate): at N=32 two
/// shards of 24 files ran 13.973s and 13.480s isolated = **0.57 s/file**, matching the DESIGN's
/// 0.554 s/file (425.736s / 769).
///
/// Sized to the DEFAULT 15s-warn / 30s-kill budget, so the gate carries NO `nextest.toml` override
/// in any profile (the three 1500s/3000s overrides are gone):
///
/// ```text
///   69% of the 30s kill ............................ 20.7s loaded
///   / 4.4 (top of the contention band) ............. 4.70s isolated
///   / 0.57 s/file .................................. 8 files per shard, at most
///   corpus today 770 -> N >= 97;  corpus doubled (1540, it did in five weeks) -> N >= 193
///   N = 200 -> ceil(770/200) = 4 files ~ 2.3s isolated ~ 10.0s loaded = 33% of the kill
///   the invariant breaks at 9 files/shard = 1601+ files under wat-scripts/
/// ```
///
/// Past ~1600 files, raise THIS number — never a budget.
const N_SHARDS: usize = 200;

/// The floor the walk must clear. 770 `.wat` under `wat-scripts/` on 2026-09-23 (445 on
/// 2026-09-01). Deliberately far under it: this catches a walk gone blind — a moved root, a renamed
/// directory — without rotting as the tree grows or when a scratch file is honestly deleted.
const CORPUS_FLOOR: usize = 200;

fn collect_wat(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display())) {
        let p = entry.expect("dir entry").path();
        if p.is_dir() {
            collect_wat(&p, out);
        } else if p.extension().is_some_and(|x| x == "wat") {
            out.push(p);
        }
    }
}

/// The whole corpus, sorted — every shard walks the same list, so the stride is deterministic.
fn corpus() -> Vec<PathBuf> {
    let mut entries = Vec::new();
    collect_wat(Path::new("wat-scripts"), &mut entries);
    entries.sort();

    // NON-VACUITY: a walk that comes back empty asserts nothing over nothing and reports PASS, and
    // every verdict downstream inherits that silence. The floor sits well under the 770 .wat
    // file(s) this walk finds today (driven 2026-09-23), so it catches a walk gone blind — a moved
    // root, a renamed directory — without rotting as the tree grows.
    assert!(
        entries.len() > CORPUS_FLOOR,
        "the wat-scripts load walk found only {} .wat file(s) — it is not \
         reaching the tree it claims to guard, so its green means nothing",
        entries.len()
    );
    entries
}

/// This shard's slice: every `N_SHARDS`-th file starting at `shard`. The slices over
/// `0..N_SHARDS` partition the corpus — `stride_slices_partition_the_corpus` proves it.
fn slice(entries: &[PathBuf], shard: usize) -> Vec<&PathBuf> {
    entries.iter().skip(shard).step_by(N_SHARDS).collect()
}

/// Load every file in this shard's slice; return one failure line per file that does not load.
fn check_shard(shard: usize) -> (usize, Vec<String>) {
    let entries = corpus();
    let mine = slice(&entries, shard);

    // NON-VACUITY: a shard whose slice is empty passes over nothing. That happens exactly when
    // N_SHARDS outgrows the corpus — the sharding arithmetic, not the tree, has gone wrong — and
    // it must RED rather than let a generated test report a green it never earned.
    assert!(
        !mine.is_empty(),
        "shard {shard}/{N_SHARDS} covers no files — the sharding arithmetic is wrong and this \
         test's green is vacuous"
    );

    let mut failures = Vec::new();
    for path in &mine {
        let rel = path.to_str().expect("utf8 path");
        // FsLoader (the disk loader cargo-wat uses) — NOT InMemoryLoader — so a script's relative
        // `(:wat::load-file! "../lib/…")` resolves against its own dir exactly as it does when run.
        // The gate must measure the way the script actually runs, or it lies (false positives).
        let src = std::fs::read_to_string(rel).unwrap_or_else(|e| panic!("read {rel}: {e}"));
        if let Err(e) = startup_from_source(&src, Some(rel), Arc::new(FsLoader)) {
            failures.push(format!("  {rel}\n      {e:?}"));
        }
    }
    (mine.len(), failures)
}

macro_rules! shards {
    ($($name:ident = $idx:expr;)*) => {
        $(
            #[test]
            fn $name() {
                let (checked, failures) = check_shard($idx);
                assert!(
                    failures.is_empty(),
                    "{} of {} wat-scripts/ files in shard {}/{N_SHARDS} do not load on the \
                     current runtime (rotted):\n{}",
                    failures.len(),
                    checked,
                    $idx,
                    failures.join("\n")
                );
            }
        )*

        /// Every generated shard index, emitted from the SAME list as the tests themselves, so the
        /// partition proof below cannot drift from what actually runs.
        const SHARD_INDICES: &[usize] = &[$($idx),*];
    };
}

shards! {
    every_wat_scripts_file_loads_on_the_current_runtime_shard_000 = 0;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_001 = 1;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_002 = 2;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_003 = 3;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_004 = 4;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_005 = 5;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_006 = 6;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_007 = 7;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_008 = 8;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_009 = 9;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_010 = 10;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_011 = 11;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_012 = 12;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_013 = 13;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_014 = 14;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_015 = 15;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_016 = 16;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_017 = 17;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_018 = 18;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_019 = 19;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_020 = 20;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_021 = 21;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_022 = 22;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_023 = 23;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_024 = 24;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_025 = 25;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_026 = 26;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_027 = 27;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_028 = 28;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_029 = 29;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_030 = 30;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_031 = 31;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_032 = 32;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_033 = 33;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_034 = 34;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_035 = 35;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_036 = 36;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_037 = 37;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_038 = 38;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_039 = 39;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_040 = 40;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_041 = 41;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_042 = 42;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_043 = 43;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_044 = 44;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_045 = 45;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_046 = 46;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_047 = 47;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_048 = 48;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_049 = 49;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_050 = 50;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_051 = 51;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_052 = 52;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_053 = 53;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_054 = 54;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_055 = 55;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_056 = 56;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_057 = 57;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_058 = 58;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_059 = 59;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_060 = 60;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_061 = 61;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_062 = 62;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_063 = 63;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_064 = 64;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_065 = 65;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_066 = 66;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_067 = 67;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_068 = 68;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_069 = 69;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_070 = 70;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_071 = 71;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_072 = 72;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_073 = 73;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_074 = 74;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_075 = 75;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_076 = 76;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_077 = 77;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_078 = 78;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_079 = 79;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_080 = 80;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_081 = 81;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_082 = 82;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_083 = 83;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_084 = 84;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_085 = 85;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_086 = 86;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_087 = 87;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_088 = 88;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_089 = 89;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_090 = 90;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_091 = 91;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_092 = 92;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_093 = 93;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_094 = 94;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_095 = 95;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_096 = 96;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_097 = 97;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_098 = 98;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_099 = 99;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_100 = 100;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_101 = 101;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_102 = 102;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_103 = 103;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_104 = 104;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_105 = 105;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_106 = 106;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_107 = 107;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_108 = 108;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_109 = 109;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_110 = 110;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_111 = 111;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_112 = 112;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_113 = 113;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_114 = 114;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_115 = 115;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_116 = 116;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_117 = 117;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_118 = 118;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_119 = 119;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_120 = 120;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_121 = 121;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_122 = 122;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_123 = 123;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_124 = 124;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_125 = 125;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_126 = 126;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_127 = 127;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_128 = 128;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_129 = 129;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_130 = 130;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_131 = 131;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_132 = 132;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_133 = 133;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_134 = 134;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_135 = 135;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_136 = 136;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_137 = 137;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_138 = 138;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_139 = 139;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_140 = 140;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_141 = 141;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_142 = 142;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_143 = 143;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_144 = 144;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_145 = 145;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_146 = 146;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_147 = 147;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_148 = 148;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_149 = 149;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_150 = 150;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_151 = 151;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_152 = 152;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_153 = 153;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_154 = 154;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_155 = 155;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_156 = 156;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_157 = 157;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_158 = 158;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_159 = 159;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_160 = 160;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_161 = 161;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_162 = 162;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_163 = 163;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_164 = 164;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_165 = 165;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_166 = 166;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_167 = 167;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_168 = 168;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_169 = 169;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_170 = 170;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_171 = 171;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_172 = 172;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_173 = 173;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_174 = 174;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_175 = 175;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_176 = 176;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_177 = 177;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_178 = 178;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_179 = 179;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_180 = 180;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_181 = 181;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_182 = 182;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_183 = 183;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_184 = 184;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_185 = 185;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_186 = 186;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_187 = 187;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_188 = 188;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_189 = 189;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_190 = 190;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_191 = 191;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_192 = 192;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_193 = 193;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_194 = 194;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_195 = 195;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_196 = 196;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_197 = 197;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_198 = 198;
    every_wat_scripts_file_loads_on_the_current_runtime_shard_199 = 199;
}

/// The split must not change WHAT is checked: the generated shards are exactly `0..N_SHARDS`, and
/// their slices cover every file in the corpus exactly once. Pure arithmetic over the real walk —
/// no file is loaded — so it costs nothing and reds on a mis-edited shard list or stride.
#[test]
fn every_wat_scripts_file_loads_on_the_current_runtime_stride_slices_partition_the_corpus() {
    assert_eq!(
        SHARD_INDICES,
        (0..N_SHARDS).collect::<Vec<_>>().as_slice(),
        "the generated shard list must be exactly 0..N_SHARDS — a missing index is a slice of \
         the corpus no test checks"
    );
    let entries = corpus();
    let mut seen: Vec<&PathBuf> = SHARD_INDICES
        .iter()
        .flat_map(|&s| slice(&entries, s))
        .collect();
    seen.sort();
    let all: Vec<&PathBuf> = entries.iter().collect();
    assert_eq!(
        seen, all,
        "the shard slices must cover every wat-scripts/ file exactly once"
    );
}
