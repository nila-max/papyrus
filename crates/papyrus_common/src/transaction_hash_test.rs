use serde::{Deserialize, Serialize};
use starknet_api::core::ChainId;
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::transaction::Transaction;
use test_utils::read_json_file;

use super::{ascii_as_felt, get_transaction_hash, validate_transaction_hash};

#[test]
fn test_ascii_as_felt() {
    let sn_main_id = ChainId("SN_MAIN".to_owned());
    let sn_main_felt = ascii_as_felt(sn_main_id.0.as_str()).unwrap();
    // This is the result of the Python snippet from the Chain-Id documentation.
    let expected_sn_main = StarkFelt::from(23448594291968334_u128);
    assert_eq!(sn_main_felt, expected_sn_main);
}

#[derive(Deserialize, Serialize)]
struct TransactionWithHash {
    transaction: Transaction,
    transaction_hash: StarkHash,
}

#[test]
fn test_transaction_hash() {
    // The details were taken from Starknet Goerli. You can found the transactions by hash in:
    // https://alpha4.starknet.io/feeder_gateway/get_transaction?transactionHash=<transaction_hash>
    let chain_id = ChainId("SN_GOERLI".to_owned());
    let transactions_with_hash: Vec<TransactionWithHash> =
        serde_json::from_value(read_json_file("transaction_hash.json")).unwrap();

    for transaction_with_hash in transactions_with_hash {
        assert!(
            validate_transaction_hash(
                &transaction_with_hash.transaction,
                &chain_id,
                transaction_with_hash.transaction_hash
            )
            .unwrap()
        );
        let actual_transaction_hash =
            get_transaction_hash(&transaction_with_hash.transaction, &chain_id).unwrap();
        assert_eq!(actual_transaction_hash, transaction_with_hash.transaction_hash);
    }
}

#[test]
fn test_deprecated_transaction_hash() {
    // The details were taken from Starknet Goerli. You can found the transactions by hash in:
    // https://alpha4.starknet.io/feeder_gateway/get_transaction?transactionHash=<transaction_hash>
    let chain_id = ChainId("SN_GOERLI".to_owned());
    let transactions_with_hash: Vec<TransactionWithHash> =
        serde_json::from_value(read_json_file("deprecated_transaction_hash.json")).unwrap();

    for transaction_with_hash in transactions_with_hash {
        assert!(
            validate_transaction_hash(
                &transaction_with_hash.transaction,
                &chain_id,
                transaction_with_hash.transaction_hash
            )
            .unwrap()
        );
    }
}