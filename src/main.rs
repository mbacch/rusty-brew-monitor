
extern crate linux_embedded_hal as hal;
extern crate max31855;
extern crate influent;

use std::thread;
use std::time::Duration;

use max31855::{Max31855, Measurement as Meas, Units};
use hal::spidev::{self, SpidevOptions};
use hal::{Pin, Spidev};
use hal::sysfs_gpio::Direction;

use influent::create_client;
use influent::client::{Client, Credentials};
use influent::measurement::{Measurement, Value};

fn main() {

    // Configure the influxdb interface
    let credentials = Credentials {
        username: "root",
        password: "root",
        database: "woodu"
    };
    let hosts = vec!["http://ceres:8086"];
    let client = create_client(credentials, hosts);

    // Configure SPI
    let mut spi = Spidev::open("/dev/spidev0.0").unwrap();
    let options = SpidevOptions::new()
        .bits_per_word(8)
        .max_speed_hz(1_000_000)
        .mode(spidev::SPI_MODE_0)
        .build();
    spi.configure(&options).unwrap();

    // Configure Digital I/O Pin to be used as Chip Select
    let cs = Pin::new(4);
    cs.export().unwrap();
    while !cs.is_exported() {}
    cs.set_direction(Direction::Out).unwrap();
    cs.set_value(1).unwrap();

    // Configure the MAX31855 driver
    let mut max31855 = Max31855::new(spi, cs).unwrap();
    let mut data: Meas;

    loop {
        let mut measurement = Measurement::new("max31855_1");
        data = max31855.read_all(Units::Fahrenheit).unwrap();
        measurement.add_field("temperature", Value::Float(data.temperature as f64));
        measurement.add_field("cold ref", Value::Float(data.cold_reference as f64));
        measurement.add_field("fault", Value::Integer(data.fault as i64));
        measurement.add_field("scv", Value::Integer(data.scv as i64));
        measurement.add_field("scg", Value::Integer(data.scg as i64));
        measurement.add_field("oc", Value::Integer(data.oc as i64));
        measurement.add_tag("location", "basement");
        measurement.add_tag("device", "woodu-dev");
        client.write_one(measurement, None).unwrap();
        thread::sleep(Duration::from_millis(10000));
    }
}
