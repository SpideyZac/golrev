use rayon::prelude::*;

type Matrix = Vec<Vec<i32>>;

#[allow(dead_code)]
fn count_ones(matrix: &Matrix) -> usize {
    matrix
        .iter()
        .flat_map(|row| row.iter())
        .filter(|&&x| x == 1)
        .count()
}

struct GameConfig {
    alive_patterns: Vec<Matrix>,
    dead_patterns: Vec<Matrix>,
}

impl GameConfig {
    fn new() -> Self {
        let alive_patterns = Self::generate_patterns(1);
        let dead_patterns = Self::generate_patterns(0);
        Self {
            alive_patterns,
            dead_patterns,
        }
    }

    fn generate_patterns(end_state: i32) -> Vec<Matrix> {
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

            let neighbor_counts: Vec<i32> = (0..3).chain(4..9).collect();
            for neighbors in neighbor_counts {
                patterns.extend(Self::generate_single_pattern(0, neighbors));
            }
        }

        patterns.sort_by(|a, b| count_ones(a).cmp(&count_ones(b)));
        patterns
    }

    fn generate_single_pattern(cell_state: i32, num_neighbors: i32) -> Vec<Matrix> {
        let mut patterns = Vec::new();

        // Generate all possible 3x3 matrices (2^9 possibilities)
        for i in 0..(1 << 9) {
            let mut matrix = vec![vec![0; 3]; 3];
            for bit in 0..9 {
                let y = bit / 3;
                let x = bit % 3;
                matrix[y][x] = (i >> bit) & 1;
            }

            // Check if matrix matches our criteria
            let center_correct = matrix[1][1] == cell_state;
            let neighbor_sum = matrix.iter().flat_map(|row| row.iter()).sum::<i32>() - matrix[1][1];

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
        let height = target_state.len();
        let width = target_state[0].len();
        let config = GameConfig::new();

        Self {
            target_state,
            height,
            width,
            config,
            solve_flow_y,
        }
    }

    pub fn find_previous_state(&mut self) -> Option<Matrix> {
        let mut initial_state = vec![vec![-1; self.width]; self.height];
        let alive_patterns = self.config.alive_patterns.clone();
        let dead_patterns = self.config.dead_patterns.clone();
        let result = self.solve_recursively(
            0,
            0,
            &mut initial_state,
            &alive_patterns,
            &dead_patterns,
            true,
            0,
        );

        result
    }

    fn solve_recursively(
        &self,
        x: usize,
        y: usize,
        state: &mut Matrix,
        alive_patterns: &Vec<Vec<Vec<i32>>>,
        dead_patterns: &Vec<Vec<Vec<i32>>>,
        first: bool,
        thread: usize,
    ) -> Option<Matrix> {
        #[cfg(feature = "debug")]
        println!("Exploring position ({}, {}) on thread {}", x, y, thread);

        let patterns = if self.target_state[y][x] == 1 {
            alive_patterns
        } else {
            dead_patterns
        };

        if !first {
            for pattern in patterns {
                let mut current_state = state.clone();

                if self.try_pattern(x, y, &mut current_state, pattern, thread) {
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
                        if let Some(result) = self.solve_recursively(
                            next_x,
                            next_y,
                            &mut current_state,
                            alive_patterns,
                            dead_patterns,
                            false,
                            thread,
                        ) {
                            return Some(result);
                        }
                    }
                }
            }

            return None;
        } else {
            let results = patterns.par_iter().find_map_any(|pattern| {
                let mut current_state = state.clone();

                if self.try_pattern(x, y, &mut current_state, pattern, thread) {
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
                        if let Some(result) = self.solve_recursively(
                            next_x,
                            next_y,
                            &mut current_state,
                            alive_patterns,
                            dead_patterns,
                            false,
                            rand::random::<usize>(),
                        ) {
                            return Some(result);
                        }
                    }
                }

                None
            });

            return results;
        }
    }

    fn try_pattern(
        &self,
        x: usize,
        y: usize,
        state: &mut Matrix,
        pattern: &Matrix,
        thread: usize,
    ) -> bool {
        for dy in -1..=1 {
            for dx in -1..=1 {
                let new_x = x as i32 + dx;
                let new_y = y as i32 + dy;

                #[cfg(feature = "debug")]
                println!(
                    "Checking position ({}, {}) on thread {}",
                    new_x, new_y, thread
                );

                if !self.is_valid_position(new_x, new_y) {
                    if self.pattern_conflicts_with_boundary(pattern, new_x, new_y, thread) {
                        return false;
                    }
                    continue;
                }

                let pattern_value = pattern[(dy + 1) as usize][(dx + 1) as usize];
                let new_x = new_x as usize;
                let new_y = new_y as usize;

                if state[new_y][new_x] != -1 && state[new_y][new_x] != pattern_value {
                    #[cfg(feature = "debug")]
                    println!("Failed due to cell conflict on thread {}", thread);
                    return false;
                }

                state[new_y][new_x] = pattern_value;
            }
        }

        true
    }

    fn is_valid_position(&self, x: i32, y: i32) -> bool {
        x >= 0 && (x as usize) < self.width && y >= 0 && (y as usize) < self.height
    }

    #[allow(unused_variables)]
    fn pattern_conflicts_with_boundary(
        &self,
        pattern: &Matrix,
        new_x: i32,
        new_y: i32,
        thread: usize,
    ) -> bool {
        if new_x < 0 && pattern.iter().any(|row| row[0] != 0) {
            #[cfg(feature = "debug")]
            println!("Failed at left edge on thread {}", thread);
            return true;
        }
        if new_x >= self.width as i32 && pattern.iter().any(|row| row[2] != 0) {
            #[cfg(feature = "debug")]
            println!("Failed at right edge on thread {}", thread);
            return true;
        }
        if new_y < 0 && pattern[0].iter().any(|&cell| cell != 0) {
            #[cfg(feature = "debug")]
            println!("Failed at top edge on thread {}", thread);
            return true;
        }
        if new_y >= self.height as i32 && pattern[2].iter().any(|&cell| cell != 0) {
            #[cfg(feature = "debug")]
            println!("Failed at bottom edge on thread {}", thread);
            return true;
        }
        false
    }

    fn is_complete(&self, x: usize, y: usize) -> bool {
        y == self.height - 1 && x == self.width - 1
    }

    pub fn visualize_state(state: &Matrix) {
        for row in state {
            for &cell in row {
                match cell {
                    1 => print!("+"),
                    0 => print!("-"),
                    -1 => print!("?"),
                    _ => print!("x"),
                }
            }
            println!();
        }
    }
}

// Create Game of Life simulation
pub struct GameOfLife {
    pub state: Matrix,
    height: usize,
    width: usize,
}

impl GameOfLife {
    pub fn new(initial_state: Matrix) -> Self {
        let height = initial_state.len();
        let width = initial_state[0].len();
        Self {
            state: initial_state,
            height,
            width,
        }
    }

    pub fn step(&mut self) {
        let mut new_state = vec![vec![0; self.width]; self.height];

        for y in 0..self.height {
            for x in 0..self.width {
                let neighbors = self.count_neighbors(x, y);
                let cell = self.state[y][x];

                if cell == 1 && (neighbors == 2 || neighbors == 3) {
                    new_state[y][x] = 1;
                } else if cell == 0 && neighbors == 3 {
                    new_state[y][x] = 1;
                } else {
                    new_state[y][x] = 0;
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
                    && self.state[new_y as usize][new_x as usize] == 1
                {
                    count += 1;
                }
            }
        }

        count
    }
}
