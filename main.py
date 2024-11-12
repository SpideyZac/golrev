# pylint: skip-file

from typing import Optional

import numpy as np


def generate_reversed_matrix(cell_state: int, num_neighbors: int):
    all_reversed_matrix = []

    for i in range(2**9):
        binary = f"{i:09b}"
        matrix = np.array([int(bit) for bit in binary]).reshape(3, 3)

        if np.sum(matrix) == num_neighbors + cell_state and matrix[1][1] == cell_state:
            all_reversed_matrix.append(matrix.tolist())

    return all_reversed_matrix


def generate_reversed_matricies(end_cell_state: int) -> list:
    reversed_matrices = []
    if end_cell_state == 1:
        reversed_matrices.extend(generate_reversed_matrix(1, 2))
        reversed_matrices.extend(generate_reversed_matrix(1, 3))
        reversed_matrices.extend(generate_reversed_matrix(0, 3))
    else:
        reversed_matrices.extend(generate_reversed_matrix(1, 1))
        reversed_matrices.extend(generate_reversed_matrix(1, 0))

        for k in range(4, 9):
            reversed_matrices.extend(generate_reversed_matrix(1, k))

        for k in range(0, 3):
            reversed_matrices.extend(generate_reversed_matrix(0, k))

        for k in range(4, 9):
            reversed_matrices.extend(generate_reversed_matrix(0, k))

    return reversed_matrices


LOGGING = False
ALIVE_REVERSED_MATRICES = generate_reversed_matricies(1)
DEAD_REVERSED_MATRICES = generate_reversed_matricies(0)


class GOLReverser:
    def __init__(self, end_state: np.ndarray):
        self.end_state = end_state
        self.height = end_state.shape[0]
        self.width = end_state.shape[1]

    def find_solution(self) -> Optional[np.ndarray]:
        state = np.zeros((self.height, self.width), dtype=int)
        state.fill(-1)

        solution = self._collapse(0, 0, state)
        return solution

    def _collapse(self, x: int, y: int, state: np.ndarray) -> Optional[np.ndarray]:
        init_state = state.copy()
        log(f"Collapsing at {x}, {y}")
        for matrix_tuple in (
            ALIVE_REVERSED_MATRICES
            if self.end_state[y][x] == 1
            else DEAD_REVERSED_MATRICES
        ):
            applied_matrix = np.array(matrix_tuple)
            log_matrix(applied_matrix)
            log(f"Applying matrix at {x}, {y}")
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
                                log("Failed at first column")
                                break
                            continue
                        if x + dx >= self.width:
                            # check if any 1s in the last column
                            if np.any(applied_matrix[:, 2]):
                                failed = True
                                log("Failed at last column")
                                break
                            continue
                        if y + dy < 0:
                            # check if any 1s in the first row
                            if np.any(applied_matrix[0, :]):
                                failed = True
                                log("Failed at first row")
                                break
                            continue
                        if y + dy >= self.height:
                            # check if any 1s in the last row
                            if np.any(applied_matrix[2, :]):
                                failed = True
                                log("Failed at last row")
                                break
                            continue
                    if (
                        state[y + dy][x + dx] != -1
                        and state[y + dy][x + dx] != applied_matrix[dy + 1][dx + 1]
                    ):
                        failed = True
                        log("Failed at existing cell")
                        break
                    state[y + dy][x + dx] = applied_matrix[dy + 1][dx + 1]

            if failed:
                state = init_state.copy()
                continue

            if y == self.height - 1 and x == self.width - 1:
                log_matrix(state)
                log("\n")
                return state

            log_matrix(state)
            log("\n")
            res = self._collapse(
                x + 1 if x + 1 < self.width else 0,
                y + 1 if y + 1 < self.height and x + 1 == self.width else y,
                state.copy(),
            )
            if res is not None:
                return res
            log(f"Failed at {x}, {y}")
            log_matrix(state)
            log("\n")
            state = init_state.copy()

        log_matrix(state)
        log("\n")
        return None


def log(msg: str):
    if LOGGING:
        print(msg)


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


def log_matrix(matrix: np.ndarray):
    if LOGGING:
        print_matrix(matrix)


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
