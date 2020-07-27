use reqwest;
use colored::*;

use std::env;

use std::time::Duration;

use std::fs::File;
use std::io::{self, prelude::*, BufReader};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn handler(e: reqwest::Error) {

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

    let url = &args[1];
    let file = &args[2];

    if !url.contains("@@") {
		println!("Fuzzing indicator not present!");
		std::process::exit(1);
    }

    let file_lines = load_file(&file)?;

    for line in file_lines {
		let new_url = url.replace("@@", &line);

		let result = request(&new_url);
		match result {
			Ok(code) => {
				println!("[{}] - {}", code.to_string().green(), new_url);
				success_vec.push(new_url);
			},
			Err(e) => {
				println!("{:?}", e);
				failure_vec.push(new_url);
			}
		}
    }

    println!("Success: {} - Failure: {}", success_vec.len(), failure_vec.len());
    println!("Finishing...");

    return Ok(());
}
