use std::fmt::{Display, Formatter};
use std::fmt::Result as FmtResult;
use std::io::{Write, Read};
use num_cpus;
use rustc_serialize::{Encodable, Encoder, Decodable, Decoder};
use toml;
use types::{CoordSum, Depth, Time};

const CONFIG_STR: &'static str = "config";

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum UcbType {
  Winrate,
  Ucb1,
  Ucb1Tuned
}

const WINRATE_STR: &'static str = "Winrate";

const UCB1_STR: &'static str = "Ucb1";

const UCB1_TUNED_STR: &'static str = "Ucb1Tuned";

impl UcbType {
  pub fn as_str(&self) -> &'static str {
    match self {
      &UcbType::Winrate => WINRATE_STR,
      &UcbType::Ucb1 => UCB1_STR,
      &UcbType::Ucb1Tuned => UCB1_TUNED_STR
    }
  }

  pub fn from_str(s: &str) -> Option<UcbType> {
    match s {
      WINRATE_STR => Some(UcbType::Winrate),
      UCB1_STR => Some(UcbType::Ucb1),
      UCB1_TUNED_STR => Some(UcbType::Ucb1Tuned),
      _ => None
    }
  }
}

impl Display for UcbType {
  fn fmt(&self, f: &mut Formatter) -> FmtResult {
    write!(f, "{}", self.as_str())
  }
}

impl Encodable for UcbType {
  fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
    s.emit_str(self.as_str())
  }
}

impl Decodable for UcbType {
  fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
    d.read_str().and_then(|s| UcbType::from_str(s.as_str()).ok_or(d.error("Invalid string!")))
  }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum UctKomiType {
  None,
  Static,
  Dynamic
}

const NONE_STR: &'static str = "None";

const STATIC_STR: &'static str = "Static";

const DYNAMIC_STR: &'static str = "Dynamic";

impl UctKomiType {
  pub fn as_str(&self) -> &'static str {
    match self {
      &UctKomiType::None => NONE_STR,
      &UctKomiType::Static => STATIC_STR,
      &UctKomiType::Dynamic => DYNAMIC_STR
    }
  }

  pub fn from_str(s: &str) -> Option<UctKomiType> {
    match s {
      NONE_STR => Some(UctKomiType::None),
      STATIC_STR => Some(UctKomiType::Static),
      DYNAMIC_STR => Some(UctKomiType::Dynamic),
      _ => None
    }
  }
}

impl Display for UctKomiType {
  fn fmt(&self, f: &mut Formatter) -> FmtResult {
    write!(f, "{}", self.as_str())
  }
}

impl Encodable for UctKomiType {
  fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
    s.emit_str(self.as_str())
  }
}

impl Decodable for UctKomiType {
  fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
    d.read_str().and_then(|s| UctKomiType::from_str(s.as_str()).ok_or(d.error("Invalid string!")))
  }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Solver {
  Uct,
  Heuristic
}

const UCT_STR: &'static str = "Uct";

const HEURISTIC_STR: &'static str = "Heuristic";

impl Solver {
  pub fn as_str(&self) -> &'static str {
    match self {
      &Solver::Uct => UCT_STR,
      &Solver::Heuristic => HEURISTIC_STR
    }
  }

  pub fn from_str(s: &str) -> Option<Solver> {
    match s {
      UCT_STR => Some(Solver::Uct),
      HEURISTIC_STR => Some(Solver::Heuristic),
      _ => None
    }
  }
}

impl Display for Solver {
  fn fmt(&self, f: &mut Formatter) -> FmtResult {
    write!(f, "{}", self.as_str())
  }
}

impl Encodable for Solver {
  fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
    s.emit_str(self.as_str())
  }
}

impl Decodable for Solver {
  fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
    d.read_str().and_then(|s| Solver::from_str(s.as_str()).ok_or(d.error("Invalid string!")))
  }
}

#[derive(Clone, RustcDecodable, RustcEncodable)]
struct Config {
  uct: UctConfig,
  bot: BotConfig
}

#[derive(Clone, RustcDecodable, RustcEncodable)]
struct UctConfig {
  radius: CoordSum,
  ucb_type: UcbType,
  final_ucb_type: UcbType,
  draw_weight: f32,
  uctk: f32,
  when_create_children: usize,
  depth: Depth,
  komi_type: UctKomiType,
  red: f32,
  green: f32,
  komi_min_iterations: usize
}

#[derive(Clone, RustcDecodable, RustcEncodable)]
struct BotConfig {
  threads_count: Option<usize>,
  time_gap: Time
}

const DEFAULT_UCT_CONFIG: UctConfig = UctConfig {
  radius: 3,
  ucb_type: UcbType::Ucb1Tuned,
  final_ucb_type: UcbType::Winrate,
  draw_weight: 0.4,
  uctk: 1.0,
  when_create_children: 2,
  depth: 8,
  komi_type: UctKomiType::Dynamic,
  red: 0.45,
  green: 0.5,
  komi_min_iterations: 3000
};

const DEFAULT_BOT_CONFIG: BotConfig = BotConfig {
  threads_count: None,
  time_gap: 100
};

const DEFAULT_CONFIG: Config = Config {
  uct: DEFAULT_UCT_CONFIG,
  bot: DEFAULT_BOT_CONFIG
};

static mut NUM_CPUS: usize = 4;

static mut CONFIG: Config = DEFAULT_CONFIG;

#[inline]
fn config() -> &'static Config {
  unsafe { &CONFIG }
}

static RAVE: bool = true;

static FINAL_RAVE: bool = false;

static RAVE_BIAS: f32 = 0.01;

pub fn init() {
  let num_cpus = num_cpus::get();
  unsafe {
    NUM_CPUS = num_cpus;
  }
  info!(target: CONFIG_STR, "Default threads count is {}.", num_cpus);
}

pub fn read<T: Read>(input: &mut T) {
  let mut string = String::new();
  input.read_to_string(&mut string).ok();
  if let Some(config) = toml::decode_str::<Config>(string.as_str()) {
    unsafe {
      CONFIG = config
    }
    debug!(target: CONFIG_STR, "Config has been loaded.");
  } else {
    error!(target: CONFIG_STR, "Bad config file!");
  }
}

pub fn write<T: Write>(output: &mut T) {
  write!(output, "{}", toml::encode(config())).ok();
  info!(target: CONFIG_STR, "Config has been written.");
}

#[inline]
pub fn uct_radius() -> CoordSum {
  config().uct.radius
}

#[inline]
pub fn ucb_type() -> UcbType {
  config().uct.ucb_type
}

#[inline]
pub fn final_ucb_type() -> UcbType {
  config().uct.final_ucb_type
}

#[inline]
pub fn uct_draw_weight() -> f32 {
  config().uct.draw_weight
}

#[inline]
pub fn uctk() -> f32 {
  config().uct.uctk
}

#[inline]
pub fn uct_when_create_children() -> usize {
  config().uct.when_create_children
}

#[inline]
pub fn uct_depth() -> Depth {
  config().uct.depth
}

#[inline]
pub fn threads_count() -> usize {
  config().bot.threads_count.unwrap_or(unsafe { NUM_CPUS })
}

#[inline]
pub fn uct_komi_type() -> UctKomiType {
  config().uct.komi_type
}

#[inline]
pub fn uct_red() -> f32 {
  config().uct.red
}

#[inline]
pub fn uct_green() -> f32 {
  config().uct.green
}

#[inline]
pub fn uct_komi_min_iterations() -> usize {
  config().uct.komi_min_iterations
}

#[inline]
pub fn time_gap() -> Time {
  config().bot.time_gap
}

#[inline]
pub fn rave() -> bool {
  RAVE
}

#[inline]
pub fn final_rave() -> bool {
  FINAL_RAVE
}

#[inline]
pub fn rave_bias() -> f32 {
  RAVE_BIAS
}
