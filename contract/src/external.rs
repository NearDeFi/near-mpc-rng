use near_sdk::{ext_contract, AccountId};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SignRequest {
    pub payload: [u8; 32],
    pub path: String,
    pub key_version: u32,
}
#[allow(dead_code)]
#[ext_contract(mpc_contract)]
trait MPCContract {
    fn sign(&self, request: SignRequest);
}
#[allow(dead_code)]
#[ext_contract(rng_contract)]
trait RNGContract {
    fn sign_callback(&self, account_id: AccountId);
}
