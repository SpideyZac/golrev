use indicatif::ProgressBar;
use rand::Rng;

type Matrix = Vec<Vec<i32>>;

#[derive(Debug)]
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

        patterns.sort_by(|a, b| {
            let count_ones = |matrix: &Matrix| {
                matrix
                    .iter()
                    .flat_map(|row| row.iter())
                    .filter(|&&x| x == 1)
                    .count()
            };
            count_ones(a).cmp(&count_ones(b))
        });
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

struct GameOfLifeReverser {
    target_state: Matrix,
    height: usize,
    width: usize,
    debug: bool,
    config: GameConfig,
    progress_bar: Option<ProgressBar>,
    solve_flow_y: bool,
}

impl GameOfLifeReverser {
    fn new(target_state: Matrix, solve_flow_y: bool, debug: bool, show_progress: bool) -> Self {
        let height = target_state.len();
        let width = target_state[0].len();
        let config = GameConfig::new();
        let progress_bar = if show_progress {
            Some(ProgressBar::new((height * width) as u64))
        } else {
            None
        };

        Self {
            target_state,
            height,
            width,
            debug,
            config,
            progress_bar,
            solve_flow_y,
        }
    }

    fn find_previous_state(&mut self) -> Option<Matrix> {
        let mut initial_state = vec![vec![-1; self.width]; self.height];
        let alive_patterns = self.config.alive_patterns.clone();
        let dead_patterns = self.config.dead_patterns.clone();
        let result =
            self.solve_recursively(0, 0, &mut initial_state, &alive_patterns, &dead_patterns);

        if let Some(progress_bar) = &self.progress_bar {
            progress_bar.finish();
        }

        result
    }

    fn solve_recursively(
        &mut self,
        x: usize,
        y: usize,
        state: &mut Matrix,
        alive_patterns: &Vec<Vec<Vec<i32>>>,
        dead_patterns: &Vec<Vec<Vec<i32>>>,
    ) -> Option<Matrix> {
        if self.debug {
            println!("Exploring position ({}, {})", x, y);
        }

        let patterns = if self.target_state[y][x] == 1 {
            alive_patterns
        } else {
            dead_patterns
        };

        for pattern in patterns {
            let mut current_state = state.clone();

            if self.try_pattern(x, y, &mut current_state, pattern) {
                if self.is_complete(x, y) {
                    return Some(current_state);
                }

                if let Some(progress_bar) = &self.progress_bar {
                    progress_bar.inc(1);
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
                    ) {
                        return Some(result);
                    }
                    if let Some(progress_bar) = &self.progress_bar {
                        let cur_pos = progress_bar.position();
                        progress_bar.set_position(cur_pos - 1);
                    }
                }
            }
        }

        None
    }

    fn try_pattern(&self, x: usize, y: usize, state: &mut Matrix, pattern: &Matrix) -> bool {
        for dy in -1..=1 {
            for dx in -1..=1 {
                let new_x = x as i32 + dx;
                let new_y = y as i32 + dy;

                if self.debug {
                    println!("Checking position ({}, {})", new_x, new_y);
                }

                if !self.is_valid_position(new_x, new_y) {
                    if self.pattern_conflicts_with_boundary(pattern, new_x, new_y) {
                        return false;
                    }
                    continue;
                }

                let pattern_value = pattern[(dy + 1) as usize][(dx + 1) as usize];
                let new_x = new_x as usize;
                let new_y = new_y as usize;

                if state[new_y][new_x] != -1 && state[new_y][new_x] != pattern_value {
                    if self.debug {
                        println!("Failed due to cell conflict");
                    }
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

    fn pattern_conflicts_with_boundary(&self, pattern: &Matrix, new_x: i32, new_y: i32) -> bool {
        if new_x < 0 && pattern.iter().any(|row| row[0] != 0) {
            if self.debug {
                println!("Failed at left edge");
            }
            return true;
        }
        if new_x >= self.width as i32 && pattern.iter().any(|row| row[2] != 0) {
            if self.debug {
                println!("Failed at right edge");
            }
            return true;
        }
        if new_y < 0 && pattern[0].iter().any(|&cell| cell != 0) {
            if self.debug {
                println!("Failed at top edge");
            }
            return true;
        }
        if new_y >= self.height as i32 && pattern[2].iter().any(|&cell| cell != 0) {
            if self.debug {
                println!("Failed at bottom edge");
            }
            return true;
        }
        false
    }

    fn is_complete(&self, x: usize, y: usize) -> bool {
        y == self.height - 1 && x == self.width - 1
    }

    fn visualize_state(state: &Matrix) {
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

fn main() {
    let show_progress = true;
    let do_logging = false;
    let flow_y = true;

    let target_state = vec![
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 1, 1, 0, 0, 0, 0, 0, 0],
        vec![0, 1, 0, 0, 1, 0, 0, 0, 0, 0],
        vec![0, 1, 1, 1, 1, 0, 0, 0, 0, 0],
        vec![0, 1, 0, 0, 1, 0, 0, 0, 0, 0],
        vec![0, 0, 1, 1, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];

    println!("Target state:");
    GameOfLifeReverser::visualize_state(&target_state);
    println!();

    let mut reverser =
        GameOfLifeReverser::new(target_state.clone(), flow_y, do_logging, show_progress);
    let mut solution = reverser.find_previous_state();
    let mut current_target = target_state;

    if solution.is_none() {
        println!("No initial solution found");
        let mut rng = rand::thread_rng();

        while solution.is_none() {
            println!("Adding random noise");
            let rand_x = rng.gen_range(0..current_target[0].len());
            let rand_y = rng.gen_range(0..current_target.len());
            current_target[rand_y][rand_x] = if current_target[rand_y][rand_x] == 0 {
                1
            } else {
                0
            };

            reverser =
                GameOfLifeReverser::new(current_target.clone(), flow_y, do_logging, show_progress);
            solution = reverser.find_previous_state();
        }

        println!("New target state:");
        GameOfLifeReverser::visualize_state(&current_target);
        println!();
    }

    println!("Found solution:");
    if let Some(solution) = solution {
        GameOfLifeReverser::visualize_state(&solution);
    }
}
