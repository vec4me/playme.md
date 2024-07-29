use http_body_util::Full;
use hyper::{Response, header};
use std::collections::VecDeque;
use std::convert::Infallible;
use std::env;
use std::io::Write;
use std::net::SocketAddr;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

macro_rules! fix {
	($v: expr) => {
		$v << SHIFT
	}
}

macro_rules! defix {
	($v: expr) => {
		$v >> SHIFT
	}
}

macro_rules! float {
	($v: expr) => {
		$v as f64/fix!(1) as f64
	}
}

macro_rules! multiply {
	($a: expr, $b: expr) => {
		($a*$b) >> SHIFT
	}
}

macro_rules! radian {
	($v: expr) => {
		multiply!(PI, $v)
	}
}

macro_rules! divide {
	($a: expr, $b: expr) => {
		($a << SHIFT)/$b
	}
}

const PI: i32 = 64;
const SHIFT: i32 = 7;
const CLOUD_HEIGHT: i32 = fix!(200);
const VIEW_SIZE_X: i32 = 320;
const VIEW_SIZE_Y: i32 = 200;
const R: usize = 0;
const G: usize = 1;
const B: usize = 2;
const HEADER_SIZE: usize = 15;
const BUFFER_SIZE: usize = (VIEW_SIZE_X*VIEW_SIZE_Y*3) as usize;

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
	// (a*b) >> SHIFT
// }

// fn divide(a: i32, b: i32) -> i32 {
	// (a << SHIFT)/b
// }

// fn fix(v: i32) -> i32 {
	// v << SHIFT
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

const TRIG: ([i32; 256], [i32; 256]) = {
	let sin: [i32; 256] = [
		0, 3, 6, 9, 12, 16, 19, 22, 25, 28, 31, 34, 37, 40, 43, 46, 49, 51, 54, 57, 60, 63, 65, 68, 71, 73, 76, 78, 81,
		83, 85, 88, 90, 92, 94, 96, 98, 100, 102, 104, 106, 107, 109, 111, 112, 113, 115, 116, 117, 118, 120, 121, 122,
		122, 123, 124, 125, 125, 126, 126, 126, 127, 127, 127, 127, 127, 127, 127, 126, 126, 126, 125, 125, 124, 123,
		122, 122, 121, 120, 118, 117, 116, 115, 113, 112, 111, 109, 107, 106, 104, 102, 100, 98, 96, 94, 92, 90, 88,
		85, 83, 81, 78, 76, 73, 71, 68, 65, 63, 60, 57, 54, 51, 49, 46, 43, 40, 37, 34, 31, 28, 25, 22, 19, 16, 12, 9,
		6, 3, 0, -3, -6, -9, -12, -16, -19, -22, -25, -28, -31, -34, -37, -40, -43, -46, -49, -51, -54, -57, -60, -63,
		-65, -68, -71, -73, -76, -78, -81, -83, -85, -88, -90, -92, -94, -96, -98, -100, -102, -104, -106, -107, -109,
		-111, -112, -113, -115, -116, -117, -118, -120, -121, -122, -122, -123, -124, -125, -125, -126, -126, -126,
		-127, -127, -127, -127, -127, -127, -127, -126, -126, -126, -125, -125, -124, -123, -122, -122, -121, -120,
		-118, -117, -116, -115, -113, -112, -111, -109, -107, -106, -104, -102, -100, -98, -96, -94, -92, -90, -88,
		-85, -83, -81, -78, -76, -73, -71, -68, -65, -63, -60, -57, -54, -51, -49, -46, -43, -40, -37, -34, -31, -28,
		-25, -22, -19, -16, -12, -9, -6, -3,
	];

	let mut cos = [0; 256];
	let mut i=0;
	//for loops are not allowed at compile time
	while i<256{
		cos[i] = sin[(i + 65) & 255];
		i+=1;
	}

	(cos, sin)
};

fn sun_direction(latitude: i32) -> Vec3 {
	let total_seconds = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map(|n| n.as_secs() + 9*3600)
		.unwrap_or(0);

	let hour = divide!(total_seconds%86400, 3600) as i32;

	let (cos, sin) = &TRIG;

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
	let sx = cos!(radian!(hour)/24);
	let sy = sin!(radian!(hour)/24);
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

fn get_gif_buffer(header: &[u8], state: State) -> Vec<u8> {
	let (cos, sin) = &TRIG;

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
			x: view_offset.x*camera_right_x/VIEW_SIZE_Y + camera_right_z,
			y: divide!(view_offset.y, VIEW_SIZE_Y),
			z: view_offset.x*-camera_right_z/VIEW_SIZE_Y + camera_right_x
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

			let sky = cos!(hit.z >> 9)/2 + cos!(fix!(200 + sin!(hit.z >> 11)*6) + hit.x >> 9) + 32;
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
		.spawn()
		.expect("Failed to spawn ffmpeg process");

	let stdin = ffmpeg.stdin.as_mut().expect("Failed to open stdin");
	stdin.write_all(&image).expect("Failed to write to stdin");

	let output = ffmpeg.wait_with_output().expect("Failed to read ffmpeg output");
	let gif_buffer = output.stdout;

	gif_buffer
}

fn get_state_action(mut state_lock: std::sync::MutexGuard<'_, State>, input_action: InputAction) -> StateAction {
	let (cos, sin) = &TRIG;

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

	match input_action {
		// if the input_action is to render, make a copy of the current state
		// so the lock can be dropped while the state is used for the render
		InputAction::Render => return StateAction::Render(state_lock.clone()),
		InputAction::Right => state_lock.camera_heading += PI/2,
		InputAction::Left => state_lock.camera_heading -= PI/2,
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
	req: hyper::Request<hyper::body::Incoming>,
	state: Arc<Mutex<State>>,
) -> Result<Response<Full<VecDeque<u8>>>, Infallible> {
	let input_action = req.uri().path().parse::<InputAction>(); // parse() uses fromstr

	// state_lock is moved into state_action here
	let state_action = get_state_action(state.lock().unwrap(), input_action.unwrap());
	// state_lock is dropped at the end of state_action, meaning the lock is freed

	let response = match state_action {
		StateAction::Render(state) => {
			// render a cloned state while being free to serve more requests
			let gif_buffer = get_gif_buffer(header, state);

			Response::builder()
				.header(header::CACHE_CONTROL, "max-age=0")
				.header(header::CONTENT_TYPE, "image/gif")
				.body(Full::new(VecDeque::from(gif_buffer)))
				.unwrap()
		},
		StateAction::DoNothing => {
			// don't render
			Response::builder()
				.status(302)
				.header(header::CACHE_CONTROL, "max-age=0")
				.header(header::LOCATION, "https://github.com/blocksrey")
				.body(Full::default())
				.unwrap()
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
async fn main() {
	let port = env::var("PORT").map_or(7890,
		|port_env| port_env.parse().expect("Invalid port number")
	);
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

	// https:// github.com/hyperium/hyper/blob/master/examples/hello.rs
	let listener = tokio::net::TcpListener::bind(address).await.unwrap();

	println!("Listening on http://{}", address);

	loop {
		let (tcp, _) = listener.accept().await.unwrap();
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
