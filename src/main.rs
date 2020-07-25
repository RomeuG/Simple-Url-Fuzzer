use reqwest;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn request(url: &str) -> Result<u16> {
	let resp = reqwest::blocking::get(url)?
		.status()
		.as_u16();

	return Ok(resp);
}

fn main() -> Result<()> {
	let result = request("https://google.com/")?;
	println!("{}", result);
	// match resp {
	// 	Ok(v) => println!("working with version: {:?}", v),
	// 	Err(e) => println!("error parsing header: {:?}", e),
	// }

    return Ok(());
}
