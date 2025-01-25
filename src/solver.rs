use ndarray::{Array2, Axis};
use rayon::prelude::*;

type Matrix = Array2<i8>;

fn count_ones(matrix: &Matrix) -> usize {
    matrix.iter().filter(|&&x| x == 1).count()
}

struct GameConfig {
    alive_patterns: Vec<Matrix>,
    dead_patterns: Vec<Matrix>,
}

impl GameConfig {
    fn new() -> Self {
        let alive_patterns = Self::generate_patterns(1);
        let dead_patterns = Self::generate_patterns(0);

        println!("Alive patterns: {}", alive_patterns.len());
        println!("Dead patterns: {}", dead_patterns.len());

        Self {
            alive_patterns,
            dead_patterns,
        }
    }

    fn generate_patterns(end_state: i8) -> Vec<Matrix> {
        let mut patterns = Vec::new();

        if end_state == 1 {
            // Rules for cell becoming/staying alive
            for &neighbors in &[2, 3] {
                patterns.extend(Self::generate_single_pattern(1, neighbors));
            }
            patterns.extend(Self::generate_single_pattern(0, 3));
        } else {
            // Rules for cell dying/staying dead
            patterns.extend(Self::generate_single_pattern(1, 1));
            patterns.extend(Self::generate_single_pattern(1, 0));

            // Add patterns for underpopulation and overpopulation
            for neighbors in 4..9 {
                patterns.extend(Self::generate_single_pattern(1, neighbors));
            }

            let neighbor_counts: Vec<i8> = (0..3).chain(4..9).collect();
            for neighbors in neighbor_counts {
                patterns.extend(Self::generate_single_pattern(0, neighbors));
            }
        }

        patterns.sort_by(|a, b| count_ones(a).cmp(&count_ones(b)));
        patterns
    }

    fn generate_single_pattern(cell_state: i8, num_neighbors: i8) -> Vec<Matrix> {
        let mut patterns = Vec::new();

        // Generate all possible 3x3 matrices (2^9 possibilities)
        for i in 0..(1 << 9) {
            let mut matrix = Array2::zeros((3, 3));
            for bit in 0..9 {
                let y = bit / 3;
                let x = bit % 3;
                matrix[[y, x]] = (i >> bit) & 1;
            }

            // Check if matrix matches our criteria
            let center_correct = matrix[[1, 1]] == cell_state;
            let neighbor_sum = matrix.sum() - matrix[[1, 1]];

            if center_correct && neighbor_sum == num_neighbors {
                patterns.push(matrix);
            }
        }

        patterns
    }
}

pub struct GameOfLifeReverser {
    target_state: Matrix,
    height: usize,
    width: usize,
    config: GameConfig,
    solve_flow_y: bool,
}

impl GameOfLifeReverser {
    pub fn new(target_state: Matrix, solve_flow_y: bool) -> Self {
        let height = target_state.nrows();
        let width = target_state.ncols();
        let config = GameConfig::new();

        let per_cell_patterns = config.alive_patterns.len() + config.dead_patterns.len();
        let big_int = num_bigint::BigUint::from(per_cell_patterns);
        println!("Number of patterns per cell {}", big_int);
        println!(
            "Total number of patterns {}",
            num_traits::pow(big_int, (height * width) as usize)
        );

        Self {
            target_state,
            height,
            width,
            config,
            solve_flow_y,
        }
    }

    pub fn find_previous_state(&mut self) -> Option<Matrix> {
        let mut initial_state = Array2::from_elem((self.height, self.width), -1);
        let result = self.solve_recursively(0, 0, &mut initial_state, true);

        result
    }

    fn solve_recursively(
        &self,
        x: usize,
        y: usize,
        state: &mut Matrix,
        first: bool,
    ) -> Option<Matrix> {
        let patterns = if self.target_state[[y, x]] == 1 {
            &self.config.alive_patterns
        } else {
            &self.config.dead_patterns
        };

        if !first {
            for pattern in patterns {
                let mut current_state = state.clone();

                if self.try_pattern(x, y, &mut current_state, pattern) {
                    if self.is_complete(x, y) {
                        return Some(current_state);
                    }

                    let next_x;
                    let next_y;
                    if self.solve_flow_y {
                        next_y = (y + 1) % self.height;
                        next_x = if next_y == 0 { x + 1 } else { x };
                    } else {
                        next_x = (x + 1) % self.width;
                        next_y = if next_x == 0 { y + 1 } else { y };
                    }

                    if next_y < self.height {
                        if let Some(result) =
                            self.solve_recursively(next_x, next_y, &mut current_state, false)
                        {
                            return Some(result);
                        }
                    }
                }
            }

            return None;
        } else {
            let results = patterns.par_iter().find_map_any(|pattern| {
                let mut current_state = state.clone();

                if self.try_pattern(x, y, &mut current_state, pattern) {
                    if self.is_complete(x, y) {
                        return Some(current_state);
                    }

                    let next_x;
                    let next_y;
                    if self.solve_flow_y {
                        next_y = (y + 1) % self.height;
                        next_x = if next_y == 0 { x + 1 } else { x };
                    } else {
                        next_x = (x + 1) % self.width;
                        next_y = if next_x == 0 { y + 1 } else { y };
                    }

                    if next_y < self.height {
                        if let Some(result) =
                            self.solve_recursively(next_x, next_y, &mut current_state, false)
                        {
                            return Some(result);
                        }
                    }
                }

                None
            });

            return results;
        }
    }

    fn try_pattern(&self, x: usize, y: usize, state: &mut Matrix, pattern: &Matrix) -> bool {
        for dy in -1..=1 {
            for dx in -1..=1 {
                let new_x = x as i32 + dx;
                let new_y = y as i32 + dy;

                if !self.is_valid_position(new_x, new_y) {
                    if self.pattern_conflicts_with_boundary(pattern, new_x, new_y) {
                        return false;
                    }
                    continue;
                }

                let pattern_value = pattern[[(dy + 1) as usize, (dx + 1) as usize]];
                let new_x = new_x as usize;
                let new_y = new_y as usize;

                if state[[new_y, new_x]] != -1 && state[[new_y, new_x]] != pattern_value {
                    return false;
                }

                state[[new_y, new_x]] = pattern_value;
            }
        }

        true
    }

    fn is_valid_position(&self, x: i32, y: i32) -> bool {
        x >= 0 && (x as usize) < self.width && y >= 0 && (y as usize) < self.height
    }

    fn pattern_conflicts_with_boundary(&self, pattern: &Matrix, new_x: i32, new_y: i32) -> bool {
        if new_x < 0 && pattern.index_axis(Axis(1), 0).iter().any(|&cell| cell != 0) {
            return true;
        }
        if new_x >= self.width as i32
            && pattern.index_axis(Axis(1), 2).iter().any(|&cell| cell != 0)
        {
            return true;
        }
        if new_y < 0 && pattern.index_axis(Axis(0), 0).iter().any(|&cell| cell != 0) {
            return true;
        }
        if new_y >= self.height as i32
            && pattern.index_axis(Axis(0), 2).iter().any(|&cell| cell != 0)
        {
            return true;
        }
        false
    }

    fn is_complete(&self, x: usize, y: usize) -> bool {
        y == self.height - 1 && x == self.width - 1
    }

    pub fn visualize_state(state: &Matrix) {
        for row in state.rows() {
            for &cell in row.iter() {
                match cell {
                    1 => print!("#"),
                    0 => print!("-"),
                    -1 => print!("?"),
                    _ => print!("x"),
                }
            }
            println!();
        }
    }
}

pub struct GameOfLife {
    pub state: Matrix,
    height: usize,
    width: usize,
}

impl GameOfLife {
    pub fn new(initial_state: Matrix) -> Self {
        let height = initial_state.nrows();
        let width = initial_state.ncols();
        Self {
            state: initial_state,
            height,
            width,
        }
    }

    pub fn step(&mut self) {
        let mut new_state = Array2::zeros((self.height, self.width));

        for y in 0..self.height {
            for x in 0..self.width {
                let neighbors = self.count_neighbors(x, y);
                let cell = self.state[[y, x]];

                if cell == 1 && (neighbors == 2 || neighbors == 3) {
                    new_state[[y, x]] = 1;
                } else if cell == 0 && neighbors == 3 {
                    new_state[[y, x]] = 1;
                } else {
                    new_state[[y, x]] = 0;
                }
            }
        }

        self.state = new_state;
    }

    pub fn count_neighbors(&self, x: usize, y: usize) -> i32 {
        let mut count = 0;

        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let new_x = x as i32 + dx;
                let new_y = y as i32 + dy;

                if new_x >= 0
                    && new_x < self.width as i32
                    && new_y >= 0
                    && new_y < self.height as i32
                    && self.state[[new_y as usize, new_x as usize]] == 1
                {
                    count += 1;
                }
            }
        }

        count
    }
}
