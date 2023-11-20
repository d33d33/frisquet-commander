use deku::prelude::*;

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(ctx = "length: u8", id = "length", endian = "big")]
pub enum SatellitePayload {
    #[deku(id = "17")]
    SatelliteInitMessage {
        static_part: [u8; 7],
        message_part: [u8; 3],
    },
    #[deku(id = "8")]
    SatelliteInitEmptyMessage {
        #[deku(count = "length - 6")]
        data: Vec<u8>,
    },

    #[deku(id = "10")]
    SatelliteAssocationAnnounceMessage { unknown: u8, version: [u8; 3] },

    #[deku(id = "23")]
    SatelliteSetTemperatureMessage {
        static_part: [u8; 3],
        unknown1: u8,
        static_part_end: [u8; 3],
        unknown2: u8,
        message_static_part: u8,
        temperature: i16,
        consigne: i16,
        unknown3: u8,
        #[deku(bits = "1")]
        unknown_mode1: u8,
        #[deku(bits = "1")]
        boost: bool,
        #[deku(bits = "1")]
        unknown_mode2: u8,
        #[deku(bits = "1")]
        hors_gel: bool,
        #[deku(bits = "2")]
        unknown_mode3: u8,
        #[deku(bits = "1")]
        derogation: bool,
        #[deku(bits = "1")]
        soleil: bool, // confort
        signature: [u8; 2],
    },
    #[deku(id = "63")]
    SatelliteProgMessage {
        unknown1: [u8; 15],
        sunday: [u8; 6],
        monday: [u8; 6],
        tuesday: [u8; 6],
        wednesday: [u8; 6],
        thursday: [u8; 6],
        friday: [u8; 6],
        saturday: [u8; 6],
    },
    #[deku(id_pat = "_")]
    SatelliteUnknowMessage {
        #[deku(count = "length - 6")]
        data: Vec<u8>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::frisquet::proto::common::unhexify;
    use crate::frisquet::proto::FrisquetMetadata;

    #[test]
    fn test_satellite_set_temperature_message() {
        let payload = unhexify("17800819E40117A0290015A02F00040800B200AA002400C6");
        let (rest, metadata) = FrisquetMetadata::from_bytes((payload.as_ref(), 0)).unwrap();

        let (_rest, mmm) =
            SatellitePayload::read(deku::bitvec::BitSlice::from_slice(rest.0), metadata.length)
                .unwrap();
        // assert_eq!(payload.length, 23);
        // assert_eq!(payload.from_addr, 8);
        // assert_eq!(payload.to_addr, 128);
        // assert_eq!(payload.request_id, 6628);
        // assert_eq!(payload.req_or_answer, 1);
        // assert_eq!(payload.msg_type, 23);
        println!("Parsed input: {metadata:?}");
        println!("Parsed input: {mmm:?}");
    }
}
