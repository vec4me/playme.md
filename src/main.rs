use hyper::{Body, Request, Response, Server, header};
use hyper::service::{make_service_fn, service_fn};
use std::sync::{Arc, Mutex};
use std::env;
use std::net::SocketAddr;
use std::process::Command;

#[derive(Debug)]
struct AppState {
    arg: [i32; 7],
}

async fn handle_request(
    req: Request<Body>,
    state: Arc<Mutex<AppState>>,
) -> Result<Response<Body>, hyper::Error> {
    enum Actions {
        Cx,
        Cy,
        Cz,
        Ry,
        Sdx,
        Sdy,
        Sdz,
    }

    let path = req.uri().path();
    let path_after_slash = if path.len() > 1 {
        &path[1..]
    } else {
        ""
    };
    let action = path_after_slash.chars().next().unwrap_or(' ') as char;

    let mut state = state.lock().unwrap();
    println!("{action}");

    if action == 'v' {
        println!("{}", state.arg[Actions::Ry as usize]);

        let cmd = format!(
            "./asahi_renderer {} {} {} {} {} {} {} | ffmpeg -loglevel 0 -i - -f gif -vf 'split[a][b];[a]palettegen[p];[b][p]paletteuse' -",
            state.arg[Actions::Cx as usize],
            state.arg[Actions::Cy as usize],
            state.arg[Actions::Cz as usize],
            state.arg[Actions::Ry as usize],
            state.arg[Actions::Sdx as usize],
            state.arg[Actions::Sdy as usize],
            state.arg[Actions::Sdz as usize]
        );

        let image = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()
            .unwrap();

        let response = Response::builder()
            .header(header::CACHE_CONTROL, "max-age=0")
            .header(header::CONTENT_TYPE, "image/gif")
            .body(Body::from(image.stdout))
            .unwrap();

        Ok(response)
    } else {
        match action {
            'h' => state.arg[Actions::Ry as usize] += 32,
            'l' => state.arg[Actions::Ry as usize] -= 32,
            'k' => {
                let sin: Vec<i32> = (0..=255).map(|x| (x as f32).sin() as i32).collect();
                let cos: Vec<i32> = (0..=255).map(|x| (x as f32).cos() as i32).collect();
                state.arg[Actions::Cx as usize] -= 4000 * sin[state.arg[Actions::Ry as usize] as usize & 255];
                state.arg[Actions::Cz as usize] += 4000 * cos[state.arg[Actions::Ry as usize] as usize & 255];
            }
            'j' => state.arg[Actions::Ry as usize] += 64,
            _ => {}
        }

        let redirect_url = "https://github.com/blocksrey";
        let response = Response::builder()
            .status(302)
            .header(header::CACHE_CONTROL, "max-age=0")
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
    let state = Arc::new(Mutex::new(AppState { arg: [0, 4000, 0, 0, 0, 0, 127] }));

    let make_svc = make_service_fn(move |_conn| {
        let state = Arc::clone(&state);
        async move { Ok::<_, hyper::Error>(service_fn(move |req| handle_request(req, state.clone()))) }
    });

    let server = Server::bind(&address).serve(make_svc);

    println!("Listening on http://{}", address);
    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
}
