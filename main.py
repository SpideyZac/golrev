# pylint: disable=trailing-whitespace
# pylint: disable=missing-module-docstring
# pylint: disable=missing-function-docstring
# pylint: disable=missing-class-docstring

from typing import Optional, List
from dataclasses import dataclass

import numpy as np
from numpy.typing import NDArray
from tqdm import tqdm
from numba import njit

# Type aliases for clarity
Matrix = NDArray[np.int_]


@dataclass
class GameConfig:
    alive_patterns: List[Matrix] = None  # type: ignore
    dead_patterns: List[Matrix] = None  # type: ignore

    def __post_init__(self):
        self.alive_patterns = self._generate_patterns(end_state=1)
        self.dead_patterns = self._generate_patterns(end_state=0)

    def _generate_patterns(self, end_state: int) -> List[Matrix]:
        patterns = []

        if end_state == 1:
            # Rules for cell becoming/staying alive
            for neighbors in [2, 3]:
                patterns.extend(self._generate_single_pattern(1, neighbors))
            patterns.extend(self._generate_single_pattern(0, 3))
        else:
            # Rules for cell dying/staying dead
            patterns.extend(self._generate_single_pattern(1, 1))
            patterns.extend(self._generate_single_pattern(1, 0))

            # Add patterns for underpopulation and overpopulation
            for neighbors in range(4, 9):
                patterns.extend(self._generate_single_pattern(1, neighbors))

            for neighbors in list(range(0, 3)) + list(range(4, 9)):
                patterns.extend(self._generate_single_pattern(0, neighbors))

        # sort by sum of 1s in the matrix ascending if the cell is dead and descending if the cell is alive
        patterns = sorted(patterns, key=np.sum, reverse=end_state == 1)

        return patterns

    @staticmethod
    def _generate_single_pattern(cell_state: int, num_neighbors: int) -> List[Matrix]:
        patterns = []

        # Generate all possible 3x3 matrices (2^9 possibilities)
        for i in range(2**9):
            binary = f"{i:09b}"
            matrix = np.array([int(bit) for bit in binary]).reshape(3, 3)

            # Check if matrix matches our criteria
            center_correct = matrix[1][1] == cell_state
            neighbor_sum_correct = np.sum(matrix) == num_neighbors + cell_state

            if center_correct and neighbor_sum_correct:
                patterns.append(matrix)

        return patterns


@njit
def _is_valid_position(x: int, y: int, width: int, height: int) -> bool:
    return 0 <= x < width and 0 <= y < height


@njit
def _pattern_conflicts_with_boundary(
    pattern: Matrix, new_x: int, new_y: int, width: int, height: int
) -> bool:
    if new_x < 0 and np.any(pattern[:, 0]):
        return True
    if new_x >= width and np.any(pattern[:, 2]):
        return True
    if new_y < 0 and np.any(pattern[0, :]):
        return True
    if new_y >= height and np.any(pattern[2, :]):
        return True
    return False


class GameOfLifeReverser:
    def __init__(
        self, target_state: Matrix, debug: bool = False, show_progress: bool = False
    ):
        self.target_state = target_state
        self.height, self.width = target_state.shape
        self.debug = debug
        self.show_progress = show_progress
        self.config = GameConfig()
        if self.show_progress:
            self.progress_bar = tqdm(total=self.height * self.width, desc="Solving")

    def find_previous_state(self) -> Optional[Matrix]:
        initial_state = np.full((self.height, self.width), -1, dtype=int)
        solution = self._solve_recursively(0, 0, initial_state)

        if self.show_progress:
            self.progress_bar.close()

        return solution

    def _solve_recursively(self, x: int, y: int, state: Matrix) -> Optional[Matrix]:
        current_state = state.copy()
        if self.debug:
            self._log(f"Exploring position ({x}, {y})")

        patterns = (
            self.config.alive_patterns
            if self.target_state[y][x] == 1
            else self.config.dead_patterns
        )

        for pattern in patterns:
            if self.debug:  # so that numpy array print is not being called every time
                self._log(f"Trying pattern: {pattern}")
            if self._try_pattern(x, y, current_state, pattern):
                # Check if we've filled the entire grid
                if self._is_complete(x, y):
                    return current_state

                if self.show_progress:
                    self.progress_bar.update(1)

                # Calculate next position
                next_x = (x + 1) % self.width
                next_y = y + 1 if next_x == 0 else y

                # Recursively try to solve the rest
                if next_y < self.height:
                    result = self._solve_recursively(
                        next_x, next_y, current_state.copy()
                    )
                    if result is not None:
                        return result
                    elif self.show_progress:
                        self.progress_bar.update(-1)

            current_state = state.copy()

        return None

    def _try_pattern(self, x: int, y: int, state: Matrix, pattern: Matrix) -> bool:
        for dy in range(-1, 2):
            for dx in range(-1, 2):
                new_x, new_y = x + dx, y + dy

                # Handle edge cases
                if self.debug:
                    self._log(f"Checking position ({new_x}, {new_y})")
                if not self._is_valid_position(new_x, new_y):
                    if self._pattern_conflicts_with_boundary(pattern, new_x, new_y):
                        return False
                    continue

                # Check if pattern conflicts with existing cells
                pattern_value = pattern[dy + 1][dx + 1]
                if state[new_y][new_x] != -1 and state[new_y][new_x] != pattern_value:
                    if self.debug:
                        self._log("Failed due to cell conflict")
                    return False

                state[new_y][new_x] = pattern_value

        return True

    def _is_valid_position(self, x: int, y: int) -> bool:
        return _is_valid_position(x, y, self.width, self.height)

    def _pattern_conflicts_with_boundary(
        self, pattern: Matrix, new_x: int, new_y: int
    ) -> bool:
        if not self.debug:
            return _pattern_conflicts_with_boundary(
                pattern, new_x, new_y, self.width, self.height
            )
        if new_x < 0 and np.any(pattern[:, 0]):  # Left edge
            self._log("Failed at left edge")
            return True
        if new_x >= self.width and np.any(pattern[:, 2]):  # Right edge
            self._log("Failed at right edge")
            return True
        if new_y < 0 and np.any(pattern[0, :]):  # Top edge
            self._log("Failed at top edge")
            return True
        if new_y >= self.height and np.any(pattern[2, :]):  # Bottom edge
            self._log("Failed at bottom edge")
            return True
        return False

    def _is_complete(self, x: int, y: int) -> bool:
        return y == self.height - 1 and x == self.width - 1

    def _log(self, message: str) -> None:
        if self.debug:
            print(message)

    @staticmethod
    def visualize_state(state: Matrix) -> None:
        symbols = {1: "+", 0: "-", -1: "?"}
        for row in state:
            print("".join(symbols[cell] for cell in row))


def main():
    show_progress = True
    do_logging = False

    # Example pattern (a simple oscillator)
    target_state = np.array(
        [
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        ]
    )

    print("Target state:")
    GameOfLifeReverser.visualize_state(target_state)
    print()

    reverser = GameOfLifeReverser(
        target_state, debug=do_logging, show_progress=show_progress
    )
    solution = reverser.find_previous_state()

    if solution is None:
        print("No initial solution found")
        while solution is None:
            print("Adding random noise")
            rand_x = np.random.randint(0, target_state.shape[1])
            rand_y = np.random.randint(0, target_state.shape[0])
            target_state[rand_y][rand_x] = 1 if target_state[rand_y][rand_x] == 0 else 0

            reverser = GameOfLifeReverser(
                target_state, debug=do_logging, show_progress=show_progress
            )
            solution = reverser.find_previous_state()

        print("New target state:")
        GameOfLifeReverser.visualize_state(target_state)
        print()

    print("Found solution:")
    GameOfLifeReverser.visualize_state(solution)


if __name__ == "__main__":
    main()
