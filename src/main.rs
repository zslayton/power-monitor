extern crate embedded_hal;
extern crate linux_embedded_hal;
#[macro_use(block)]
extern crate nb;
extern crate ads1x1x;
extern crate reqwest;
extern crate serde;

use serde::{Serialize, Deserialize};

use ads1x1x::{channel, Ads1x1x, DataRate16Bit, SlaveAddr, FullScaleRange};
use embedded_hal::adc::OneShot;
use linux_embedded_hal::I2cdev;
use chrono::prelude::*;

use std::{time, thread};

const SAMPLES_PER_CHANNEL: usize = 125;

const MINIMUM_READING: i16 = -8_000;
const MAXIMUM_READING: i16 = 8_000;

const API_ENDPOINT: &str = "http://170e94.nyc/power";

#[derive(Debug, Clone, Copy, Serialize)]
struct ChannelPowerReading {
    min: i16,
    max: i16,
    timestamp: DateTime<Local>
}

impl ChannelPowerReading {
    fn new() -> ChannelPowerReading {
        ChannelPowerReading {
            min: MAXIMUM_READING,
            max: MINIMUM_READING,
            timestamp: Local::now()
        }
    }

    fn add_sample(&mut self, value: i16) {
        if value < self.min {
            self.min = value;
        } else if value > self.max {
            self.max = value;
        }
    }

    fn difference(&self) -> i16 {
      self.max - self.min
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
struct PowerReading {
    a0: ChannelPowerReading,
    a1: ChannelPowerReading,
    a2: ChannelPowerReading,
    a3: ChannelPowerReading,
}

impl PowerReading {
    fn new(
        a0: ChannelPowerReading,
        a1: ChannelPowerReading,
        a2: ChannelPowerReading,
        a3: ChannelPowerReading,
    ) -> PowerReading {
        PowerReading { a0, a1, a2, a3 }
    }
}

fn main() {
    let dev = I2cdev::new("/dev/i2c-1").unwrap();
    let address = SlaveAddr::default();
    let mut adc = Ads1x1x::new_ads1115(dev, address);
    adc.set_data_rate(DataRate16Bit::Sps250).unwrap();
    adc.set_full_scale_range(FullScaleRange::Within0_256V).unwrap();

    let http_client = reqwest::Client::new();

    loop {
    // The channels are each a different data type, making it difficult to reuse code without
    // macros.
    let mut a0_reading = ChannelPowerReading::new();
    for _i in 0..SAMPLES_PER_CHANNEL {
        let value = block!(adc.read(&mut channel::SingleA0));
        if value.is_ok() {
            a0_reading.add_sample(value.unwrap());
        } else {
            eprintln!("Could not read from a0: {:#?}", value);
        }
    }
    println!("Channel A0: {:#?} => {}", a0_reading, a0_reading.difference()); 

    let mut a1_reading = ChannelPowerReading::new();
    for _i in 0..SAMPLES_PER_CHANNEL {
        let value = block!(adc.read(&mut channel::SingleA1));
        if value.is_ok() {
            a1_reading.add_sample(value.unwrap());
        } else {
            eprintln!("Could not read from a1: {:#?}", value);
        }
    }
    println!("Channel A1: {:#?} => {}", a1_reading, a1_reading.difference());

    let mut a2_reading = ChannelPowerReading::new();
    for _i in 0..SAMPLES_PER_CHANNEL {
        let value = block!(adc.read(&mut channel::SingleA2));
        if value.is_ok() {
            a2_reading.add_sample(value.unwrap());
        } else {
            eprintln!("Could not read from a2: {:#?}", value);
        }
    }
    println!("Channel A2: {:#?} => {}", a2_reading, a2_reading.difference());

    let mut a3_reading = ChannelPowerReading::new();
    for _i in 0..SAMPLES_PER_CHANNEL {
        let value = block!(adc.read(&mut channel::SingleA3));
        if value.is_ok() {
            a3_reading.add_sample(value.unwrap());
        } else {
            eprintln!("Could not read from a3: {:#?}", value);
        }
    }
    println!("Channel A3: {:#?} => {}", a3_reading, a3_reading.difference());

    let reading = PowerReading::new(
        a0_reading,
        a1_reading,
        a2_reading,
        a3_reading,
    );

    //println!("Reading: {:#?}", reading);

    let result = http_client.post(API_ENDPOINT).json(&reading).send();

    println!("POST: {:#?}", result);
    thread::sleep(time::Duration::from_millis(2_000));
    }
    // get I2C device back
    let _dev = adc.destroy_ads1115();
}
