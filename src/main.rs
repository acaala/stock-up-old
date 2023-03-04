use std::env;
use std::error::Error;

use bytes::{self, Bytes};
use std::process;
use dotenv::dotenv;
use serde_json;
use image::GenericImageView;
use rand::thread_rng;
use rand::seq::SliceRandom;


struct Arguments {
    flag: String,
    seed: String,
    path: String,
}

impl Arguments {
    fn new(args: &[String]) -> Result<Arguments, &'static str> {

        if args.len() < 2 {
            let seeds = vec!["Poker".to_owned(), "Nature".to_owned(), "Science".to_owned(), "Animals".to_owned()];
            let mut rng = thread_rng();
            let random_seed = seeds.choose(&mut rng).unwrap();
            
            return Ok(Arguments{ flag: String::from(""), seed: random_seed.to_string(), path: String::from("./example.png")});
        } else {
            if args[1].contains("-h") || args[1].contains("-help") && args.len() == 2 {
                // println!("Usage: -c to copy filename to clipboard \r\n -h or -help to show this help message");
                println!("Usage: scriptname optional_seed optional_path \r\n -h or -help to show this help message");
                return Err("help");

            } else {
                let path = if args[2].is_empty() { "./example.png".to_owned() } else { args[2].clone() };

                return Ok(Arguments {flag: String::from(""), seed: args[1].clone(), path })
            };
        }
    }
}


#[tokio::main]
async fn get_image_from_unsplash(seed: String) -> Result<Bytes, Box<dyn Error>> {
    let url: String = "https://api.unsplash.com/search/photos".to_owned() + "?query=" + &seed + "&client_id=" + &std::env::var("API_KEY").unwrap();

    println!("Searching for images with seed: {}", seed);

    let res = reqwest::get(url).await?;
    let body: String = res.text().await?;
    let parsed_body: serde_json::Value = serde_json::from_str(&body)?;

    println!("Found image: {}", parsed_body["results"][0]["description"]);
    println!("Downloading image...");
    let image_download_url = parsed_body["results"][0]["urls"]["raw"].as_str().unwrap();

    let download_response = reqwest::get(image_download_url).await?;

    return Ok(download_response.bytes().await?);
}



fn get_image(arguments: Arguments) -> Result<(), Box<dyn Error>> {
    let content = get_image_from_unsplash(arguments.seed).unwrap();
    
    println!("Saving image at {}", arguments.path);
    let img = image::load_from_memory(&content)?;

    img.save(arguments.path).unwrap();

    Ok(())
}

fn main() {
    dotenv().ok(); 
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let arguments = Arguments::new(&args).unwrap_or_else(
        |err| {
            if err.contains("help") {
                process::exit(0);
            } else {
                eprint!("{} problem parsing argumens: {}", program, err);
                process::exit(0);
            }
        }
    );

    get_image(arguments);

    println!("Image saved");

   

    // println!("{:?} {:?} {:?}", arguments.flag, arguments.seed, arguments.filename);
}
