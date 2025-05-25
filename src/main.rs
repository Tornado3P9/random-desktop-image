use rand::rng;
use reqwest::blocking::get;
use serde::Deserialize;
use std::fs::{self, File};
use std::io;
use std::path::Path;
use rand::seq::IndexedRandom;
use std::process::Command;

#[derive(Deserialize)]
struct Config {
    save_path: String,
    width: u32,
    height: u32,
    local_background_folder: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read the config file
    let home_dir = dirs::home_dir().expect("Could not find home directory");
    let config_path = home_dir.join(".config/random-desktop-image/config.json");
    let config: Config = serde_json::from_reader(File::open(config_path)?)?;

    // Construct the URL with the specified width and height
    let url: String = format!("https://picsum.photos/{}/{}", config.width, config.height);

    // Attempt to download the image
    let response = get(&url);

    match response {
        Ok(resp) => {
            // Save the online image to the specified path
            let mut file: File = File::create(Path::new(&config.save_path))?;
            io::copy(&mut resp.bytes()?.as_ref(), &mut file)?;
            println!("Image saved to {}", config.save_path);

            // Change the background using xfconf-query
            // (Get the available properties: xfconf-query -c xfce4-desktop -l)
            Command::new("xfconf-query")
                .args(&[
                    "--channel", "xfce4-desktop",
                    "--property", "/backdrop/screen0/monitorHDMI-0/workspace0/last-image",
                    "--set", &config.save_path,
                ])
                .status()?;
        }
        Err(_) => {
            // If the online image cannot be reached, choose a random local image
            let paths: Vec<_> = fs::read_dir(&config.local_background_folder)?
                .filter_map(Result::ok)
                .filter(|entry| entry.path().is_file())
                .collect();

            if let Some(random_entry) = paths.choose(&mut rng()) {
                if let Some(random_image_path) = random_entry.path().to_str() {
                    // Change the background using xfconf-query
                    // (Get the available properties: xfconf-query -c xfce4-desktop -l)
                    Command::new("xfconf-query")
                        .args(&[
                            "--channel", "xfce4-desktop",
                            "--property", "/backdrop/screen0/monitorHDMI-0/workspace0/last-image",
                            "--set", random_image_path,
                        ])
                        .status()?;
                }
            } else {
                println!("No images found in the local background folder.");
            }
        }
    }

    Ok(())
}
