use jsonrpsee::core::Error;
use jsonrpsee::http_client::HttpClientBuilder;
use jsonrpsee::types::EmptyParams;
use starknet_api::{
    shash, BlockBody, BlockHash, BlockHeader, CallData, ClassHash, DeployTransaction,
    DeployedContract, Fee, StarkHash, StateDiffForward, StorageDiff, StorageEntry,
    TransactionVersion,
};

use super::api::*;
use super::*;
use crate::storage::components::{
    storage_test_utils, BodyStorageWriter, HeaderStorageWriter, StateStorageWriter,
};

#[tokio::test]
async fn test_block_number() -> Result<(), anyhow::Error> {
    let storage_components = storage_test_utils::get_test_storage();
    let storage_reader = storage_components.block_storage_reader;
    let mut storage_writer = storage_components.block_storage_writer;
    let module = JsonRpcServerImpl { storage_reader }.into_rpc();

    // No blocks yet.
    let err = module
        .call::<_, BlockNumber>("starknet_blockNumber", EmptyParams::new())
        .await
        .unwrap_err();
    assert_matches!(err, Error::Call(CallError::Custom(err)) if err == ErrorObject::owned(
        JsonRpcError::NoBlocks as i32,
        JsonRpcError::NoBlocks.to_string(),
        None::<()>,
    ));

    // Add a block and check again.
    storage_writer
        .begin_rw_txn()?
        .append_header(BlockNumber(0), &BlockHeader::default())?
        .commit()?;
    let block_number =
        module.call::<_, BlockNumber>("starknet_blockNumber", EmptyParams::new()).await?;
    assert_eq!(block_number, BlockNumber(0));
    Ok(())
}

#[tokio::test]
async fn test_get_block_w_transaction_hashes() -> Result<(), anyhow::Error> {
    let storage_components = storage_test_utils::get_test_storage();
    let storage_reader = storage_components.block_storage_reader;
    let mut storage_writer = storage_components.block_storage_writer;
    let module = JsonRpcServerImpl { storage_reader }.into_rpc();

    let block_number = BlockNumber(0);
    let block_hash =
        BlockHash(shash!("0x642b629ad8ce233b55798c83bb629a59bf0a0092f67da28d6d66776680d5483"));
    let header = BlockHeader { block_hash, number: block_number, ..BlockHeader::default() };
    storage_writer.begin_rw_txn()?.append_header(header.number, &header)?.commit()?;

    let expected_block =
        Block { header: header.into(), transactions: Transactions::Hashes(vec![]) };

    // Get block by hash.
    let block = module
        .call::<_, Block>("starknet_getBlockWithTxHashes", [BlockId::Hash(block_hash)])
        .await
        .unwrap();
    assert_eq!(block, expected_block);

    // Get block by number.
    let block = module
        .call::<_, Block>("starknet_getBlockWithTxHashes", [BlockId::Number(block_number)])
        .await?;
    assert_eq!(block, expected_block);

    // Ask for the latest block.
    let block = module
        .call::<_, Block>("starknet_getBlockWithTxHashes", [BlockId::Tag(Tag::Latest)])
        .await?;
    assert_eq!(block, expected_block);

    // Ask for an invalid block hash.
    let err = module
        .call::<_, Block>(
            "starknet_getBlockWithTxHashes",
            [BlockId::Hash(BlockHash(shash!(
                "0x642b629ad8ce233b55798c83bb629a59bf0a0092f67da28d6d66776680d5484"
            )))],
        )
        .await
        .unwrap_err();
    assert_matches!(err, Error::Call(CallError::Custom(err)) if err == ErrorObject::owned(
        JsonRpcError::InvalidBlockId as i32,
        JsonRpcError::InvalidBlockId.to_string(),
        None::<()>,
    ));

    // Ask for an invalid block number.
    let err = module
        .call::<_, Block>("starknet_getBlockWithTxHashes", [BlockId::Number(BlockNumber(1))])
        .await
        .unwrap_err();
    assert_matches!(err, Error::Call(CallError::Custom(err)) if err == ErrorObject::owned(
        JsonRpcError::InvalidBlockId as i32,
        JsonRpcError::InvalidBlockId.to_string(),
        None::<()>,
    ));
    Ok(())
}

#[tokio::test]
async fn test_get_block_w_full_transactions() -> Result<(), anyhow::Error> {
    let storage_components = storage_test_utils::get_test_storage();
    let storage_reader = storage_components.block_storage_reader;
    let mut storage_writer = storage_components.block_storage_writer;
    let module = JsonRpcServerImpl { storage_reader }.into_rpc();

    let block_number = BlockNumber(0);
    let block_hash =
        BlockHash(shash!("0x642b629ad8ce233b55798c83bb629a59bf0a0092f67da28d6d66776680d5483"));
    let header = BlockHeader { block_hash, number: block_number, ..BlockHeader::default() };
    storage_writer.begin_rw_txn()?.append_header(header.number, &header)?.commit()?;

    let expected_block = Block { header: header.into(), transactions: Transactions::Full(vec![]) };

    // Get block by hash.
    let block =
        module.call::<_, Block>("starknet_getBlockWithTxs", [BlockId::Hash(block_hash)]).await?;
    assert_eq!(block, expected_block);

    // Get block by number.
    let block = module
        .call::<_, Block>("starknet_getBlockWithTxs", [BlockId::Number(block_number)])
        .await?;
    assert_eq!(block, expected_block);

    // Ask for the latest block.
    let block =
        module.call::<_, Block>("starknet_getBlockWithTxs", [BlockId::Tag(Tag::Latest)]).await?;
    assert_eq!(block, expected_block);

    // Ask for an invalid block hash.
    let err = module
        .call::<_, Block>(
            "starknet_getBlockWithTxs",
            [BlockId::Hash(BlockHash(shash!(
                "0x642b629ad8ce233b55798c83bb629a59bf0a0092f67da28d6d66776680d5484"
            )))],
        )
        .await
        .unwrap_err();
    assert_matches!(err, Error::Call(CallError::Custom(err)) if err == ErrorObject::owned(
        JsonRpcError::InvalidBlockId as i32,
        JsonRpcError::InvalidBlockId.to_string(),
        None::<()>,
    ));

    // Ask for an invalid block number.
    let err = module
        .call::<_, Block>("starknet_getBlockWithTxs", [BlockId::Number(BlockNumber(1))])
        .await
        .unwrap_err();
    assert_matches!(err, Error::Call(CallError::Custom(err)) if err == ErrorObject::owned(
        JsonRpcError::InvalidBlockId as i32,
        JsonRpcError::InvalidBlockId.to_string(),
        None::<()>,
    ));
    Ok(())
}

#[tokio::test]
async fn test_get_storage_at() -> Result<(), anyhow::Error> {
    let storage_components = storage_test_utils::get_test_storage();
    let storage_reader = storage_components.block_storage_reader;
    let mut storage_writer = storage_components.block_storage_writer;
    let module = JsonRpcServerImpl { storage_reader }.into_rpc();

    let block_number = BlockNumber(0);
    let block_hash =
        BlockHash(shash!("0x642b629ad8ce233b55798c83bb629a59bf0a0092f67da28d6d66776680d5483"));
    let header = BlockHeader { number: block_number, block_hash, ..BlockHeader::default() };
    let address = ContractAddress(shash!("0x11"));
    let class_hash = ClassHash(shash!("0x4"));
    let key = StorageKey(shash!("0x1001"));
    let value = shash!("0x200");
    let diff = StateDiffForward {
        deployed_contracts: vec![DeployedContract { address, class_hash }],
        storage_diffs: vec![StorageDiff {
            address,
            diff: vec![StorageEntry { key: key.clone(), value }],
        }],
    };
    storage_writer
        .begin_rw_txn()?
        .append_header(header.number, &header)?
        .append_state_diff(BlockNumber(0), &diff)?
        .commit()?;

    // Get storage by block hash.
    let res = module
        .call::<_, StarkFelt>(
            "starknet_getStorageAt",
            (address, key.clone(), BlockId::Hash(block_hash)),
        )
        .await?;
    assert_eq!(res, value);

    // Get storage by block number.
    let res = module
        .call::<_, StarkFelt>(
            "starknet_getStorageAt",
            (address, key.clone(), BlockId::Number(block_number)),
        )
        .await?;
    assert_eq!(res, value);

    // Ask for an invalid contract.
    let err = module
        .call::<_, StarkFelt>(
            "starknet_getStorageAt",
            (ContractAddress(shash!("0x12")), key.clone(), BlockId::Hash(block_hash)),
        )
        .await
        .unwrap_err();
    assert_matches!(err, Error::Call(CallError::Custom(err)) if err == ErrorObject::owned(
        JsonRpcError::ContractNotFound as i32,
        JsonRpcError::ContractNotFound.to_string(),
        None::<()>,
    ));

    // Ask for an invalid block hash.
    let err = module
        .call::<_, StarkFelt>(
            "starknet_getStorageAt",
            (
                address,
                key.clone(),
                BlockId::Hash(BlockHash(shash!(
                    "0x642b629ad8ce233b55798c83bb629a59bf0a0092f67da28d6d66776680d5484"
                ))),
            ),
        )
        .await
        .unwrap_err();
    assert_matches!(err, Error::Call(CallError::Custom(err)) if err == ErrorObject::owned(
        JsonRpcError::InvalidBlockId as i32,
        JsonRpcError::InvalidBlockId.to_string(),
        None::<()>,
    ));

    // Ask for an invalid block number.
    let err = module
        .call::<_, StarkFelt>(
            "starknet_getStorageAt",
            (address, key.clone(), BlockId::Number(BlockNumber(1))),
        )
        .await
        .unwrap_err();
    assert_matches!(err, Error::Call(CallError::Custom(err)) if err == ErrorObject::owned(
        JsonRpcError::InvalidBlockId as i32,
        JsonRpcError::InvalidBlockId.to_string(),
        None::<()>,
    ));

    Ok(())
}

#[tokio::test]
async fn test_get_transaction_by_hash() -> Result<(), anyhow::Error> {
    let storage_components = storage_test_utils::get_test_storage();
    let storage_reader = storage_components.block_storage_reader;
    let mut storage_writer = storage_components.block_storage_writer;
    let module = JsonRpcServerImpl { storage_reader }.into_rpc();

    let transaction_hash = TransactionHash(StarkHash::from_u64(0));
    let transaction = Transaction::Deploy(DeployTransaction {
        transaction_hash,
        max_fee: Fee(100),
        version: TransactionVersion(shash!("0x1")),
        contract_address: ContractAddress(shash!("0x2")),
        constructor_calldata: CallData(vec![shash!("0x3")]),
    });
    let body = BlockBody { transactions: vec![transaction.clone()] };
    storage_writer.begin_rw_txn()?.append_body(BlockNumber(0), &body)?.commit()?;

    let res = module
        .call::<_, Transaction>("starknet_getTransactionByHash", [transaction_hash])
        .await
        .unwrap();
    assert_eq!(res, transaction.clone());

    // Ask for an invalid transaction.
    let err = module
        .call::<_, Transaction>(
            "starknet_getTransactionByHash",
            [TransactionHash(StarkHash::from_u64(1))],
        )
        .await
        .unwrap_err();
    assert_matches!(err, Error::Call(CallError::Custom(err)) if err == ErrorObject::owned(
        JsonRpcError::InvalidTransactionHash as i32,
        JsonRpcError::InvalidTransactionHash.to_string(),
        None::<()>,
    ));
    Ok(())
}

#[tokio::test]
async fn test_get_transaction_by_block_id_and_index() -> Result<(), anyhow::Error> {
    let storage_components = storage_test_utils::get_test_storage();
    let storage_reader = storage_components.block_storage_reader;
    let mut storage_writer = storage_components.block_storage_writer;
    let module = JsonRpcServerImpl { storage_reader }.into_rpc();

    let transaction_hash = TransactionHash(StarkHash::from_u64(0));
    let transaction = Transaction::Deploy(DeployTransaction {
        transaction_hash,
        max_fee: Fee(100),
        version: TransactionVersion(shash!("0x1")),
        contract_address: ContractAddress(shash!("0x2")),
        constructor_calldata: CallData(vec![shash!("0x3")]),
    });
    let block_number = BlockNumber(0);
    let block_hash =
        BlockHash(shash!("0x642b629ad8ce233b55798c83bb629a59bf0a0092f67da28d6d66776680d5483"));
    let header = BlockHeader { block_hash, number: block_number, ..BlockHeader::default() };
    let body = BlockBody { transactions: vec![transaction.clone()] };
    storage_writer
        .begin_rw_txn()?
        .append_header(header.number, &header)?
        .append_body(header.number, &body)?
        .commit()?;

    // Get transaction by block hash.
    let res = module
        .call::<_, Transaction>(
            "starknet_getTransactionByBlockIdAndIndex",
            (BlockId::Hash(block_hash), 0),
        )
        .await
        .unwrap();
    assert_eq!(res, transaction.clone());

    // Get transaction by block number.
    let res = module
        .call::<_, Transaction>(
            "starknet_getTransactionByBlockIdAndIndex",
            (BlockId::Number(block_number), 0),
        )
        .await
        .unwrap();
    assert_eq!(res, transaction.clone());

    // Ask for an invalid block hash.
    let err = module
        .call::<_, Transaction>(
            "starknet_getTransactionByBlockIdAndIndex",
            (
                BlockId::Hash(BlockHash(shash!(
                    "0x642b629ad8ce233b55798c83bb629a59bf0a0092f67da28d6d66776680d5484"
                ))),
                0,
            ),
        )
        .await
        .unwrap_err();
    assert_matches!(err, Error::Call(CallError::Custom(err)) if err == ErrorObject::owned(
        JsonRpcError::InvalidBlockId as i32,
        JsonRpcError::InvalidBlockId.to_string(),
        None::<()>,
    ));

    // Ask for an invalid block number.
    let err = module
        .call::<_, Transaction>(
            "starknet_getTransactionByBlockIdAndIndex",
            (BlockId::Number(BlockNumber(1)), 0),
        )
        .await
        .unwrap_err();
    assert_matches!(err, Error::Call(CallError::Custom(err)) if err == ErrorObject::owned(
        JsonRpcError::InvalidBlockId as i32,
        JsonRpcError::InvalidBlockId.to_string(),
        None::<()>,
    ));

    // Ask for an invalid transaction index.
    let err = module
        .call::<_, Transaction>(
            "starknet_getTransactionByBlockIdAndIndex",
            (BlockId::Hash(block_hash), 1),
        )
        .await
        .unwrap_err();
    assert_matches!(err, Error::Call(CallError::Custom(err)) if err == ErrorObject::owned(
        JsonRpcError::InvalidTransactionIndex as i32,
        JsonRpcError::InvalidTransactionIndex.to_string(),
        None::<()>,
    ));
    Ok(())
}

#[tokio::test]
async fn test_run_server() -> Result<(), anyhow::Error> {
    let storage_reader = storage_test_utils::get_test_storage().block_storage_reader;
    let (addr, _handle) =
        run_server(GatewayConfig { server_ip: String::from("127.0.0.1:0") }, storage_reader)
            .await?;
    let client = HttpClientBuilder::default().build(format!("http://{:?}", addr))?;
    let err = client.block_number().await.unwrap_err();
    assert_matches!(err, Error::Call(CallError::Custom(err)) if err == ErrorObject::owned(
        JsonRpcError::NoBlocks as i32,
        JsonRpcError::NoBlocks.to_string(),
        None::<()>,
    ));
    Ok(())
}