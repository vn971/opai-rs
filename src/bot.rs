use std::sync::Arc;
use rand::{XorShiftRng, SeedableRng};
use player::Player;
use config;
use config::Solver;
use zobrist::Zobrist;
use field;
use field::Field;
use uct::UctRoot;
use heuristic;
use minimax;

const BOT_STR: &'static str = "bot";

const MIN_COMPLEXITY: u32 = 0;

const MAX_COMPLEXITY: u32 = 100;

const MIN_UCT_ITERATIONS: usize = 0;

const MAX_UCT_ITERATIONS: usize = 500000;

const MIN_MINIMAX_DEPTH: u32 = 0;

const MAX_MINIMAX_DEPTH: u32 = 8;

pub struct Bot {
  rng: XorShiftRng,
  zobrist: Arc<Zobrist>,
  field: Field,
  uct: UctRoot
}

impl Bot {
  pub fn new(width: u32, height: u32, seed: u64) -> Bot {
    info!(target: BOT_STR, "Initialization with width {0}, height {1}, seed {2}.", width, height, seed);
    let length = field::length(width, height);
    let seed_array = [3, seed as u32, 7, (seed >> 32) as u32];
    let mut rng = XorShiftRng::from_seed(seed_array);
    let zobrist = Arc::new(Zobrist::new(length * 2, &mut rng));
    let field_zobrist = zobrist.clone();
    Bot {
      rng: rng,
      zobrist: zobrist,
      field: Field::new(width, height, field_zobrist),
      uct: UctRoot::new(length)
    }
  }

  pub fn best_move(&mut self, player: Player) -> Option<(u32, u32)> {
    self.best_move_with_complexity(player, (MAX_COMPLEXITY - MIN_COMPLEXITY) / 2 + MIN_COMPLEXITY)
  }

  pub fn best_move_with_time(&mut self, player: Player, time: u32) -> Option<(u32, u32)> {
    match config::solver() {
      Solver::Uct => {
        self.uct.best_move_with_time(&self.field, player, &mut self.rng, time - config::time_gap())
          .or_else(|| { heuristic::heuristic(&self.field, player) })
          .map(|pos| (self.field.to_x(pos), self.field.to_y(pos)))
      },
      Solver::Minimax => {
        minimax::minimax_with_time(&mut self.field, player, &mut self.rng, time)
          .or_else(|| { heuristic::heuristic(&self.field, player) })
          .map(|pos| (self.field.to_x(pos), self.field.to_y(pos)))
      },
      Solver::Heuristic => {
        heuristic::heuristic(&self.field, player).map(|pos| (self.field.to_x(pos), self.field.to_y(pos)))
      }
    }
  }

  pub fn best_move_with_complexity(&mut self, player: Player, complexity: u32) -> Option<(u32, u32)> {
    match config::solver() {
      Solver::Uct => {
        let iterations_count = (complexity - MIN_COMPLEXITY) as usize * (MAX_UCT_ITERATIONS - MIN_UCT_ITERATIONS) / (MAX_COMPLEXITY - MIN_COMPLEXITY) as usize + MIN_UCT_ITERATIONS;
        self.uct.best_move_with_iterations_count(&self.field, player, &mut self.rng, iterations_count)
          .or_else(|| { heuristic::heuristic(&self.field, player) })
          .map(|pos| (self.field.to_x(pos), self.field.to_y(pos)))
      },
      Solver::Minimax => {
        let depth = (complexity - MIN_COMPLEXITY) * (MAX_MINIMAX_DEPTH - MIN_MINIMAX_DEPTH) / (MAX_COMPLEXITY - MIN_COMPLEXITY) + MIN_MINIMAX_DEPTH;
        minimax::minimax(&mut self.field, player, &mut self.rng, depth)
          .or_else(|| { heuristic::heuristic(&self.field, player) })
          .map(|pos| (self.field.to_x(pos), self.field.to_y(pos)))
      },
      Solver::Heuristic => {
        heuristic::heuristic(&self.field, player).map(|pos| (self.field.to_x(pos), self.field.to_y(pos)))
      }
    }
  }

  pub fn put_point(&mut self, x: u32, y: u32, player: Player) -> bool {
    let pos = self.field.to_pos(x, y);
    self.field.put_point(pos, player)
  }

  pub fn undo(&mut self) -> bool {
    self.field.undo()
  }
}
