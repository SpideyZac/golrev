# Game of Life Reverser: A Temporal Reconstruction Tool

The Game of Life Reverser is a tool designed to reverse the progression of Conway's Game of Life, aiming to identify a potential board configuration that could have led to a given end state.

## Approach Overview

This tool introduces a novel approach to reversing the Game of Life by utilizing **pattern matrices**. These matrices define the conditions that must have existed in a previous state to arrive at a specific cell configuration in the end state. By systematically analyzing the end board and applying these matrices, the tool iteratively works backward in time to reconstruct a possible earlier state.

### Key Concept: Pattern Matrices

For each cell in the final configuration, the program uses predefined pattern matrices that correspond to the possible transitions according to Conway's rules. For example, a cell in the final state that is alive must have come from one of the following configurations:

- A cell that was alive with exactly 2 neighbors.
- A cell that was alive with exactly 3 neighbors.
- A cell that was dead with exactly 3 neighbors.

Consider the following matrix, which represents the scenario where a cell is alive with 2 neighbors:

```
[
    [0, 0, 0],
    [0, 1, 1],
    [1, 0, 0]
]
```

In this matrix, the central cell represents the current cell in the final state, and the surrounding cells specify the required state of its neighbors in the previous configuration.

### Matrix Application and Backtracking

The tool works by iterating through each cell in the final state and attempting to apply a matching matrix. For each cell, the program checks whether the chosen matrix conflicts with the matrices of neighboring cells. If a conflict is detected, the program backtracks and tries alternative matrices, progressively reconstructing earlier states.

For example, if we start with a cell at position (0, 0) and apply a matrix that represents a dead cell with no neighbors:

```
[
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0]
]
```

This application might lead to the following board:

```
[
    [0, 0, ?, ?],
    [0, 0, ?, ?],
    [?, ?, ?, ?],
    [?, ?, ?, ?]
]
```

Subsequently, if we apply a matrix at position (1, 0) that suggests the cell is alive with two neighbors:

```
[
    [1, 0, 0],
    [1, 0, 0],
    [0, 0, 0]
]
```

The board might update as follows:

```
[
    [?, 0, 0, ?],
    [0, 0, 0, ?],
    [?, ?, ?, ?],
    [?, ?, ?, ?]
]
```

In this case, the conflicting patterns—specifically the overlap of pre-existing matrix constraints—forces the tool to backtrack and choose new matrices, refining the solution until a valid configuration is found.

### Some edge cases

If a matrix were to be applied to a cell where it would go out of bounds (i.e. the cell is on the edge of the game board), there are rules which determine whether it can still be applied:

- If the area of the matrix which goes out of bounds does not have any 1s, then it can still be applied as these out-of-bounds 0s will not effect the neighbor count.
- If the area of the matrix which goes out of bounds does contain 1s, then it cannot be applied as these out-of-bounds 1s will effect the neighbor count.

## Running

Without debugging:
```sh
cargo run --release
```

With:
```sh
cargo run --release --features debug
```

## Config

- `FLOW_Y`: Whether to solve in the vertical or horizontal direction. `true` generally leads to better peformance, but can very based on the state.
- `FILE_TO_READ`: The file path to read the initial state from.
- `NUMBER_OF_PASSES`: The number of times to solve for the previous state. For example, if you wanted to calculate 3 states prior, this would be `3`.

## General Tips

Giving the input some buffer 0s (`-`s) can help it solve quicker. This means adding a lot of `-`s to the beginnings and endings of the input.
