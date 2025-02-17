#![no_main]

use bls_utils;
use dvt_abi::AbiFinalizationData;

#[guests_macro::proving_entrypoint]
pub fn main(
    generate_settings: (u8, u8, String),
    generations: Vec<(Vec<String>, String, String, String, String)>,
    aggregate_pubkey: String,
) {
    let data = AbiFinalizationData::new(generate_settings, generations, aggregate_pubkey);
    let ok =
        bls_utils::verify_generations(&data.generations, &data.settings, &data.aggregate_pubkey);
    if ok.is_err() {
        panic!("{:?}", ok.unwrap_err().to_string());
    }
}
