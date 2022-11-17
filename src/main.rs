use chrono::Local;
use reqwest::blocking::{multipart, Client};
use rppal::gpio::Gpio;
use std::{env, error::Error, io::ErrorKind, process::Command};
use thiserror::Error;

const GPIO17: u8 = 17;
const LINE_NOTIFY_API: &str = "https://notify-api.line.me/api/notify";

#[derive(Error, Debug)]
pub enum CatCamError {
    #[error("Failed to send a request.")]
    SendRequest(#[source] reqwest::Error),
    #[error("Failed to libcamera.")]
    FailureLibcamera(#[source] std::io::Error),
}

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

                let dt = Local::now();
                let file_name = format!("{}/image_{}.jpg", image_dir, dt.format("%Y%m%d%H%M%S"));

                let result = libcam(&file_name);
                log::info!("send a LINE Notify.");
                match send_line_notify(result, &file_name, &line_token) {
                    Ok(res) => log::info!("status: {}", res.status()),
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

fn send_line_notify(
    libcam_result: Result<(), CatCamError>,
    file_name: &str,
    line_token: &str,
) -> Result<reqwest::blocking::Response, CatCamError> {
    let client = Client::new().post(LINE_NOTIFY_API).bearer_auth(&line_token);
    let result;

    match libcam_result {
        Ok(_) => {
            let form = multipart::Form::new()
                .text("message", "Detected")
                .file("imageFile", file_name);
            let req;
            match form {
                Ok(_) => req = client.multipart(form.unwrap()),
                Err(_) => {
                    req = client
                        .body("detected someone but something failed with creating request form.")
                }
            }
            match req.send() {
                Ok(res) => result = Ok(res),
                Err(e) => {
                    result = Err(CatCamError::SendRequest(e));
                }
            }
        }
        Err(_) => {
            let req = client.body("detected someone, but failed to execute libcamera.");
            match req.send() {
                Ok(res) => result = Ok(res),
                Err(e) => result = Err(CatCamError::SendRequest(e)),
            }
        }
    }
    result
}

fn libcam(file_name: &str) -> Result<(), CatCamError> {
    let libcam = Command::new("libcamera-jpeg")
        .args(["-o", file_name])
        .args(get_options())
        .output();
    match libcam {
        Ok(_) => log::info!("libcamera-jpeg: {}", file_name),
        Err(e) => {
            log::error!("{:?}", e);
            return Err(CatCamError::FailureLibcamera(e));
        }
    }
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
        "--brightness",
        "0.2",
    ];
    libcam_args.map(|e| e.to_string()).to_vec()
}
