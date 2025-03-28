use cubing::kpuzzle::KPuzzle;

use crate::{
    _internal::search::pattern_traversal_filter_trait::PatternTraversalFilter,
    scramble::puzzles::square1::wedges::{WedgeType, WEDGE_TYPE_LOOKUP},
};

pub(crate) struct Square1ShapeTraversalFilter;

const SLOTS_THAT_ARE_AFTER_SLICES: [u8; 4] = [0, 6, 12, 18];

impl PatternTraversalFilter<KPuzzle> for Square1ShapeTraversalFilter {
    fn is_valid(pattern: &cubing::kpuzzle::KPattern) -> bool {
        let orbit_info = &pattern.kpuzzle().data.ordered_orbit_info[0];
        assert_eq!(orbit_info.name.0, "WEDGES");

        for slot in SLOTS_THAT_ARE_AFTER_SLICES {
            let value = pattern.get_piece(orbit_info, slot);

            // Note: the `WEDGE_TYPE_LOOKUP` lookup is not necessary for phase 1, but it is needed for a single-pahse search.
            if WEDGE_TYPE_LOOKUP[value as usize] == WedgeType::CornerUpper {
                return false;
            }
        }

        true
    }
}
