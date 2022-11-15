use chrono::Local;
use reqwest::blocking::{multipart, Client};
use rppal::gpio::Gpio;
use std::{env, error::Error, io::ErrorKind, process::Command};

const GPIO17: u8 = 17;
const LINE_NOTIFY_API: &str = "https://notify-api.line.me/api/notify";

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    log::info!("start!");

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

    loop {
        match pir.poll_interrupt(true, None) {
            Ok(_) => {
                log::info!("Detect someone!");

                let client = Client::new();
                let dt = Local::now();
                let file_name = format!("{}/image_{}.jpg", image_dir, dt.format("%Y%m%d%H%M%S"));

                if let Err(_) = libcam(&file_name, &line_token) {
                    continue;
                }

                let form = multipart::Form::new()
                    .text("message", "Detected")
                    .file("imageFile", file_name);
                if let Err(e) = form {
                    log::error!("{}", e);
                    unreachable!()
                }

                let req = client
                    .post(LINE_NOTIFY_API)
                    .bearer_auth(&line_token)
                    .multipart(form.unwrap());
                match req.send() {
                    Ok(res) => log::info!("{:?}", res),
                    Err(e) => log::error!("{}", e),
                }
            }
            e => {
                log::error!("{:?}", e);
                unreachable!()
            }
        }
    }
}

fn libcam(file_name: &str, line_token: &str) -> Result<(), std::io::Error> {
    let client = Client::new();
    let libcam = Command::new("libcamera-jpeg")
        .args(["-o", file_name])
        .args(get_options())
        .output();

    if let Err(e) = libcam {
        log::error!("{}", e);
        let req = client
            .post(LINE_NOTIFY_API)
            .body("detected, but failed to snap.")
            .bearer_auth(&line_token);
        match req.send() {
            Ok(res) => log::info!("{:?}", res),
            Err(e) => log::error!("{}", e),
        }
        return Err(e);
    }
    log::info!("snap: {}", file_name);
    Ok(())
}

fn get_options() -> Vec<String> {
    let ev = 0.5.to_string();
    let shutter = 2000000.to_string();
    let width = 1600.to_string();
    let height = 900.to_string();
    let libcam_args = [
        "--nopreview",
        "--ev",
        &ev,
        "--shutter",
        &shutter,
        "--width",
        &width,
        "--height",
        &height,
    ];
    libcam_args.map(|e| e.to_string()).to_vec()
}
