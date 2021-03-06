use player::Player;
use field::{Pos, Field};

static CG_SUM: [i32; 9] = [-5, -1, 0, 0, 1, 2, 5, 20, 30];

fn heuristic_estimation(field: &Field, pos: Pos, player: Player) -> i32 {
  let enemy = player.next();
  let g1 = field.number_near_groups(pos, player) as i32;
  let g2 = field.number_near_groups(pos, enemy) as i32;
  let c1 = CG_SUM[field.number_near_points(pos, player) as usize];
  let c2 = CG_SUM[field.number_near_points(pos, enemy) as usize];
  let mut result = (g1 * 3 + g2 * 2) * (5 - (g1 - g2).abs()) - c1 - c2;
  if let Some(&last_pos) = field.points_seq().last() {
    if field.is_near(last_pos, pos) {
      result += 5;
    }
  }
  result
}

pub fn heuristic(field: &Field, player: Player) -> Option<Pos> {
  let mut best_estimation = i32::min_value();
  let mut result = None;
  for pos in field.min_pos() .. field.max_pos() + 1 {
    if field.is_putting_allowed(pos) {
      let cur_estimation = heuristic_estimation(field, pos, player);
      if cur_estimation > best_estimation { //TODO: check for stupid move.
        best_estimation = cur_estimation;
        result = Some(pos);
      }
    }
  }
  result
}
