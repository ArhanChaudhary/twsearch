# Twizzle Search (`twsearch`)

A twisty puzzle search program to that can find algs and scrambles for [WCA](https://www.worldcubeassociation.org/regulations/) puzzles and a wide variety of other permutation puzzles. `twsearch` is inspired by [KSolve](https://github.com/cubing/ksolve) and can handle the same puzzles, often with better performance or additional functionality.

Twizzle Search powers alg search and scramble functionality for [Twizzle](https://alpha.twizzle.net/), and can be used from the commandline or as a library in many environments.

Twizzle Search is currently implemented in C++, and we are prototyping a version in Rust for easier use with WASM/JavaScript.

## Project Goals

1. Maintainability
  - We want `twsearch` to serve as a foundation for the cubing software ecosystem for a long time.
  - The project has multiple maintainers from the start, and we want to focus on a sustainable model for stewardship.
2. Ease of use
  - Powerful APIs that are easy to get started with.
  - Can either be used directly, or as a library in other projects.
  - Ability to scale from mobile devices all the way to native binaries that can fully utilize high-end hardware.
  - Can be used on any website through
    [`cubing.js`](https://js.cubing.net/cubing/), either by running in the
    browser itself or optionally connecting to a computer.
3. Performance
  - Great performance out of the box for a wide variety of puzzles.
  - Tunable optimizations for heavy-duty searches, including reusable prune tables for time-memory tradeoff.

## Building and running the `twsearch` CLI

If you have a C++ toolchain on your computer, you can run:

```shell
# Check out and build the binary
git clone https://github.com/cubing/twsearch && cd twsearch
make build

# Run a search
./build/bin/twsearch samples/main/3x3x3.tws samples/main/tperm.scr
```

If you're on macOS, you can also use [Homebrew](https://brew.sh/) to install the binary:

```shell
env HOMEBREW_NO_INSTALL_FROM_API=1 brew install --HEAD cubing/cubing/twsearch
```

Otherwise, you may find it easiest to work with the JSON-based web interface at <https://experiments.cubing.net/cubing.js/twsearch/text-ui.html>

### Usage

Important options (you should specify these):

   `-M` *#* megabytes of memory to use max; should be ~ 1/2 of your RAM and ideally a power of two; defaults to 8192 (8GB).

   `-t` *#*  number of threads to use; defaults to 4.

   `--nowrite`  don't write pruning tables to disk (regenerate each time).

Sample usage:

```shell
./build/bin/twsearch samples/main/3x3x3.tws samples/main/tperm.scr

./build/bin/twsearch -g samples/main/2x2x2.tws

./build/bin/twsearch -c 20 --moves 2L,2R,U,F samples/main/4x4x4.tws samples/main/flip.scr

./build/bin/twsearch --moves F,R,D,B,L --scramblealg U samples/main/3x3x3.tws

./build/bin/twsearch --moves U,R,F -q -g samples/main/kilominx.tws
```

The maximum memory setting should be used carefully; on a machine running
Windows or OS-X with heavy browser usage and other programs, you may want
to set it to only one quarter of your physical RAM.  On a dedicated Linux
or BSD machine, you can probably set it to 90% of your physical RAM.
When using large pruning tables (such as when searching for solutions),
the pruning tables are always a power of two in size, so specify a value
somewhat larger than a power of two (so on a 64GB machine you might specify
-M 40000 to permit it to build a 32GB pruning table).  Be aware that the
memory size you set also sets the size of the pruning tables that are
written to disk.  Although they are compressed, the compression can
vary from 1.1X to more than 5X, but in general the longer the pruning
table takes to generate, the worse the compression.  All pruning tables
are written with extensions of .dat, so you might want to clean them
up occasionally if you start to run out of disk space.

Options:

`-a`    Set the maximum number of antipodes to print; the default is 20.

`-A`    Search for algorithms that hold most pieces in place.  You can
      give a digit immediately after the option to specify the type
      of search; 1 is normal iterated depth-first search; 2 is repeated
      sequences; 3 is conjugates.  By default all three are done in
      parallel.

`-c` *#*  Set the number of solutions to print (you might get more than this).

`-C`    Calculate and print information about canonical sequences.

`--cancelseqs`  Read sequences on standard in and perform move cancellations
                to minimize the length of the sequences.

`--checkbeforesolve`  Check that a position is legal before attempting to
                    solve it.  This may take extra time or memory for
                    large puzzles.

`--cachedir`  Write pruning tables here.  Expands a leading tilde with the value
              of the HOME environment variable.

`-d` *#*  Set the max depth to search.

`--distinguishall`  Distinguish all pieces, despite any identical
        piece definitions in the tws file.

`-F`    Force the use of arrays rather than bitmaps in God calculations.

`-g`    Calculate God's algorithm.

`-H`    When doing God's algorithm calculations, use 128-bit hash to
        encode states rather than actual packed state representation.

`-i`    Read sequences from stdin and write inverted sequences to stdout.

`-m` *#*  Set the max depth to search.

`-M` *#*  Set the maximum memory size as an integer number of megabytes.

`--mergeseqs`  Read move sequences on standard input and merge the sequences
               into canonical sequences.

`--mindepth`   Start solving at this depth.

`--moves` *moves*  Gives a comma-separated list of moves to use.
               All multiples of these moves are considered.  For instance,
               --moves U,F,R2 only permits half-turns on R, and all
               possible turns on U and F.

`--newcanon` *#*  Instead of using standard canonical sequences based on
              commuting moves, use canonical sequences based on unique
              positions seen through depth n.  This can help prune the
              search space for certain puzzles if n is tuned properly.

`--nocenters`   Ignore centers in the input .tws file.  Centers are sets
              whose name starts with 'ce' (ignoring case).

`--nocorners`   Ignore corners in the input .tws file.  Corners are sets
              whose name starts with 'c' and whose third letter is 'r'
              (ignoring case).

`--noearlysolutions`  Don't print trivial solutions (those of length zero).

`--noedges`     Ignore edges in the input .tws file.  Edges are sets
              whose name starts with 'ed' (ignoring case).

`--noorientation`  Ignore all orientation information in the tws file.

`--nowrite`     Don't write pruning tables to disk; regenerate them every time.

`--writeprunetables` `[always|auto|never]'  Write pruning tables always (after
                 level 5), never (same as `nowrite` above), or automatically
                 depending on table occupancy.  Default is auto.

`-o`    Print the order of every scramble from standard input.

`--orientationgroup` *#* For puzzles using adjacent element permutations rather
                     than explicit orientations for orientation (as is needed
                     when the orientation is not cyclic, as for the 2x2x2x2),
                     this sets the number of adjacent elements that comprise
                     a single cubie.

`-q`   Use quarter-turn metric.

`-r`   Generate and show a random position.

`-R` *#*  Set the seed for the random number generator.

`-s`    Given a set of scrambles on standard in, solve each and write
      a solution to standard out.

`-S`    Do a solve test; for the given tws file, solve sequences of
      increasing length.  The number of moves to add each time can
      be provided immediately after the argument (as in -S20).

`--schreiersims`    Run the Schreier-Sims algorithm to calculate the supergroup
                  size.  Does not account for identical pieces.

`--scramblealg` *scr*  Give a scramble to solve directly on the command line.

`--showmoves`    Given a set of scrambles on standard input (one per line),
               writes to standard output the move-format for those
               scrambles.  This way you can build composite moves; for
               instance, to make a move specification for the antislice
               move "U D" (a clockwise turn of both the up and down faces)
               on the 3x3x3, use the command

                  echo "U D" | ./build/bin/twsearch --showmoves samples/main/3x3x3.tws

`--showpositions`  Given a set of scrambles on standard input (one per line),
                 writes to standard output the scramble-format for those
                 scrambles.

`-t` *#*  Specify the number of threads to use.

`-T`    Do a timing test.

`-u`    Read scrambles on standard input and only output unique ones (by
      the position they reach) on standard output.

`-v`   Increase verbosity.  If followed immediately by a digit, that digit
     sets the verbosity.

## Pruning Tables

The pruning tables are written to a system-dependent cache directory.
On Unix, the default system directory is `~/.cache/`, and on Apple
platforms the default system directory is `~/Library/Caches/`; on
both of these platforms the default can be overrriden by setting the
`XDG_CACHE_HOME` environment variable.
On Windows, the default location is obtained from the `LOCALAPPDATA`
environment variable.

Unless a directory is explicitly specified with the `--cachedir` option,
a `twsearch` subdirectory will be created for the pruning tables.

In environment values and in the `--cachedir` option, a leading tilde
will be expanded with the contents of the `HOME` environment variable.

## Status

What is working so far:

* Parsing ksolve file
* God's algorithm
* Optimal solver for random positions
* Canonical sequence data
* Tree search using canonical sequences
* Write pruning tables
* Read pruning tables
* Parse scramble file
* Solve scramble positions
* QTM solves/pruning tables
* Symmetry reduction (except mirroring)

Things to do:

* Add algebraic support for when reading scrambles
* Add grip information; derive moves according to SiGN
* Print antipodes on two-bit God's algorithm
* Coset solvers

Things to consider:

* Ignore pieces
* Blocking moves


## Rust implementation

This repo contains a pure Rust implementation of the core `twsearch` functionality. It does not support multi-threading yet.

Examples:

```shell
cargo run --release -- search \
  --moves "U,F,R" \
  --min-num-solutions 10 \
  samples/json/3x3x3/3x3x3-Reid.def.json \
  samples/json/3x3x3/T-perm.scramble.json

cargo run --release -- gods-algorithm \
  --moves U,F,R \
  samples/json/2x2x2/2x2x2.kpuzzle.json
```


To get completions in your shell, install using one of:

- `brew install --HEAD cubing/cubing/twsearch`
- `cargo install --path .; twsearch completions <your shell>`

## `twsearch-cpp-wrapper`

This repo also contains a Rust build that wraps the C++ implementation (in a single binary). This allows running a server to connect with web UIs:

```
cargo run --package twsearch-cpp-wrapper -- serve
// Now open https://experiments.cubing.net/cubing.js/twsearch/text-ui.html
```

To run commands similar to the examples above:

```shell
cargo run --package twsearch-cpp-wrapper -- search samples/main/3x3x3.tws samples/main/tperm.scr
cargo run --package twsearch-cpp-wrapper -- gods-algorithm samples/main/2x2x2.tws
cargo run --package twsearch-cpp-wrapper -- search --check-before-solve never --min-num-solutions 20 --generator-moves 2L,2R,U,F samples/main/4x4x4.tws samples/main/flip.scr
cargo run --package twsearch-cpp-wrapper -- search --generator-moves F,R,D,B,L --scramble-alg U samples/main/3x3x3.tws
cargo run --package twsearch-cpp-wrapper -- gods-algorithm --metric quantum --generator-moves U,R,F samples/main/kilominx.tws
```

To get completions in your shell, install using one of:

- `brew install --HEAD cubing/cubing/twsearch-cpp-wrapper`
- `cargo install --path .; twsearch-cpp-wrapper completions <your shell>`

## License

This work is dual-licensed under the Mozilla Public License 2.0 and GPL 3.0 (or
any later version). If you use this work, you can choose either (or both) license terms to adhere to.

`SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later`
