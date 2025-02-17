use serde::de::DeserializeOwned;
use serde::Deserialize;

use validator::Validate;

use hex::decode;
use std::fs::File;
use std::io::Read;

use std::error::Error;

pub const BLS_SIGNATURE_SIZE: usize = 96;
pub const BLS_PUBKEY_SIZE: usize = 48;
pub const BLS_SECRET_SIZE: usize = 32;
pub const BLS_ID_SIZE: usize = 32;
pub const GEN_ID_SIZE: usize = 16;
pub const SHA256_SIZE: usize = 32;

pub type BLSPubkey = [u8; BLS_PUBKEY_SIZE];
pub type BLSSecret = [u8; BLS_SECRET_SIZE];
pub type BLSId = [u8; BLS_ID_SIZE];
pub type BLSSignature = [u8; BLS_SIGNATURE_SIZE];
pub type SHA256 = [u8; SHA256_SIZE];

macro_rules! decode {
    ($str:ident) => { hex::decode($str).unwrap().try_into().unwrap() };
}

#[derive(Debug, Deserialize)]
pub struct DvtVerificationVector {
    #[serde(rename(deserialize = "base_pubkeys"))]
    pub pubkeys: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct DvtGenerateSettings {
    pub n: u8,
    pub k: u8,
    pub gen_id: String,
}

pub type DvtVerificationHashes = Vec<String>;

#[derive(Debug, Deserialize, Validate)]
pub struct DvtInitialCommitment {
    pub hash: String,
    pub settings: DvtGenerateSettings,
    #[serde(rename(deserialize = "vvector"))]
    pub verification_vector: DvtVerificationVector,
}

#[derive(Debug, Deserialize)]
pub struct DvtCommitment {
    pub hash: String,
    pub pubkey: String,
    pub signature: String,
}

#[derive(Debug, Deserialize)]
pub struct DvtShareExchangeCommitment {
    pub initial_commitment_hash: String,
    #[serde(rename(deserialize = "ssecret"))]
    pub shared_secret: DvtExchangedSecret,
    pub commitment: DvtCommitment,
}

#[derive(Debug, Deserialize)]
pub struct DvtExchangedSecret {
    #[serde(rename(deserialize = "dst_share_id"))]
    pub dst_id: String,
    #[serde(rename(deserialize = "src_share_id"))]
    pub src_id: String,
    #[serde(rename(deserialize = "shared_secret"))]
    pub secret: String,
    #[serde(rename(deserialize = "dst_base_hash"))]
    pub dst_base_hash: String,
}

#[derive(Debug, Deserialize)]
pub struct DvtShare {
    pub id: String,
    pub pubkey: String,
}

#[derive(Debug, Deserialize)]
pub struct DvtBlsSharedData {
    #[serde(rename(deserialize = "base_hashes"))]
    verification_hashes: DvtVerificationHashes,
    initial_commitment: DvtInitialCommitment,
    seeds_exchange_commitment: DvtShareExchangeCommitment,
}

#[derive(Debug, Deserialize)]
pub struct DvtGeneration {
    #[serde(rename(deserialize = "base_pubkeys"))]
    verification_vector: Vec<String>,
    base_hash: String,
    partial_pubkey: String,
    message_cleartext: String,
    message_signature: String,
}

#[derive(Debug, Deserialize)]
pub struct DvtFinalizationData {
    settings: DvtGenerateSettings,
    generations: Vec<DvtGeneration>,
    aggregate_pubkey: String,
}

#[derive(Debug)]
pub struct AbiVerificationVector {
    pub pubkeys: Vec<BLSPubkey>,
}

#[derive(Debug)]
pub struct AbiGenerateSettings {
    pub n: u8,
    pub k: u8,
    pub gen_id: [u8; GEN_ID_SIZE],
}

impl AbiGenerateSettings {
    fn new((n, k, gen_id): (u8, u8, String)) -> AbiGenerateSettings {
        AbiGenerateSettings { n, k, gen_id: decode!(gen_id) }
    }
}

pub type AbiVerificationHashes = Vec<SHA256>;

#[derive(Debug)]
pub struct AbiInitialCommitment {
    pub hash: SHA256,
    pub settings: AbiGenerateSettings,
    pub verification_vector: AbiVerificationVector,
}

#[derive(Debug)]
pub struct AbiExchangedSecret {
    pub src_id: BLSId,
    pub dst_id: BLSId,
    pub dst_base_hash: SHA256,
    pub secret: BLSSecret,
}

#[derive(Debug)]
pub struct AbiCommitment {
    pub hash: SHA256,
    pub pubkey: BLSPubkey,
    pub signature: BLSSignature,
}

#[derive(Debug)]
pub struct AbiSeedExchangeCommitment {
    pub initial_commitment_hash: SHA256,
    pub shared_secret: AbiExchangedSecret,
    pub commitment: AbiCommitment,
}

#[derive(Debug)]
pub struct AbiBlsSharedData {
    pub verification_hashes: AbiVerificationHashes,
    pub initial_commitment: AbiInitialCommitment,
    pub seeds_exchange_commitment: AbiSeedExchangeCommitment,
}

#[derive(Debug, Clone)]
pub struct AbiGeneration {
    pub verification_vector: Vec<BLSPubkey>,
    pub base_hash: SHA256,
    pub partial_pubkey: BLSPubkey,
    pub message_cleartext: Vec<u8>,
    pub message_signature: BLSSignature,
}

impl AbiGeneration {
    fn new(
        (vector, base_hash, partial_pubkey, cleartext, signature): (Vec<String>, String, String, String, String),
    ) -> AbiGeneration {
        AbiGeneration {
            verification_vector: vector.into_iter().map(|x| decode!(x)).collect(),
            base_hash: decode!(base_hash),
            partial_pubkey: decode!(partial_pubkey),
            message_cleartext: cleartext.into(),
            message_signature: decode!(signature),
        }
    }
}

#[derive(Debug)]
pub struct AbiFinalizationData {
    pub settings: AbiGenerateSettings,
    pub generations: Vec<AbiGeneration>,
    pub aggregate_pubkey: BLSPubkey,
}

impl AbiFinalizationData {
    pub fn new(
        generate_settings: (u8, u8, String),
        generations: Vec<(Vec<String>, String, String, String, String)>,
        aggregate_pubkey: String,
    ) -> AbiFinalizationData {
        AbiFinalizationData {
            settings: AbiGenerateSettings::new(generate_settings),
            generations: generations.into_iter().map(|x| AbiGeneration::new(x)).collect(),
            aggregate_pubkey: decode!(aggregate_pubkey),
        }
    }
}
