use deku::prelude::*;

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(ctx = "length: u8", id = "length")]
pub enum ConnectPayload {
    #[deku(id = "63")]
    ConnectCmdMessage {
        unknown_start: [u8; 9], // adresse?
        temp_confort: u8, // start at 5°C - 0 is 50
        temp_reduit: u8, // start at 5°C - 0 is 50
        temp_hors_gel: u8, // start at 5°C - 0 is 50
        mode: u8, // 05 auto - 06 confort - 07 reduit - 08 hors gel
        #[deku(bits = "1")]
        unknow_mode: bool,
        #[deku(bits = "1")]
        boost: bool,
        #[deku(bits = "2")]
        unknown_mode2: u8,
        #[deku(bits = "2")]
        unknown_mode3: u8,
        #[deku(bits = "1")]
        derogation: bool,
        #[deku(bits = "1")]
        confort: bool,
        unknown_data: u8,
        sunday: [u8; 6],
        monday: [u8; 6],
        tuesday: [u8; 6],
        wednesday: [u8; 6],
        thursday: [u8; 6],
        friday: [u8; 6],
        saturday: [u8; 6],
    },

    #[deku(id_pat = "_")]
    ConnectUnknownMessage {
        #[deku(count = "length - 6")]
        data: Vec<u8>,
    },
}
