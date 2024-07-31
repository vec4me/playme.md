use anyhow::Result as Anyhow;
use http_body_util::Full;
use hyper::{Response, header};
use std::collections::VecDeque;
use std::env;
use std::io::Write;
use std::net::SocketAddr;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

macro_rules! fix {
	($v: expr) => {
		$v << DIGITS
	}
}

macro_rules! defix {
	($v: expr) => {
		$v >> DIGITS
	}
}

macro_rules! float {
	($v: expr) => {
		$v as f64/fix!(1) as f64
	}
}

macro_rules! multiply {
	($a: expr, $b: expr) => {
		($a*$b) >> DIGITS
	}
}

macro_rules! radian {
	($v: expr) => {
		multiply!(TAU as i32, $v)/2
	}
}

macro_rules! divide {
	($a: expr, $b: expr) => {
		($a << DIGITS)/$b
	}
}

const R: usize = 0;
const G: usize = 1;
const B: usize = 2;

const BUFFER_SIZE: usize = (3*VIEW_SIZE_X*VIEW_SIZE_Y) as usize;

const CLOUD_HEIGHT: i32 = fix!(200);
const DIGITS: i32 = 7;
const HEADER_SIZE: usize = 15;
const TAU: usize = 256;
const TRIG_SCALE: i64 = 10_000;
const TRIG_TERMS: usize = 4;

const VIEW_SIZE_X: i32 = 320;
const VIEW_SIZE_Y: i32 = 200;

fn get_square_root(n: i32) -> i32 {
	let (mut f, mut p, mut r) = (0, 1 << 30, n);
	while p > r {
		p >>= 2;
	}
	while p != 0 {
		if r >= f + p {
			r -= f + p;
			f += p << 1;
		}
		f >>= 1;
		p >>= 2;
	}
	f
}

fn get_dot_product(a: &Vec3, b: &Vec3) -> i32 {
	// multiply!(a.x, b.x) + multiply!(a.y, b.y) + multiply!(a.z, b.z)
	a.x*b.x + a.y*b.y + a.z*b.z
}

fn get_length(v: &Vec3) -> i32 {
	get_square_root(get_dot_product(v, v))
}

// fn multiply(a: i32, b: i32) -> i32 {
	// (a*b) >> DIGITS
// }

// fn divide(a: i32, b: i32) -> i32 {DIGITS
	// (a << DIGITS)/b
// }

// fn fix(v: i32) -> i32 {
	// v << DIGITS
// }

fn unitize(v: &mut Vec3) {
	let n = get_length(v);
	v.x = divide!(v.x, n);
	v.y = divide!(v.y, n);
	v.z = divide!(v.z, n);
}

#[derive(Debug)]
enum InputAction {
	Render,
	Right,
	Left,
	Forward,
	Back
}

#[derive(Debug)]
struct InvalidInputAction;
impl std::fmt::Display for InvalidInputAction{
	fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{
		write!(f,"{self:?}")
	}
}

impl std::error::Error for InvalidInputAction{}
impl std::str::FromStr for InputAction {
	type Err = InvalidInputAction;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"/v" => Ok(InputAction::Render),
			"/l" => Ok(InputAction::Right),
			"/h" => Ok(InputAction::Left),
			"/k" => Ok(InputAction::Forward),
			"/j" => Ok(InputAction::Back),
			_ => Err(InvalidInputAction)
		}
	}
}

enum StateAction {
	Render(State),
	DoNothing
}

const fn get_factorial(n: i32) -> i32 {
	let mut result = 1;
	let mut i = 1;
	while i <= n {
		result *= i;
		i += 1;
	}
	result
}

// const fn get_taylor_cos_fast(t: i64) -> i32 {
// 	let mut sum = TRIG_SCALE;
// 	let tt = t*t/TRIG_SCALE;
// 	// Only the first term of the Taylor series for cos(t)
// 	sum -= tt/2; // Since get_factorial(2) is 2
// 	sum as i32
// }

const fn get_fixed_point_pow(x: i64, exp: u64) -> i64 {
	let mut result = TRIG_SCALE;
	let mut i = 0;
	while i < exp {
		result = result*x/TRIG_SCALE;
		i += 1;
	}
	result
}

const fn get_taylor_cos(x: i64) -> i32 {
	let mut sum = 0;
	let mut n = 0;
	while n < TRIG_TERMS {
		let sign = if n % 2 == 0 { 1 } else { -1 };
		let numerator = sign*get_fixed_point_pow(x, (2*n) as u64);
		let denominator = get_factorial((2*n) as i32) as i64;
		let term = numerator/denominator;
		sum += term;
		n += 1;
	}
	sum as i32
}

const fn get_trig_tables() -> ([i32; TAU], [i32; TAU]) {
	let mut cos = [0; TAU];
	let mut sin = [0; TAU];

	macro_rules! cos {
		($t: expr) => {
			cos[($t & 255) as usize]
		}
	}

	macro_rules! sin {
		($t: expr) => {
			sin[($t & 255) as usize]
		}
	}

	let mut t = 0;
	while 4*t < TAU {
		// Honestly I just guessed these constants...
		let x = get_taylor_cos(247*t as i64)/78;

		// https://www.desmos.com/calculator/zltsqq6kvl
		cos!(0*TAU/4 + t) = x;
		cos!(2*TAU/4 + t) = -x;
		cos!(2*TAU/4 - t) = -x;
		cos!(4*TAU/4 - t) = x;

		sin!(1*TAU/4 - t) = x;
		sin!(1*TAU/4 + t) = x;
		sin!(3*TAU/4 - t) = -x;
		sin!(3*TAU/4 + t) = -x;

		t += 1
	}

	(cos, sin)
}

const TRIG_TABLES: ([i32; TAU], [i32; TAU]) = get_trig_tables();

fn sun_direction(latitude: i32) -> Vec3 {
	let time = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map(|n| n.as_secs() + 9*3600)
		.unwrap_or(0);

	let time = divide!(time%86400, 3600) as i32;

	let (cos, sin) = &TRIG_TABLES;

	macro_rules! cos {
		($t: expr) => {
			cos[($t & 255) as usize]
		}
	}

	macro_rules! sin {
		($t: expr) => {
			sin[($t & 255) as usize]
		}
	}

	// gotta be in angle space
	let sx = cos!(radian!(time)/24);
	let sy = sin!(radian!(time)/24);
	let lx = cos!(radian!(latitude));
	let ly = sin!(radian!(latitude));

	let x = sx;
	let y = multiply!(sy, lx);
	let z = ly;

	println!("{}, {}, {}, {}", sx, sy, lx, ly);
	println!("{}, {}, {}", x, y, z);

	Vec3 {
		x: x,
		y: y,
		z: z
	}
}

fn get_gif_buffer(header: &[u8], state: State) -> Anyhow<Vec<u8>> {
	let (cos, sin) = &TRIG_TABLES;

	sun_direction(0);

	macro_rules! cos {
		($t: expr) => {
			cos[($t & 255) as usize]
		}
	}

	macro_rules! sin {
		($t: expr) => {
			sin[($t & 255) as usize]
		}
	}

	let mut image = header.to_owned();

	let camera_right_z = -sin!(state.camera_heading);
	let camera_right_x = cos!(state.camera_heading);

	for pixel_index in 0..(VIEW_SIZE_X*VIEW_SIZE_Y) as usize {
		macro_rules! color {
			($channel: expr) => {
				image[HEADER_SIZE + 3*pixel_index + $channel]
			}
		}

		let pixel_position = Vec2 {
			x: pixel_index as i32%VIEW_SIZE_X,
			y: pixel_index as i32/VIEW_SIZE_X
		};

		let view_offset = Vec2 {
			x: VIEW_SIZE_X - (pixel_position.x << 1),
			y: VIEW_SIZE_Y - (pixel_position.y << 1)
		};

		let mut pixel_direction = Vec3 {
			x: camera_right_x*view_offset.x/VIEW_SIZE_Y + camera_right_z,
			y: divide!(view_offset.y, VIEW_SIZE_Y),
			z: -camera_right_z*view_offset.x/VIEW_SIZE_Y + camera_right_x
		};
		unitize(&mut pixel_direction);

		let light_direction = Vec3 {
			x: 0,
			y: 0,
			z: fix!(1)
		};

		let pixel_distance =
		if pixel_direction.y > 0 {
			divide!(CLOUD_HEIGHT, pixel_direction.y)
		}
		else if pixel_direction.y < 0 {
			divide!(-state.camera_position.y, pixel_direction.y)
		}
		else {
			0
		};

		let hit = Vec3 {
			x: state.camera_position.x + multiply!(pixel_distance, pixel_direction.x),
			y: state.camera_position.y + multiply!(pixel_distance, pixel_direction.y),
			z: state.camera_position.z + multiply!(pixel_distance, pixel_direction.z)
		};

		if pixel_direction.y >= 0 {
			color!(R) = 188;
			color!(G) = 0;
			color!(B) = 45;

			let sky = cos!(hit.z >> 9)/2 + cos!(fix!(200 + 6*sin!(hit.z >> 11)) + hit.x >> 9) + 32;
			if sky < 0 {
				color!(R) = sky as u8;
				color!(G) = sky as u8;
				color!(B) = sky as u8;
			}
			else if get_dot_product(&pixel_direction, &light_direction) < 128*fix!(1) {
				color!(R) = (128 - multiply!(128, pixel_direction.y)) as u8;
				color!(G) = (179 - multiply!(179, pixel_direction.y)) as u8;
				color!(B) = (255 - multiply!(76, pixel_direction.y)) as u8;
			}
		}
		else {
			color!(R) = 77;
			color!(G) = 40;
			color!(B) = 0;

			if !(((hit.x >> 13)%7)*((hit.z >> 13)%9) != 0) {
				color!(R) = 100;
				// color!(G) = 100 + 4*(hit.x/20&31) as u8;
				color!(G) = 100;
				color!(B) = 110;
			}
			else {
				color!(R) = 60;
				color!(G) = (sin!(hit.x/20)/2 + 55) as u8;
				color!(B) = 0;

				// checking if it's negative(overflow)
				if color!(G) > 200 {
					color!(G) = 60;
					color!(B) = 120;
				}
			}
		}
	}

	let mut ffmpeg = Command::new("ffmpeg")
		.arg("-loglevel")
		.arg("0")
		.arg("-i")
		.arg("-")
		.arg("-f")
		.arg("gif")
		.arg("-vf")
		.arg("split[a][b];[a]palettegen[p];[b][p]paletteuse")
		.arg("-")
		.stdin(Stdio::piped())
		.stdout(Stdio::piped())
		.spawn()?;

	let stdin = ffmpeg.stdin.as_mut().ok_or(anyhow::anyhow!("Failed to open stdin"))?;
	stdin.write_all(&image)?;

	let output = ffmpeg.wait_with_output()?;
	let gif_buffer = output.stdout;

	Ok(gif_buffer)
}

fn get_state_action(mut state_lock: std::sync::MutexGuard<'_, State>, input_action: InputAction) -> StateAction {
	let (cos, sin) = &TRIG_TABLES;

	macro_rules! cos {
		($t: expr) => {
			cos[($t & 255) as usize]
		}
	}

	macro_rules! sin {
		($t: expr) => {
			sin[($t & 255) as usize]
		}
	}

	// for t in (0..=TAU).step_by(1) {
	// 	println!("{}", sin!(t as i64));
	// }

	match input_action {
		// if the input_action is to render, make a copy of the current state
		// so the lock can be dropped while the state is used for the render
		InputAction::Render => return StateAction::Render(state_lock.clone()),
		InputAction::Right => state_lock.camera_heading += TAU as i32/8,
		InputAction::Left => state_lock.camera_heading -= TAU as i32/8,
		InputAction::Forward => {
			state_lock.camera_position.z += fix!(1)*cos!(state_lock.camera_heading)/4;
			state_lock.camera_position.x -= fix!(1)*sin!(state_lock.camera_heading)/4;
		},
		InputAction::Back => {
			state_lock.camera_position.z -= fix!(1)*cos!(state_lock.camera_heading)/4;
			state_lock.camera_position.x += fix!(1)*sin!(state_lock.camera_heading)/4;
		}
	}
	// otherwise do nothing after the input_action
	StateAction::DoNothing
}

async fn handle_request(
	header: &[u8],
	request: hyper::Request<hyper::body::Incoming>,
	state: Arc<Mutex<State>>,
) -> Anyhow<Response<Full<VecDeque<u8>>>> {
	let input_action = request.uri().path().parse::<InputAction>()?;

	let state_lock = state.lock().map_err(|_|anyhow::anyhow!("Lock failed"))?;
	// state_lock is moved into state_action here
	let state_action = get_state_action(state_lock, input_action);
	// state_lock is dropped at the end of state_action, meaning the lock is freed

	let response = match state_action {
		StateAction::Render(state) => {
			// render a cloned state while being free to serve more requests
			let gif_buffer = get_gif_buffer(header, state)?;

			Response::builder()
				.header(header::CACHE_CONTROL, "max-age=0")
				.header(header::CONTENT_TYPE, "image/gif")
				.body(Full::new(VecDeque::from(gif_buffer)))?
		},
		StateAction::DoNothing => {
			// don't render
			Response::builder()
				.status(302)
				.header(header::CACHE_CONTROL, "max-age=0")
				.header(header::LOCATION, "https://github.com/blocksrey")
				.body(Full::default())?
		}
	};

	Ok(response)
}

#[derive(Clone)]
struct State {
	camera_position: Vec3,
	camera_heading: i32
}

#[derive(Copy, Clone)]
struct Vec2 {
	x: i32,
	y: i32
}

#[derive(Copy, Clone)]
struct Vec3 {
	x: i32,
	y: i32,
	z: i32
}

#[tokio::main]
async fn main() -> Anyhow<()> {
	let port = env::var("PORT").map_or(Ok(7890),
		|port| port.parse()
	)?;
	let address = SocketAddr::from(([0, 0, 0, 0], port));

	let mut header = format!("P6\n{} {}\n255\n", VIEW_SIZE_X, VIEW_SIZE_Y).into_bytes();
	assert_eq!(header.len(), HEADER_SIZE, "Update header size");
	header.resize(HEADER_SIZE + BUFFER_SIZE, 0);

	// use a memory leak to make a static reference to a byte slice
	// it only runs once so does it really count as a memory leak?
	// coerce &'static mut[u8]into &'static[u8]so the reference can be copied
	let static_header: &'static[u8] = Box::new(header).leak();

	let state = Arc::new(Mutex::new(State {
		camera_position: Vec3 {x: 0, y: fix!(30), z: 0},
		camera_heading: 0
	}));

	// https://github.com/hyperium/hyper/blob/master/examples/hello.rs
	let listener = tokio::net::TcpListener::bind(address).await?;

	println!("Listening on http://{}", address);

	loop {
		let (tcp, _) = listener.accept().await?;
		let io = hyper_util::rt::TokioIo::new(tcp);
		let state = state.clone();
		tokio::spawn(async move {
			if let Err(err) = hyper::server::conn::http1::Builder::new()
			.timer(hyper_util::rt::TokioTimer::new())
			.serve_connection(io, hyper::service::service_fn(move |body|
				// i couldn't tell you why it has to be cloned twice
				handle_request(static_header, body, state.clone())
			))
			.await {
				println!("Error serving connection: {:?}", err);
			}
		});
	}
}
