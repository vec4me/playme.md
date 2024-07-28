use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Response, Server, header};
use std::convert::Infallible;
use std::env;
use std::io::Write;
use std::net::SocketAddr;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

macro_rules! fixed {
	($v: expr) => {
		$v << SHIFT
	};
}

macro_rules! normal {
	($v: expr) => {
		$v >> SHIFT
	};
}

macro_rules! multiply {
	($a: expr, $b: expr) => {
		($a*$b) >> SHIFT
	};
}

macro_rules! divide {
	($a: expr, $b: expr) => {
		($a << SHIFT)/$b
	};
}

const SHIFT: i32 = 7;
const CLOUD_HEIGHT: i32 = fixed!(200);
const VIEW_SIZE_X: i32 = 320;
const VIEW_SIZE_Y: i32 = 200;
const R: usize = 0;
const G: usize = 1;
const B: usize = 2;
const PI: i32 = 64;

fn sqrt(n: i32) -> i32 {
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

fn dot3(a: &Vec3, b: &Vec3) -> i32 {
	// multiply!(a.x, b.x) + multiply!(a.y, b.y) + multiply!(a.z, b.z)
	a.x*b.x + a.y*b.y + a.z*b.z
}

fn norm3(v: &Vec3) -> i32 {
	sqrt(dot3(v, v))
}

// fn multiply(a: i32, b: i32) -> i32 {
// 	(a*b) >> SHIFT
// }

// fn divide(a: i32, b: i32) -> i32 {
// 	(a << SHIFT)/b
// }

// fn fixed(v: i32) -> i32 {
// 	v << SHIFT
// }

fn unit3(v: &mut Vec3) {
	let n = norm3(v);
	v.x = divide!(v.x, n);
	v.y = divide!(v.y, n);
	v.z = divide!(v.z, n);
}

fn get_trig_tables() -> ([i32; 256], [i32; 256]) {
	let sin_table: [i32; 256] = [
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

	let mut cos_table = [0; 256];
	for i in 0..256 {
		cos_table[i as usize] = sin_table[((i + 65) & 255) as usize];
	}

	(cos_table, sin_table)
}

async fn handle_request(
	req: hyper::Request<Body>,
	state: Arc<Mutex<State>>,
) -> Result<Response<Body>, Infallible> {
	let action = req.uri().path().chars().nth(1).unwrap_or(' ');

	let mut state_lock = state.lock().unwrap();

	println!("{action}");

	let (cos_table, sin_table) = get_trig_tables();

	macro_rules! cos {
		($t: expr) => {
			cos_table[($t & 255) as usize]
		};
	}

	macro_rules! sin {
		($t: expr) => {
			sin_table[($t & 255) as usize]
		};
	}

	let camera_right_x = cos!(state_lock.camera_heading);
	let camera_right_z = -sin!(state_lock.camera_heading);

	match action {
		'v' => {
			for pixel_index in 0..(VIEW_SIZE_X*VIEW_SIZE_Y) as usize {
				macro_rules! color {
					($channel: expr) => {
						state_lock.image_buffer[15 + 3*pixel_index + $channel]
					};
				}

				let pixel_position = Vec2 {
					x: pixel_index as i32%VIEW_SIZE_X,
					y: pixel_index as i32/VIEW_SIZE_X,
				};

				let view_offset = Vec2 {
					x: VIEW_SIZE_X - (pixel_position.x << 1),
					y: VIEW_SIZE_Y - (pixel_position.y << 1),
				};

				let mut pixel_direction = Vec3 {
					x: view_offset.x*camera_right_x/VIEW_SIZE_Y + camera_right_z,
					y: divide!(view_offset.y, VIEW_SIZE_Y),
					z: view_offset.x*-camera_right_z/VIEW_SIZE_Y + camera_right_x,
				};
				unit3(&mut pixel_direction);

				let pixel_distance =
				if pixel_direction.y > 0 {
					divide!(CLOUD_HEIGHT, pixel_direction.y)
				}
				else if pixel_direction.y < 0 {
					divide!(-state_lock.camera_position.y, pixel_direction.y)
				}
				else {
					0
				};

				let hit = Vec3 {
					x: state_lock.camera_position.x + multiply!(pixel_distance, pixel_direction.x),
					y: state_lock.camera_position.y + multiply!(pixel_distance, pixel_direction.y),
					z: state_lock.camera_position.z + multiply!(pixel_distance, pixel_direction.z),
				};

				if pixel_direction.y >= 0 {
					color!(R) = 188;
					color!(G) = 0;
					color!(B) = 45;

					let sky = cos!(((cos!((hit.z >> 11) + 291) + (hit.x >> 8)) >> 1)) + cos!(hit.z/500)/4 + 30;
					if sky < 0 {
						color!(R) = sky as u8;
						color!(G) = sky as u8;
						color!(B) = sky as u8;
					}
					else if dot3(&pixel_direction, &state_lock.light_direction) < 128*fixed!(1) {
						color!(R) = (128 - 128*pixel_direction.y/255) as u8;
						color!(G) = (179 - 179*pixel_direction.y/255) as u8;
						color!(B) = (255 - 76*pixel_direction.y/255) as u8;
					}
				}
				else {
					color!(R) = 77;
					color!(G) = 40;
					color!(B) = 0;

					if !(((hit.x >> 13)%7)*((hit.z >> 13)%9) != 0) {
						color!(R) = 100;
						color!(G) = 100;
						color!(B) = 110;
					}
					else {
						color!(R) = 60;
						color!(G) = (sin!((hit.x/20))/2 + 55) as u8;
						color!(B) = 0;

						// Checking if it's negative (overflow)
						if color!(G) > 200 {
							color!(G) = 60;
							color!(B) = 120;
						}
					}
				}
			}

			let image_buffer = state_lock.image_buffer.clone();

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

			let body = Body::from(gif_buffer);
			let response = Response::builder()
				.header(header::CACHE_CONTROL, "max-age=0")
				.header(header::CONTENT_TYPE, "image/gif")
				.body(body)
				.unwrap();

			return Ok(response);
		}
		'l' => state_lock.camera_heading += PI/2,
		'h' => state_lock.camera_heading -= PI/2,
		'k' => {
			state_lock.camera_position.z += fixed!(1)*cos!(state_lock.camera_heading)/4;
			state_lock.camera_position.x -= fixed!(1)*sin!(state_lock.camera_heading)/4;
		}
		'j' => state_lock.camera_heading += PI,
		_ => {},
	}

	let response = Response::builder()
		.status(302)
		.header(header::CACHE_CONTROL, "max-age=0")
		.header(header::LOCATION, "https://github.com/blocksrey")
		.body(Body::empty())
		.unwrap();

	Ok(response)
}

#[derive(Clone)]
struct State {
	image_buffer: Vec<u8>,
	camera_position: Vec3,
	camera_heading: i32,
	light_direction: Vec3,
}

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

#[tokio::main]
async fn main() {
	let port = env::var("PORT").unwrap_or_else(|_| "7890".to_string());
	let address = SocketAddr::from(([0, 0, 0, 0], port.parse().expect("Invalid port number")));

	let header = format!("P6\n{} {}\n255\n", VIEW_SIZE_X, VIEW_SIZE_Y).into_bytes();
	let buffer_size = header.len() + (3*VIEW_SIZE_X*VIEW_SIZE_Y) as usize;

	let mut state = State {
		image_buffer: {
			let mut buffer = vec![0; buffer_size];
			buffer[..header.len()].copy_from_slice(&header);
			buffer
		},
		camera_position: Vec3 { x: 0, y: fixed!(30), z: 0 },
		camera_heading: 0,
		light_direction: Vec3 { x: 0, y: 0, z: fixed!(1) },
	};
	unit3(&mut state.light_direction);

	let shared_state = Arc::new(Mutex::new(state));

	let make_svc = make_service_fn(move |_conn| {
		let state = Arc::clone(&shared_state);
		async move {
			Ok::<_, hyper::Error>(service_fn(move |req| {
				let state = Arc::clone(&state);
				handle_request(req, state)
			}))
		}
	});

	let server = Server::bind(&address).serve(make_svc);

	println!("Listening on http://{}", address);
	if let Err(e) = server.await {
		eprintln!("Server error: {}", e);
	}
}
