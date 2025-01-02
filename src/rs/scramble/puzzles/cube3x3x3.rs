use std::sync::Mutex;

use cubing::{
    alg::{parse_move, Alg, AlgNode, Move},
    kpuzzle::{KPattern, KPuzzle},
};
use lazy_static::lazy_static;

use crate::{
    _internal::search::{
        idf_search::{IDFSearch, IndividualSearchOptions},
        mask_pattern::apply_mask,
        prune_table_trait::Depth,
    },
    scramble::{
        collapse::collapse_adjacent_moves,
        randomize::{basic_parity, BasicParity, OrbitRandomizationConstraints},
        scramble_search::{move_list_from_vec, FilteredSearch},
    },
};

use super::{
    super::randomize::{
        randomize_orbit_naïve, OrbitOrientationConstraint, OrbitPermutationConstraint,
    },
    definitions::{cube3x3x3_centerless_g1_target_kpattern, cube3x3x3_centerless_kpuzzle},
    static_move_list::{add_random_suffixes_from, static_parsed_list, static_parsed_opt_list},
};

pub struct Scramble3x3x3TwoPhase {
    kpuzzle: KPuzzle,

    filtering_idfs: IDFSearch<KPuzzle>,

    phase1_target_pattern: KPattern,
    phase1_idfs: IDFSearch<KPuzzle>,

    phase2_idfs: IDFSearch<KPuzzle>,
}

impl Default for Scramble3x3x3TwoPhase {
    fn default() -> Self {
        let kpuzzle = cube3x3x3_centerless_kpuzzle().clone();
        let generators = move_list_from_vec(vec!["U", "L", "F", "R", "B", "D"]);
        let filtering_idfs = FilteredSearch::basic_idfs(
            &kpuzzle,
            generators.clone(),
            Some(32),
            kpuzzle.default_pattern(),
        );

        let phase1_target_pattern = cube3x3x3_centerless_g1_target_kpattern().clone();
        let phase1_idfs = FilteredSearch::idfs_with_target_pattern(
            &kpuzzle,
            generators.clone(),
            phase1_target_pattern.clone(),
            Some(1 << 24),
        );

        let phase2_generators = move_list_from_vec(vec!["U", "L2", "F2", "R2", "B2", "D"]);
        let phase2_idfs = FilteredSearch::idfs_with_target_pattern(
            &kpuzzle,
            phase2_generators.clone(),
            kpuzzle.default_pattern(),
            Some(1 << 24),
        );

        Self {
            kpuzzle,
            filtering_idfs,

            phase1_target_pattern,
            phase1_idfs,

            phase2_idfs,
        }
    }
}

pub fn random_3x3x3_pattern() -> KPattern {
    let kpuzzle = cube3x3x3_centerless_kpuzzle();
    let mut scramble_pattern = kpuzzle.default_pattern();
    let edge_order = randomize_orbit_naïve(
        &mut scramble_pattern,
        0,
        "EDGES",
        OrbitRandomizationConstraints {
            orientation: Some(OrbitOrientationConstraint::SumToZero),
            ..Default::default()
        },
    );
    let each_orbit_parity = basic_parity(&edge_order);
    randomize_orbit_naïve(
        &mut scramble_pattern,
        1,
        "CORNERS",
        OrbitRandomizationConstraints {
            permutation: Some(match each_orbit_parity {
                BasicParity::Even => OrbitPermutationConstraint::EvenParity,
                BasicParity::Odd => OrbitPermutationConstraint::OddParity,
            }),
            orientation: Some(OrbitOrientationConstraint::SumToZero),
            ..Default::default()
        },
    );
    scramble_pattern
}

pub(crate) enum PrefixOrSuffixConstraints {
    None,
    ForFMC,
}

impl Scramble3x3x3TwoPhase {
    pub(crate) fn solve_3x3x3_pattern(
        &mut self,
        pattern: &KPattern,
        constraints: PrefixOrSuffixConstraints,
    ) -> Alg {
        let (canonical_fsm_pre_moves, canonical_fsm_post_moves) = match constraints {
            PrefixOrSuffixConstraints::None => (None, None),
            PrefixOrSuffixConstraints::ForFMC => {
                // For the pre-moves, we don't have to specify R' and U' because we know the FSM only depends on the final `F` move.
                // For similar reasons, we only have to specify R' for the post-moves.
                (Some(vec![parse_move!("F")]), Some(vec![parse_move!("R'")]))
            }
        };

        let phase1_alg = {
            let phase1_search_pattern = apply_mask(pattern, &self.phase1_target_pattern).unwrap();
            self.phase1_idfs
                .search(
                    &phase1_search_pattern,
                    IndividualSearchOptions {
                        min_num_solutions: Some(1),
                        canonical_fsm_pre_moves,
                        canonical_fsm_post_moves, // TODO: We currently need to pass this in case phase 2 return the empty alg. Can we handle this in another way?
                        ..Default::default()
                    },
                )
                .next()
                .unwrap()
        };

        let mut phase2_alg = {
            let phase2_search_pattern = pattern
                .apply_transformation(&self.kpuzzle.transformation_from_alg(&phase1_alg).unwrap());
            self.phase2_idfs
                .search(
                    &phase2_search_pattern,
                    IndividualSearchOptions {
                        min_num_solutions: Some(1),
                        ..Default::default()
                    },
                )
                .next()
                .unwrap()
        };

        let mut nodes = phase1_alg.nodes;
        nodes.append(&mut phase2_alg.nodes);
        Alg { nodes }
    }

    // TODO: rely on the main search to find patterns at a low depth?
    pub fn is_valid_scramble_pattern(&mut self, pattern: &KPattern) -> bool {
        self.filtering_idfs
            .search(
                pattern,
                IndividualSearchOptions {
                    min_num_solutions: Some(1),
                    min_depth: Some(Depth(0)),
                    max_depth: Some(Depth(2)),
                    ..Default::default()
                },
            )
            .next()
            .is_none()
    }

    pub(crate) fn scramble_3x3x3(&mut self, constraints: PrefixOrSuffixConstraints) -> Alg {
        loop {
            let scramble_pattern = random_3x3x3_pattern();
            if !self.is_valid_scramble_pattern(&scramble_pattern) {
                continue;
            }
            return self.solve_3x3x3_pattern(&scramble_pattern, constraints);
        }
    }
}

// TODO: switch to `LazyLock` once that's stable: https://doc.rust-lang.org/nightly/std/cell/struct.LazyCell.html
lazy_static! {
    static ref SCRAMBLE3X3X3_TWO_PHASE: Mutex<Scramble3x3x3TwoPhase> =
        Mutex::new(Scramble3x3x3TwoPhase::default());
}

pub fn scramble_3x3x3() -> Alg {
    SCRAMBLE3X3X3_TWO_PHASE
        .lock()
        .unwrap()
        .scramble_3x3x3(PrefixOrSuffixConstraints::None)
}

pub fn scramble_3x3x3_bld() -> Alg {
    let s1 = static_parsed_opt_list(&["", "Rw", "Rw2", "Rw'", "Fw", "Fw'"]);
    let s2 = static_parsed_opt_list(&["", "Uw", "Uw2", "Uw'"]);
    add_random_suffixes_from(scramble_3x3x3(), [s1, s2])
}

const FMC_AFFIX: [&str; 3] = ["R'", "U'", "F"];

pub fn scramble_3x3x3_fmc() -> Alg {
    let mut nodes = Vec::<AlgNode>::new();

    let prefix_and_suffix: Vec<Move> = static_parsed_list(&FMC_AFFIX);
    for r#move in prefix_and_suffix {
        nodes.push(r#move.into());
    }

    nodes.append(
        &mut SCRAMBLE3X3X3_TWO_PHASE
            .lock()
            .unwrap()
            .scramble_3x3x3(PrefixOrSuffixConstraints::ForFMC)
            .nodes,
    );

    let affix: Vec<Move> = static_parsed_list(&FMC_AFFIX);
    for r#move in affix {
        nodes.push(r#move.into());
    }

    // Note: `collapse_adjacent_moves(…)` is technically overkill, as it's only
    // possible for a single move to overlap without completely cancelling.
    // However, it's safer to use a common function for this instead of a one-off implementation.
    collapse_adjacent_moves(Alg { nodes }, 4, -1)
}
