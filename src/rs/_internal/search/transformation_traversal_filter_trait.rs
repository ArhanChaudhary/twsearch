use crate::_internal::{
    canonical_fsm::search_generators::MoveTransformationInfo,
    puzzle_traits::puzzle_traits::SemiGroupActionPuzzle,
};

use super::prune_table_trait::Depth;

pub trait TransformationTraversalFilter<TPuzzle: SemiGroupActionPuzzle> {
    // TODO: if performance is not good enough, apply this filter earlier during the iterators?
    // TODO: figure out the appropriate params.
    fn keep_move(
        move_transformation_info: &MoveTransformationInfo<TPuzzle>,
        remaining_depth: Depth,
    ) -> bool;
}

// TODO: unify struct with `AlwaysValid`?
pub struct TransformationTraversalFilterNoOp;

impl<TPuzzle: SemiGroupActionPuzzle> TransformationTraversalFilter<TPuzzle>
    for TransformationTraversalFilterNoOp
{
    fn keep_move(
        _move_transformation_info: &MoveTransformationInfo<TPuzzle>,
        _remaining_depth: Depth,
    ) -> bool {
        true
    }
}
