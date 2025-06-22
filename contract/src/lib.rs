use hex::encode;
use near_sdk::{
    env, near, require, store::IterableMap, AccountId, Gas, NearToken, PanicOnDefault, Promise,
    PromiseError,
};
use omni_transaction::signer::types::SignatureResponse;

const CALLBACK_GAS: Gas = Gas::from_tgas(5);

mod ecdsa;
mod external;
mod utils;
use utils::vec_to_fixed;

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct CandidateRNG {
    random_seed: [u8; 32],
}

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Contract {
    pub owner_id: AccountId,
    pub candidate_by_account_id: IterableMap<AccountId, CandidateRNG>,
}

#[near]
impl Contract {
    #[init]
    #[private]
    pub fn init(owner_id: AccountId) -> Self {
        Self {
            owner_id,
            candidate_by_account_id: IterableMap::new(b"b"),
        }
    }

    #[payable]
    pub fn random(&mut self) -> Promise {
        let deposit = env::attached_deposit();
        require!(
            deposit > NearToken::from_millinear(1),
            "Deposit must be greater than 0.001 NEAR"
        );
        env::log_str(&format!("Deposited {}", deposit));

        let account_id = env::predecessor_account_id();
        let random_seed = env::random_seed_array();

        self.candidate_by_account_id
            .insert(account_id.clone(), CandidateRNG { random_seed });

        ecdsa::get_sig(random_seed, account_id.to_string(), 0).then(
            external::rng_contract::ext(env::current_account_id())
                .with_static_gas(CALLBACK_GAS)
                .sign_callback(account_id),
        )
    }

    #[private]
    pub fn sign_callback(
        &mut self,
        #[callback_result] call_result: Result<SignatureResponse, PromiseError>,
        account_id: AccountId,
    ) -> String {
        match call_result {
            Ok(signature_response) => {
                // extract r and s from the signature response
                let affine_point_bytes = hex::decode(signature_response.big_r.affine_point)
                    .expect("failed to decode affine_point to bytes");
                // extract r from the affine_point_bytes
                let r_bytes: [u8; 32] = vec_to_fixed(affine_point_bytes[1..33].to_vec());

                let s_bytes: [u8; 32] = vec_to_fixed(
                    hex::decode(signature_response.s.scalar)
                        .expect("failed to decode scalar to bytes"),
                );

                // update the commit candidate
                let candidate = self
                    .candidate_by_account_id
                    .get(&account_id)
                    .expect("cannot find candidate");

                let rng: String = encode(env::sha256(
                    &[candidate.random_seed, r_bytes, s_bytes].concat(),
                ));

                self.candidate_by_account_id.remove(&account_id);

                rng
            }
            Err(error) => {
                env::log_str(&format!("callback failed with error: {:?}", error));
                "".to_owned()
            }
        }
    }
}
