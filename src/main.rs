use chrono::Local;
use reqwest::blocking::{multipart, Client};
use rppal::gpio::Gpio;
use std::{env, error::Error, io::ErrorKind, process::Command};

const GPIO17: u8 = 17;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let mut pir = Gpio::new()?.get(GPIO17)?.into_input();
    pir.set_interrupt(rppal::gpio::Trigger::RisingEdge)?;

    let line_token =
        env::var("LINE_TOKEN").expect("LINE_TOKEN is empty. Set the access token to LINE_TOKEN");
    let image_dir = "/tmp/cat-sv";

    match std::fs::create_dir(image_dir) {
        Err(e) if e.kind() == ErrorKind::AlreadyExists => log::info!("{}", e),
        Err(e) => {
            log::error!("{}", e);
            return Err(Box::new(e));
        }
        Ok(_) => (),
    }

    let ev = 0.5.to_string();
    let shutter = 2000000.to_string();
    let width = 1600.to_string();
    let height = 900.to_string();

    loop {
        match pir.poll_interrupt(true, None) {
            Ok(_) => {
                let dt = Local::now();
                let file_name = format!("{}/image_{}.jpg", image_dir, dt.format("%Y%m%d%H%M%S"));

                Command::new("libcamera-jpeg")
                    .args([
                        "-o",
                        file_name.as_str(),
                        "--nopreview",
                        "-ev",
                        &ev,
                        "--shutter",
                        &shutter,
                        "--width",
                        &width,
                        "--height",
                        &height,
                    ])
                    .output()?;
                log::info!("snap: {}", file_name);

                let client = Client::new();
                let form = multipart::Form::new()
                    .text("message", "Detected")
                    .file("imageFile", file_name)?;
                let req = client
                    .post("https://notify-api.line.me/api/notify")
                    .bearer_auth(&line_token)
                    .multipart(form);

                let res = req.send()?;
                log::info!("response: {:?}", res);
            }
            e => log::error!("{:?}", e),
        }
    }
}
