mod config;
mod solver;

use rand::Rng;

use config::*;
use solver::{GameOfLife, GameOfLifeReverser};

fn main() {
    let to_convert = std::fs::read_to_string(FILE_TO_READ).unwrap();

    let mut target_state = vec![];
    for line in to_convert.lines() {
        let mut row = vec![];
        for c in line.chars() {
            match c {
                '#' => row.push(1),
                '-' => row.push(0),
                _ => (),
            }
        }
        target_state.push(row);
    }

    let mut row_len = None;
    for row in &target_state {
        if row_len.is_none() {
            row_len = Some(row.len());
        } else {
            assert!(row.len() == row_len.unwrap());
        }
    }

    let mut target_state = ndarray::Array2::from_shape_vec(
        (target_state.len(), target_state[0].len()),
        target_state.into_iter().flatten().collect(),
    )
    .unwrap();

    for pass in 0..NUMBER_OF_PASSES {
        println!("Running pass {}", pass + 1);
        println!("Target state:");
        GameOfLifeReverser::visualize_state(&target_state);
        println!();

        let mut reverser = GameOfLifeReverser::new(target_state.clone(), FLOW_Y);
        let mut solution = reverser.find_previous_state();
        println!();
        let mut current_target = target_state;

        if solution.is_none() {
            println!("No initial solution found");
            let mut rng = rand::thread_rng();

            while solution.is_none() {
                println!("Adding random noise");
                let rand_x = rng.gen_range(0..current_target.ncols());
                let rand_y = rng.gen_range(0..current_target.nrows());
                current_target[[rand_y, rand_x]] = if current_target[[rand_y, rand_x]] == 0 {
                    1
                } else {
                    0
                };

                reverser = GameOfLifeReverser::new(current_target.clone(), FLOW_Y);
                solution = reverser.find_previous_state();
                println!();
            }

            println!("New target state:");
            GameOfLifeReverser::visualize_state(&current_target);
            println!();
        }

        println!("Found solution:");
        if let Some(solution) = solution {
            GameOfLifeReverser::visualize_state(&solution);
            let mut game = GameOfLife::new(solution.clone());
            game.step();
            assert!(game.state == current_target);
            println!();
            println!("Next state:");
            GameOfLifeReverser::visualize_state(&game.state);
            target_state = solution;
        } else {
            break;
        }

        println!();
    }
}
