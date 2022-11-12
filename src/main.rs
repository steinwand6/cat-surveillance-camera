use rppal::gpio::{Gpio, InputPin, Level, OutputPin};
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};

const GPIO17: u8 = 17;

fn main() -> Result<(), Box<dyn Error>> {
    let mut pir = Gpio::new()?.get(GPIO17)?.into_input();
    pir.set_interrupt(rppal::gpio::Trigger::Both)?;

    loop {
        match pir.poll_interrupt(true, None) {
            Ok(trigger) => match trigger {
                Some(rppal::gpio::Level::High) => {
                    println!("!!");
                }
                Some(rppal::gpio::Level::Low) => {
                    println!("...");
                }
                None => (),
            },
            _ => break,
        }
    }
    Ok(())
}
