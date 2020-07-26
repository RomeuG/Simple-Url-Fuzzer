use reqwest;

use std::fs::File;
use std::io::{self, prelude::*, BufReader};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn request(url: &str) -> Result<u16> {
	let resp = reqwest::blocking::get(url)?
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
	let result = request("https://google.com/")?;
	println!("{}", result);
	// match resp {
	// 	Ok(v) => println!("working with version: {:?}", v),
	// 	Err(e) => println!("error parsing header: {:?}", e),
	// }

	let file_lines = load_file("wordlist.txt")?;
	for line in file_lines {
		println!("{}", line);
	}

    return Ok(());
}
