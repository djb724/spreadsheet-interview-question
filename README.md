# Spreadsheet

This was posed to me in an interview where I feel like I got the high-level plan of it correct but fumbled a lot of the implementation. Had to reimplement it for piece of mind.

## Requirements

Create a ```Spreadsheet``` struct that contains a ```get``` and ```set``` method.

The ```get``` function must be in O(1) time but ```set``` can be in any time.

Cells can hold either numeric values or a sum of two other cells.

## Implementation

Use a hash table instead of a 2D array to store cell data. The width and height of the sheet don't need to be known at any given time and this is more memory efficient for arbitrary-sized, sparse data.

Use an additional table of sets for tracking dependencies. If cell B1 references cell A1 in its internal formula, then A1 must cause an update to B1 when it is changed.

When making a call to spreadsheet.set(<A1>), make the change to the internal formula, update the stored value, then trigger updates to all the cell indeces in its set of dependents.

The private function check_circular_dependencies will perform a breadth-first search for self-referential cell references or formulas, preventing infinite loops.

This is not final and there are definitely optimizations that can be made by moving functions around and whatnot.
