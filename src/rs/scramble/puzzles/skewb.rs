use cubing::alg::Alg;


use super::{
    super::randomize::{
        randomize_orbit_naïve, OrbitOrientationConstraint, OrbitPermutationConstraint,
    },
    super::scramble_search::{filtered_search, generators_from_vec_str},
    definitions::skewb_fixed_corner_with_co_tweaks_kpuzzle,
};

pub fn scramble_skewb() -> Alg {
    let kpuzzle = skewb_fixed_corner_with_co_tweaks_kpuzzle();
    loop {
        let mut scramble_pattern = kpuzzle.default_pattern();

        /* The total orientation of each corner orbit is constrained by the permutation of the other. That is, suppose we have a valid state of Skewb with some placeholders:
         *
         * (Take note of the values highlighted by ↓↓ and ↑↑.)
         *
         *                                                               ↓↓
         *                                                               ↓↓
         * {
         *     "CORNERS1": { "pieces": [@2, @2, @2],     "orientation": [#1, @1, @1] },
         *     "CORNERS2": { "pieces": [@1, @1, @1, @1], "orientation": [#2, @2, @2, @2]},
         *     "CENTERS":  { … }
         * }                                                             ↑↑
         *                                                               ↑↑
         *
         * Then:
         *
         * - The orientation of value `#1` is determined by the values labeled `@1`.
         * - The orientation of value `#2` is determined by the values labeled `@2`.
         *
         * Now, we could either:
         *
         * - Do a bit of math to determine the values `#1` and `#2.`
         * - Set the orientations of `#1` and `#2` to "ignored" by using the `orientationMod` feature.
         *
         * We choose to do the latter with respect to the solved state, then generate a random permutation of this pattern
         * (taking into account permutation parity for each orbit) and solve it. In the resulting state:
         *
         * - All the `@1` values match the solved state, so `#1` must also match the solved state.
         * - All the `@2` values match the solved state, so `#2` value also match the solved state.
         *
         * That is: the entire puzzle is solved, and we can use this to return a uniform random scramble (subject to other filtering).
         *
         * This approach does not have any performance implications, and also has the benefit that it allows us to randomize each orbit independently.
         *
         * The numbers check out, as this gives us the following number of distinct states:
         *
         * | Orbit    | Calculation    | Number of possibilities |
         * |----------|----------------|-------------------------|
         * | CORNERS1 | 4! / 2 * 3^3   | 324                     |
         * | CORNERS2 | 3! / 2 * 3^2   | 27                      |
         * | CENTERS  | 6! / 2         | 360                     |
         * |----------|----------------|-------------------------|
         * | Overall  | 324 * 27 * 360 | 3149280                 |
         *
         * This matches: https://www.jaapsch.net/puzzles/skewb.htm
         */


        let orbit_info = &kpuzzle.data.ordered_orbit_info[0];
        assert_eq!(orbit_info.name.0, "CORNERS1");
        randomize_orbit_naïve(
            &mut scramble_pattern,
            orbit_info,
            OrbitPermutationConstraint::SingleOrbitEvenParity,
            OrbitOrientationConstraint::SetPieceZeroToIgnoredOrientation,
        );
        
        let orbit_info = &kpuzzle.data.ordered_orbit_info[1];
        assert_eq!(orbit_info.name.0, "CORNERS2");
        randomize_orbit_naïve(
            &mut scramble_pattern,
            orbit_info,
            OrbitPermutationConstraint::SingleOrbitEvenParity,
            OrbitOrientationConstraint::SetPieceZeroToIgnoredOrientation,
        );

        let orbit_info = &kpuzzle.data.ordered_orbit_info[2];
        assert_eq!(orbit_info.name.0, "CENTERS");
        randomize_orbit_naïve(
            &mut scramble_pattern,
            orbit_info,
            OrbitPermutationConstraint::SingleOrbitEvenParity,
            OrbitOrientationConstraint::OrientationsMustSumToZero,
        );

        let generators = generators_from_vec_str(vec!["U", "L", "R", "B"]); // TODO: cache
        if let Some(scramble) = filtered_search(&scramble_pattern, generators, 7, Some(11)) {
            return scramble;
        }
    }
}
