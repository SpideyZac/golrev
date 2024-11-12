# pylint: skip-file

import numpy as np
from functools import lru_cache
from typing import Optional


class GOLReverser:
    def __init__(self, end_state: np.ndarray):
        self.end_state = end_state
        self.height = end_state.shape[0]
        self.width = end_state.shape[1]

    @lru_cache(maxsize=None)
    def _generate_reversed_matricies(self):
        reversed_matrices = []

        for i in range(self.height):
            row = []
            for j in range(self.width):
                cell_matrices = []

                if self.end_state[i][j] == 1:
                    cell_matrices.extend(self._generate_reversed_matrix(1, 2))
                    cell_matrices.extend(self._generate_reversed_matrix(1, 3))
                    cell_matrices.extend(self._generate_reversed_matrix(0, 3))
                else:
                    cell_matrices.extend(self._generate_reversed_matrix(1, 1))
                    cell_matrices.extend(self._generate_reversed_matrix(1, 0))

                    for k in range(4, 9):
                        cell_matrices.extend(self._generate_reversed_matrix(1, k))

                    for k in range(0, 3):
                        cell_matrices.extend(self._generate_reversed_matrix(0, k))

                    for k in range(4, 9):
                        cell_matrices.extend(self._generate_reversed_matrix(0, k))

                row.append(tuple(cell_matrices))  # Convert to tuple for immutability
            reversed_matrices.append(tuple(row))

        return tuple(reversed_matrices)

    def _generate_reversed_matrix(self, cell_state: int, num_neighbors: int):
        all_reversed_matrix = []

        for i in range(2**9):
            binary = f"{i:09b}"
            matrix = np.array([int(bit) for bit in binary]).reshape(3, 3)

            if (
                np.sum(matrix) == num_neighbors + cell_state
                and matrix[1][1] == cell_state
            ):
                all_reversed_matrix.append(matrix.tolist())

        return all_reversed_matrix

    def find_solution(self) -> Optional[np.ndarray]:
        state = np.zeros((self.height, self.width), dtype=int)
        state.fill(-1)

        solution = self._collapse(0, 0, state)
        return solution

    def _collapse(self, x: int, y: int, state: np.ndarray) -> Optional[np.ndarray]:
        init_state = state.copy()
        reversed_matricies = self._generate_reversed_matricies()
        # print(f"Collapsing at {x}, {y}")
        for matrix_tuple in reversed_matricies[y][x]:
            applied_matrix = np.array(matrix_tuple)
            # print_matrix(applied_matrix)
            # print(f"Applying matrix at {x}, {y}")
            failed = False
            for dx in range(-1, 2):
                for dy in range(-1, 2):
                    if (
                        x + dx < 0
                        or x + dx >= self.width
                        or y + dy < 0
                        or y + dy >= self.height
                    ):
                        if x + dx < 0:
                            # check if any 1s in the first column
                            if np.any(applied_matrix[:, 0]):
                                failed = True
                                # print("Failed at first column")
                                break
                            continue
                        if x + dx >= self.width:
                            # check if any 1s in the last column
                            if np.any(applied_matrix[:, 2]):
                                failed = True
                                # print("Failed at last column")
                                break
                            continue
                        if y + dy < 0:
                            # check if any 1s in the first row
                            if np.any(applied_matrix[0, :]):
                                failed = True
                                # print("Failed at first row")
                                break
                            continue
                        if y + dy >= self.height:
                            # check if any 1s in the last row
                            if np.any(applied_matrix[2, :]):
                                failed = True
                                # print("Failed at last row")
                                break
                            continue
                    if (
                        state[y + dy][x + dx] != -1
                        and state[y + dy][x + dx] != applied_matrix[dy + 1][dx + 1]
                    ):
                        failed = True
                        # print("Failed at existing cell")
                        break
                    state[y + dy][x + dx] = applied_matrix[dy + 1][dx + 1]

            if failed:
                state = init_state.copy()
                continue

            if y == self.height - 1 and x == self.width - 1:
                # print_matrix(state)
                # print()
                return state

            # print_matrix(state)
            # print()
            res = self._collapse(
                x + 1 if x + 1 < self.width else 0,
                y + 1 if y + 1 < self.height and x + 1 == self.width else y,
                state.copy(),
            )
            if res is not None:
                return res
            # print(f"Failed at {x}, {y}")
            # print_matrix(state)
            # print()
            state = init_state.copy()

        # print_matrix(state)
        # print()
        return None


def print_matrix(matrix: np.ndarray):
    for row in matrix:
        string = ""
        for cell in row:
            if cell == 1:
                string += "+"
            elif cell == 0:
                string += "-"
            else:
                string += "?"
        print(string)


if __name__ == "__main__":
    to_rev = np.array(
        [
            [0, 0, 0, 0, 0, 0],
            [0, 0, 1, 1, 0, 0],
            [0, 1, 0, 0, 1, 0],
            [0, 1, 1, 1, 1, 0],
            [0, 1, 0, 0, 1, 0],
            [0, 0, 1, 1, 0, 0],
            [0, 0, 0, 0, 0, 0],
        ]
    )

    print("Target state:")
    print_matrix(to_rev)
    print()

    reverser = GOLReverser(to_rev)
    solution = reverser.find_solution()

    if solution is not None:
        print("Found solution:")
        print_matrix(solution)
    else:
        print("No solution found; Adding random noise until a solution is found")
        while solution is None:
            print("Adding random noise")
            rand_x = np.random.randint(0, to_rev.shape[1])
            rand_y = np.random.randint(0, to_rev.shape[0])
            to_rev[rand_y][rand_x] = 1 if to_rev[rand_y][rand_x] == 0 else 0
            reverser = GOLReverser(to_rev)
            solution = reverser.find_solution()

        print("New Target state:")
        print_matrix(to_rev)

        print("Found solution:")
        print_matrix(solution)