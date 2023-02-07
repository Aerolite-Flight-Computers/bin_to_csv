#![feature(read_buf)]
#![feature(cursor_remaining)]

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, Cursor};
use std::process;
use byteorder::{LittleEndian, ReadBytesExt};
use cgmath::{Euler, Quaternion};

fn checksum_calculator(data: &[u8]) -> u16 {
    let curr_crc: u16 = 0x0000;
    let mut sum1 = curr_crc as u8;
    let mut sum2 = (curr_crc >> 8) as u8;

    for index in 0..data.len() {
        sum1 = ((sum1 as u16 + data[index as usize] as u16) % 255) as u8;
        sum2 = ((sum2 as u16 + sum1 as u16) % 255) as u8;
    }

    (sum1 as u16) << 8| (sum2 as u16)
}

fn run(reader: &mut BufReader<File>) -> Result<(), Box<dyn Error>> {
    let buf = reader.fill_buf()?;
    let length = buf.len() / 78;
    println!("Records Found: {}",length);

    let file_path = "output.csv";
    let mut wtr = csv::Writer::from_path(file_path)?;
    let mut rdr = Cursor::new(buf);
    wtr.write_record(&[
        "Time Stamp",
        "Altitude",
        "Position X",
        "Position Y",
        "Position Z",
        "Velocity X",
        "Velocity Y",
        "Velocity Z",
        "Acceleration X",
        "Acceleration Y",
        "Acceleration Z",
        "Direction X",
        "Direction Y",
        "Direction Z",
        "Angular Rate X",
        "Angular Rate Y",
        "Angular Rate Z",
        "Device State"
    ]).expect("TODO: panic message");

    // let l = File::open("data18.bin")?;
    // let mut reader2 = BufReader::new(l);

    for i in 0..length {
        let index = i*78;
        rdr.set_position(index as u64);
        let mut buffer = &buf[index..(index)+78];

        let ccs = checksum_calculator(&buffer[0..76]);
        println!("Calculated Checksum: {ccs}");

        let byte = buffer[76];
        let byte2 = buffer[77];
        let rcs = (byte as u16) | (byte2 as u16) << 8;

        println!("Received Checksum: {rcs}");
        if rcs == ccs {
            println!("Checksums Approved! Exporting data to CSV...");

            let datetime = rdr.read_u32::<LittleEndian>().unwrap();
            let altitude = rdr.read_f32::<LittleEndian>().unwrap();
            let position = (
                rdr.read_f32::<LittleEndian>().unwrap(),
                rdr.read_f32::<LittleEndian>().unwrap(),
                rdr.read_f32::<LittleEndian>().unwrap()
            );
            let velocity = (
                rdr.read_f32::<LittleEndian>().unwrap(),
                rdr.read_f32::<LittleEndian>().unwrap(),
                rdr.read_f32::<LittleEndian>().unwrap()
            );
            let acceleration = (
                rdr.read_f32::<LittleEndian>().unwrap(),
                rdr.read_f32::<LittleEndian>().unwrap(),
                rdr.read_f32::<LittleEndian>().unwrap()
            );
            rdr.read_f32::<LittleEndian>().unwrap();
            rdr.read_f32::<LittleEndian>().unwrap();
            rdr.read_f32::<LittleEndian>().unwrap();
            rdr.read_f32::<LittleEndian>().unwrap();
            let direction_tuple = (0f32, 0f32, 0f32, 1f32);
            let direction_quaternion = Quaternion::try_from(direction_tuple).unwrap();
            let direction_euler = Euler::from(direction_quaternion);
            let direction = (direction_euler.x.0, direction_euler.y.0, direction_euler.z.0);
            let angular_rate = (
                rdr.read_f32::<LittleEndian>().unwrap(),
                rdr.read_f32::<LittleEndian>().unwrap(),
                rdr.read_f32::<LittleEndian>().unwrap()
            );

            let device_state = rdr.read_u32::<LittleEndian>().unwrap();

            wtr.write_record(&[datetime.to_string(),
                altitude.to_string(),
                position.0.to_string(),
                position.1.to_string(),
                position.2.to_string(),
                velocity.0.to_string(),
                velocity.1.to_string(),
                velocity.2.to_string(),
                acceleration.0.to_string(),
                acceleration.1.to_string(),
                acceleration.2.to_string(),
                direction.0.to_string(),
                direction.1.to_string(),
                direction.2.to_string(),
                angular_rate.0.to_string(),
                angular_rate.1.to_string(),
                angular_rate.2.to_string(),
                device_state.to_string()
            ]).expect("TODO: panic message");
        } else {
            println!("Checksum Failed")
        }
        wtr.flush()?;
    }
    println!("Finished Exporting to CSV: \"{}\"!", file_path);

    Ok(())
}

fn main() -> std::io::Result<()>{

    let f = File::open("data18.bin").expect("No file found");
    let mut reader = BufReader::new(f);
    if let Err(err) = run(&mut reader) {
        println!("{}", err);
        process::exit(1);
    }
    Ok(())
}
