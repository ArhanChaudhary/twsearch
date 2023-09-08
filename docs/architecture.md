The twsearch program is my attempt to solve, generate algorithms
for, and otherwise explore twisty puzzles such as the ones generated
by the puzzle geometry component of cubing.js.  This document
describes some of the architectural decisions and implementation
details.

Representing Moves and States:  ksolve vs twsearch

The program originated from my work with ksolve.  When I realized
that the licensing for ksolve was unclear, I started completely
from scratch, using nothing from the ksolve source.  But I did want
to be able to have ksolve compatibility so users of that program
could simply switch to mine with no effort.

This may have been an unfortunate decision, as the ksolve input
format is unnecessarily complicated and verbose.  In particular,
the syntax used for moves and positions is identical, but the
semantics are completely different with respect to orientations.
I can find no trace of why this was done this way, but for compatibility
twsearch also does this.

We start by discussing the puzzle representation used for ksolve
move definitions and positions, internal representation, and
alternatives.  We will start by teasing out the semantics of just
the permutations, and follow this up with the necessary description
of the orientations.

Permutations are well understood, with standard notations that are
well agreed upon.  We will assume the left-to-right application
order, so (p1 * p2) means apply permutation p1 first, then permutation
p2.  Further, we permit this to be written as simply p1 p2 (as we
normally do with puzzle sequence notation).

We will first discuss writing permutations on S_n as a list of n
objects (without parentheses), and later discuss the cycle notation.

For this discussion we will use the standard convention permuting
the set of numbers 1..n.  Internally we will probably instead use
the set of numbers 0..n-1, to match the convention in most programming
languages of 0-based arrays, and all input and output will be
appropriately transformed.

A permutation can be written either as a transformation (the active
form), or as a ordered sequence (the passive form).

In the active form, a permutation written as 2 3 1 4 means that the
item currently in the first slot is moved to the second position
(so sigma(1)=2)), and an item in the second position is moved to
the third position (so sigma(2)=3), and an item in the third position
is moved to the first position (so sigma(3)=1), and the item in the
fourth position is left alone.  Another way to interpret this is
that the item that used to be in the first position is in position
two, the item that used to be in position two is in position three,
and the item that used to be in position three is in position one.
When interpreting such a permutation as a position, the permutation
maps elements to slots.

In the passive form, a permutation written as 2 3 1 4 means that
the second item is currently in the first position, the third item
is currently in the second position, the first item is currently
in the third position, and the fourth item is in the fourth position.
To interpret this as a permutation, this says move the element in
the 2nd slot to the first position, and so on.  When interpreting
such a permutation as a position, the permutation maps slots to
elements, and as such is typically used for positions.

These representations are directly isomorphic; if you invert a
passive permutation, you get the active representation of the
original permutation, and vice versa.  Inversion can be performed
by the following code:

   for (i in 1..n) inv[p[i]] = i ;

So it doesn't matter if we use passive or active representations,
as long as we are clear in what we use.

So what is the difference?  Consider how we multiply two permutations.
If we are using the active representation and left-to-right application
order, then sigma_p1(a)=b means piece a moves to position b, and
then sigma_p2(b)=c means that it moves further to c, so sigma_(p1*p2)(a)
= sigma_p2(sigma_p1(a)).  Code to multiply two permutations looks
like this:

   for (i in 1..n) p3[i] = p2[p1[i]]

But if we are using the passive representation, with left-to-right
application order, then sigma_p1(a)=b means the piece that was in
b moves to a, and sigma_p2(c)=a means that the piece that was in a
moves to c, so sigma_(p1*p2)(a) = sigma_p1(sigma_p2(a)), or in code:

   for (i in 1..n) p3[i]=p1[p2[i]]

These are very different multiplication methods; getting multiplication
correct is fundamental.

In ksolve, according to the documentation, both moves and positions
are given as active permutations.  This is the standard in mathematics,
as well.  The quote from the readme is as follows:

"A permutation vector is a list of the numbers from 1 to n in some
order, such as 231456. This describes where each piece goes - for
instance, the 2 in the first spot means that piece number 1 is in
spot 2."

So this says that ksolve is using the element to slot notation,
which is the active form.  Yet, if you look at the 4x4 centers
definition file distributed with ksolve, the permutation vector for
the solved position is given as

    1 1 1 1 2 2 2 2 3 3 3 3 4 4 4 4 5 5 5 5 6 6 6 6

and this cannot be an active permutation since it would try to cram
multiple elements into the same place.  This has to be a passive
permutation, mapping slot to element, with some identical elements.
Further, if you use the standard 3x3 def file, which has the solved
position for corners as 12345678, and the U move as 41235678 (the
corner orders are given as URF, ULF, ULB, URB, DRF, DLF, DLB, DRB),
then the U move says that the 4 (URB) in the first (URF) spot means
that piece 1 (URF) is in slot 4 (URB)".  But this is absolutely not
what the U move does; the U move puts the piece 4 (URB) in slot 1
(URF).  So ksolve is clearly using passive format throughout.

If you look at the ksolve code, in the move routine and in the merge
moves routine (which are both effectively multiplication, but
actually differ in terms of orientation, which we will be getting
to), the actual code used is:

   for (i in 1..n) p3[i] = p1[p2[i]]

This matches the passive mode multiplication procedure we derived above.

In group theory, with no identical pieces, there is no a-priori
difference between a move and a position; they are both simply a
permutation.  One can think of a move as the position obtained after
making that move from the identity position; similarly, one can
think of a position as the composite move required to obtain that
position from the identity.

So far this agrees with the tests I've performed on ksolve as well
as its code, though it contradicts its documentation.  It's also
the most logical code; if we need to support identical pieces, our
positions must be passive, not active, and if our positions are
passive, it's easiest if our moves are passive as well.  This also
means that the move descriptions are the same as descriptions of
positions after the move, assuming the solved state is the identity
permutation, which makes writing them fairly easy.  Further, the
permutation vector in the solved state doesn't matter at all, except
in that it allows the definition of identical pieces.

Let's move on to orientations.

Orientations are just a concise way of specifying larger permutations.
For instance, on the 3x3x3 cube, there are 54 stickers, 9 on each
face; if you assume the centers do not move, then there are only
48.  When describing this as a group to software such as GAP or
Mathematica, you would describe it as a group generated by elements
of S_48 or S_54, and be done.  But there are good reasons to separate
the permutation into orbits (what ksolve calls sets) and further
to rewrite some of the orbits in terms of smaller permutations and
orientations rather than a larger permutation.

Each orientation value expands a single permutation element in the
small permutation to several in the larger permutation element.
These elements collectively move together, potentially cycling due
to rotations in three-dimensional space.  So instead of having 24
stickers moving on eight cubies, we can instead have the eight
cubies moving, and maintain how far each cubie is rotated against
some standard configuration.  This standard configuration involves
marking a single face as the primary face on every cubie, and then
keeping track of how far clockwise from the marked slot face the
actual cubie face is.

In passive notation the permutation maps slots to elements, so our
orientation should also map slots to orientations.  So if the value
at index a in our permutation is b and in our orientation is c,
that would seem to mean that piece b is currently in slot a in
orientation c.  This is a good representation in that it has the
following property:  orientation can be separated from permutation
for state exploration purposes, because when multiplying to states
s1 and s2, the orientation change depends only on s2 (the "move",
usually) and not on s1.  So we can index the orientation separately
from the permutation, which enables a lot of nice functionality.
Other orientation conventions complicate this.

(This is even nicer if you can arrange so the largest number of
moves do not affect the orientation in the solved state, because
then a pruning table created from orientation tends to have somewhat
higher values.)

Ksolve uses this representation for states, but critically, not for
moves.  For moves, ksolve maps cubie numbers to orientations.  So
ksolve states and moves are not the same representation of a
permutation, and this affects (and complicates) the code.

To convert a state to a move:

   m.o[p[i]] = s.o[i]

Note that the permutations are the same.  To convert a move to a
state:

   s.o[i] = m.o[p[i]]

So for move 13425,01201 we convert to state 13425,02011.
For state 13425,02011 we convert to move 13425,01201.

sub movetostate      no[i]=o[p[i]] ;
sub statetomove      no[p[i]]=o[i] ;
sub sss      no[i] = (o1[p2[i]]+o2[i])%omod ;
sub sms      no[i] = (o1[p2[i]]+o2[p2[i]])%omod ;
sub mss      no[i] = (o1[p1[p2[i]]]+o2[i])%omod ;
sub mms      no[i] = (o1[p1[p2[i]]]+o2[p2[i]])%omod ;
sub mmm      no[p1[i]] = (o1[p1[i]]+o2[i])%omod ;
sub smm      no[p1[i]] = (o1[i]+o2[i])%omod ;
sub msm      no[p1[p2[i]]] = (o1[p1[p2[i]]]+o2[i])%omod ;
sub ssm      no[p1[p2[i]]] = (o1[p2[i]]+o2[i])%omod ;
sub sinv      no[p[i]] = (omod-o[i])%omod ;
sub minv      no[i] = (omod-o[p[i]])%omod ;

The twsearch program uses state representation everywhere.  In order
for this to work we need to convert the ksolve moves read in to be
in state representation rather than ksolve's move representation.

With identical pieces states always have to be on the left side of
any multiplication, but this is true for ksolve as well.

The twsearch input description

The twsearch input description format is cribbed directly from ksolve
(although I don't support all the features that ksolve has, and I do
support some features that ksolve does not have).  The parser for the
format is ad-hoc C++ code because I did not want to introduce extra
dependencies on parsers or lexers.  The input description is line-based,
with comments preceded by a hash symbol (#) and blank lines ignored.

To motivate this section, we list here the initial part of the 3x3x3
twsearch definition file (omitting some moves for brevity):

# PuzzleGeometry 0.1 Copyright 2018 Tomas Rokicki.
Name PuzzleGeometryPuzzle

Set EDGE 12 2
Set CORNER 8 3

Solved
EDGE
1 2 3 4 5 6 7 8 9 10 11 12
0 0 0 0 0 0 0 0 0 0 0 0
CORNER
1 2 3 4 5 6 7 8
0 0 0 0 0 0 0 0
End

Move F
EDGE
10 1 3 4 2 6 7 8 9 5 11 12
1 1 0 0 1 0 0 0 0 1 0 0
CORNER
7 1 3 2 5 6 4 8
2 1 0 2 0 0 1 0
End

Tokens on a line are whitespace separated.  The first command in the file
must be a Name command, which must be followed by a single token that
is used as the puzzle name.  (The name is not presently used for anything.)

After that there must be one or more Set commands.  Each Set command
defines pieces of a particular orbit, which consists of pieces that might be
exchanged for each other.  For instance, on the standard 3x3x3, the two
orbits are corners and edges; corners can never be swapped with edges, and
edges can never be swapped with corners.

Each Set command must have a total of four tokens: first, the Set command
itself, then a name for the set (which must be unique among the sets),
then a count of pieces in that set, and finally, a count of the number
of ways that a piece might be oriented (which must be an integer between
1 and 126 inclusive).  The orientation count must be supplied even if it is
1.  There is no current limit on the count of sets.

Following the Set commands there must be a Solved command.  The Solved
command is a single token on one line, followed by a position block,
which is terminated by an End line.  The position block contains zero
or more set position chunks.  Each set position chunk consists of two or
three lines.  The first line is the name of the set, on a single line.
Next is the identity of elements of that set.  Finally, we have the
optional orientation values of elements of that set; if not provided,
all zeros are assumed.

As a future-looking extension, the "Solved" command can be replaced with
the "StartState" command, with the following change in behavior.
The element numbering is zero-based rather than one-based.  This is
consistent with cubing.js states (and permutations) and with the
internal representation.

For set position blocks, we use element identities which are numbered
from 1 to the count of distinct identities.  There may be duplications
in the case some elements are indistinguishable, but the range of
values must be contiguous from 1.  If all pieces are distinguishable,
then the element identity must consist of some permutation of the
numbers 1 through the number of elements.

In a position block, if any sets are not listed, they default to the
identity permutation (with all pieces distinguishable) and the all-zero
orientation vector.

For the element orientation specification, in addition to values from
0 to one less than the orientation count for the set, an element can
have the distinguished value '?' which indicates an orientation
wildcard.  All elements that are indistinguishable and share the same
numeric identifier must either have a numeric orientation or they must
all be orientation wildcards.

After the Solved block, we then have a sequence of Move or Illegal
blocks.  We'll describe the Move blocks first.

Each Move block describes either a move (something that changes the
puzzle state) or a rotation (something that rotates the whole puzzle
in space without actually changing the position of any piece with
respect to any other piece).  The Move command is on a line with two
tokens, and the second token is the unique name of the move or rotation.

Rotations are distinguished from Moves because they consist of one or
more upper case letters or underscores followed by a lowercase v, or
they consist of x, y, or z possibly followed by 2 or a single quote.
Anything else is a move.  So Uv, FRBv, Z_Av, and x' are rotations,
while X, F, xx, and T are just moves.

Following the Move command line is a transformation block.  The
transformation block has the same syntax as a position block, except
the element identity line is instead a permutation line, and must
always contain a proper permutation with the numbers 1 through the
number of elements.  Also, if an orientation vector is provided for
a set transformation block, all values must be numeric; orientation
wildcards are not permitted as part of a transformation.

Identical pieces are useful in puzzles like Rubik's Revenge (the 4x4x4)
where there are some pieces that are generally indistinguishable,
but they, with orientation wildcards, are also useful when evaluating
different steps of solution methods.  For instance, when solving the
last layer edge orientation step of the 3x3x3, you might make all last
layer edges identical and oriented, and all last layer corners identical
but unoriented.

Move extension

When specifying moves, only a "base" move needs to be described.  A base
move is one like "U" on the 3x3x3; it's the one that generally is not
decorated with a suffix like an apostrophe or number.  For every such
move or rotation given, twsearch will generate the decorated versions by
calculating the order of the move (the number of times it must be
executed to yield the identity), and then selecting appropriate decorations.
For instance, on the 3x3x3, twsearch displays the following output
indicating what decorated moves it generates:

Created new moves F2 F' B2 B' D2 D' U2 U' L2 L' R2 R'

For the megaminx, it will generate moves including U2'; in general
it generates repetitions in both the clockwise direction of up to
and including half a full rotation, and in the counterclockwise
direction of up to but not including half a full rotation.

As a future-looking extension, the "Move" command can be replaced
with the "MoveTransformation" command, with the following changes
in behavior.  First, the numbering is zero-based rather than
one-based.  Secondly, the orientations are given in the same way
as for states, rather than in the awkward ksolve-style representation.
We recommend any new software interacting with twsearch use this
format as it is more logical and easier to reason about than the
ksolve format.

Symmetry and rotations

In general when calculating solutions or most other operations,
rotations are not used; only moves are used.  However, rotations
can be used when providing an algorithm or scramble sequence as
input.  Further, when generating pruning tables or performing a
God's algorithm search, rotations are used to perform symmetry
reduction, which can increase the speed and memory effectiveness
of the algorithms.  Twsearch will calculate the full rotation group
from the given rotations and perform symmetry reduction on that
group.  Twsearch does not yet perform mirror symmetry reduction or
inverse "symmetry" reduction (but it may in the fugure).  If a move
subset is specified, it will only use the symmetries that preserve
that move subset; for instance, on the 3x3x3, if you specify the
move subset U,R,F, then only the three-element symmetry group that
preserves the URF corner will be used.

Internal puzzle representation

The primary internal data structures are the puzzle definition
(puzdef), the set definition (setdef), and the set value (setval).
The setval is a bad name because it actually stores the full state
of all sets in one contiguous memory array.  We will describe these
structures from the bottom up, starting with the setval, then the
setdef, then the puzdef.

The setval is just a contiguous array of unsigned bytes, with two
bytes for each set element.  The values for the sets are stored in
the same order the sets occur in the input file, which is the same
order they occur in the setdef array in the puzdef.  For a particular
set, with say n elements, we have n unsigned characters storing the
permutation (for the case of a transformation) or the element numbers
(in the case of a state); then we have a further n unsigned characters
storing the orientation values in the same order.  Permutations and
element ids are stored using a base of zero (despite being read
with a base of 1).

The permutation portion of a state or transformation is always
represented in passive mode:  array element i holding value j means
that a piece with id j is in slot i (for a state), or that the
element in slot j should be moved to slot i (for a permutation).
States can hold duplicated values within the permutation elements,
but transformations must always contain each of the values 0..n-1
exactly once.

The orientation is stored exactly as given in the input for states,
with the value in slot i representing how much twisting is applied
to the cubie that is currently in slot i (thus, orientation is moved
along with the cubies).  If an orientation is a wildcard (given as
a question mark on the input), then it is stored as the special
value 2m (where m is the modulus of the orientation).  As a special
performance optimization, to avoid unpredictable branches and slow
division operations, when two orientations are added together when
performing a move, a small table is used rather than an actual
arithmetic instruction, and this table preserves the don't care
special values of 2m.

Storing the entire state of the puzzle (or transformation in the
case of a move) in one contiguous sequence of bytes helps cache
locality and simplifies some operations such as hashing of a state,
as we will see.

The setval structure itself is just a pointer to the actual array
of unsigned characters; since it is a single pointer, it is normally
stored in a register.  The setval constructor takes a pointer to
an appropriately-sized memory chunk, but does not manage the
lifetime of the memory chunk itself.

Currently twsearch uses two distinct types, stacksetval and
allocsetval, which manage lifetimes of setvals.  allocsetval is
intended for general but infrequent use, as in when reading a
puzzle definition or doing low-frequency things.  stacksetval is
intended to be used in a stack-based manner, as in automatic
values that track some recursive control flow.  I intended
stacksetval to be very fast (that is, not churn the memory
allocator) by maintaining a stack of available values on the
puzzle definition, but this fails to take into account threading.
So all the performance-critical routines, such as search,
allocate a vector of allocsetvals, one vector per thread, and
pass this vector into the recursion to use so memory allocation is
not on the critical path.

<><> setdef <><>

<><> puzdef <><>

Canonical sequences

The notion of canonical sequences is at the root of effective search
in twisty puzzles.  Solving twisty puzzles usually involves doing
a meet-in-the-middle search from both a given position and the solved
position, attempting to identify a position approximately midway for
which we have or can easily constract a path from the given position
to solved through that position.  Since the solved position is fixed
for any position we might want to solve, we can precalculate the
solved half of the meet-in-the-middle search and store it in what's
called a pattern database or pruning table, and reuse that computation
across many positions.

Calculating all the positions near a given position would normally
be done with a breadth-first search, but a breadth-first search requires
all the positions at one or more levels to be in memory at one time.
Managing a large set of positions in memory like this, and checking
whether a position is new or has been seen before, can be slow and
consume a lot of memory.  On the other hand, naive depth-first search
can explore duplicated positions an excessive number of times.  The
concept of canonical sequences, and restricting the search tree to
these sequences, can give search in twisty puzzles the speed of dfs
with very close to the node efficiency of breadth-first search.

The basic concept is to identify redundancies in the search tree that
are caused by moves that commute, and restrict the search to the first
lexicographical path among those that have commuting moves.  For instance,
on the 3x3x3, the moves L and R commute.  So, the path L R leads to the
same position as R L, and so we can simply restrict search from going
down any path that has an R immediately followed by an L.  This would
also apply to moves such as R2 L'.

For the 3x3x3, the restriction is very simple; we can simply disallow
any R followed by any L, or any F followed by any B, or any U followed
by any D, and we've eliminated most redundancies in the search graph.
There are still a few (such as D2 U2 L2 R2 leading to the same position
as L2 R2 D2 U2), but these remaining redundancies do not significantly
increase the runtime.

The 3x3x3 and larger cubes can be handled so simply because the commutes
relation between moves is associative; if A commutes with B, and B commutes
with C, so simply ordering the sets of moves that commute solves the
problem.  But other puzzles, like the megaminx, have a more complicated
commutes relation, and thus require somewhat more sophistication.  On
the megaminx, L commutes with R, and L commutes with BR, but R does not
commute with BR.  The solution is to say a path is legal if there is no
other path that can be reached just by swapping adjacent commuting
moves, and get a path that is lexicographically smaller.

In order to do this in the half-turn metric, we can simply remember the
set of moves in the search sequence so far that are not followed by a
move that does not commute with that move.  This set of moves can be
considered a representation of the state in a state machine, and when
in that state, if a move commutes with any of the moves in this state
and occurs earlier lexicographically, then this move is not allowed.
Also, if a move twists the same grip as one of the moves in the set,
it's also not allowed as being redundant.

To make the code to perform this calculation during search as simple
as possible, we precalculate a finite state machine, with an integer
state, a next state table indexed by the state and the move class, and a
bitmask of unpermitted moves for each state.  The move class is just
an integer referring to the base move, so U, U2, and U' all have the
same move class.  Twsearch currently limits the number of move classes
to 63 when using canonical sequences; this should suffice for some
very large puzzles, and it's unlikely search will be performed for
puzzles with more classes.

For the quarter-turn metric, the approach is the same, except in
addition to commuting moves, we need to track how many quarter turns
have been performed for the most recent move, and what direction
they are in; for instance, on the 3x3x3 we permit U, U U, and U',
but no other sequence of U moves.  This is blended into the state
machine (and each distinct direction of a particular move gets
its own move class).

The number of states of such a machine might get very large if there
are a lot of commuting moves.  For instance, on the 20x20x20, there
are roughly three million states in the finite state machine that is
computed using the above procedure.  To help limit the number of
states without changing the semantics, we perform the following
"big cube optimization".  If any two move classes have an exactly
identical commutes-with relation (as do any pair of move classes on
the same axis of a big cube), then we only retain the lexicographically
least of these in our set of moves state.  Thus, if 4U < 6U in our
move class ordering, and the current state includes 6U, and we move
4U, we would drop 6U.  This will not change the set of moves
permitted, since the commutes-with relation is the same between these
moves, and since any move that is lexicographically less than 4U is
also lexicographically less than 6U.  For the 20x20x20, this reduces
our state machine size to 61 states in the face-turn metric.

Returning to our original objective, which was to gain the memory
efficiency and speed of depth-first search with the redundancy
elimination of breadth-first search, let's see how well our approach
works.  There are a total of 232,248,063,316 unique positions at
distance 10 for the Rubik's cube.  If we perform a depth-first
search without any move restrictions, we'd explore 18^10 or 
3,570,467,226,624 leaves, which is about 15 times too many.  Just
ensuring that we don't repeat moves on a face reduces this to
18 * 15^9 or 691,980,468,750, still very close to three times too
many.  But our canonical sequence approach as described above
only explores 244,686,773,808 nodes at distance 10, which is an
excess of only 5.3%.  That's an excellent result.

For puzzles where one piece always remains in place, like the
core on odd cubes or the megaminx, this approach works great.  But
some puzzles, like the pentultimate, the 2x2x2, the 4x4x4, and
others, permute all parts of the puzzle with some moves.  Thus,
there are combinations of moves that simply spatially rotate the
puzzle without effectively changing the relation of pieces to one
another.  For these puzzles, the canonical sequences approach does
not perform quite as well.  For instance, on the 2x2x2, allowing
all six face turns, the branching factor is 13.33.  Yet, we can
solve the 3x3x3 only turning three of the faces; any twist of one
of the other faces can be performed by a twist of the opposite
face followed by a whole cube rotation.

The best approach to remedy this is to generate a move set that
fixes some part of the puzzle.  For instance, a corner cubie can be
fixed on even cubies, and other moves changed to compensate.  For
the 4x4x4 and outer block turn metric, we simply eliminate all
outer block turns that turn (say) the BLD corner and perform our
search; postprocessing the results can expand given solutions into
many alternate solutions using all moves, if desired.  For the 
slice turn metric on the 4x4x4, we'd again fix the BLD corner, but
now we'd transform the D move into 3u and require postprocessing
to fix the sequence.  Note that cubing.js can easily generate
twizzle .tws files with fixed corners or edges using the --fixcorner
or --fixedge option.

But twsearch does have a facility for helping to improve search
for puzzles like this in cases where you do not want to, or cannot,
generate a modified set of moves.  If you specify --newcanon n where
n is an integer number greater than zero, it will generate only
sequences where every subsequence of length n or shorter is the
lexicographically first that generates a particular sequence.  You
will need to experiment with what number n works for a given puzzle;
larger n will take longer and generate larger finite state machines.
But with a reasonable value of n, some portion of the branching
factor inflation due to full-puzzle rotations, or other types of
short identity sequences, will be reduced.  For example, on the
2x2x2, we list here the options and the resulting branching factor:

      option       states  branching factor
    ------------  -------  ----------------
    (default)           7          13.3485
    --newcanon 1       19          13.3485
    --newcanon 2      262          11.8098
    --newcanon 3     3136          10.1869
    --newcanon 4    31136           8.0426
    --newcanon 5   236552           6.2356

Now let's do the same for the pentultimate:

      option       states  branching factor
    ------------  -------  ----------------
    (default)          13          41.9089
    --newcanon 1       49          41.9089
    --newcanon 2     2065          40.2458
    --newcanon 3    83065          38.0152
    --newcanon 4  3138180          35.0140

As you can see, this approach is very effective on the 2x2x2, but that's
a trivial puzzle anyway; on the Pentultimate, it's only a little effective.
For comparison, using a fixed-corner metric on the 2x2x2 gives a branching
factor of 6, and on the pentultimate it gives a branching factor of 24.

With the -C option to twsearch, the count of canonical sequences at
each distance is generated; this can be used to calculate a lower bound
on God's number for a particular puzzle (if you know the number of states
of the puzzle, which you can get with the --schreiersims option and
possibly take into account full-puzzle rotations and identical pieces).

Hashing

Indexing

Compact internal state representation

Pruning tables in memory

Reading and writing pruning tables

Compression of pruning tables

Solve routines

Schreier-Sims

God's Algorithm searches

Compact external puzzle representation

Utility routines
* Uniquify
* Conversion algorithm -> position
* Shorten
* Invert

All options

Extra stuff

To do

* Make solve be iterative rather than recursive
  * Can prefetch/parallelize memory lookups better
  * Can pause and restart search (ala generators)
  * Might be faster

* Improve compression and decompression speed

* Support moves-from-algorithms in input description

* When --showpositions or --showmoves, use StartState and/or
  MoveRepresentation rather than the older formats

* Tighten setdef; if orientationmod=1 don't store orientation;
  consider combining orientaiton and permutation if their
  product is small enough (and don't cares "work").

* Fix allocsetval and stacksetval to be the same class and to
  properly manage the state memory.
