//! ★ THE PORT CHECK — `native` vs `$oracle`, on the floor, on every sized grid axis.
//!
//! `run-axis.sh:289-296` names three pairings and says what each diagnoses:
//!
//! ```text
//!   oracle vs clara   MISMATCH  =>  the SPEC is wrong        (needs the JVM)
//!   native vs clara   MISMATCH  =>  the fast path is wrong   (needs the JVM — 23 grids ran it)
//!   oracle vs native  MISMATCH  =>  a PORT bug               (needs NOTHING but this binary)
//! ```
//!
//! Every sized axis already fires BOTH engines on the SAME staged session and emits both answers
//! (`:derived` and `:oracle-derived`) on one `#grid/Result` line. The third pairing therefore costs
//! one subprocess per axis and no JDK — and until this gate it was computed and thrown away, because
//! the only consumer was `run-axis.sh`, which no test invokes and CI cannot run.
//!
//! ## ⛔ WHY THIS IS NOT A DUPLICATE OF `wat_scripts_grid_axes_live.rs`
//!
//! That test appears to compare the two columns, and its comment says it does. **It cannot.**
//! `run_sized_axis` there first calls `skip_oracle_fire`, which rewrites
//! `:wat::rete::fire-rules$oracle`'s call site into a `FireOutcome::Fired` wrapping the ALREADY-FIRED
//! NATIVE session — so `ofired` IS `fired`, `:oracle-derived` is the native answer, and the
//! comparison is `X == X`. Measured 2026-09-03 on `min-finding [100 3]`: `:oracle-ns` falls from
//! **544,437,493 ns** (the interpreted oracle really running) to **5,608 ns** (a match on a
//! constructed value) under that rewrite. The rewrite is right for a LIVENESS test — it is asking
//! whether native runs — but it makes the oracle column a mirror. That gate's own span-overlap
//! guard (`derived_at == oracle_at`) defends the *parse* against reading one field twice; nothing
//! defended the *source* against firing one engine twice.
//!
//! **Driven, not merely read.** With D7's cure reverted (`git checkout 523152b31 --
//! src/rete/kernel/fire/pass/alpha.rs`) — a live silent fact-drop in the engine —
//! `grid_axes_run_and_derive_nonvacuously` as it stood at `daa92c3b0`, oracle comparison and all,
//! **passed green**. This gate reds on the same tree.
//!
//! **This gate runs each axis from its path on disk, byte-for-byte, with no rewrite of any kind**,
//! and asserts below that the source still contains the oracle verb — so a future rewrite that
//! neuters the pairing again goes red here instead of passing green.
//!
//! ## ⛔ EQUALITY IS SATISFIED BY ABSENCE — four guards, and their order matters
//!
//! An empty `:derived` compares equal to an empty `:oracle-derived` and reports agreement. This is
//! not hypothetical: driven 2026-09-03, `fanout` handed the two-element size `[20 5]` (it takes a
//! SINGLE number, `fanout.wat:5`) emits `:size [20] :derived [] :oracle-derived []` — it read the
//! first element, ignored the second, derived nothing, and the pairing "agreed". A gate that cannot
//! tell *they agree* from *there is nothing to disagree about* is the C16 defect rebuilt.
//!
//! So each axis clears FOUR guards, **in this order**:
//!
//! 1. **The echoed `:size` equals the size we sent.** This catches the arity mistake AT ITS CAUSE
//!    rather than at its symptom — `fanout [20 5]` fails here, naming the extra element.
//! 2. **Neither set is empty.**
//! 3. **The two sets are equal element-for-element**, and a mismatch prints BOTH SETS in full plus
//!    the symmetric difference. ⛔ NOT A COUNT: D7 produced a right-sized WRONG answer (`d_alpha`
//!    indexing elements that had moved under it), which a cardinality check passes.
//! 4. **The ORACLE set's cardinality equals the count derived from that axis's own SHAPE FORMULA**
//!    (the `derivation` column below re-derives every one from the axis's header; none is a number
//!    read off a run). This is the anti-vacuity instrument for the case guard 2 cannot see — a run
//!    that derives a few facts where the shape says many, both engines agreeing about the shrunken
//!    answer.
//!
//! ⛔ **THE ORDER OF 3 AND 4 IS ITSELF A FINDING, made under the D7 mutation.** Written the other
//! way round, reverting D7's cure reported *"derived 334 element(s), but this axis's own shape
//! predicts 600 — either the workload changed or the run is not the one this row describes"*:
//! every word a diagnosis of the TEST TABLE, for what was a silent fact-drop in the engine. An
//! anti-vacuity instrument must never pre-empt the correctness verdict, and it is checked on the
//! ORACLE column so a native-side defect cannot reach it at all.
//!
//! ## Discovery, not a list
//!
//! The directory is walked and the walk is held against `CORRECTNESS_SIZES` by exact set equality,
//! so a new sized axis cannot land without a deliberate size and a deleted one cannot vanish
//! quietly. The `where-*` expressivity corpus is a different instrument (no stdin, no
//! `#grid/Result`) and is covered against the oracle by
//! `wat_scripts_grid_axes_live::spec_equals_native_on_every_where_family`.
//!
//! ## Sizes are CORRECTNESS sizes, not the perf ladder
//!
//! `run-all.sh`'s LADDER is a published artifact and must not drift toward these, nor these toward
//! it. These are the smallest sizes that make each axis's derived set structurally interesting;
//! the whole gate runs in ~7 s.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// One row per sized (non-`where-*`) grid axis:
/// `(stem, correctness size, expected element count, derivation of that count from the axis's own
/// header formula)`.
///
/// The count is the ANTI-VACUITY instrument. Every one is re-derived from the shape the axis's own
/// doc-comment states — not read off a run — so a workload change that alters what an axis derives
/// reds this gate and names itself instead of quietly shrinking the corpus.
const CORRECTNESS_SIZES: &[(&str, &[i64], usize, &str)] = &[
    (
        "accum",
        &[10, 20],
        50,
        "size=[groups readings]; W=20>=1 makes min/max Some, so every group emits all FIVE derived \
         facts (count/sum/min/max/exists) — 10 groups * 5 = 50.",
    ),
    (
        "asym-join",
        &[100],
        200,
        "size=[items]; R1 derives B(k) from every input A(k) and R2's join derives C(k) for every k \
         once caught up — 2 * 100 = 200.",
    ),
    (
        "deep-cascade",
        &[5, 20],
        200,
        "size=[depth width]; the joins never drop anyone, so every seeded id survives every level — \
         2 * depth * width = 2 * 5 * 20 = 200.",
    ),
    (
        "fanout",
        &[500],
        400,
        "size=[items] — a SINGLE number; items = keys * F^2 with F fixed at 20, so items=500 gives \
         keys=1 and F^2 = 400 Pair facts. ⛔ Handed a TWO-element size this axis reads only the \
         first, derives [], and 'agrees' with the oracle on two empty sets — guard 1 (the echoed \
         :size) is what refuses that.",
    ),
    (
        "leading-exists",
        &[20],
        20,
        "size=[items]; every loc in [0,items) is asserted twice as Wind(loc) and the leading \
         :exists binds ONE token per DISTINCT loc — exactly [0..20), independent of the inert \
         S1..S6 cascade (which is the property this axis exists to hold).",
    ),
    (
        "min-finding",
        &[100, 3],
        49,
        "size=[stations threshold]; readings(loc) = loc mod 2T = loc mod 6, and a station activates \
         iff that is >= T=3, i.e. loc mod 6 in {3,4,5}. Over [0,100): 16 whole periods * 3 = 48, \
         plus loc 99 (99 mod 6 = 3) = 49.",
    ),
    (
        "negation",
        &[50],
        25,
        "size=[items]; Bad is seeded for even k and Ok fires for odd k — the 25 odd keys in [0,50).",
    ),
    (
        "neg-consumer",
        &[50],
        25,
        "size=[items]; same odd/even split as `negation`, with Final(k) :- Ok(k), Tag(k) the \
         positive consumer downstream of the negation — 25 odd keys in [0,50).",
    ),
    (
        "node-share",
        &[10, 20],
        20,
        "size=[rules items]; every k in [0,items) satisfies EXACTLY one of the N rules \
         (i == k mod N) by construction, so the derived Out set is items-many regardless of \
         `rules` — 20.",
    ),
    (
        "parametric-erasure",
        &[200],
        600,
        "size=[items]; ONE erased class `pe::Box` whose instances cycle i64 / String / Tag fillers \
         beside the uniformly-packable `pe::Plain`. Every key derives Hit, PlainHit and Pair — \
         3 * 200 = 600. ★ This is the axis that carries D7's shape; see its header.",
    ),
    (
        "strat-neg",
        &[3, 50],
        75,
        "size=[strata items]; S0 marks even k (25), S1 marks NOT-S0 i.e. odd k (25), S2 marks \
         NOT-S1 i.e. even k again (25) — 3 strata * 25 = 75.",
    ),
    (
        "user-reduce",
        &[5, 20],
        5,
        "size=[locs reads]; sum-of-squares over each location's non-empty reading vector emits \
         exactly ONE Agg fact per location — 5.",
    ),
];

/// The oracle verb every sized axis must still call. Held as a NAME, never as a form — an inlined
/// wat form in a test is its own lint (`no_inlined_wat_in_tests`), and a name is what we need.
const ORACLE_VERB: &str = ":wat::rete::fire-rules$oracle";

fn grid_dir() -> PathBuf {
    Path::new("wat-scripts/perf/grid").to_path_buf()
}

/// Every `.wat` directly under the grid whose stem is not `where-*`, sorted. Walked, never listed.
fn discover_sized_stems() -> Vec<String> {
    let dir = grid_dir();
    let mut stems: Vec<String> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .map(|entry| entry.expect("dir entry").path())
        .filter(|p| p.extension().is_some_and(|x| x == "wat"))
        .filter_map(|p| p.file_stem().and_then(|s| s.to_str()).map(str::to_string))
        .filter(|s| !s.starts_with("where-"))
        .collect();
    stems.sort();
    stems
}

/// Render a size vector as the EDN the axis reads on stdin.
///
/// Built with `push`, not a `"[{}]"` scaffold: `no_inlined_edn` flags any string literal whose
/// trimmed content opens with `[`, and cannot tell a format scaffold from a complete inlined EDN
/// vector. Delimiters as `char`s sidestep it and read the same.
fn size_edn(size: &[i64]) -> String {
    let mut s = String::new();
    s.push('[');
    s.push_str(
        &size
            .iter()
            .map(i64::to_string)
            .collect::<Vec<_>>()
            .join(" "),
    );
    s.push(']');
    s
}

/// Run one axis **from its path on disk, unmodified**, piping `size` on stdin.
///
/// ⛔ The absence of a rewrite here is the whole point — see this file's header. There is no temp
/// copy, so there is no place for a substitution to hide.
fn run_axis(stem: &str, size: &[i64]) -> (bool, String, String) {
    let bin = env!("CARGO_BIN_EXE_wat");
    let path = grid_dir().join(format!("{stem}.wat"));
    let mut child = Command::new(bin)
        .arg(&path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("spawn {bin} {}: {e}", path.display()));
    let payload = format!("{}\n", size_edn(size));
    child
        .stdin
        .take()
        .expect("child stdin")
        .write_all(payload.as_bytes())
        .unwrap_or_else(|e| panic!("write size {payload:?} to {stem}: {e}"));
    let output = child.wait_with_output().expect("wait for child");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// The one `#grid/Result` LINE, isolated before any field is read.
///
/// Parsing whole stdout would be wrong for `fanout`, which prints a `#fan/QuerySplit` timing line
/// FIRST and the `#grid/Result` line second.
fn grid_result_line(stdout: &str) -> Option<&str> {
    stdout.lines().find(|l| l.trim_start().starts_with("#grid/Result"))
}

/// Extract one `<key> #wat.core/PersistentVector [...]` bracket from the result line: its contents
/// and the byte offset the key matched at.
///
/// ⚠ THE KEY IS SPACE-DELIMITED, and the offsets are returned so the caller can prove the two keys
/// matched DIFFERENT spans. Both are cheap insurance against one field being read twice, which
/// would degrade the comparison to `X == X` — the shape of blindness this gate exists to remove,
/// one layer down.
///
/// ⛔ A CORRECTION, because the claim is in the tree twice and is wrong both times. The neighbouring
/// gate warns that `":oracle-derived"` CONTAINS `":derived"` so a plain `find` "can land inside the
/// oracle field". **It cannot**: the colon is part of the needle, and `:oracle-derived` holds
/// `-derived`, not `:derived`. Driven 2026-09-03 —
/// `grep -oP ':derived\s+(?:#wat\.core/PersistentVector\s+)?\K\[[^]]*\]'` (run-axis.sh:277, the
/// extraction this gate was told to reuse) returns exactly ONE match on a full `#grid/Result` line
/// and returns NOTHING when handed the `:oracle-derived` field alone. So the hazard is LATENT, not
/// live, and the delimiter and span check are what keep it latent — e.g. against a future
/// `:spec-derived`, or a `:derived` appearing where a leading space is the only separator.
///
/// Elements are plain i64s (no nested brackets), so the first `]` after the `[` closes it.
fn extract_vector_field(line: &str, key: &str) -> Option<(String, usize)> {
    let delimited = format!(" {key} ");
    let key_pos = line.find(&delimited)?;
    let after = &line[key_pos + delimited.len()..];
    let open = after.find('[')?;
    let close = after[open..].find(']')?;
    Some((after[open + 1..open + close].to_string(), key_pos))
}

fn elements(raw: &str) -> Vec<&str> {
    raw.split_whitespace().collect()
}

/// Elements present in `a` but not in `b`, rendered for a failure message.
fn only_in(a: &[&str], b: &[&str]) -> String {
    a.iter()
        .filter(|x| !b.contains(x))
        .copied()
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn every_grid_axis_native_matches_its_oracle() {
    let stems = discover_sized_stems();
    let on_disk: Vec<&str> = stems.iter().map(String::as_str).collect();
    // Both sides sorted: this is SET equality, not an ordering contract on the table above.
    let mut expected: Vec<&str> = CORRECTNESS_SIZES.iter().map(|(n, _, _, _)| *n).collect();
    expected.sort_unstable();
    assert_eq!(
        on_disk, expected,
        "the sized (non-`where-*`) grid axes on disk do not match CORRECTNESS_SIZES in \
         tests/rete/wat_scripts_grid_port_check.rs. A new axis must be given a correctness size AND \
         a derived-count derivation deliberately; a deleted one must be removed deliberately. This \
         assertion is also what makes an empty glob or a moved wat-scripts/perf/grid/ fail loudly \
         instead of passing vacuously."
    );

    let mut failures: Vec<String> = Vec::new();
    let mut compared = 0usize;

    for &(stem, size, want_n, derivation) in CORRECTNESS_SIZES {
        // ── guard 0: the axis still FIRES the oracle ────────────────────────────────────────
        // A source that no longer calls the oracle verb cannot produce an oracle column, and a
        // rewrite that redirects it (see the header) makes the column a mirror of native.
        let path = grid_dir().join(format!("{stem}.wat"));
        let src = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        if src.matches(ORACLE_VERB).count() == 0 {
            failures.push(format!(
                "  {stem}: source does NOT call {ORACLE_VERB} — there is no oracle answer to \
                 compare against, so this axis's port pairing is not running at all"
            ));
            continue;
        }

        let (ok, stdout, stderr) = run_axis(stem, size);
        if !ok {
            failures.push(format!(
                "  {stem} (size {size:?}): process did NOT exit successfully.\n      stdout: \
                 {stdout:?}\n      stderr: {stderr:?}"
            ));
            continue;
        }
        let Some(line) = grid_result_line(&stdout) else {
            failures.push(format!(
                "  {stem} (size {size:?}): exited 0 but stdout carries NO #grid/Result line.\n      \
                 stdout: {stdout:?}\n      stderr: {stderr:?}"
            ));
            continue;
        };

        // ── guard 1: the echoed :size is the size we SENT ───────────────────────────────────
        // Catches an arity mistake at its cause. `fanout [20 5]` echoes `:size [20]` — it read one
        // element and ignored the other — and then derives nothing and "agrees" with the oracle.
        match extract_vector_field(line, ":size") {
            None => {
                failures.push(format!(
                    "  {stem} (size {size:?}): #grid/Result carries no :size field to check the \
                     size against.\n      line: {line}"
                ));
                continue;
            }
            Some((echoed, _)) => {
                let sent: Vec<String> = size.iter().map(i64::to_string).collect();
                let got = elements(&echoed);
                if got != sent.iter().map(String::as_str).collect::<Vec<_>>() {
                    failures.push(format!(
                        "  {stem}: SIZE ARITY MISMATCH — this gate sent {} element(s) {sent:?} but \
                         the axis echoed {} element(s) {got:?}. The axis silently ignored part of \
                         the size; whatever it derived is not the workload this row asked for.\n\
                         \x20     line: {line}",
                        sent.len(),
                        got.len()
                    ));
                    continue;
                }
            }
        }

        let Some((native, native_at)) = extract_vector_field(line, ":derived") else {
            failures.push(format!(
                "  {stem} (size {size:?}): #grid/Result present but no :derived \
                 #wat.core/PersistentVector [...] shape in it.\n      line: {line}"
            ));
            continue;
        };
        let Some((oracle, oracle_at)) = extract_vector_field(line, ":oracle-derived") else {
            failures.push(format!(
                "  {stem} (size {size:?}): #grid/Result carries :derived but NO :oracle-derived — \
                 every sized axis fires the $oracle on the same staged session and must emit its \
                 answer, or this differential is silently not running.\n      line: {line}"
            ));
            continue;
        };
        if native_at == oracle_at {
            failures.push(format!(
                "  {stem}: :derived and :oracle-derived resolved to the SAME span — the comparison \
                 below would be X == X"
            ));
            continue;
        }

        // ── guard 2: non-vacuity — neither set may be EMPTY ─────────────────────────────────
        let n_elems = elements(&native);
        let o_elems = elements(&oracle);
        if n_elems.is_empty() || o_elems.is_empty() {
            failures.push(format!(
                "  {stem} (size {size:?}): VACUOUS — native has {} element(s), oracle has {}. An \
                 empty set compares EQUAL to an empty set and reports agreement while proving \
                 nothing. Expected {want_n}: {derivation}\n      line: {line}",
                n_elems.len(),
                o_elems.len()
            ));
            continue;
        }

        // ── guard 3: THE PORT PAIRING. oracle != native => a PORT bug. ──────────────────────
        //
        // ⛔ THIS RUNS BEFORE THE CARDINALITY GUARD, AND THE ORDER IS THE FINDING. Written the
        // other way round — cardinality first — the D7 mutation (revert `src/rete/kernel/fire/
        // pass/alpha.rs` to 523152b31) reported *"derived 334 element(s), but this axis's own
        // shape predicts 600 — either the workload changed or the run is not the one this row
        // describes"*. Every word of that is a diagnosis of the TEST TABLE. The actual event was
        // a silent fact-drop in the engine, and the anti-vacuity instrument had shadowed the
        // correctness verdict with a maintenance complaint. A port bug must report as a PORT BUG.
        if n_elems != o_elems {
            failures.push(format!(
                "  {stem} (size {size:?}): ⛔ PORT BUG — NATIVE AND $ORACLE DISAGREE.\n      \
                 native ({} elems): {}\n      oracle ({} elems): {}\n      only in native: {}\n   \
                 \x20  only in oracle: {}\n      (oracle vs native MISMATCH diagnoses a PORT bug — \
                 run-axis.sh:291-296. Not a count: a right-sized wrong answer is exactly what D7 \
                 produced.)",
                n_elems.len(),
                native.trim(),
                o_elems.len(),
                oracle.trim(),
                only_in(&n_elems, &o_elems),
                only_in(&o_elems, &n_elems),
            ));
            continue;
        }

        // ── guard 4: the cardinality the axis's own SHAPE FORMULA predicts ──────────────────
        //
        // Checked on the ORACLE column — the reference answer — and only AFTER the pairing has
        // already agreed, so this can never be the thing that speaks when an engine is at fault.
        // It is the anti-vacuity instrument, catching the case guard 2 cannot: a run that derives
        // a FEW facts where the shape says many (a size read at the wrong arity, a workload
        // silently narrowed) and whose two engines agree about the shrunken answer.
        if o_elems.len() != want_n {
            failures.push(format!(
                "  {stem} (size {size:?}): native and oracle AGREE, but on {} element(s) where \
                 this axis's own shape predicts {want_n}. The engines are not implicated — either \
                 the workload changed (update the derivation, do not update the number) or the \
                 run is not the one this row describes.\n      derivation: {derivation}\n      \
                 line: {line}",
                o_elems.len()
            ));
            continue;
        }
        compared += 1;
    }

    assert!(
        failures.is_empty(),
        "{} of {} grid axes FAILED the native-vs-oracle port check ({} axes agreed before the \
         failures below):\n{}",
        failures.len(),
        CORRECTNESS_SIZES.len(),
        compared,
        failures.join("\n")
    );
}
