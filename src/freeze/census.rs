//! The **boot census** — env-gated timing of the startup pipeline, **default off**.
//!
//! ⛔ THIS MODULE MEASURES. IT CHANGES NOTHING. Drawn under
//! `docs/excursus/2026/08/001-sns-sqs/boot-names-where-its-time-goes/DESIGN.md`, whose first
//! trap-door is "do not optimise anything", because `3f159b0c5` is a commit whose whole body is a
//! retraction for *"describing a mechanism and prescribing a remedy in the same breath, having
//! measured neither"* — on this exact subject. Its closing line is the rule: **a prescription is a
//! claim.** So this file adds a reading and nothing else.
//!
//! # What it answers
//!
//! `FINDING-boot-is-99-percent-stdlib.md` established WHERE boot goes (99.5 % of a `wat` process's
//! startup is re-deriving the baked stdlib) and left open WHICH PHASE owns it. Two independent
//! readings pointed at macro expansion; both were inferences from marginal cost, not a profile.
//! This is the profile.
//!
//! # The knob
//!
//! ```text
//! WAT_BOOT_CENSUS=phases   per-PHASE breakdown only      (~40 Instant::now() pairs per boot)
//! WAT_BOOT_CENSUS=files    phases + per-MANIFEST-ENTRY   (one pair per top-level stdlib form)
//! (unset / anything else)  OFF — not one clock is read
//! ```
//!
//! Read **once** per process through a `OnceLock`, mirroring
//! `WAT_STARTUP_HANDSHAKE_DEADLINE_MS` (`src/intrinsic/program.rs`): a diagnostic that changes
//! mid-run is a different bug. An unrecognised value is OFF **with one line on stderr** — same
//! split as that precedent, and for the same reason: it must not fail (this is on the path of every
//! spawned test program), and it must not read as deliberate.
//!
//! ## ⚠ THE TWO LEVELS EXIST SO THE INSTRUMENT CAN REPORT ITS OWN DISTORTION
//!
//! `[[feedback_the_measurement_contains_the_measurer]]`. `phases` wraps ~40 whole passes, so its
//! own cost is ~40 clock reads against a 430 ms boot — unmeasurable. `files` reads a clock twice
//! per **top-level stdlib form** (thousands), and that IS measurable. Running both and diffing the
//! totals is how the report states which numbers it distorts, rather than asserting it doesn't.
//!
//! # ⛔ THE PHASES ARE THE ONES THE LOADER ACTUALLY HAS
//!
//! The DESIGN guessed "parse / expand / check / freeze" and told the reader not to trust it. The
//! real pipeline is `freeze::startup_from_source` → `startup_from_forms` →
//! `startup_from_forms_post_config` → `freeze::env::build_env`, and `build_env` is where the seams
//! are: it is a numbered sequence of ~26 named calls (steps 3a–7.8). [`PHASE_ORDER`] lists every
//! leaf exactly as that code runs them. **Leaves only — nothing nests**, so the sum is the
//! accounted total and no reading is double-counted. Work inside `build_env` that is between two
//! named calls is deliberately NOT wrapped; it lands in the report's unaccounted remainder, which
//! is the honest place for it.
//!
//! # ⛔ WHERE THERE IS NO SEAM, AND THAT IS A FINDING
//!
//! `stdlib_forms()` (`src/load/stdlib.rs`) is the **last** place in the whole boot that knows which
//! manifest entry a form came from: it parses 55 files in a loop and returns ONE flat
//! `Vec<WatAST>`. Every later pass — defmacro registration, expansion, type registration, define
//! registration — consumes that flat vector and has no file boundary at all. So a per-manifest-entry
//! curve past parse cannot come from a phase boundary; it has to be reconstructed from each
//! top-level form's `Span::file`, which survives. That is what [`file_guard`] does, and it is why
//! the per-file rows are per *pass* rather than per *phase*: five passes iterate top-level stdlib
//! forms, and each one stamps the file its current form came from.
//!
//! # Reconciliation
//!
//! The report prints three totals, not one:
//!
//! - **accounted** — the sum of the leaf phases.
//! - **pipeline wall** — first census call → report, monotonic. `pipeline − accounted` is work
//!   inside the pipeline that no phase name covers.
//! - **since process boot** — `crate::time::process_boot_instant()` (primed at the earliest
//!   wat-controlled point in `distribution::run_with_args`) → report. Minus the pipeline wall,
//!   that is everything before step 1: batteries, argv, the entry-file read.
//!
//! A large remainder in either gap is itself a finding — it means the cost is somewhere nobody has
//! a name for.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

/// The env var that arms the census. Named for the census, and it reaches nothing else.
pub const BOOT_CENSUS_ENV: &str = "WAT_BOOT_CENSUS";

/// How much the census measures. `Off` is the default and reads no clock at all.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Mode {
    /// Default. Every [`phase`] call is exactly `f()`; [`file_guard`] returns `None`.
    Off,
    /// Per-PHASE leaf timings. ~40 clock pairs per boot.
    Phases,
    /// Per-phase **and** per-manifest-entry attribution. Thousands of clock pairs per boot;
    /// its own overhead is measurable, which is the point (see the module header).
    Files,
}

impl Mode {
    /// Is any clock read at all?
    fn on(self) -> bool {
        !matches!(self, Mode::Off)
    }
}

/// Parse the env var's text. Split out from [`mode`] so the wording and the accepted
/// spellings are testable without an environment — the same split
/// `ring_census_text`/`ring_census` uses in `src/comms/process.rs`.
///
/// `Err(raw)` means "unrecognised": the caller warns once and stays OFF.
fn mode_from_raw(raw: &str) -> Result<Mode, ()> {
    match raw.trim() {
        "" | "0" | "off" | "false" => Ok(Mode::Off),
        "1" | "phases" | "phase" | "on" | "true" => Ok(Mode::Phases),
        "files" | "file" | "2" => Ok(Mode::Files),
        _ => Err(()),
    }
}

/// Read once per process, never per phase.
static MODE: OnceLock<Mode> = OnceLock::new();

/// This process's census mode, reading the environment at most once.
///
/// Absent → `Off`, silently. **Unrecognised → `Off`, with ONE line on stderr**, naming the
/// variable, the offending text and the accepted spellings — the
/// `WAT_STARTUP_HANDSHAKE_DEADLINE_MS` contract, for the same two reasons stated there.
pub fn mode() -> Mode {
    *MODE.get_or_init(|| {
        let Ok(raw) = std::env::var(BOOT_CENSUS_ENV) else {
            return Mode::Off;
        };
        mode_from_raw(&raw).unwrap_or_else(|()| {
            eprintln!("{}", unrecognised_mode_text(&raw));
            Mode::Off
        })
    })
}

/// The one stderr line an unrecognised value earns. Split from [`mode`] so the wording is
/// under test.
fn unrecognised_mode_text(raw: &str) -> String {
    format!(
        "wat: {BOOT_CENSUS_ENV}={raw:?} is not a census mode — expected `phases` \
         (per-phase breakdown) or `files` (per-phase + per-manifest-entry attribution). \
         The boot census stays OFF."
    )
}

/// Force the census mode for a test, the way `crate::time::set_process_boot_instant` exists for
/// tests to inject a known boot instant. Returns `false` if the mode was already resolved in this
/// process (env read, or an earlier force) — a caller that needs the mode it asked for must
/// assert on this rather than assume.
///
/// ⚠ `pub` only because an integration test in `tests/` is a separate crate and the census's
/// own end-to-end proof — "arming it actually attributes real boot work to real manifest
/// entries" — cannot be made from inside `src/`. Nothing in the substrate calls it.
pub fn force_mode(m: Mode) -> bool {
    MODE.set(m).is_ok()
}

// ─── Phase names — the seams the loader ACTUALLY has ──────────────────────────────────────
//
// Numbered with the step numbers `freeze.rs` / `freeze/env.rs` already use in their own
// comments, so a row in the report can be found in the source by grep. LEAVES ONLY.

/// Step 1 — `parse_all_with_file` over the entry source (`startup_from_source`).
pub(crate) const P_ENTRY_PARSE: &str = "1    entry-parse";
/// Step 2 — `collect_entry_file`, the config pass (`startup_from_forms`).
pub(crate) const P_CONFIG: &str = "2    config-pass";
/// Step 3 — `resolve_loads`, recursive `load!` resolution.
pub(crate) const P_RESOLVE_LOADS: &str = "3    resolve-loads";
/// Step 3a — `stdlib_forms()`: parse all 55 baked manifest entries. **The last seam that knows
/// which file a form came from.**
pub(crate) const P_STDLIB_PARSE: &str = "3a   stdlib-parse";
/// Excursus 001 Tier A — the boot cache's READ path: the whole stdlib derivation (`3a` parse,
/// `4` stdlib-defmacro/kwargs-companions/stdlib-expand, `5` typeenv-with-builtins/
/// stdlib-types-register, `6` stdlib-defines-register, `6a` defclause-stub-preregister,
/// `6b` stdlib-runtime-def-filter) replaced by one file read + decode. When this leaf has a
/// reading, every one of those has none — that is what "elided" looks like in the report.
pub(crate) const P_CACHE_LOAD: &str = "3a-6b boot-cache-load";
/// The WRITE path — first boot after a rebuild only, and the bill row 12 asks for.
pub(crate) const P_CACHE_STORE: &str = "7.9  boot-cache-store";
/// Step 3b — `extract_rete_defn_names` + `rewrite_rete_defn_heads` (user forms only).
pub(crate) const P_RETE_DEFN_SCAN: &str = "3b   rete-defn-scan";
/// Step 4 — `register_stdlib_defmacros`.
pub(crate) const P_STDLIB_DEFMACRO: &str = "4    stdlib-defmacro-register";
/// Step 4 — `register_defmacros` (user).
pub(crate) const P_USER_DEFMACRO: &str = "4    user-defmacro-register";
/// Step 4 — `register_aggregate_kwargs_companions` (arc 294 item 9a class closure).
pub(crate) const P_KWARGS_COMPANIONS: &str = "4    kwargs-companions";
/// Step 4 — `preregister_acronyms` into the expand-time `SymbolTable`.
pub(crate) const P_ACRONYMS_MACRO: &str = "4    acronym-preregister(macro)";
/// Step 4 — `expand_all_with(… Privilege::Stdlib)`: **macro expansion of the whole stdlib.**
pub(crate) const P_STDLIB_EXPAND: &str = "4    stdlib-expand";
/// Step 4 — `expand_all` over user forms.
pub(crate) const P_USER_EXPAND: &str = "4    user-expand";
/// Step 4b — the bare-legacy + arc-170 legacy-callsite walkers (user forms only).
pub(crate) const P_LEGACY_WALKERS: &str = "4b   legacy-walkers(user)";
/// Step 5 — `TypeEnv::with_builtins()`.
pub(crate) const P_TYPEENV_BUILTINS: &str = "5    typeenv-with-builtins";
/// Step 5 — `register_stdlib_types`.
pub(crate) const P_STDLIB_TYPES: &str = "5    stdlib-types-register";
/// Step 5 — `register_types_with_acronyms` (user).
pub(crate) const P_USER_TYPES: &str = "5    user-types-register";
/// Step 5 — `validate_aggregate_containment` (arc 293.W).
pub(crate) const P_CONTAINMENT: &str = "5    aggregate-containment";
/// Step 6 — `register_stdlib_defines`.
pub(crate) const P_STDLIB_DEFINES: &str = "6    stdlib-defines-register";
/// Step 6a — the `preregister_stdlib_defclause_stub` loop over the stdlib residue.
pub(crate) const P_DEFCLAUSE_STUBS: &str = "6a   defclause-stub-preregister";
/// Step 6b — the `stdlib_runtime_def_forms` filter.
pub(crate) const P_RUNTIME_DEF_FILTER: &str = "6b   stdlib-runtime-def-filter";
/// Step 6 — `register_defines` (user).
pub(crate) const P_USER_DEFINES: &str = "6    user-defines-register";
/// Steps 6a–6.9 — struct / enum / newtype / aggregate accessor / type-predicate codegen. One
/// leaf because all five walk the SAME fully-populated `TypeEnv`, stdlib types included.
pub(crate) const P_AUTO_METHODS: &str = "6a-9 auto-method-codegen";
/// Step 6.8 — the `inventory` `RestrictionEntry` drain.
pub(crate) const P_RESTRICTION_DRAIN: &str = "6.8  restriction-entry-drain";
/// Step 6.96 — `preregister_acronyms` into the runtime `SymbolTable`.
pub(crate) const P_ACRONYMS_RUNTIME: &str = "6.96 acronym-preregister(runtime)";
/// Step 6.97 — `symbols.types_insert(Arc::new(types.clone()))`: a full `TypeEnv` clone.
pub(crate) const P_TYPEENV_ATTACH: &str = "6.97 typeenv-clone-attach";
/// Step 7 — `normalize_symbol_refs` (user residue).
pub(crate) const P_NORMALIZE: &str = "7    normalize-symbol-refs";
/// Step 7 — `resolve_references` (user residue; error deferred, arc 278).
pub(crate) const P_RESOLVE_REFS: &str = "7    resolve-references";
/// Step 7.6 — `register_stdlib_runtime_defs`.
pub(crate) const P_STDLIB_RUNTIME_DEFS: &str = "7.6  stdlib-runtime-defs-register";
/// Step 7.7 — `preregister_extend_type_in_do_let` over the user residue.
pub(crate) const P_EXTEND_TYPE_PREREG: &str = "7.7  extend-type-preregister(user)";
/// Step 7.8 — the `inventory` `FreezeValidator` drain (the rete `defrule` wall, today).
pub(crate) const P_FREEZE_VALIDATORS: &str = "7.8  freeze-validator-drain";
// ─── Step 8, partitioned — and the partition is a FINDING, not a convenience ───────────────
//
// ⭐ `check_program(&bundle.residue, …)` takes the USER residue as its `forms` argument, so reading
// the call site suggests it costs what the user's program costs. IT DOES NOT. Four of its passes
// never look at `forms` at all — they sweep `sym.function_values()` / `sym.functions_iter()`, which
// by step 8 holds every function the STDLIB registered. Measured on a one-line user program, step 8
// was 130–139 ms, ~32 % of boot. The whole of that is stdlib work, and the argument name hid it.
//
// So step 8 is recorded as SIX leaves rather than one, partitioning `check_program` exactly, and
// each leaf's name says whose forms it walks. ⛔ Their sum is step 8; there is no `P_CHECK` row to
// add to it. `check_program` is also called outside boot (the REPL turn, `--check`); these leaves
// accumulate there too, which is why the report prints hit counts.

/// Step 8 — `CheckEnv::from_symbols`: build the check environment from the symbol table.
pub(crate) const P_CHECK_ENV: &str = "8a   check:env-from-symbols";
/// Step 8 — five retired-syntax walkers over **every fn body in the symbol table**.
pub(crate) const P_CHECK_LEGACY_SWEEP: &str = "8b   check:retired-syntax(ALL fns)";
/// Step 8 — `walk_for_restricted_call` over **every fn body in the symbol table**.
pub(crate) const P_CHECK_RESTRICTED: &str = "8c   check:restricted-call(ALL fns)";
/// Step 8 — `validate_def_position*` over the user forms **and every fn body**.
pub(crate) const P_CHECK_DEF_POS: &str = "8d   check:def-position(ALL fns)";
/// Step 8 — the sequential `check_form` loop. **The only pass whose cost is the user program's.**
pub(crate) const P_CHECK_FORMS: &str = "8e   check:form-loop(user residue)";
/// Step 8 — `check_function_body` over **every fn in the symbol table**: full type inference on
/// every stdlib body, every boot.
pub(crate) const P_CHECK_BODIES: &str = "8f   check:body-infer(ALL fns)";
/// Step 8 — `check_impls_completeness` over the subtype edges.
pub(crate) const P_CHECK_IMPLS: &str = "8g   check:impls-completeness";
/// Step 9 — `FrozenWorld::freeze`.
pub(crate) const P_FREEZE: &str = "9    frozen-world-freeze";

/// Every leaf phase, in pipeline order. **The report prints in this order**, so a run is
/// comparable to any other run, and `every_recorded_phase_is_in_the_order` (below) makes a
/// phase name that is recorded but not listed a RED rather than an invisible omission —
/// `[[feedback_gate_on_the_property_not_the_path]]`.
pub(crate) const PHASE_ORDER: &[&str] = &[
    P_ENTRY_PARSE,
    P_CONFIG,
    P_RESOLVE_LOADS,
    P_CACHE_LOAD,
    P_STDLIB_PARSE,
    P_RETE_DEFN_SCAN,
    P_STDLIB_DEFMACRO,
    P_USER_DEFMACRO,
    P_KWARGS_COMPANIONS,
    P_ACRONYMS_MACRO,
    P_STDLIB_EXPAND,
    P_USER_EXPAND,
    P_LEGACY_WALKERS,
    P_TYPEENV_BUILTINS,
    P_STDLIB_TYPES,
    P_USER_TYPES,
    P_CONTAINMENT,
    P_STDLIB_DEFINES,
    P_DEFCLAUSE_STUBS,
    P_RUNTIME_DEF_FILTER,
    P_USER_DEFINES,
    P_AUTO_METHODS,
    P_RESTRICTION_DRAIN,
    P_ACRONYMS_RUNTIME,
    P_TYPEENV_ATTACH,
    P_NORMALIZE,
    P_RESOLVE_REFS,
    P_STDLIB_RUNTIME_DEFS,
    P_EXTEND_TYPE_PREREG,
    P_FREEZE_VALIDATORS,
    P_CACHE_STORE,
    P_CHECK_ENV,
    P_CHECK_LEGACY_SWEEP,
    P_CHECK_RESTRICTED,
    P_CHECK_DEF_POS,
    P_CHECK_FORMS,
    P_CHECK_BODIES,
    P_CHECK_IMPLS,
    P_FREEZE,
];

// ─── Per-pass labels for the per-manifest-entry attribution ───────────────────────────────
//
// Five passes iterate top-level stdlib forms. `expand` and `type-register` are SHARED with the
// user path (one `expand_all_with`, one `register_types_impl`), which is exactly why these are
// per-PASS labels rather than per-phase: the file column separates stdlib from user for free.

/// `stdlib_forms()` — the only genuine per-file seam.
pub(crate) const F_PARSE: &str = "parse";
/// `register_stdlib_defmacros`'s loop.
pub(crate) const F_DEFMACRO: &str = "defmacro-reg";
/// `expand_all_with`'s loop (stdlib **and** user; the file column tells them apart).
pub(crate) const F_EXPAND: &str = "expand";
/// `register_types_impl`'s loop (stdlib **and** user).
pub(crate) const F_TYPES: &str = "type-reg";
/// `register_stdlib_defines`'s loop.
pub(crate) const F_DEFINES: &str = "define-reg";

/// Every per-pass label, in pipeline order — same gate as [`PHASE_ORDER`].
pub(crate) const PASS_ORDER: &[&str] = &[F_PARSE, F_DEFMACRO, F_EXPAND, F_TYPES, F_DEFINES];

// ─── The accumulator ─────────────────────────────────────────────────────────────────────

/// One reading: total time and how many times it fired.
#[derive(Copy, Clone, Default, PartialEq, Eq, Debug)]
pub(crate) struct Reading {
    pub(crate) total: Duration,
    pub(crate) hits: u64,
}

/// What the census has accumulated so far. `pub(crate)` fields — this is a diagnostic
/// accumulator, not an invariant-bearing type.
#[derive(Default)]
pub(crate) struct Table {
    /// Leaf phase → reading.
    pub(crate) phases: HashMap<&'static str, Reading>,
    /// (per-pass label, manifest entry path) → reading.
    pub(crate) files: HashMap<(&'static str, String), Reading>,
    /// First census call in this process, monotonic. `None` until the first record.
    pub(crate) started: Option<Instant>,
}

fn table() -> &'static Mutex<Table> {
    static TABLE: OnceLock<Mutex<Table>> = OnceLock::new();
    TABLE.get_or_init(|| Mutex::new(Table::default()))
}

fn with_table<T>(f: impl FnOnce(&mut Table) -> T) -> T {
    let mut g = table().lock().unwrap_or_else(|e| e.into_inner());
    if g.started.is_none() {
        g.started = Some(Instant::now());
    }
    f(&mut g)
}

/// Record one leaf-phase reading. `pub(crate)` so the report's own tests can build a table
/// without an environment or a boot.
pub(crate) fn record_phase(name: &'static str, d: Duration) {
    with_table(|t| {
        let e = t.phases.entry(name).or_default();
        e.total += d;
        e.hits += 1;
    });
}

/// Record one per-manifest-entry reading.
pub(crate) fn record_file(pass: &'static str, file: &str, d: Duration) {
    with_table(|t| {
        let e = t.files.entry((pass, file.to_string())).or_default();
        e.total += d;
        e.hits += 1;
    });
}

/// Time one leaf phase of the startup pipeline.
///
/// **When the census is off this is exactly `f()`** — one relaxed read of an already-initialised
/// `OnceLock`, no clock, no lock, no allocation. `#[inline]` so the off path is the bare call.
#[inline]
pub(crate) fn phase<T>(name: &'static str, f: impl FnOnce() -> T) -> T {
    if !mode().on() {
        return f();
    }
    let t0 = Instant::now();
    let out = f();
    record_phase(name, t0.elapsed());
    out
}

/// A live leaf-phase measurement; records on drop. Sibling of [`phase`] for the case a closure
/// cannot wrap: a span that is not one expression, because the code in the middle needs a mutable
/// borrow the closure would capture. `check_program`'s step-8e span is the one such site — its
/// defclause pre-pass and its `check_form` loop both mutate `env` and are separated by a block
/// that the census must NOT include, so the span is bracketed by this guard and an explicit
/// `drop`.
pub(crate) struct PhaseGuard {
    name: &'static str,
    t0: Instant,
}

impl Drop for PhaseGuard {
    fn drop(&mut self) {
        record_phase(self.name, self.t0.elapsed());
    }
}

/// Open a leaf-phase measurement that ends when the returned guard is dropped. `None` when the
/// census is off, so the off path is one bool test.
#[inline]
pub(crate) fn pass_guard(name: &'static str) -> Option<PhaseGuard> {
    if !mode().on() {
        return None;
    }
    Some(PhaseGuard {
        name,
        t0: Instant::now(),
    })
}

/// A live per-manifest-entry measurement; records on drop, so an early `?` out of the loop body
/// still books the time it spent.
pub(crate) struct FileGuard {
    pass: &'static str,
    file: String,
    t0: Instant,
}

impl Drop for FileGuard {
    fn drop(&mut self) {
        record_file(self.pass, &self.file, self.t0.elapsed());
    }
}

/// Open a per-manifest-entry measurement for the form about to be processed, attributing it to
/// `file` — which the caller reads off the form's own `Span`, because past
/// [`P_STDLIB_PARSE`] that span is the ONLY thing left that knows which manifest entry a form
/// came from (see the module header).
///
/// `None` unless `WAT_BOOT_CENSUS=files`, so `Mode::Phases` and `Mode::Off` pay one bool test
/// and never touch the clock, the lock, or the allocator.
#[inline]
pub(crate) fn file_guard(pass: &'static str, file: &str) -> Option<FileGuard> {
    if mode() != Mode::Files {
        return None;
    }
    Some(FileGuard {
        pass,
        file: file.to_string(),
        t0: Instant::now(),
    })
}

/// A copy of the accumulated readings plus the pipeline anchor — what the report renders from.
pub(crate) struct Snapshot {
    pub(crate) phases: HashMap<&'static str, Reading>,
    pub(crate) files: HashMap<(&'static str, String), Reading>,
    /// First census call in this process, monotonic; `None` if nothing was ever recorded.
    pub(crate) started: Option<Instant>,
}

/// Snapshot the accumulated table. Used by the report and by the census's own tests.
pub(crate) fn snapshot() -> Snapshot {
    with_table(|t| Snapshot {
        phases: t.phases.clone(),
        files: t.files.clone(),
        started: t.started,
    })
}

// ─── The report ──────────────────────────────────────────────────────────────────────────

/// Print the census for the FIRST completed startup in this process, once.
///
/// **First only, and deliberately.** `startup_from_forms_post_config` is re-entered by every
/// sandbox, every `spawn-process` child and every REPL turn; those freezes re-derive the same
/// stdlib and would bury the boot reading in repeats. The latch means the accumulators keep
/// growing after the report but nothing reads them again — harmless, and stated rather than
/// hidden. A boot that FAILS never reaches here: the census reports a successful freeze.
pub(crate) fn report() {
    if !mode().on() {
        return;
    }
    static REPORTED: OnceLock<()> = OnceLock::new();
    if REPORTED.set(()).is_err() {
        return;
    }
    let s = snapshot();
    let pipeline = s.started.map(|t| t.elapsed());
    let since_boot = (chrono::Utc::now() - crate::time::process_boot_instant())
        .to_std()
        .ok();
    eprint!("{}", report_text(mode(), &s.phases, &s.files, pipeline, since_boot));
}

/// Render the census's CURRENTLY accumulated table — the same text [`report`] prints, returned
/// instead of emitted.
///
/// ⚠ `pub` for one reason: the census's own end-to-end proof ("an armed boot really does attribute
/// real time to real manifest entries") can only be made from an integration test, which is a
/// separate crate and cannot see `pub(crate)` internals, and `report` writes to the real fd 2 where
/// a test cannot read it. Nothing in the substrate calls this.
pub fn rendered_report() -> String {
    let s = snapshot();
    report_text(
        mode(),
        &s.phases,
        &s.files,
        s.started.map(|t| t.elapsed()),
        (chrono::Utc::now() - crate::time::process_boot_instant()).to_std().ok(),
    )
}

/// The three numbers a test needs to prove the armed instrument COUNTED, rather than merely
/// printed a table of zeroes: `(accounted total, distinct manifest entries attributed, file_guard
/// open/close pairs)`. `pub` for the same reason as [`rendered_report`].
pub fn totals() -> (Duration, usize, u64) {
    with_table(|t| {
        (
            t.phases.values().map(|r| r.total).sum(),
            t.files
                .keys()
                .map(|(_, f)| f.as_str())
                .collect::<std::collections::HashSet<_>>()
                .len(),
            t.files.values().map(|r| r.hits).sum(),
        )
    })
}

/// The report's wording, for GIVEN readings. Split from [`report`] for exactly the reason
/// `ring_census_text` is split from `ring_census` in `src/comms/process.rs`: the numbers vary
/// per run, the wording must not, and only a pure function can be asserted on.
pub(crate) fn report_text(
    mode: Mode,
    phases: &HashMap<&'static str, Reading>,
    files: &HashMap<(&'static str, String), Reading>,
    pipeline: Option<Duration>,
    since_boot: Option<Duration>,
) -> String {
    let ms = |d: Duration| d.as_secs_f64() * 1000.0;
    let accounted: Duration = phases.values().map(|r| r.total).sum();
    let mut out = String::new();

    out.push_str(&format!(
        "\n── wat boot census ─────────────────────────────── {BOOT_CENSUS_ENV}={} ──\n",
        match mode {
            Mode::Off => "off",
            Mode::Phases => "phases",
            Mode::Files => "files",
        }
    ));
    out.push_str(
        "   LEAF phases, nothing nested, in pipeline order — the sum IS the accounted total.\n\
         \x20  A phase with 0 hits did not run in this boot (a user-side pass on a stdlib-only\n\
         \x20  freeze, say); it is still listed, because an absent row and a zero row are\n\
         \x20  different facts.\n\n",
    );
    out.push_str("     phase                                 ms      %acc   hits\n");
    for name in PHASE_ORDER {
        let r = phases.get(name).copied().unwrap_or_default();
        let pct = if accounted.is_zero() {
            0.0
        } else {
            100.0 * r.total.as_secs_f64() / accounted.as_secs_f64()
        };
        out.push_str(&format!(
            "     {name:<34} {:>8.2}  {pct:>6.2}  {:>5}\n",
            ms(r.total),
            r.hits
        ));
    }
    out.push_str(&format!(
        "\n     {:<34} {:>8.2}\n",
        "ACCOUNTED (sum of leaves)",
        ms(accounted)
    ));

    // Any phase recorded but not in PHASE_ORDER would be invisible above. Say so loudly
    // rather than silently dropping it — the report must not be able to hide a reading.
    let mut stray: Vec<&&str> = phases.keys().filter(|k| !PHASE_ORDER.contains(k)).collect();
    stray.sort();
    if !stray.is_empty() {
        out.push_str(&format!(
            "     ⛔ {} phase name(s) recorded but ABSENT from PHASE_ORDER, so excluded from\n\
             \x20       every row above: {:?}. This is an instrument bug — the accounted total\n\
             \x20       is WRONG by their sum.\n",
            stray.len(),
            stray
        ));
    }

    match (pipeline, since_boot) {
        (Some(p), Some(b)) => {
            let unnamed = p.saturating_sub(accounted);
            let pre = b.saturating_sub(p);
            out.push_str(&format!(
                "     {:<34} {:>8.2}\n     {:<34} {:>8.2}   ← inside the pipeline, no phase names it\n\
                 \x20    {:<34} {:>8.2}\n     {:<34} {:>8.2}   ← batteries, argv, entry-file read\n",
                "PIPELINE WALL (1st census call→here)", ms(p),
                "unaccounted, in-pipeline", ms(unnamed),
                "SINCE PROCESS BOOT", ms(b),
                "pre-pipeline (boot→step 1)", ms(pre),
            ));
        }
        _ => out.push_str(
            "     ⚠ no pipeline / boot anchor available — reconciliation not printed.\n",
        ),
    }

    if !files.is_empty() {
        out.push_str(&format!(
            "\n   PER-MANIFEST-ENTRY, {} pass(es) × file. ⚠ THE PASSES BELOW ARE NOT THE PHASES\n\
             \x20  ABOVE: past stdlib-parse the form vector is FLAT and carries no file boundary,\n\
             \x20  so these rows are reconstructed from each top-level form's Span::file. Two of\n\
             \x20  the passes (expand, type-reg) are shared with the user path — the file column\n\
             \x20  is what separates them.\n\n",
            PASS_ORDER.len()
        ));
        // `forms` is the EXPAND pass's hit count for the file — how many top-level forms it
        // contributed. It is printed because "this file is expensive" and "this file is big" are
        // different facts, and ms/form is what separates them: a file with four forms and 35 ms
        // is a different problem from one with four hundred.
        let mut per_file: HashMap<&str, (Duration, [Duration; 5], u64)> = HashMap::new();
        for ((pass, file), r) in files {
            let slot = PASS_ORDER.iter().position(|p| p == pass);
            let e = per_file
                .entry(file.as_str())
                .or_insert((Duration::ZERO, [Duration::ZERO; 5], 0));
            e.0 += r.total;
            if let Some(i) = slot {
                e.1[i] += r.total;
            }
            if *pass == F_EXPAND {
                e.2 += r.hits;
            }
        }
        let files_total: Duration = per_file.values().map(|v| v.0).sum();
        let mut rows: Vec<(&str, Duration, [Duration; 5], u64)> =
            per_file.into_iter().map(|(f, (t, b, n))| (f, t, b, n)).collect();
        rows.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
        out.push_str(&format!(
            "     {:<36} {:>8}  {:>6} {:>8} {:>8} {:>8} {:>8} {:>8} {:>6} {:>8}\n",
            "manifest entry", "ms", "%files", PASS_ORDER[0], PASS_ORDER[1], PASS_ORDER[2],
            PASS_ORDER[3], PASS_ORDER[4], "forms", "us/form",
        ));
        for (i, (file, total, by_pass, forms)) in rows.iter().enumerate() {
            let pct = if files_total.is_zero() {
                0.0
            } else {
                100.0 * total.as_secs_f64() / files_total.as_secs_f64()
            };
            let us_per_form = if *forms == 0 {
                0.0
            } else {
                by_pass[2].as_secs_f64() * 1_000_000.0 / *forms as f64
            };
            out.push_str(&format!(
                "  {:>2} {:<36} {:>8.2}  {pct:>6.2} {:>8.2} {:>8.2} {:>8.2} {:>8.2} {:>8.2} {forms:>6} {us_per_form:>8.1}\n",
                i + 1,
                file,
                ms(*total),
                ms(by_pass[0]),
                ms(by_pass[1]),
                ms(by_pass[2]),
                ms(by_pass[3]),
                ms(by_pass[4]),
            ));
        }
        // ⚠ THE MEASURER'S OWN SIZE, printed with its output. `guards` is how many times a
        // `file_guard` opened and closed — two `Instant::now()` reads, one `String` allocation
        // and one `HashMap` entry each. It is the ONLY number from which this mode's overhead
        // can be bounded, so the report states it rather than leaving the reader to assume the
        // instrument is free. `[[feedback_the_measurement_contains_the_measurer]]`.
        let guards: u64 = files.values().map(|r| r.hits).sum();
        out.push_str(&format!(
            "\n     {} entries, {:.2} ms attributed to a named file, from {guards} file_guard \
             open/close pairs — THIS MODE'S OWN COST scales with that count, not with the\n\
             \x20    boot. Compare a `phases` run (0 pairs) to bound it.\n",
            rows.len(),
            ms(files_total)
        ));
        // ⛔ THE INSTRUMENT CAN BE BLINDED, AND IT MUST SAY SO. `file_guard` opens inside
        // `stdlib_forms()` — the last place in the boot that knows which manifest entry a form
        // came from. When the boot cache answers, that function never runs, so the per-entry
        // table below covers ONLY what was derived, which on a warm cache is the user's own
        // entry file and nothing else. A near-empty table with no explanation is the shape that
        // gets read as "the stdlib is free". It is not free; it was elided.
        if phases.get(P_CACHE_LOAD).is_some_and(|r| r.hits > 0) {
            out.push_str(
                "\n     ⛔ THE BOOT CACHE ANSWERED, so `stdlib-parse` never ran and NO stdlib\n                 \x20       manifest entry could be attributed. The table above covers only what\n                 \x20       this boot DERIVED. Re-run with WAT_BOOT_CACHE=off to measure the\n                 \x20       derivation this mode exists to measure.\n",
            );
        }
    }
    out.push_str("──────────────────────────────────────────────────────────────────────────\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭑ THE DEFAULT. Absent env ⇒ `Off`, and `phase` is then pass-through.
    #[test]
    fn off_is_the_default_and_phase_is_pass_through() {
        assert_eq!(mode_from_raw(""), Ok(Mode::Off));
        // `phase` must return the closure's value byte-for-byte whatever the mode is; the
        // default case is the one every process takes.
        assert_eq!(phase("unlisted-on-purpose", || 7_u8 + 1), 8);
    }

    /// Every spelling the knob accepts, and the shape of a refusal.
    #[test]
    fn mode_parses_the_documented_spellings_only() {
        for raw in ["1", "phases", "phase", "on", "true", " phases "] {
            assert_eq!(mode_from_raw(raw), Ok(Mode::Phases), "{raw:?}");
        }
        for raw in ["files", "file", "2"] {
            assert_eq!(mode_from_raw(raw), Ok(Mode::Files), "{raw:?}");
        }
        for raw in ["", "0", "off", "false"] {
            assert_eq!(mode_from_raw(raw), Ok(Mode::Off), "{raw:?}");
        }
        for raw in ["yes", "all", "PHASES", "3", "per-file"] {
            assert_eq!(mode_from_raw(raw), Err(()), "{raw:?}");
        }
    }

    /// An unrecognised value must name the variable, the offending text, both accepted spellings,
    /// and that the census stayed OFF — otherwise an operator reads a silent default as a
    /// deliberate one. The whole line is pinned: a loose `contains` would pass on a message that
    /// had quietly lost the "stays OFF" half, which is the half that prevents the misreading.
    #[test]
    fn unrecognised_mode_text_names_the_variable_the_input_and_the_outcome() {
        assert_eq!(
            unrecognised_mode_text("per-file"),
            "wat: WAT_BOOT_CENSUS=\"per-file\" is not a census mode — expected `phases` \
             (per-phase breakdown) or `files` (per-phase + per-manifest-entry attribution). \
             The boot census stays OFF."
        );
    }

    /// ⭑ THE REPORT MUST COUNT, AND IT MUST RECONCILE — and the whole rendered table is pinned,
    /// not sampled. Precedent: `ring_census_counts_every_ring_it_hands_out`
    /// (`src/comms/process.rs`) — a census that does not demonstrably count is decoration.
    ///
    /// Readings are FIXED, so `report_text` is a pure function of them and the output is
    /// deterministic to the byte. That is what makes the exact form available here, and the exact
    /// form is what catches a dropped row, a reordered column, a share computed against the wrong
    /// denominator, or a reconciliation line that stopped being printed — none of which a
    /// `contains` on three numbers can see.
    ///
    /// The arithmetic it pins: accounted = 300+60+1 = 361 ms; in-pipeline remainder =
    /// 400−361 = 39 ms; pre-pipeline = 430−400 = 30 ms; expansion's share = 300/361 = 83.10 %.
    /// And every phase that did NOT fire is still listed at 0.00 with 0 hits, because an absent
    /// row and a zero row are different facts.
    #[test]
    fn report_text_sums_leaves_and_reconciles_against_both_anchors() {
        let mut phases = HashMap::new();
        phases.insert(P_STDLIB_EXPAND, Reading { total: Duration::from_millis(300), hits: 1 });
        phases.insert(P_STDLIB_PARSE, Reading { total: Duration::from_millis(60), hits: 1 });
        phases.insert(P_CHECK_FORMS, Reading { total: Duration::from_millis(1), hits: 1 });
        assert_eq!(
            report_text(
                Mode::Phases,
                &phases,
                &HashMap::new(),
                Some(Duration::from_millis(400)),
                Some(Duration::from_millis(430)),
            ),
            EXPECTED_PHASES_REPORT
        );
    }

    /// ⛔ A reading the report cannot show must SAY it cannot show it. A phase name recorded
    /// outside `PHASE_ORDER` would otherwise vanish from every row and silently falsify the
    /// accounted total — "a green gate is silent, not clean". The whole text is pinned so the
    /// shout cannot be softened into something a reader skims past.
    #[test]
    fn report_text_shouts_about_a_phase_it_cannot_place() {
        let mut phases = HashMap::new();
        phases.insert("99 invented-phase", Reading { total: Duration::from_millis(5), hits: 1 });
        assert_eq!(
            report_text(Mode::Phases, &phases, &HashMap::new(), None, None),
            EXPECTED_STRAY_PHASE_REPORT
        );
    }

    /// The per-file table must rank by cost, name the file, split by pass, carry its own guard
    /// count, and keep the "these are not phase boundaries" caveat travelling with the numbers —
    /// that is the whole of row 3's deliverable. Pinned whole for the same reason as above: the
    /// ordering and the denominators are the claim.
    ///
    /// The arithmetic it pins: `wat/queue.wat` = 120+6 = 126 ms, ranked first; `wat/core.wat` =
    /// 9 ms; attributed total 135 ms; queue's share 126/135 = 93.33 %; guard pairs 40+1+200 = 241;
    /// and µs/form from the EXPAND column only — 120 ms / 40 forms = 3000 µs.
    #[test]
    fn report_text_ranks_manifest_entries_by_cost_and_splits_by_pass() {
        let mut files = HashMap::new();
        files.insert((F_EXPAND, "wat/queue.wat".to_string()), Reading { total: Duration::from_millis(120), hits: 40 });
        files.insert((F_PARSE, "wat/queue.wat".to_string()), Reading { total: Duration::from_millis(6), hits: 1 });
        files.insert((F_EXPAND, "wat/core.wat".to_string()), Reading { total: Duration::from_millis(9), hits: 200 });
        assert_eq!(
            report_text(Mode::Files, &HashMap::new(), &files, None, None),
            EXPECTED_FILES_REPORT
        );
    }

    /// ⭑ The property, not the path: every per-pass label the code can stamp must be placeable in
    /// the report's columns. A sixth label added without extending `PASS_ORDER` would land in no
    /// column and be silently dropped from the per-pass split.
    #[test]
    fn every_pass_label_has_a_column() {
        for l in [F_PARSE, F_DEFMACRO, F_EXPAND, F_TYPES, F_DEFINES] {
            assert!(PASS_ORDER.contains(&l), "pass label {l:?} has no column in the report");
        }
        assert_eq!(PASS_ORDER.len(), 5, "the report's per-pass columns are hard-sized at 5");
    }

    /// `PHASE_ORDER` is the report's whole table; a duplicate would print a phase twice and
    /// double its apparent share.
    #[test]
    fn phase_order_has_no_duplicates() {
        let mut seen = std::collections::HashSet::new();
        for p in PHASE_ORDER {
            assert!(seen.insert(*p), "duplicate phase name in PHASE_ORDER: {p:?}");
        }
    }

    /// The accumulator must actually accumulate — two records of the same key add, and hits
    /// count. (Independent of the env gate: `record_*` is the door the gate opens.)
    #[test]
    fn records_accumulate_and_count() {
        record_phase(P_CONTAINMENT, Duration::from_millis(3));
        record_phase(P_CONTAINMENT, Duration::from_millis(4));
        record_file(F_PARSE, "wat/core.wat", Duration::from_micros(500));
        let s = snapshot();
        let r = s.phases[&P_CONTAINMENT];
        assert_eq!(r.hits, 2);
        assert_eq!(r.total, Duration::from_millis(7));
        assert_eq!(s.files[&(F_PARSE, "wat/core.wat".to_string())].hits, 1);
        assert!(s.started.is_some(), "the pipeline anchor must be stamped on first record");
    }

    // ─── The goldens ─────────────────────────────────────────────────────────────────────
    //
    // Captured from the renderer, not hand-written — per CONVENTIONS' golden rubric ("capture the
    // whole value, never guess"). Inline `const`s rather than co-located files because these are
    // rendered TEXT tables, not structured EDN: `assert_eq!` against the literal is already the
    // byte-identical form, and keeping the expectation beside the arithmetic it encodes is what
    // makes a diff readable when the renderer changes.

    const EXPECTED_PHASES_REPORT: &str = r"
── wat boot census ─────────────────────────────── WAT_BOOT_CENSUS=phases ──
   LEAF phases, nothing nested, in pipeline order — the sum IS the accounted total.
   A phase with 0 hits did not run in this boot (a user-side pass on a stdlib-only
   freeze, say); it is still listed, because an absent row and a zero row are
   different facts.

     phase                                 ms      %acc   hits
     1    entry-parse                       0.00    0.00      0
     2    config-pass                       0.00    0.00      0
     3    resolve-loads                     0.00    0.00      0
     3a-6b boot-cache-load                  0.00    0.00      0
     3a   stdlib-parse                     60.00   16.62      1
     3b   rete-defn-scan                    0.00    0.00      0
     4    stdlib-defmacro-register          0.00    0.00      0
     4    user-defmacro-register            0.00    0.00      0
     4    kwargs-companions                 0.00    0.00      0
     4    acronym-preregister(macro)        0.00    0.00      0
     4    stdlib-expand                   300.00   83.10      1
     4    user-expand                       0.00    0.00      0
     4b   legacy-walkers(user)              0.00    0.00      0
     5    typeenv-with-builtins             0.00    0.00      0
     5    stdlib-types-register             0.00    0.00      0
     5    user-types-register               0.00    0.00      0
     5    aggregate-containment             0.00    0.00      0
     6    stdlib-defines-register           0.00    0.00      0
     6a   defclause-stub-preregister        0.00    0.00      0
     6b   stdlib-runtime-def-filter         0.00    0.00      0
     6    user-defines-register             0.00    0.00      0
     6a-9 auto-method-codegen               0.00    0.00      0
     6.8  restriction-entry-drain           0.00    0.00      0
     6.96 acronym-preregister(runtime)      0.00    0.00      0
     6.97 typeenv-clone-attach              0.00    0.00      0
     7    normalize-symbol-refs             0.00    0.00      0
     7    resolve-references                0.00    0.00      0
     7.6  stdlib-runtime-defs-register      0.00    0.00      0
     7.7  extend-type-preregister(user)     0.00    0.00      0
     7.8  freeze-validator-drain            0.00    0.00      0
     7.9  boot-cache-store                  0.00    0.00      0
     8a   check:env-from-symbols            0.00    0.00      0
     8b   check:retired-syntax(ALL fns)     0.00    0.00      0
     8c   check:restricted-call(ALL fns)     0.00    0.00      0
     8d   check:def-position(ALL fns)       0.00    0.00      0
     8e   check:form-loop(user residue)     1.00    0.28      1
     8f   check:body-infer(ALL fns)         0.00    0.00      0
     8g   check:impls-completeness          0.00    0.00      0
     9    frozen-world-freeze               0.00    0.00      0

     ACCOUNTED (sum of leaves)            361.00
     PIPELINE WALL (1st census call→here)   400.00
     unaccounted, in-pipeline              39.00   ← inside the pipeline, no phase names it
     SINCE PROCESS BOOT                   430.00
     pre-pipeline (boot→step 1)            30.00   ← batteries, argv, entry-file read
──────────────────────────────────────────────────────────────────────────
";
    const EXPECTED_STRAY_PHASE_REPORT: &str = r#"
── wat boot census ─────────────────────────────── WAT_BOOT_CENSUS=phases ──
   LEAF phases, nothing nested, in pipeline order — the sum IS the accounted total.
   A phase with 0 hits did not run in this boot (a user-side pass on a stdlib-only
   freeze, say); it is still listed, because an absent row and a zero row are
   different facts.

     phase                                 ms      %acc   hits
     1    entry-parse                       0.00    0.00      0
     2    config-pass                       0.00    0.00      0
     3    resolve-loads                     0.00    0.00      0
     3a-6b boot-cache-load                  0.00    0.00      0
     3a   stdlib-parse                      0.00    0.00      0
     3b   rete-defn-scan                    0.00    0.00      0
     4    stdlib-defmacro-register          0.00    0.00      0
     4    user-defmacro-register            0.00    0.00      0
     4    kwargs-companions                 0.00    0.00      0
     4    acronym-preregister(macro)        0.00    0.00      0
     4    stdlib-expand                     0.00    0.00      0
     4    user-expand                       0.00    0.00      0
     4b   legacy-walkers(user)              0.00    0.00      0
     5    typeenv-with-builtins             0.00    0.00      0
     5    stdlib-types-register             0.00    0.00      0
     5    user-types-register               0.00    0.00      0
     5    aggregate-containment             0.00    0.00      0
     6    stdlib-defines-register           0.00    0.00      0
     6a   defclause-stub-preregister        0.00    0.00      0
     6b   stdlib-runtime-def-filter         0.00    0.00      0
     6    user-defines-register             0.00    0.00      0
     6a-9 auto-method-codegen               0.00    0.00      0
     6.8  restriction-entry-drain           0.00    0.00      0
     6.96 acronym-preregister(runtime)      0.00    0.00      0
     6.97 typeenv-clone-attach              0.00    0.00      0
     7    normalize-symbol-refs             0.00    0.00      0
     7    resolve-references                0.00    0.00      0
     7.6  stdlib-runtime-defs-register      0.00    0.00      0
     7.7  extend-type-preregister(user)     0.00    0.00      0
     7.8  freeze-validator-drain            0.00    0.00      0
     7.9  boot-cache-store                  0.00    0.00      0
     8a   check:env-from-symbols            0.00    0.00      0
     8b   check:retired-syntax(ALL fns)     0.00    0.00      0
     8c   check:restricted-call(ALL fns)     0.00    0.00      0
     8d   check:def-position(ALL fns)       0.00    0.00      0
     8e   check:form-loop(user residue)     0.00    0.00      0
     8f   check:body-infer(ALL fns)         0.00    0.00      0
     8g   check:impls-completeness          0.00    0.00      0
     9    frozen-world-freeze               0.00    0.00      0

     ACCOUNTED (sum of leaves)              5.00
     ⛔ 1 phase name(s) recorded but ABSENT from PHASE_ORDER, so excluded from
        every row above: ["99 invented-phase"]. This is an instrument bug — the accounted total
        is WRONG by their sum.
     ⚠ no pipeline / boot anchor available — reconciliation not printed.
──────────────────────────────────────────────────────────────────────────
"#;
    const EXPECTED_FILES_REPORT: &str = r"
── wat boot census ─────────────────────────────── WAT_BOOT_CENSUS=files ──
   LEAF phases, nothing nested, in pipeline order — the sum IS the accounted total.
   A phase with 0 hits did not run in this boot (a user-side pass on a stdlib-only
   freeze, say); it is still listed, because an absent row and a zero row are
   different facts.

     phase                                 ms      %acc   hits
     1    entry-parse                       0.00    0.00      0
     2    config-pass                       0.00    0.00      0
     3    resolve-loads                     0.00    0.00      0
     3a-6b boot-cache-load                  0.00    0.00      0
     3a   stdlib-parse                      0.00    0.00      0
     3b   rete-defn-scan                    0.00    0.00      0
     4    stdlib-defmacro-register          0.00    0.00      0
     4    user-defmacro-register            0.00    0.00      0
     4    kwargs-companions                 0.00    0.00      0
     4    acronym-preregister(macro)        0.00    0.00      0
     4    stdlib-expand                     0.00    0.00      0
     4    user-expand                       0.00    0.00      0
     4b   legacy-walkers(user)              0.00    0.00      0
     5    typeenv-with-builtins             0.00    0.00      0
     5    stdlib-types-register             0.00    0.00      0
     5    user-types-register               0.00    0.00      0
     5    aggregate-containment             0.00    0.00      0
     6    stdlib-defines-register           0.00    0.00      0
     6a   defclause-stub-preregister        0.00    0.00      0
     6b   stdlib-runtime-def-filter         0.00    0.00      0
     6    user-defines-register             0.00    0.00      0
     6a-9 auto-method-codegen               0.00    0.00      0
     6.8  restriction-entry-drain           0.00    0.00      0
     6.96 acronym-preregister(runtime)      0.00    0.00      0
     6.97 typeenv-clone-attach              0.00    0.00      0
     7    normalize-symbol-refs             0.00    0.00      0
     7    resolve-references                0.00    0.00      0
     7.6  stdlib-runtime-defs-register      0.00    0.00      0
     7.7  extend-type-preregister(user)     0.00    0.00      0
     7.8  freeze-validator-drain            0.00    0.00      0
     7.9  boot-cache-store                  0.00    0.00      0
     8a   check:env-from-symbols            0.00    0.00      0
     8b   check:retired-syntax(ALL fns)     0.00    0.00      0
     8c   check:restricted-call(ALL fns)     0.00    0.00      0
     8d   check:def-position(ALL fns)       0.00    0.00      0
     8e   check:form-loop(user residue)     0.00    0.00      0
     8f   check:body-infer(ALL fns)         0.00    0.00      0
     8g   check:impls-completeness          0.00    0.00      0
     9    frozen-world-freeze               0.00    0.00      0

     ACCOUNTED (sum of leaves)              0.00
     ⚠ no pipeline / boot anchor available — reconciliation not printed.

   PER-MANIFEST-ENTRY, 5 pass(es) × file. ⚠ THE PASSES BELOW ARE NOT THE PHASES
   ABOVE: past stdlib-parse the form vector is FLAT and carries no file boundary,
   so these rows are reconstructed from each top-level form's Span::file. Two of
   the passes (expand, type-reg) are shared with the user path — the file column
   is what separates them.

     manifest entry                             ms  %files    parse defmacro-reg   expand type-reg define-reg  forms  us/form
   1 wat/queue.wat                          126.00   93.33     6.00     0.00   120.00     0.00     0.00     40   3000.0
   2 wat/core.wat                             9.00    6.67     0.00     0.00     9.00     0.00     0.00    200     45.0

     2 entries, 135.00 ms attributed to a named file, from 241 file_guard open/close pairs — THIS MODE'S OWN COST scales with that count, not with the
     boot. Compare a `phases` run (0 pairs) to bound it.
──────────────────────────────────────────────────────────────────────────
";

}
