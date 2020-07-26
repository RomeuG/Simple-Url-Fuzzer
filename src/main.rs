use reqwest;

use std::env;

use std::time::Duration;

use std::fs::File;
use std::io::{self, prelude::*, BufReader};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn request(url: &str) -> Result<u16> {
	println!("Initiating request to {}...", url);

	let client = reqwest::blocking::Client::builder()
		.timeout(Duration::from_secs(5))
		.build()?;

	let resp = client.get(url).send()?
		.status()
		.as_u16();

	return Ok(resp);
}

fn load_file(file: &str) -> Result<Vec<String>> {
	let mut vector = Vec::new();

	let file = File::open(file)?;
	let reader = BufReader::new(file);

	for line in reader.lines() {
        //println!("{}", line?);
		vector.push(line?);
    }

	return Ok(vector);
}

fn main() -> Result<()> {

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
		//let new_url = url.to_owned() + &line;
		let new_url = url.replace("@@", &line);
		let result = request(&new_url)?;
		println!("{:?}", result);
	}

	println!("Finishing...");

    return Ok(());
}
