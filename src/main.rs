use reqwest;
use colored::*;

use std::env;

use std::time::Duration;

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use std::sync::{Arc, Mutex};

use std::fs::File;
use std::io::{self, prelude::*, BufReader};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Default)]
struct Statistics {
	codes: HashMap<String, Vec<String>>,
}

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

fn worker(thread_id: u32, url: String, lines: Arc<Mutex<Vec<String>>>, stats: Arc<Mutex<Statistics>>) {
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

				let mut stats_mutex = stats.lock().unwrap();

				match stats_mutex.codes.entry(code.to_string()) {
					Entry::Vacant(e) => { e.insert(vec![new_url.clone()]); },
					Entry::Occupied(mut e) => { e.get_mut().push(new_url.clone()); }
				}

				std::mem::drop(stats_mutex);

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

    let file_lines = Arc::new(Mutex::new(load_file(&file)?));

	// threading stuff
	let mut threads = Vec::new();
	let mut _stats = Statistics {
		codes: HashMap::new(),
	};
	let mut stats: Arc<Mutex<Statistics>> = Arc::new(Mutex::new(_stats));
;
	for thread_id in 0..64 {
		let t_url = url.clone();
		let vec_clone = file_lines.clone();
		let stats_clone = stats.clone();

		threads.push(std::thread::spawn(move || worker(thread_id, t_url, vec_clone, stats_clone)));
	}

	for thr in threads {
		thr.join().unwrap();
	}

	let mut stats_mutex = stats.lock().unwrap();

	for (key, value) in stats_mutex.codes.clone() {
		for item in value {
			println!("{} / {}", key, item);
		}
	}

    println!("Finishing...");

    return Ok(());
}
