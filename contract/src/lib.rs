use hex::encode;
use near_sdk::{
    env, near, require, store::IterableMap, AccountId, Gas, NearToken, PanicOnDefault, Promise,
    PromiseError,
};
use omni_transaction::signer::types::SignatureResponse;

const CALLBACK_GAS: Gas = Gas::from_tgas(50);
const ZERO_PAYLOAD: [u8; 32] = [0; 32];

mod ecdsa;
mod external;
mod utils;
use utils::vec_to_fixed;

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct CandidateRNG {
    commit_hash: String,
    random_seed_1: [u8; 32],
    random_seed_2: [u8; 32],
    r_bytes: [u8; 32],
    s_bytes: [u8; 32],
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

    pub fn commit(&mut self, commit_hash: String) -> Promise {
        let account_id = env::predecessor_account_id();
        let random_seed_1 = env::random_seed_array();

        self.candidate_by_account_id.insert(
            account_id.clone(),
            CandidateRNG {
                random_seed_1,
                random_seed_2: ZERO_PAYLOAD,
                r_bytes: ZERO_PAYLOAD,
                s_bytes: ZERO_PAYLOAD,
                commit_hash: commit_hash.clone(),
            },
        );

        ecdsa::get_sig(random_seed_1, commit_hash, 0).then(
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
    ) -> bool {
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

                let new_candidate = CandidateRNG {
                    commit_hash: candidate.commit_hash.clone(),
                    random_seed_1: candidate.random_seed_1,
                    random_seed_2: env::random_seed_array(),
                    r_bytes,
                    s_bytes,
                };

                self.candidate_by_account_id
                    .insert(account_id, new_candidate);

                // ready for reveal
                true
            }
            Err(error) => {
                env::log_str(&format!("callback failed with error: {:?}", error));
                false
            }
        }
    }

    pub fn reveal(&mut self, commit_value: String) -> String {
        let account_id = env::predecessor_account_id();
        let candidate = self
            .candidate_by_account_id
            .get(&account_id)
            .expect("cannot find candidate");

        require!(candidate.random_seed_2 != ZERO_PAYLOAD, "not ready");

        require!(
            encode(env::sha256(commit_value.as_bytes())) == candidate.commit_hash,
            "commit value does not hash to candidate commit_hash"
        );

        let rng: String = encode(env::sha256(
            &[
                candidate.random_seed_1,
                candidate.random_seed_2,
                candidate.r_bytes,
                candidate.s_bytes,
                vec_to_fixed(format!("{:0>32}", commit_value).as_bytes().to_vec()),
            ]
            .concat(),
        ));

        self.candidate_by_account_id.remove(&account_id);

        rng
    }
}
