use std::process::Command;
use std::env;
use std::path::Path;

fn main() {
	let out_dir = env::var("OUT_DIR").unwrap();
	let repo_url = "https://github.com/blocksrey/asahi_renderer.git";
	let repo_dir = Path::new(&out_dir).join("asahi_renderer");

	// Clone the repository if it doesn't already exist
	if !repo_dir.exists() {
		Command::new("git")
			.args(&["clone", repo_url, repo_dir.to_str().unwrap()])
			.status()
			.expect("Failed to clone repository");
	}

	// Change to the repository directory
	Command::new("git")
		.current_dir(&repo_dir)
		.args(&["checkout", "rust"])
		.status()
		.expect("Failed to checkout branch");

	// Build the executable
	Command::new("cargo")
		.current_dir(&repo_dir)
		.args(&["build", "--release"])
		.status()
		.expect("Failed to build executable");

	// Optionally, copy the built executable to OUT_DIR
	let target_dir = repo_dir.join("target").join("release");
	let executable_path = target_dir.join("asahi_renderer"); // Adjust if the executable has a different name

	if executable_path.exists() {
		std::fs::copy(&executable_path, Path::new(".").join("asahi_renderer"))
			.expect("Failed to copy executable");
	}
}
