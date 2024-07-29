use hyper::{Response, header};
use http_body_util::Full;
use std::collections::VecDeque;
use std::convert::Infallible;
use std::env;
use std::net::SocketAddr;
use std::process::{Command, Stdio};
use std::io::Write;
use std::sync::{Arc, Mutex};

const VIEW_SIZE_X: usize = 320;
const VIEW_SIZE_Y: usize = 200;
const HEADER_SIZE: usize = 15;
const BUFFER_SIZE: usize = VIEW_SIZE_X * VIEW_SIZE_Y * 3;
const R: usize = 0;
const G: usize = 1;
const B: usize = 2;

#[derive(Copy, Clone)]
struct Vec2 {
	x: i32,
	y: i32,
}

#[derive(Copy, Clone)]
struct Vec3 {
	x: i32,
	y: i32,
	z: i32,
}

#[derive(Clone)]
struct State {
	camera_position: Vec3,
	camera_heading: i32,
	light_direction: Vec3,
}

const fn sqrt(n: u32) -> i32 {
	let (mut f, mut p, mut r) = (0u32, 1u32 << 30, n);
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
	f as i32
}

const fn dot3(a: &Vec3, b: &Vec3) -> i32 {
	a.x * b.x + a.y * b.y + a.z * b.z
}

const fn norm3(v: &Vec3) -> i32 {
	sqrt(dot3(v, v) as u32)
}

macro_rules! multiply {
	($value:expr, $factor:expr) => {
		($value * $factor) >> 8
	};
}

macro_rules! divide {
	($value:expr, $n:expr) => {
		($value << 8) / $n
	};
}

fn unit3(v: &mut Vec3) {
	let n = norm3(v);
	v.x = divide!(v.x, n);
	v.y = divide!(v.y, n);
	v.z = divide!(v.z, n);
}

const SIN_COS:([i32;256],[i32;256])=(
	[127, 127, 127, 126, 126, 126, 125, 125, 124, 123, 122, 122, 121, 120, 118, 117, 116, 115, 113, 112, 111, 109, 107, 106, 104, 102, 100, 98, 96, 94, 92, 90, 88, 85, 83, 81, 78, 76, 73, 71, 68, 65, 63, 60, 57, 54, 51, 49, 46, 43, 40, 37, 34, 31, 28, 25, 22, 19, 16, 12, 9, 6, 3, 0, -3, -6, -9, -12, -16, -19, -22, -25, -28, -31, -34, -37, -40, -43, -46, -49, -51, -54, -57, -60, -63, -65, -68, -71, -73, -76, -78, -81, -83, -85, -88, -90, -92, -94, -96, -98, -100, -102, -104, -106, -107, -109, -111, -112, -113, -115, -116, -117, -118, -120, -121, -122, -122, -123, -124, -125, -125, -126, -126, -126, -127, -127, -127, -127, -127, -127, -127, -126, -126, -126, -125, -125, -124, -123, -122, -122, -121, -120, -118, -117, -116, -115, -113, -112, -111, -109, -107, -106, -104, -102, -100, -98, -96, -94, -92, -90, -88, -85, -83, -81, -78, -76, -73, -71, -68, -65, -63, -60, -57, -54, -51, -49, -46, -43, -40, -37, -34, -31, -28, -25, -22, -19, -16, -12, -9, -6, -3, 0, 3, 6, 9, 12, 16, 19, 22, 25, 28, 31, 34, 37, 40, 43, 46, 49, 51, 54, 57, 60, 63, 65, 68, 71, 73, 76, 78, 81, 83, 85, 88, 90, 92, 94, 96, 98, 100, 102, 104, 106, 107, 109, 111, 112, 113, 115, 116, 117, 118, 120, 121, 122, 122, 123, 124, 125, 125, 126, 126, 126, 127, 127, 127, 127],
	[0, 3, 6, 9, 12, 16, 19, 22, 25, 28, 31, 34, 37, 40, 43, 46, 49, 51, 54, 57, 60, 63, 65, 68, 71, 73, 76, 78, 81, 83, 85, 88, 90, 92, 94, 96, 98, 100, 102, 104, 106, 107, 109, 111, 112, 113, 115, 116, 117, 118, 120, 121, 122, 122, 123, 124, 125, 125, 126, 126, 126, 127, 127, 127, 127, 127, 127, 127, 126, 126, 126, 125, 125, 124, 123, 122, 122, 121, 120, 118, 117, 116, 115, 113, 112, 111, 109, 107, 106, 104, 102, 100, 98, 96, 94, 92, 90, 88, 85, 83, 81, 78, 76, 73, 71, 68, 65, 63, 60, 57, 54, 51, 49, 46, 43, 40, 37, 34, 31, 28, 25, 22, 19, 16, 12, 9, 6, 3, 0, -3, -6, -9, -12, -16, -19, -22, -25, -28, -31, -34, -37, -40, -43, -46, -49, -51, -54, -57, -60, -63, -65, -68, -71, -73, -76, -78, -81, -83, -85, -88, -90, -92, -94, -96, -98, -100, -102, -104, -106, -107, -109, -111, -112, -113, -115, -116, -117, -118, -120, -121, -122, -122, -123, -124, -125, -125, -126, -126, -126, -127, -127, -127, -127, -127, -127, -127, -126, -126, -126, -125, -125, -124, -123, -122, -122, -121, -120, -118, -117, -116, -115, -113, -112, -111, -109, -107, -106, -104, -102, -100, -98, -96, -94, -92, -90, -88, -85, -83, -81, -78, -76, -73, -71, -68, -65, -63, -60, -57, -54, -51, -49, -46, -43, -40, -37, -34, -31, -28, -25, -22, -19, -16, -12, -9, -6, -3]
);

/// note that state is immutable here
fn render_to_gif(header:&'static [u8],state:State)->Vec<u8>{
	let (cos, sin) = &SIN_COS;

	let camera_heading_cos = cos[state.camera_heading as usize & 255];
	let camera_heading_sin = sin[state.camera_heading as usize & 255];

	let mut image_buffer = header.to_owned();

	for pixel_index in 0..VIEW_SIZE_X * VIEW_SIZE_Y {
		macro_rules! color {
			($channel:expr) => {
				image_buffer[HEADER_SIZE + pixel_index * 3 + $channel]
			};
		}

		let pixel_position = Vec2 {
			x: (pixel_index % VIEW_SIZE_X) as i32,
			y: (pixel_index / VIEW_SIZE_X) as i32,
		};

		let view_offset = Vec2 {
			x: VIEW_SIZE_X as i32 - (pixel_position.x << 1) as i32,
			y: VIEW_SIZE_Y as i32 - (pixel_position.y << 1) as i32,
		};

		let mut pixel_direction = Vec3 {
			x: view_offset.x * camera_heading_cos / VIEW_SIZE_Y as i32 - camera_heading_sin,
			y: (view_offset.y << 7) / VIEW_SIZE_Y as i32,
			z: view_offset.x * camera_heading_sin / VIEW_SIZE_Y as i32 + camera_heading_cos,
		};
		unit3(&mut pixel_direction);

		let pixel_distance = if pixel_direction.y != 0 {
			divide!(150, pixel_direction.y)
		} else {
			0
		};

		let hit = Vec3 {
			x: state.camera_position.x + pixel_distance * pixel_direction.x,
			y: state.camera_position.y + pixel_distance * pixel_direction.y,
			z: state.camera_position.z + pixel_distance * pixel_direction.z,
		};

		if pixel_direction.y > 0 {
			color!(R) = 188 as u8;
			color!(G) = 0 as u8;
			color!(B) = 45 as u8;

			let sky = cos[((cos[((hit.z >> 11) & 255) as usize] + (hit.x >> 8)) >> 1 & 255) as usize]
				+ cos[(hit.z / 500 & 255) as usize] / 4
				+ 30;
			if sky < 0 {
				color!(R) = sky as u8;
				color!(G) = sky as u8;
				color!(B) = sky as u8;
			} else if dot3(&pixel_direction, &state.light_direction) < 64000 {
				color!(R) = (128 - 128 * pixel_direction.y / 255) as u8;
				color!(G) = (179 - 179 * pixel_direction.y / 255) as u8;
				color!(B) = (255 - 76 * pixel_direction.y / 255) as u8;
			}
		} else if pixel_direction.y < 0 {
			color!(R) = 77 as u8;
			color!(G) = 40 as u8;
			color!(B) = 0 as u8;

			if !(((hit.x >> 13) % 7) * ((hit.z >> 13) % 9) != 0) {
				color!(R) = 100 as u8;
				color!(G) = 100 as u8;
				color!(B) = 110 as u8;
			} else {
				color!(R) = 60 as u8;
				color!(G) = (sin[(hit.x / 20 & 255) as usize] / 2 + 55) as u8;
				color!(B) = 0 as u8;

				// Checking if it's negative (overflow)
				if color!(G) > 200 {
					color!(G) = 60 as u8;
					color!(B) = 120 as u8;
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
    stdin.write_all(&image_buffer).expect("Failed to write to stdin");

	let output = ffmpeg.wait_with_output().expect("Failed to read ffmpeg output");
	let gif_buffer = output.stdout;

	gif_buffer
}

#[derive(Debug)]
enum Action{
	Render,
	Right,
	Left,
	Forward,
	Back,
}
#[derive(Debug)]
struct InvalidAction;
impl std::str::FromStr for Action{
	type Err=InvalidAction;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s{
			"v"=>Ok(Action::Render),
			"l"=>Ok(Action::Right),
			"h"=>Ok(Action::Left),
			"k"=>Ok(Action::Forward),
			"j"=>Ok(Action::Back),
			_=>Err(InvalidAction),
		}
	}
}

enum AfterAction{
	Render(State),
	DoNothing,
}

/// returns AfterAction
fn do_action(mut state_lock: std::sync::MutexGuard<'_,State>,action:Action)->AfterAction{
	match action {
		//if the action is to render, make a copy of the current state
		//so the lock can be dropped while the state is used for the render
		Action::Render => return AfterAction::Render(state_lock.clone()),
		Action::Right => state_lock.camera_heading += 32,
		Action::Left => state_lock.camera_heading -= 32,
		Action::Forward => {
			let (cos, sin) = &SIN_COS;
			state_lock.camera_position.x -= 4000 * sin[state_lock.camera_heading as usize & 255];
			state_lock.camera_position.z += 4000 * cos[state_lock.camera_heading as usize & 255];
		},
		Action::Back => {
			let (cos, sin) = &SIN_COS;
			state_lock.camera_position.x += 4000 * sin[state_lock.camera_heading as usize & 255];
			state_lock.camera_position.z -= 4000 * cos[state_lock.camera_heading as usize & 255];
		},
	}
	//otherwise do nothing after the action
	AfterAction::DoNothing
}

async fn handle_request(
	header: &'static [u8],
	req: hyper::Request<hyper::body::Incoming>,
	state: Arc<Mutex<State>>,
) -> Result<Response<Full<VecDeque<u8>>>, Infallible> {
	let action = req.uri().path().parse::<Action>();//parse() uses FromStr

	println!("{action:?}");

	//state_lock is moved into do_action here
	let after_action=do_action(state.lock().unwrap(),action.unwrap());
	//state_lock is DROPPED at the end of do_action, meaning the lock is freed

	let response=match after_action{
		AfterAction::Render(state)=>{
			//render a cloned state while being free to serve more requests
			let gif_buffer = render_to_gif(header,state);

			Response::builder()
			    .header(header::CACHE_CONTROL, "max-age=0")
			    .header(header::CONTENT_TYPE, "image/gif")
			    .body(Full::new(VecDeque::from(gif_buffer)))
			    .unwrap()
		},
		AfterAction::DoNothing=>{
			//don't render
			Response::builder()
				.status(302)
				.header(header::CACHE_CONTROL, "max-age=0")
				.header(header::LOCATION, "https://github.com/blocksrey")
				.body(Full::default())
				.unwrap()
		},
	};

	Ok(response)
}

#[tokio::main]
async fn main() {
	let port = env::var("PORT").map_or(7890,
		|port_env|port_env.parse().expect("Invalid port number")
	);
	let address = SocketAddr::from(([0, 0, 0, 0], port));

	let mut header=format!("P6\n{} {}\n255\n", VIEW_SIZE_X, VIEW_SIZE_Y).into_bytes();
	assert_eq!(header.len(),HEADER_SIZE,"Update HEADER_SIZE");
	header.resize(HEADER_SIZE+BUFFER_SIZE,0);

	//use a memory leak to make a static reference to a byte slice
	//it only runs once so does it really count as a memory leak?
	let static_header = Box::new(header).leak();

	let state = Arc::new(Mutex::new(State {
		camera_position: Vec3 { x: 0, y: 4000, z: 0 },
		camera_heading: 0,
		light_direction: Vec3 { x: 0, y: 0, z: 127 },
	}));

	//https://github.com/hyperium/hyper/blob/master/examples/hello.rs
	let listener=tokio::net::TcpListener::bind(address).await.unwrap();

	//coerce &'static mut [u8] into &'static [u8] so the reference can be copied
	let static_header:&'static [u8]=static_header;

	println!("Listening on http://{}",address);
	loop{
		let (tcp,_)=listener.accept().await.unwrap();
		let io=hyper_util::rt::TokioIo::new(tcp);
		let state=state.clone();
		tokio::spawn(async move{
			if let Err(err)=hyper::server::conn::http1::Builder::new()
			.timer(hyper_util::rt::TokioTimer::new())
			.serve_connection(io,hyper::service::service_fn(move|body|
				//I couldn't tell you why it has to be cloned twice
				handle_request(static_header,body,state.clone())
			))
			.await{
				println!("Error serving connection: {:?}",err);
			}
		});
	}
}
