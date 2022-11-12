use rppal::gpio::{Gpio, Level};
use std::{error::Error, process::Command};

const GPIO17: u8 = 17;

fn main() -> Result<(), Box<dyn Error>> {
    let mut pir = Gpio::new()?.get(GPIO17)?.into_input();
    pir.set_interrupt(rppal::gpio::Trigger::Both)?;

    loop {
        match pir.poll_interrupt(true, None) {
            Ok(trigger) => match trigger {
                Some(Level::High) => {
                    Command::new("libcamera-jpeg")
                        .args(["-o", "cat.jpg"])
                        .output()?;
                    println!("!!");
                }
                _ => (),
            },
            _ => break,
        }
    }
    Ok(())
}
