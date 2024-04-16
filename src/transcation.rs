use serde::Deserialize;
// pub mod transaction;
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Transaction {
    txid: String,
    // version: u8,
    // locktime: u8,
    // size: u32,
    // weight: u32,
    // fee: u64,
    vin: Vec<Vin>,
    vout: Vec<Vout>,
    pub status: Status,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Vin {
    prevout: Prevout,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Vout {
    value: u64,
    scriptpubkey_address: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Prevout {
    value: u64,
    scriptpubkey_address: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Status {
    // confirmed: bool,
    pub block_height: u128,
    // block_hash: String,
    // block_time: u64,
}

pub fn detect_okx(t: Transaction) -> bool {
    if t.vin.len() < 2 || t.vout.len() < 2 {
        // 确保越界问题
        return false;
    }

    // let mut address = vec![];
    let buyer = &t.vin[0].prevout.scriptpubkey_address;
    let seller = &t.vin[1].prevout.scriptpubkey_address;
    let value = &t.vin[1].prevout.value;

    if let Some(vout0) = t.vout.get(0) {
        if let Some(vout1) = t.vout.get(1) {
            // println!("v0 out address ----{}", vout0.scriptpubkey_address);
            // println!("v0 out amount  ----{}", vout0.value);
            // println!("v1 out address ----{}", vout1.scriptpubkey_address);
            if vout0.scriptpubkey_address == *buyer
                && vout0.value == *value
                && vout1.scriptpubkey_address == *seller
            {
                return true;
            }
        }
    }
    false
}
