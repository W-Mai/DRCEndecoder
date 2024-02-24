use std::io::{BufReader, Read};
use chrono::prelude::*;

#[allow(unused)]
struct DRCHeader {
    magic: u32,

    date_time: DateTime<Local>,

    unknown: [u8; 0x2E],
}

#[allow(unused)]
impl DRCHeader {
    pub fn new() -> Self {
        Self {
            magic: u32::from_le_bytes([0xD0, 0x07, 0x00, 0x00]),
            date_time: Local::now(),
            unknown: [0u8; 0x2E],
        }
    }

    pub fn decode<T: Read>(data: &mut BufReader<T>) -> Option<Self> {
        let mut magic_bytes = [0u8; 4];
        let mut time_stamp_string_bytes = [0u8; 0x2E];
        let mut unknown_bytes = [0u8; 0x2E];

        match data.read_exact(&mut magic_bytes) {
            Ok(_) => {}
            Err(_) => { return None; }
        }
        data.read_exact(&mut time_stamp_string_bytes).unwrap();
        data.read_exact(&mut unknown_bytes).unwrap();

        let magic: u32 = u32::from_le_bytes(magic_bytes);
        let time_stamp_string = time_stamp_string_bytes.chunks(2).map_while(
            |x| {
                let value = u16::from_le_bytes([x[0], x[1]]);
                if value == 0 {
                    None
                } else {
                    Some(value as u8 as char)
                }
            }).collect::<String>();
        println!("{:?}", time_stamp_string);
        let time_stamp = NaiveDateTime::parse_from_str(&*time_stamp_string, "%Y-%m-%d %H:%M:%S %f").unwrap();

        let date_time = time_stamp.and_local_timezone(Local).unwrap();
        Some(Self {
            magic,
            date_time,
            unknown: unknown_bytes,
        })
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(12);
        data.extend_from_slice(&self.magic.to_le_bytes());
        data.extend_from_slice(&self.date_time.timestamp().to_le_bytes());
        data
    }
}

#[allow(unused)]
struct DRCData {
    header: DRCHeader,
    data: Vec<i16>,
}

#[allow(unused)]
impl DRCData {
    pub fn new() -> Self {
        Self {
            header: DRCHeader::new(),
            data: Vec::new(),
        }
    }

    pub fn decode<T: Read>(mut data: &mut BufReader<T>) -> Vec<Self> {
        let mut result = Vec::new();

        while let Some(header) = DRCHeader::decode(&mut data) {
            let mut data_bytes = [0u8; 0x9c50];
            data.read_exact(&mut data_bytes).unwrap();

            let data = data_bytes.chunks(2).map(|x| {
                let value = i16::from_le_bytes([x[1], x[0]]);
                value
            }).collect::<Vec<i16>>();

            let mut new_data = Vec::new();

            new_data.extend_from_slice(&data[4..4000 / 2]);

            println!("len: {}", data.len());
            result.push(Self {
                header,
                data: new_data,
            });
        }
        result
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut data = self.header.encode();
        data.extend_from_slice(&self.data.iter().flat_map(|&x| x.to_le_bytes()).collect::<Vec<u8>>());
        data
    }
}

fn main() {
    let file = std::fs::File::open("./Data_No_1.drc").unwrap();

    let mut reader = BufReader::new(file);
    let data = DRCData::decode(&mut reader);

    // convert to *.wav file
    let wav_file = std::fs::File::create("./Data_No_1.wav").unwrap();
    let mut wav_writer = hound::WavWriter::new(wav_file, hound::WavSpec {
        channels: 1,
        sample_rate: 20000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    }).unwrap();

    for wave_data in data {
        for sample in wave_data.data.iter() {
            wav_writer.write_sample(*sample).unwrap();
        }
    }
}
