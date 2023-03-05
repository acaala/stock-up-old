use std::fs::File;
use std::io::{Write, Read};
use std::path::Path;
use std::{env, io};
use std::error::Error;
use std::process;


use bytes::{self, Bytes};
use dotenv::dotenv;
use serde_json;
use serde::{Deserialize, Serialize};


use rand::thread_rng;
use rand::seq::SliceRandom;


#[derive(Debug, Serialize, Deserialize)]
struct Config {
    unsplash_key: String,
    default_dir: String,
}

impl Config {
    fn new() -> Result<Config, &'static str> {
        let mut unsplash_key = String::new();
        let mut default_dir = String::new();

        println!("Enter your unsplash key:");
        io::stdin().read_line(&mut unsplash_key);


        println!("Enter the directory you want to save your images:");
        io::stdin().read_line(&mut default_dir);

        let config_file_path = "config.txt";
        let mut config_file = File::create(config_file_path).unwrap();
        
        let c = Config { unsplash_key: unsplash_key.trim().to_owned(), default_dir: default_dir.trim().to_owned() };
        let json_config = serde_json::to_string(&c).unwrap();
        config_file.write_all(json_config.as_bytes()).unwrap();


        return Ok(c);
    }

    fn set() -> Result<Config, &'static str> {
        let mut config_file = File::open("config.txt").unwrap();
        let mut contents = String::new();
        config_file.read_to_string(&mut contents).unwrap();
        let c: Config = serde_json::from_str(&contents).unwrap();

        return Ok(c);
    }
}



struct Arguments {
    flag: String,
    seed: String,
    filename: String,
}

impl Arguments {
    fn new(args: &[String]) -> Result<Arguments, &'static str> {

        if args.len() < 2 {
            let seeds = vec!["Poker".to_owned(), "Nature".to_owned(), "Science".to_owned(), "Animals".to_owned()];
            let mut rng = thread_rng();
            let random_seed = seeds.choose(&mut rng).unwrap();
            
            return Ok(Arguments{ flag: String::from(""), seed: random_seed.to_string(), filename: String::from("example.png")});
        } else {
            if args[1].contains("-h") || args[1].contains("-help") && args.len() == 2 {
                // println!("Usage: -c to copy filename to clipboard \r\n -h or -help to show this help message");
                println!("Usage: scriptname optional_seed optional_path \r\n -h or -help to show this help message");
                return Err("help");

            } else {
                let filename = if args[2].is_empty() { "example.png".to_owned() } else { args[2].clone() };

                return Ok(Arguments {flag: String::from(""), seed: args[1].clone(), filename })
            };
        }
    }
}

#[tokio::main]
async fn get_image_from_unsplash(api_key: String, seed: String) -> Result<Bytes, Box<dyn Error>> {
    let url: String = "https://api.unsplash.com/search/photos".to_owned() + "?query=" + &seed + "&client_id=" + &api_key;

    println!("Searching for images with seed: {}", seed);

    let res = reqwest::get(url).await?;
    let body: String = res.text().await?;
    let parsed_body: serde_json::Value = serde_json::from_str(&body)?;

    if parsed_body["errors"].is_array() {
        if parsed_body["errors"][0].as_str().unwrap().contains("OAuth") {
            eprintln!("Failed to authenticate");
            eprintln!("Exiting...");
            process::exit(0);
        }
    }

    if parsed_body["results"][0]["description"].is_null() {
        println!("Found image");
    } else {
        println!("Found image: {}", parsed_body["results"][0]["description"]);
    }

    println!("Downloading image...");

    let image_download_url = parsed_body["results"][0]["urls"]["raw"].as_str().unwrap();
    let download_response = reqwest::get(image_download_url).await?;

    return Ok(download_response.bytes().await?);
}



fn get_image(arguments: Arguments, config: Config) -> Result<(), Box<dyn Error>> {
    let content = get_image_from_unsplash(config.unsplash_key, arguments.seed).unwrap_or_else(
        |err| {
            eprint!("Problem getting image: {}", err);
            process::exit(0);
        }
    );
    
    println!("Saving image at {}{}",config.default_dir, arguments.filename);
    let img = image::load_from_memory(&content)?;

    img.save(format!{"{}{}",config.default_dir, arguments.filename}).unwrap();

    Ok(())
}

fn main() {
    dotenv().ok(); 
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let config;
    
    if Path::new("./config.txt").exists() {        
        config = Config::set().unwrap_or_else(  |err| {
            eprint!("{} problem setting config: {}", program, err);
            process::exit(0);
        });
    } else {
        config = Config::new().unwrap_or_else(
            |err| {
                eprint!("{} problem creating config: {}", program, err);
                process::exit(0);
            }
        );
    }

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

    get_image(arguments, config).unwrap();

    println!("Image saved");
}
