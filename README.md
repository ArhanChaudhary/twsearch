Twisty Puzzle searcher.  Much like ksolve but, due to licensing issues on
that program, we have coded it completely from scratch.  We start from a
base of compatibility but do not guarantee it.

Important options (you should specify these):

   -M xxx:  megabytes of memory to use max; should be ~ 1/2 of your RAM and ideally a power of two

   -t xxx:  number of threads to use

Sample usage:

   ./twsearch 3x3x3.ksolve tperm.scr

   ./twsearch -g 2x2x2.ksolve

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

Things to do:

* QTM solves/pruning tables
* Print antipodes on two-bit God's algorithm
* If table size too large, don't degenerate (i.e., 2a.ksolve)
* Coset solvers
* Symmetry reduction

Things to consider:

* Ignore pieces
* Blocking moves
* Are we spending too much time in the zeros?  Time to add values
  lower in the cache lines?  Two bits or four bits?
