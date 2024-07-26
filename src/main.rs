use hyper::{Body, Request, Response, Server, header};
use hyper::service::{make_service_fn, service_fn};
use std::env;
use std::net::SocketAddr;
use std::process::Command;

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
	enum Actions {
		Cx,
		Cy,
		Cz,
		Ry,
		Sdx,
		Sdy,
		Sdz,
	}

	let mut arg = [0, 10000, 0, 0, 0, 0, 127];

	let path = req.uri().path();
	let path_after_slash = if path.len() > 1 {
		&path[1..]
	} else {
		""
	};
	let action = path_after_slash.chars().next().unwrap_or(' ') as char;

	println!("{action}");

	if action == 'v' {
		let cmd = format!(
			"./asahi_renderer/render {} {} {} {} {} {} {} | ffmpeg -loglevel 0 -i - -f gif -vf 'split[a][b];[a]palettegen[p];[b][p]paletteuse' -",
			arg[Actions::Cx as usize],
			arg[Actions::Cy as usize],
			arg[Actions::Cz as usize],
			arg[Actions::Ry as usize],
			arg[Actions::Sdx as usize],
			arg[Actions::Sdy as usize],
			arg[Actions::Sdz as usize]
		);

		let image = Command::new("sh")
			.arg("-c")
			.arg(cmd)
			.output()
			.unwrap();

		// We could probably make this faster with piping
		let response = Response::builder()
			.header(header::CONTENT_TYPE, "image/gif")
			.body(Body::from(image.stdout))
			.unwrap();

		Ok(response)
	} else {
		match action {
			'r' => arg[Actions::Ry as usize] -= 32,
			'l' => arg[Actions::Ry as usize] += 32,
			'u' => {
				let sin: Vec<i32> = (0..=255).map(|x| (x as f32).sin() as i32).collect();
				let cos: Vec<i32> = (0..=255).map(|x| (x as f32).cos() as i32).collect();
				arg[Actions::Cx as usize] -= 33 * sin[arg[Actions::Ry as usize] as usize & 255];
				arg[Actions::Cz as usize] += 33 * cos[arg[Actions::Ry as usize] as usize & 255];
			}
			'd' => arg[Actions::Ry as usize] += 64,
			_ => {}
		}

		// Redirect the user to a different page
		let redirect_url = "https://github.com/blocksrey";
		let response = Response::builder()
			.status(302)
			.header(header::LOCATION, redirect_url)
			.body(Body::empty())
			.unwrap();

		Ok(response)
	}
}

#[tokio::main]
async fn main() {
	let port = env::var("PORT").unwrap_or_else(|_| "7890".to_string());
	let port: u16 = port.parse().expect("Invalid port number");

	let address = SocketAddr::from(([0, 0, 0, 0], port));
	let make_svc = make_service_fn(|_conn| async { Ok::<_, hyper::Error>(service_fn(handle_request)) });

	let server = Server::bind(&address).serve(make_svc);

	println!("Listening on http://{}", address);
	if let Err(e) = server.await {
		eprintln!("Server error: {}", e);
	}
}
