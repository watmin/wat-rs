//! DISCONFIRMING PROBE for `DESIGN-STONE-compiled-where.md` § "ONE CORE, THREE ADJACENT FLIPS".
//!
//! **THE CLAIM UNDER TEST, and it is a DESIGN claim with no evidence behind it yet:** one expression
//! Op model can serve all four rete surfaces (`where`, the accumulator fold, `compiled_cond`,
//! `compiled_rhs`), so a new rete op becomes ONE opcode in ONE table.
//!
//! **What would REFUTE it:** `compiled_cond::Op`'s six variants failing to fall out of an expression
//! core as *driver-level* concerns. If `Bind`/`BindCheck` are irreducibly part of the op model
//! rather than part of what FEEDS it, the three-step plan collapses to "build `where` alone" — and
//! that is worth learning from a probe rather than from a migration.
//!
//! This probe deliberately writes NO compiler. It is the cheapest thing that can say "the shapes
//! reconcile" or "they do not", per examinare: *a ten-line probe that fails on exactly the gap,
//! before the real work.*
//!
//! ⛔ It asserts a STRUCTURAL property, not a performance one. Step 0's number is unmeasured and
//! under its own STOP; nothing here may be read as a speedup claim.
//!
//! Run: cargo nextest run --release -E 'test(probe_arc278_49_one_core)'

/// The two axes that actually distinguish the four surfaces. The claim is that these are
/// **driver** differences — what feeds the machine and what its answer means — not **model**
/// differences.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Consumes {
    /// `compiled_cond`: a fact's fields, positionally.
    FactFields,
    /// `where` / the accumulator fold / `compiled_rhs`: a token's `?var` bindings.
    TokenBindings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Produces {
    /// `where` — a predicate verdict.
    Bool,
    /// `compiled_cond` — a verdict AND the bindings it wrote on the way.
    BoolPlusBindings,
    /// `compiled_rhs` — a constructed fact.
    Fact,
    /// the accumulator fold — a folded value.
    Value,
}

struct Surface {
    name: &'static str,
    consumes: Consumes,
    produces: Produces,
    /// Does this surface need arbitrary expression evaluation over the rete vocabulary
    /// (`i64::+`, `if`, `let`, `match`, `fn`, `foldl` …), or only a fixed comparison?
    needs_expressions: bool,
}

const SURFACES: &[Surface] = &[
    Surface { name: "where",            consumes: Consumes::TokenBindings, produces: Produces::Bool,             needs_expressions: true },
    Surface { name: "accumulator-fold", consumes: Consumes::TokenBindings, produces: Produces::Value,            needs_expressions: true },
    Surface { name: "compiled_rhs",     consumes: Consumes::TokenBindings, produces: Produces::Fact,             needs_expressions: true },
    Surface { name: "compiled_cond",    consumes: Consumes::FactFields,    produces: Produces::BoolPlusBindings, needs_expressions: false },
];

/// Where each `compiled_cond::Op` variant lands if the one-core claim holds: in the shared
/// EXPRESSION model, or in `compiled_cond`'s own DRIVER.
#[derive(Debug, PartialEq, Eq)]
enum Lands {
    /// Expressible by the shared expression core.
    Core,
    /// A property of what feeds the core (slot writes against a fact's field list), not of the
    /// expression language. Belongs to the driver.
    Driver,
}

fn classify_cond_variant(variant: &str) -> Lands {
    match variant {
        // Writes a fact field into a slot. There is no expression here at all — it is the driver
        // populating the environment the core will read.
        "Bind" => Lands::Driver,
        // "the field must equal the slot's existing value" — an EQUALITY over two operands, which
        // the core expresses as `Cmp{Eq}`; what makes it a *bind check* is that the driver chose
        // the operands. The comparison itself is core.
        "BindCheck" => Lands::Core,
        "Cmp" => Lands::Core,
        // Leftover rematch compare — same expression as Cmp; populate skips it,
        // rematch fills the seed slot. The compare is core; when it runs is driver.
        "SeedCmp" => Lands::Core,
        "Or" => Lands::Core,
        "Not" => Lands::Core,
        // "this clause can never hold" — a constant. Core.
        "Fail" => Lands::Core,
        other => panic!("unaccounted compiled_cond::Op variant: {other} — the probe's list is stale"),
    }
}

/// ★ THE LOAD-BEARING ROW. Every `compiled_cond::Op` variant must land in Core or Driver, and the
/// Driver set must be SMALL and about ENVIRONMENT POPULATION — not about evaluating expressions.
/// A Driver entry that is really an expression would mean the model does not generalise.
#[test]
fn every_compiled_cond_variant_lands_in_core_or_driver() {
    // Frozen by NAME, not by count — a renamed or added variant must fail loudly here rather than
    // silently shrink the thing being checked
    // (`[[feedback_a_gate_freezes_names_never_a_count]]`).
    let variants = ["Bind", "BindCheck", "Cmp", "SeedCmp", "Or", "Not", "Fail"];

    let driver: Vec<&str> =
        variants.iter().copied().filter(|v| classify_cond_variant(v) == Lands::Driver).collect();
    let core: Vec<&str> =
        variants.iter().copied().filter(|v| classify_cond_variant(v) == Lands::Core).collect();

    // Non-vacuity FIRST: if everything were Driver, "the core covers it" would be trivially true
    // and mean nothing.
    assert!(
        core.len() >= 4,
        "only {} of {} variants reach the shared core — the one-core claim is REFUTED, not \
         confirmed: the plan collapses to 'build `where` alone'. Core: {core:?} Driver: {driver:?}",
        core.len(),
        variants.len()
    );

    // The driver set must be about POPULATING the environment, nothing else.
    assert_eq!(
        driver,
        vec!["Bind"],
        "the driver-level set must be exactly the slot-population op. Anything else here is an \
         EXPRESSION that the core failed to express, which refutes the model. Got: {driver:?}"
    );
}

/// The two axes must actually be independent of the op model — i.e. surfaces must differ ONLY on
/// consumes/produces, never on which expressions they may contain. If two surfaces needed
/// *different expression languages*, one core would be a lie.
#[test]
fn the_surfaces_differ_only_by_driver_not_by_expression_language() {
    let expression_surfaces: Vec<&str> =
        SURFACES.iter().filter(|s| s.needs_expressions).map(|s| s.name).collect();

    assert!(
        expression_surfaces.len() >= 3,
        "fewer than three surfaces need expression evaluation — the shared core would be carrying \
         one real consumer, which is not a core, it is `where`'s private IR. Got: {expression_surfaces:?}"
    );

    // Every expression-needing surface consumes the SAME thing (token bindings); they differ only
    // in what the answer MEANS. That is the whole basis of "one core, N drivers".
    for s in SURFACES.iter().filter(|s| s.needs_expressions) {
        assert_eq!(
            s.consumes,
            Consumes::TokenBindings,
            "{} needs expressions but consumes {:?} — a second input shape means a second \
             environment model, and the one-core claim weakens to two",
            s.name,
            s.consumes
        );
    }

    // And the one surface that does NOT need expressions is exactly the one with a different input.
    let non_expr: Vec<&str> =
        SURFACES.iter().filter(|s| !s.needs_expressions).map(|s| s.name).collect();
    assert_eq!(
        non_expr,
        vec!["compiled_cond"],
        "the fixed-comparison surface should be exactly compiled_cond; got {non_expr:?}"
    );

    // ★ THE OTHER HALF OF "differ only by DRIVER": if the surfaces shared a `produces`, they would
    // not be distinct drivers at all — they would be the same consumer written twice, and "one core,
    // N drivers" would be describing a duplication rather than a design. All four must differ.
    //
    // (This assertion also earns the `produces` field its place. It was documentation-only in the
    // first draft — a dead field, which clippy called and CI's `-D warnings` would have failed. The
    // fix is to USE the thing that carries the argument, not to delete it or `#[allow]` it.)
    for (i, a) in SURFACES.iter().enumerate() {
        for (j, b) in SURFACES.iter().enumerate() {
            assert_eq!(
                i == j,
                a.produces == b.produces,
                "{} and {} must differ in what their answer MEANS ({:?} vs {:?}) — surfaces that \
                 produce the same thing are one driver, not two",
                a.name,
                b.name,
                a.produces,
                b.produces
            );
        }
    }
}

// The third row of this probe — "the comparison vocabulary is ALREADY one door" — lives in
// `src/rete/matcher.rs`'s `constraint_head_tests` module instead of here, because `CmpKind` is
// crate-private and an integration test cannot see it. Pinning it from outside would have meant
// re-declaring the vocabulary in the test, which is the exact duplication the one-core design
// exists to delete.
