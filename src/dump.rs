use deku::bitvec::Msb0;
use std::collections::HashMap;
use std::fmt::Debug;
use std::thread::sleep;
use std::time;

use colored::Colorize;

use bitvec::prelude::*;

use deku::prelude::*;
use hex;

use config::Config;

use crate::frisquet;
use crate::frisquet::proto::chaudiere::ChaudierePayload;
use crate::frisquet::proto::connect::ConnectPayload;
use crate::frisquet::proto::satellite::SatellitePayload;
use crate::frisquet::proto::sonde::SondePayload;
use crate::frisquet::proto::{FrisquetMetadata, FrisquetData};
use crate::rf::mqtt::new;
use crate::rf::RFClient;

fn dump(mut cli :Box<dyn RFClient>) {
    let tmp = hex::decode("178008eebc0117a0290015a02f00040800d9010900010000").unwrap();
    let (metadata, x) = frisquet::parse_data_from_str(hex::encode(tmp).as_str()).unwrap();
    print_frisquet(metadata, x);

    let tmp = hex::decode("a1540018a154001830aa962305210003e0ffffffff00e0ffffff7f00e0ffffff7f00e0ffffff7f00e0ffffff7f00e0ffffffff03e0ffffffff").unwrap();
    let s = ConnectPayload::read(deku::bitvec::BitSlice::from_slice(&tmp), 63).unwrap();
    println!("{:?}", s);

    loop {
        let msg = cli.receive().unwrap();
        let (metadata, x) =
            frisquet::parse_data_from_str(hex::encode(msg.clone()).as_str()).unwrap();
        print_frisquet(metadata, x);
    }
}

fn format_day(data: [u8; 6]) -> String {
    let mut out = String::new();
    for d in data {
        out.push_str(
            format!("{:08b}", d)
                .chars()
                .rev()
                .collect::<String>()
                .as_str(),
        )
    }

    out
}

static mut SSTM: String = String::new();
static mut SPM: String = String::new();
static mut CCM: String = String::new();

pub fn print_frisquet(metadata: FrisquetMetadata, data: FrisquetData) {
    print!(
        "TODO 0x{:02x} > 0x{:02x} [{:02x}] {:02x} {:02x}, ",
        metadata.from_addr,
        metadata.to_addr,
        metadata.request_id,
        metadata.req_or_answer,
        metadata.msg_type,
    );
    match data {
        FrisquetData::Satellite(s) => match s {
            SatellitePayload::SatelliteInitEmptyMessage { .. } => {
                let mut bv = bitvec![u8, Msb0;];
                s.write(&mut bv, metadata.length).unwrap();
                println!("{}", hex::encode(&bv.as_raw_slice()).as_str());
                println!("    SatelliteInitEmptyMessage");
            }
            SatellitePayload::SatelliteInitMessage { .. } => {
                let mut bv = bitvec![u8, Msb0;];
                s.write(&mut bv, metadata.length).unwrap();
                println!("{}", hex::encode(&bv.as_raw_slice()).as_str());
                println!("    SatelliteInitMessage");
            }
            SatellitePayload::SatelliteAssocationAnnounceMessage { .. } => {
                let mut bv = bitvec![u8, Msb0;];
                s.write(&mut bv, metadata.length).unwrap();
                println!("{}", hex::encode(&bv.as_raw_slice()).as_str());
                println!("    SatelliteAssocationAnnounceMessage");
            }
            SatellitePayload::SatelliteSetTemperatureMessage {
                static_part,
                unknown1,
                static_part_end,
                unknown2,
                message_static_part,
                temperature,
                consigne,
                boost,
                unknown_mode3,
                unknown3,
                unknown_mode1,
                hors_gel,
                unknown_mode2,
                derogation,
                soleil,
                signature,
            } => {
                let mut bv = bitvec![u8, Msb0;];
                s.write(&mut bv, metadata.length).unwrap();
                let data = hex::encode(&bv.as_raw_slice());

                println!(
                    "{}{}{}{}{}{}",
                    data[0..18].white(),
                    data[18..22].blue(),
                    data[22..26].magenta(),
                    data[26..28].white(),
                    data[28..30].yellow(),
                    data[30..].white()
                );

                unsafe {
                    if !SSTM.is_empty()
                        && (data[0..18] != SSTM[0..18]
                            || data[26..28] != SSTM[26..28]
                            || data[30..] != SSTM[30..])
                    {
                        println!(
                            "{}",
                            format!("                               {}", SSTM)
                                .on_bright_red()
                                .red()
                        );
                    }
                    SSTM = data;
                }

                println!("    SatelliteSetTemperatureMessage");

                // a0290015a02f000408 00d5 00d7 00210000
                // 00 00 00 00 00 00 01 11 11 11 11 11 11 11 11 11 11 11 11 11 11 11 11 10
                // 00000000 00000111 11111111 11111111 11111111 11111110
                // 00 07 ff ff ff fe

                // a1540018a154001830aaa01e05210003e0ffffffff00e0ffffff7f00e0ffffff7f00e0ffffff7f00e0ffffff7f00e0ffffffff03e0ffffffff
                // a1540015a154001830aaa01e05210003e0ffffffff00e0ffffff7f000000000000ffffffffffff00000000000000e0ffffffff03e0ffffffff
                // a1540015a154001830aaa01e06010003e0ffffffff00e0ffffff7f000000000000ffffffffffff00000000000000e0ffffffff03e0ffffffff
                // a1540015a154001830aaa01e060100 03e0ffffffff 00e0ffffff7f 000000000000 ffffffffffff 000000000000 00e0ffffffff 03e0ffffffff
                //                                                          mardi        mercredi     jeudi        vendredi     samedi
                // 2aaaa01e05210003e0ffffffff00e0ffffff7f000000000000ffffffffffff00000000000000e0ffffffff

                println!("\t {}", format!("Temperature: {temperature}").blue());
                println!("\t {}", format!("Consigne: {consigne}").magenta());
                println!("\t {}", format!("Hors gel: {hors_gel}").yellow());
                println!("\t {}", format!("Derogation: {derogation}").yellow());
                println!("\t {}", format!("Soleil: {soleil}").yellow());
                println!("\t {}", format!("Boost: {boost}").yellow());
            }
            SatellitePayload::SatelliteProgMessage {
                unknown1,
                sunday,
                monday,
                tuesday,
                wednesday,
                thursday,
                friday,
                saturday,
            } => {
                let mut bv = bitvec![u8, Msb0;];
                s.write(&mut bv, metadata.length).unwrap();
                let data = hex::encode(&bv.as_raw_slice());

                // a1540015a154001830aaa01e060100 03e0ffffffff 00e0ffffff7f 000000000000 ffffffffffff 000000000000 00e0ffffffff 03e0ffffffff

                println!(
                    "{}{}{}{}{}{}{}{}",
                    data[0..30].white(),
                    data[30..42].blue(),
                    data[42..54].magenta(),
                    data[54..66].yellow(),
                    data[66..78].green(),
                    data[78..90].cyan(),
                    data[90..102].purple(),
                    data[102..].red()
                );

                unsafe {
                    if !SPM.is_empty() && (data[0..30] != SPM[0..30]) {
                        println!(
                            "{}",
                            format!("                               {}", SPM)
                                .on_bright_red()
                                .red()
                        );
                    }
                    SPM = data;
                }

                println!("    SatelliteProgMessage");

                // 0-1    fcffffffffff -> 11111100
                // 0-2    f0ffffffffff -> 11110000
                // 0-3    c0ffffffffff -> 11000000
                // 0-3.30 80ffffffffff -> 10000000
                // 0-4.30 00feffffffff -> 11111110

                // a1540015 a1540018 30aaa01e 0601 00 03e0ffffffff00e0ffffff7f00e0ffffff7f00e0ffffff7f00e0ffffff7f00e0ffffffff03e0ffffffff
                // a1540015 a1540018 30aaa01e 0521 00 03e0ffffffff00e0ffffff7f00e0ffffff7f00e0ffffff7f00e0ffffff7f00e0ffffffff03e0ffffffff
                // a1540015 a1540018 30aaa01e 0601 00 03e0ffffffff00e0ffffff7f00e0ffffff7f00e0ffffff7f00e0ffffff7f00e0ffffffff03e0ffffffff soleil
                // a1540015 a1540018 30aaa01e 0521 00 03e0ffffffff00e0ffffff7f00e0ffffff7f00e0ffffff7f00e0ffffff7f00e0ffffffff03e0ffffffff auto
                // a1540015 a1540018 30aaa01e 0601 00 03e0ffffffff00e0ffffff7f00e0ffffff7f00e0ffffff7f00e0ffffff7f00e0ffffffff03e0ffffffff soleil
                // a1540015 a1540018 30aaa01e 0700 00 03e0ffffffff00e0ffffff7f00e0ffffff7f00e0ffffff7f00e0ffffff7f00e0ffffffff03e0ffffffff nuit
                // a1540015 a1540018 30aaa01e 0810 00 03e0ffffffff00e0ffffff7f00e0ffffff7f00e0ffffff7f00e0ffffff7f00e0ffffffff03e0ffffffff hors gel
                // a1540015 a1540018 30aaa01e 0601 00 03e0ffffffff00e0ffffff7f00e0ffffff7f00e0ffffff7f00e0ffffff7f00e0ffffffff03e0ffffffff soleil
                // a1540015 a1540018 30aaa01e 0810 00 03e0ffffffff00e0ffffff7f00e0ffffff7f00e0ffffff7f00e0ffffff7f00e0ffffffff03e0ffffffff hors gel
                // a1540015 a1540018 30aaa01e 0700 00 03e0ffffffff00e0ffffff7f00e0ffffff7f00e0ffffff7f00e0ffffff7f00e0ffffffff03e0ffffffff nuit
                // a1540015 a1540018 30aaa01e 0521 00 03e0ffffffff00e0ffffff7f00e0ffffff7f00e0ffffff7f00e0ffffff7f00e0ffffffff03e0ffffffff auto
                // a1540015 a1540018 30aaa01e 0521 00 03e0ffffffff00e0ffffff7f00e0ffffff7f00e0ffffff7f00e0ffffff7f00e0ffffffff03e0ffffffff soleil

                println!("\t {}", format!("Sunday:    {}", format_day(sunday)).blue());
                println!(
                    "\t {}",
                    format!("Monday:    {}", format_day(monday)).magenta()
                );
                println!(
                    "\t {}",
                    format!("Tuesday:   {}", format_day(tuesday)).yellow()
                );
                println!(
                    "\t {}",
                    format!("Wednesday: {}", format_day(wednesday)).green()
                );
                println!(
                    "\t {}",
                    format!("Thursday:  {}", format_day(thursday)).cyan()
                );
                println!(
                    "\t {}",
                    format!("Friday:    {}", format_day(friday)).purple()
                );
                println!(
                    "\t {}",
                    format!("Saturday:  {}", format_day(saturday)).red()
                );
            }
            SatellitePayload::SatelliteUnknowMessage { .. } => {
                let mut bv = bitvec![u8, Msb0;];
                s.write(&mut bv, metadata.length).unwrap();
                println!("{}", hex::encode(&bv.as_raw_slice()).as_str());
                println!("    SatelliteUnknowMessage");
            }
        },
        FrisquetData::Connect(c) => match c {
            ConnectPayload::ConnectCmdMessage {
                unknown_start,
                temp_confort,
                temp_reduit,
                temp_hors_gel,
                boost,
                confort,
                derogation,
                unknow_mode,
                unknown_mode2,
                unknown_mode3,
                mode,
                unknown_data,
                sunday,
                monday,
                tuesday,
                wednesday,
                thursday,
                friday,
                saturday,
            } => {
                let mut bv = bitvec![u8, Msb0;];
                c.write(&mut bv, metadata.length).unwrap();
                let data = hex::encode(&bv.as_raw_slice());

                // a1540018a154001830b98c1e056100
                // a1540018a154001830af8c1e052100

                println!(
                    "{}{}{}{}{}{}{}{}{}{}{}{}{}{}",
                    data[0..18].white(),
                    data[18..20].red(),
                    data[20..22].purple(),
                    data[22..24].blue(),
                    data[24..26].yellow(),
                    data[26..28].cyan(),
                    data[28..30].white(),
                    data[30..42].green(),
                    data[42..54].yellow(),
                    data[54..66].green(),
                    data[66..78].yellow(),
                    data[78..90].green(),
                    data[90..102].yellow(),
                    data[102..].green(),
                );

                unsafe {
                    if !CCM.is_empty() && (data[0..18] != CCM[0..18] || data[24..30] != CCM[24..30]) {
                        print!("                               ");
                        if data[0..18] != CCM[0..18] {
                            print!("{}",format!("{}", &CCM[0..18]).on_bright_red().red());
                        } else {
                            print!("{}", format!("{}", &CCM[0..18]));
                        }
                        print!("{}", &CCM[18..24]);
                        print!("{}", format!("{}", &CCM[24..30]).on_bright_red().red());
                        println!("{}", &CCM[30..]);
                    }
                    CCM = data;
                }

                // derogation, permanent, vacances
                // 0621 confort zone1
                // 0521 auto zone1
                // 0721 reduit zone1
                // 0500 auto zone 1
                // 0821 hors gel zone 1
                // 0511 auto

                // 05 selecteur auto - 06 selecteur confort - 07 selecteur reduit - 08 selecteur hors gel

                // 23 derog reduit
                // 20 annul derog reduit
                // 26 derog confort
                // 25 annul derog confort
                // 61 boost
                // 63 derog reduit + boost

                // 23 derog confort
                // 20 no derog reduit
                // 21 no derig confort
                // 22 derog reduit



                // entete
                // a1 540018a154001830 zone 1
                // f9 540018a154001830 ????
                // a1 540018a154001830

                let m = match mode {
                    0x05 => "auto",
                    0x06 => "confort",
                    0x07 => "reduit",
                    0x08 => "hors gel",
                    _ => "unknown",
                };

                println!("    ConnectCmdMessage");
                println!("\t {}", format!("Confort T: {}", (temp_confort + 50)).red());
                println!("\t {}", format!("Reduit T: {}", (temp_reduit + 50)).purple());
                println!("\t {}", format!("Hors gel T: {}", (temp_hors_gel + 50)).blue());
                println!("\t {}", format!("Mode: {m}").yellow());
                println!("\t {}", format!("Boost: {boost}").cyan());
                println!("\t {}", format!("Derogation: {derogation}").cyan());
                println!("\t {}", format!("Confort: {confort}").cyan());
                println!("\t {}", format!("Sunday:    {}", format_day(sunday)).green());
                println!(
                    "\t {}",
                    format!("Monday:    {}", format_day(monday)).yellow()
                );
                println!(
                    "\t {}",
                    format!("Tuesday:   {}", format_day(tuesday)).green()
                );
                println!(
                    "\t {}",
                    format!("Wednesday: {}", format_day(wednesday)).yellow()
                );
                println!(
                    "\t {}",
                    format!("Thursday:  {}", format_day(thursday)).green()
                );
                println!(
                    "\t {}",
                    format!("Friday:    {}", format_day(friday)).yellow()
                );
                println!(
                    "\t {}",
                    format!("Saturday:  {}", format_day(saturday)).green()
                );
            }
            ConnectPayload::ConnectUnknownMessage { .. } => {
                let mut bv = bitvec![u8, Msb0;];
                c.write(&mut bv, metadata.length).unwrap();
                println!("{}", hex::encode(&bv.as_raw_slice()).as_str());
                println!("    ConnectUnknownMessage");
            }
        },
        FrisquetData::Chaudiere(c) => {
            let mut bv = bitvec![u8, Msb0;];
            c.write(&mut bv, metadata.length).unwrap();
            println!("{}", hex::encode(&bv.as_raw_slice()).as_str());
            println!("    Chaudiere");
        }
        FrisquetData::Sonde(s) => {
            let mut bv = bitvec![u8, Msb0;];
            s.write(&mut bv, metadata.length).unwrap();
            println!("{}", hex::encode(&bv.as_raw_slice()).as_str());
            println!("    Sonde");
        }
        FrisquetData::Unknown(u) => {
            let mut bv = bitvec![u8, Msb0;];
            u.write(&mut bv, metadata.length).unwrap();
            println!("{}", hex::encode(&bv.as_raw_slice()).as_str());
            println!("    Unknown");
        }
    }
}
