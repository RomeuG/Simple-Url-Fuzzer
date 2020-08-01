use reqwest;
use colored::*;
use url::{Url, Host, Position};

extern crate clap;
use clap::{Arg, App, SubCommand};

use std::env;

use std::time::Duration;

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use std::sync::{Arc, Mutex};

use std::fs::File;
use std::io::{self, prelude::*, BufReader};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type HttpResult<T, K> = std::result::Result<T, K>;

#[derive(Default)]
struct Statistics {
    codes: HashMap<String, Vec<String>>,
    errors: HashMap<String, Vec<String>>,
}

fn error_handler(error: &reqwest::Error) -> String {
    if (error.is_body()) {
		return "BODY".to_string();
    } else if (error.is_decode()) {
		return "DECODE".to_string();
    } else if (error.is_status()) {
		return "STATUS".to_string();
    } else if (error.is_builder()) {
		return "BUILDER".to_string();
    } else if (error.is_timeout()) {
		return "TIMEOUT".to_string();
    } else if (error.is_redirect()) {
		return "REDIRECT".to_string();
    } else {
		return "UNKNOWN".to_string();
    }
}

fn request(url: &str) -> HttpResult<u16, String> {
    let client = reqwest::blocking::Client::builder()
		.timeout(Duration::from_secs(20))
		.build().unwrap();

    let req = client.get(url).send();

    let code = match req {
		Ok(resp) => resp.status().as_u16(),
		Err(e) => return Err(error_handler(&e)),
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
			},
			Err(e) => {
				let mut stats_mutex = stats.lock().unwrap();

				match stats_mutex.errors.entry(e) {
					Entry::Vacant(e) => { e.insert(vec![new_url.clone()]); },
					Entry::Occupied(mut e) => { e.get_mut().push(new_url.clone()); }
				}
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
    let matches = App::new("url-fuzzer")
    	.version("0.1")
    	.author("Romeu Gomes <romeu.bizz@gmail.com>")
    	.about("This is an URL Fuzzer.")
    	.arg(Arg::with_name("url")
             .short("u")
             .long("url")
             .value_name("URL")
             .help("Url to fuzz")
             .takes_value(true))
    	.arg(Arg::with_name("wordlist")
			 .short("w")
             .long("wordlist")
             .value_name("FILE")
             .help("Wordlist with 1 word per line")
             .takes_value(true))
    	.arg(Arg::with_name("threads")
             .short("t")
             .long("threads")
             .value_name("N")
             .help("Number of threads")
             .takes_value(true))
    	.get_matches();

    let url = matches.value_of("url").unwrap().to_owned();
    let file = matches.value_of("wordlist").unwrap();
    let nthreads = matches.value_of("threads").unwrap();

    if !url.contains("@@") {
		println!("Fuzzing indicator not present!");
		std::process::exit(1);
    }

    let file_lines = Arc::new(Mutex::new(load_file(&file)?));

	// fs
	let url_parsed = Url::parse(&url.clone())?;
	let dir_name = url_parsed.host_str().unwrap().to_owned();
	std::fs::create_dir(dir_name.clone()).unwrap_or(());

    // threading stuff
    let mut threads = Vec::new();
    let mut _stats = Statistics {
		codes: HashMap::new(),
		errors: HashMap::new(),
    };

    let mut stats: Arc<Mutex<Statistics>> = Arc::new(Mutex::new(_stats));

    for thread_id in 0..nthreads.parse::<u32>().unwrap() {
		let url_clone = url.clone();
		let vec_clone = file_lines.clone();
		let stats_clone = stats.clone();
		let dir_clone = dir_name.clone();

		threads.push(std::thread::spawn(move || worker(thread_id, url_clone.to_string(), vec_clone, stats_clone)));
    }

    for thr in threads {
		thr.join().unwrap();
    }

    let mut stats_mutex = stats.lock().unwrap();

    for (key, value) in stats_mutex.codes.clone() {

		let mut result_file = std::fs::File::create(dir_name.clone().to_string() + "/" + &key + ".txt").expect("create failed");

		for item in value {
			let formatted = format!("{}\n", item);
			result_file.write(formatted.as_bytes());
		}
    }

    println!("Finishing...");

    return Ok(());
}
