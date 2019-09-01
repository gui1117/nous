use nannou::prelude::*;
use core::time::Duration;
use nannou::color::Alpha;
use enum_iterator::IntoEnumIterator;

const NUMBER_OF_SQUARE: usize = 10;
const SQUARE_SIZE: f32 = 100.0;
const WINDOW_SIZE: f32 = SQUARE_SIZE * NUMBER_OF_SQUARE as f32;
const COLOR_DELTA: f32 = 0.0;
const COLOR_NUMBER: usize = 12;
const BACKGROUND_ALPHA: f32 = 1.0;
const BACKGROUND_S: f32 = 1.0;
const BACKGROUND_L: f32 = 0.0;
const COLORS_ALPHA: f32 = 0.4;
const TEMPO: f32 = 10.0/3.0;
const MODEL_DURATION: u64 = 3;

// TODO TODO:
// * faire un son en fonction des couleurs et des directions
// * fix dl issue: maybe we only want blocks to be aligned or half out of screen or maybe we want
// the 3 possibilies

fn main() {
	nannou::app(init)
		.update(update)
		.simple_window(view)
		.run();
}

#[derive(Clone, Copy, IntoEnumIterator)]
pub enum Dir {
	Up,
	Down,
	Left,
	Right,
	UpLeft,
	UpRight,
	DownLeft,
	DownRight,
}

impl Dir {
	fn unit_vec(&self) -> Vector2 {
		use Dir::*;

		match self {
			Up => Vector2::new(0.0, 1.0),
			Right => Vector2::new(1.0, 0.0),
			Down => -Up.unit_vec(),
			Left => -Right.unit_vec(),
			UpLeft => Up.unit_vec() + Left.unit_vec(),
			DownLeft => Down.unit_vec() + Left.unit_vec(),
			UpRight => Up.unit_vec() + Right.unit_vec(),
			DownRight => Down.unit_vec() + Right.unit_vec(),
		}
	}
}

struct Block {
	velocity: Vector2,
	position: Point2,
	color: Alpha<Hsl, f32>,
	color_position: f32,
}

impl Block {
	fn new(x: f32, y: f32, dir: Dir, block_per_beat: f32, color: usize) -> Block {
		Block {
			velocity: dir.unit_vec() * block_per_beat,
			position: Vector2::new(x + 0.5, y + 0.5),
			color: Alpha {
				color: Hsl::new(into_hue(color), 1.0, 0.0),
				alpha: COLORS_ALPHA,
			},
			color_position: 0.0,
		}
	}

	fn update(&mut self, delta_beat: f32) {
		self.position += self.velocity * delta_beat;
		self.position %= NUMBER_OF_SQUARE as f32;
		self.color_position += delta_beat;
		self.color_position %= 1.0;
	}

	fn display(&self, draw: &app::Draw) {
		let convert = |v| {
			((v / NUMBER_OF_SQUARE as f32) * 2.0 - Vector2::one()) * WINDOW_SIZE / 2.0
		};

		let mut color = self.color;
		color.lightness = if self.color_position < 0.5 {
			self.color_position * 2.0
		} else {
			(1.0 - self.color_position) * 2.0
		};

		for &dx in [-1.0, 0.0, 1.0].iter() {
			for &dy in [-1.0, 0.0, 1.0].iter() {
				draw.ellipse()
					.xy(convert(self.position + Vector2::new(dx, dy) * NUMBER_OF_SQUARE as f32))
					.width(SQUARE_SIZE*1.0)
					.height(SQUARE_SIZE*1.0)
					.color(color);
			}
		}
	}
}

fn grid(pos: f32, dpos: usize, dir: Dir, block_per_beat: f32, color: usize) -> Vec<Block> {
	let mut blocks = vec![];
	for i in 0..NUMBER_OF_SQUARE {
		for j in 0..NUMBER_OF_SQUARE {
			let block = Block::new(
				pos + (i * dpos) as f32,
				pos + (j * dpos) as f32,
				dir,
				block_per_beat,
				color,
			);
			blocks.push(block)
		}
	}
	blocks
}

fn into_hue(c: usize) -> f32 {
	c as f32 / COLOR_NUMBER as f32 * 360.0 + COLOR_DELTA
}

struct Model {
	blocks: Vec<Block>,
	tempo: f32,
	background_color: Alpha<Hsl, f32>,
	remaining_time: Duration,
}

impl Model {
	fn new() -> Self {
		use nannou::rand::seq::SliceRandom;
		let rng = &mut nannou::rand::thread_rng();
		let mut colors = (0..COLOR_NUMBER).collect::<Vec<_>>();
		let mut directions = Dir::into_enum_iter().collect::<Vec<_>>();
		colors.shuffle(rng);
		directions.shuffle(rng);

		let dl = [0.0, 0.5];
		let velocity = [1.0, 1.5, 2.0, 2.5, 3.0];
		let number_of_grid = [1, 1, 2, 2, 2, 3, 3, 3, 3, 4];

		let mut blocks = vec![];
		for _ in 0..*number_of_grid.choose(rng).unwrap() {
			let grid = grid(
				*dl.choose(rng).unwrap(),
				2,
				directions.pop().unwrap(),
				*velocity.choose(rng).unwrap(),
				colors.pop().unwrap(),
			);
			blocks.extend(grid);
		}

		Model {
			tempo: TEMPO,
			remaining_time: Duration::from_secs(MODEL_DURATION),
			blocks,
			background_color: Alpha {
				color: Hsl::new(into_hue(colors.pop().unwrap()), BACKGROUND_S, BACKGROUND_L),
				alpha: BACKGROUND_ALPHA,
			},
		}
	}
}

fn init(app: &App) -> Model {
	app.main_window().set_inner_size_points(WINDOW_SIZE, WINDOW_SIZE);

	Model::new()
}

fn update(_app: &App, model: &mut Model, update: Update) {
	let since_last_adjusted = if model.remaining_time < update.since_last {
		let since_last_adjusted = update.since_last - model.remaining_time;
		*model = Model::new();
		since_last_adjusted
	} else {
		update.since_last
	};

	model.remaining_time -= since_last_adjusted;
	let dt = duration_to_fractional(since_last_adjusted);

	for block in &mut model.blocks {
		block.update(dt * model.tempo)
	}
}

fn view(app: &App, model: &Model, frame: &Frame) {
	frame.clear(model.background_color);

	let draw = app.draw();

	for block in &model.blocks {
		block.display(&draw)
	}

	draw.to_frame(app, &frame).unwrap();
}

fn duration_to_fractional(dur: Duration) -> f32 {
	dur.as_secs() as f32 + dur.subsec_nanos() as f32 / 1_000_000_000.0
}
