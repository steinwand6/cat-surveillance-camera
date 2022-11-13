use chrono::Local;
use reqwest::blocking::Client;
use rppal::gpio::{Gpio, Level};
use std::{env, error::Error, process::Command};

const GPIO17: u8 = 17;

fn main() -> Result<(), Box<dyn Error>> {
    let mut pir = Gpio::new()?.get(GPIO17)?.into_input();
    pir.set_interrupt(rppal::gpio::Trigger::Both)?;

    let line_token =
        env::var("LINE_TOKEN").expect("LINE_TOKEN is empty. Set the access token to LINE_TOKEN");

    let image_dir = "/tmp/cat-sv";

    loop {
        match pir.poll_interrupt(true, None) {
            Ok(trigger) => match trigger {
                Some(Level::High) => {
                    let dt = Local::now();
                    let file_name =
                        format!("{}/image_{}.jpg", image_dir, dt.format("%Y%m%d%H%M%S"));
                    Command::new("libcamera-jpeg")
                        .args(["-o", file_name.as_str()])
                        .output()?;
                    println!("!!");
                    let client = Client::new();
                    let _ = client
                        .post("https://notify-api.line.me/api/notify")
                        .header(
                            reqwest::header::AUTHORIZATION,
                            format!("bearer {}", line_token),
                        )
                        .body(format!("message=Detected! imageFiles=@{}", file_name))
                        .send();
                }
                _ => (),
            },
            _ => break,
        }
    }
    Ok(())
}
