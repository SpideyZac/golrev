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

### Code Overview

For those who may be unfamiliar with Rust, a less optimized Python version of the implementation is available in the `main.py` file.

### GameConfig

The `GameConfig` class stores the pattern matrices representing alive and dead cell configurations. These matrices are sorted by the number of alive cells, from lowest to highest. This sorting strategy ensures that matrices with fewer alive cells are prioritized, as they tend to be more versatile and applicable in a wider range of scenarios.

### GameOfLifeReverser

#### `solve_recursively`

The `solve_recursively` function is responsible for attempting to solve the state of a specific cell in the current configuration. It iterates through each pattern matrix and applies the first one that is compatible with the existing state. Upon successfully applying a matrix, it proceeds to solve the next cell. The performance of this function can be influenced by the direction in which it solves the cells, controlled by the `solve_flow_y` flag. While `solve_flow_y = true` generally yields better performance, the optimal configuration may vary depending on the specific problem. If no compatible matrix can be found for a given state, the function returns `None`. If the subsequent cell's solution also returns `None`, backtracking occurs, indicating that the previously selected matrix is not applicable.

#### `try_pattern`

The `try_pattern` function attempts to apply a specific pattern matrix to the current state. It checks whether the matrix’s configuration contradicts any previously defined state values. If any contradictions are found, the pattern is deemed incompatible and cannot be applied.

#### `is_valid_position`

This function verifies whether a particular segment of a pattern matrix can be applied to the current state, taking into account potential out-of-bounds issues.

#### `pattern_conflicts_with_boundary`

In conjunction with `is_valid_position`, the `pattern_conflicts_with_boundary` function determines whether any portion of the matrix that extends beyond the boundaries of the grid would cause a conflict. If the out-of-bounds cells contain zeros, they do not affect the neighbor count, and the matrix can still be applied. However, if there are ones in the out-of-bounds region, the matrix is invalid, as it would alter the count of neighbors inappropriately.

### Main Function

The primary function first attempts to solve the exact target state. If this approach fails, the algorithm introduces random noise into the configuration to facilitate a solvable state.

# Conclusion
The Game of Life Reverser employs a methodical approach to reverse Conway's Game of Life, utilizing pattern matrices and recursive backtracking to reconstruct possible previous states from a given target configuration. Through the strategic application of various pattern matrices and the use of efficient algorithms for state validation, the tool is capable of exploring potential solutions and addressing conflicts as they arise. The flexibility in adjusting the flow direction and introducing noise for solvability further enhances the robustness of the solver. This approach offers a powerful and adaptable method for reverse engineering the Game of Life, providing valuable insights into the temporal dynamics of cellular automata.