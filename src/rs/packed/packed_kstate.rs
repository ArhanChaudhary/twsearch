use std::{
    alloc::{alloc, dealloc},
    fmt::Debug,
};

use super::{
    byte_conversions::{u8_to_usize, PackedOrientationWithMod},
    packed_kpuzzle::PackedKPuzzleOrbitInfo,
    PackedKPuzzle, PackedKTransformation,
};

use cubing::kpuzzle::KPuzzle;
#[cfg(not(feature = "no_orientation_mod"))]
use cubing::kpuzzle::{KState, KStateData};
#[cfg(not(feature = "no_orientation_mod"))]
use std::sync::Arc;

#[cfg(not(feature = "orientation_packer"))]
#[cfg(not(feature = "no_orientation_mod"))]
use super::packed_kpuzzle::{ORIENTATION_MASK, ORIENTATION_MOD_SHIFT_BITS};

pub struct PackedKState {
    pub packed_kpuzzle: PackedKPuzzle,
    pub bytes: *mut u8,
}

impl Drop for PackedKState {
    fn drop(&mut self) {
        unsafe { dealloc(self.bytes, self.packed_kpuzzle.data.layout) }
    }
}

impl PackedKState {
    pub fn new(packed_kpuzzle: PackedKPuzzle) -> Self {
        let bytes = unsafe { alloc(packed_kpuzzle.data.layout) };
        Self {
            packed_kpuzzle,
            bytes,
        }
    }

    pub fn get_piece_or_permutation(&self, orbit_info: &PackedKPuzzleOrbitInfo, i: usize) -> u8 {
        unsafe {
            self.bytes
                .add(orbit_info.pieces_or_pemutations_offset + i)
                .read()
        }
    }

    pub fn get_packed_orientation(
        &self,
        orbit_info: &PackedKPuzzleOrbitInfo,
        i: usize,
    ) -> PackedOrientationWithMod {
        unsafe { self.bytes.add(orbit_info.orientations_offset + i).read() }
    }

    pub fn set_piece_or_permutation(
        &self,
        orbit_info: &PackedKPuzzleOrbitInfo,
        i: usize,
        value: u8,
    ) {
        unsafe {
            self.bytes
                .add(orbit_info.pieces_or_pemutations_offset + i)
                .write(value)
        }
    }

    pub fn set_packed_orientation(
        &self,
        orbit_info: &PackedKPuzzleOrbitInfo,
        i: usize,
        value: PackedOrientationWithMod,
    ) {
        unsafe {
            self.bytes
                .add(orbit_info.orientations_offset + i)
                .write(value)
        }
    }

    // Adapted from https://github.com/cubing/cubing.rs/blob/b737c6a36528e9984b45b29f9449a9a330c272fb/src/kpuzzle/state.rs#L31-L82
    // TODO: dedup the implementation (but avoid runtime overhead for the shared abstraction).
    pub fn apply_transformation(&self, transformation: &PackedKTransformation) -> PackedKState {
        let mut new_state = PackedKState::new(self.packed_kpuzzle.clone());
        self.apply_transformation_into(transformation, &mut new_state);
        new_state
    }

    // Adapted from https://github.com/cubing/cubing.rs/blob/b737c6a36528e9984b45b29f9449a9a330c272fb/src/kpuzzle/state.rs#L31-L82
    // TODO: dedup the implementation (but avoid runtime overhead for the shared abstraction).
    // TODO: assign to self from another value, not into another
    pub fn apply_transformation_into(
        &self,
        transformation: &PackedKTransformation,
        into_state: &mut PackedKState,
    ) {
        for orbit_info in &self.packed_kpuzzle.data.orbit_iteration_info {
            // TODO: optimization when either value is the identity.
            for i in 0..orbit_info.num_pieces {
                let transformation_idx = transformation.get_piece_or_permutation(orbit_info, i);

                let new_piece_value =
                    self.get_piece_or_permutation(orbit_info, u8_to_usize(transformation_idx));
                into_state.set_piece_or_permutation(orbit_info, i, new_piece_value);

                let previous_packed_orientation =
                    self.get_packed_orientation(orbit_info, u8_to_usize(transformation_idx));

                #[cfg(not(feature = "orientation_packer"))]
                #[cfg(not(feature = "no_orientation_mod"))]
                let new_packed_orientation = {
                    let previous_orientation_mod =
                        previous_packed_orientation >> ORIENTATION_MOD_SHIFT_BITS;
                    let (modulus, previous_orientation) = match previous_orientation_mod {
                        0 => (orbit_info.num_orientations, previous_packed_orientation),
                        modulus => (modulus, previous_packed_orientation & ORIENTATION_MASK),
                    };
                    let new_orientation = (previous_orientation
                        + transformation.get_orientation(orbit_info, i))
                        % modulus;
                    (previous_orientation_mod << ORIENTATION_MOD_SHIFT_BITS) + new_orientation
                };
                #[cfg(feature = "orientation_packer")]
                #[cfg(not(feature = "no_orientation_mod"))]
                let new_packed_orientation = {
                    orbit_info.orientation_packer.transform(
                        previous_packed_orientation,
                        u8_to_usize(transformation.get_orientation(orbit_info, i)),
                    )
                };
                // TODO: implement an orientation packer for the `no_orientation_mod` case?
                #[cfg(feature = "no_orientation_mod")]
                let new_packed_orientation = if previous_packed_orientation
                    == orbit_info.unknown_orientation_value
                {
                    orbit_info.unknown_orientation_value
                } else {
                    (previous_packed_orientation + transformation.get_orientation(orbit_info, i))
                        % orbit_info.num_orientations
                };
                into_state.set_packed_orientation(orbit_info, i, new_packed_orientation);
            }
        }
    }

    pub fn byte_slice(&self) -> &[u8] {
        // yiss ☺️
        // https://stackoverflow.com/a/27150865
        unsafe { std::slice::from_raw_parts(self.bytes, self.packed_kpuzzle.data.num_bytes) }
    }

    pub fn hash(&self) -> u64 {
        cityhash::city_hash_64(self.byte_slice())
    }

    #[cfg(not(feature = "no_orientation_mod"))]
    pub fn unpack(&self) -> KState {
        let mut state_data = KStateData::new();
        for orbit_info in &self.packed_kpuzzle.data.orbit_iteration_info {
            let mut pieces = Vec::<usize>::new();
            let mut orientation = Vec::<usize>::new();
            let mut orientation_mod = Vec::<usize>::new();
            for i in 0..orbit_info.num_pieces {
                pieces.push(u8_to_usize(self.get_piece_or_permutation(orbit_info, i)));

                #[cfg(not(feature = "orientation_packer"))]
                #[cfg(not(feature = "no_orientation_mod"))]
                {
                    let packed_orientation = self.get_packed_orientation(orbit_info, i);
                    orientation.push(u8_to_usize(packed_orientation & ORIENTATION_MASK));
                    orientation_mod.push(u8_to_usize(
                        packed_orientation >> ORIENTATION_MOD_SHIFT_BITS,
                    ));
                }
                #[cfg(feature = "orientation_packer")]
                #[cfg(not(feature = "no_orientation_mod"))]
                {
                    let orientation_with_mod = orbit_info
                        .orientation_packer
                        .unpack(self.get_packed_orientation(orbit_info, i));
                    orientation.push(orientation_with_mod.orientation);
                    orientation_mod.push(orientation_with_mod.orientation_mod);
                };
                #[cfg(feature = "no_orientation_mod")]
                {
                    let packed_orientation = self.get_packed_orientation(orbit_info, i);
                    if packed_orientation == orbit_info.unknown_orientation_value {
                        orientation.push(0);
                        orientation_mod.push(1);
                    } else {
                        orientation.push(u8_to_usize(packed_orientation));
                        orientation_mod.push(0);
                    }
                }
            }
            let orbit_data = cubing::kpuzzle::KStateOrbitData {
                pieces,
                orientation,
                orientation_mod: Some(orientation_mod),
            };
            state_data.insert(orbit_info.name.clone(), orbit_data);
        }
        KState {
            kpuzzle: self.packed_kpuzzle.data.kpuzzle.clone(),
            state_data: Arc::new(state_data),
        }
    }
}

struct KPuzzleDebug {
    kpuzzle: KPuzzle,
}

impl Debug for KPuzzleDebug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ … name: \"{}\" … }}", &self.kpuzzle.definition().name)
    }
}

impl Debug for PackedKState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PackedKState")
            .field(
                "packed_kpuzzle",
                &KPuzzleDebug {
                    kpuzzle: self.packed_kpuzzle.data.kpuzzle.clone(),
                },
            )
            .field("bytes", &self.byte_slice())
            .finish()
    }
}
