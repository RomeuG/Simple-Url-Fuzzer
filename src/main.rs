use reqwest;
use colored::*;

use std::env;

use std::time::Duration;

use std::fs::File;
use std::io::{self, prelude::*, BufReader};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn request(url: &str) -> Result<u16> {
    let client = reqwest::blocking::Client::builder()
		.timeout(Duration::from_secs(5))
		.build()?;

    let req = client.get(url).send();

    let code = match req {
		Ok(resp) => resp.status().as_u16(),
		Err(e) => return Err(e.into()),
    };

    return Ok(code);
}

fn worker(thread_id: u32, url: String, lines: std::sync::Arc<std::sync::Mutex<Vec<String>>>) {
	// for line in file_lines {
	// 	let new_url = url.replace("@@", &line);

	// 	let result = request(&new_url);
	// 	match result {
	// 		Ok(code) => {
	// 			println!("[{}] - {}", code.to_string().green(), new_url);
	// 			success_vec.push(new_url);
	// 		},
	// 		Err(e) => {
	// 			println!("{:?}", e);
	// 			failure_vec.push(new_url);
	// 		}
	// 	}
    // }

	loop {
		let mut line_mutex = lines.lock().unwrap();

		if (line_mutex.len() < 1) {
			break;
		}

		let line = line_mutex[0].clone();
		line_mutex.remove(0);
		std::mem::drop(line_mutex);

		let new_url = url.replace("@@", &line);
		//println!("[{}] - Testing url: {}", thread_id, new_url);
		let result = request(&new_url);
		match result {
			Ok(code) => {
				println!("[{}] - {}", code.to_string().green(), new_url);
				//success_vec.push(new_url);
			},
			Err(e) => {
				println!("{:?}", e);
				//failure_vec.push(new_url);
			}
		}
	}
}

fn load_file(file: &str) -> Result<Vec<String>> {
    let mut vector = Vec::new();

    let file = File::open(file)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
		vector.push(line?);
    }

    return Ok(vector);
}

fn main() -> Result<()> {
    let mut success_vec: Vec<String> = Vec::new();
    let mut failure_vec: Vec<String> = Vec::new();

    // args
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
		println!("Not enough arguments!");
		std::process::exit(1);
    }

    let url = args[1].clone();
    let file = &args[2];

    if !url.contains("@@") {
		println!("Fuzzing indicator not present!");
		std::process::exit(1);
    }

    let file_lines = std::sync::Arc::new(std::sync::Mutex::new(load_file(&file)?));

	// threading stuff
	let mut threads = Vec::new();

	for thread_id in 0..64 {
		let t_url = url.clone();
		let vec_clone = file_lines.clone();

		threads.push(std::thread::spawn(move || worker(thread_id, t_url, vec_clone)));
	}

	for thr in threads {
		thr.join().unwrap();
	}

    println!("Success: {} - Failure: {}", success_vec.len(), failure_vec.len());
    println!("Finishing...");

    return Ok(());
}
