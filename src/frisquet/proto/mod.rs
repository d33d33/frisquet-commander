use deku::prelude::*;

use crate::frisquet::proto::chaudiere::ChaudierePayload;
use crate::frisquet::proto::connect::ConnectPayload;
use crate::frisquet::proto::satellite::SatellitePayload;
use crate::frisquet::proto::sonde::SondePayload;

pub mod chaudiere;
pub mod common;
pub mod connect;
pub mod satellite;
pub mod sonde;

#[derive(Debug, PartialEq)]
pub enum FrisquetData {
    Satellite(SatellitePayload),
    Chaudiere(ChaudierePayload),
    Sonde(SondePayload),
    Connect(ConnectPayload),
    Unknown(UnknownPayload),
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct FrisquetMetadata {
    pub length: u8,
    pub to_addr: u8,
    pub from_addr: u8,
    pub request_id: u16,
    pub req_or_answer: u8,
    pub msg_type: u8,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(ctx = "length: u8", id = "length")]
pub enum UnknownPayload {
    #[deku(id_pat = "_")]
    UnknownMessage {
        #[deku(count = "length - 6")]
        data: Vec<u8>,
    },
}
