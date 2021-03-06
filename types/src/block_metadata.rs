// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path::{AccessPath, Accesses},
    account_address::AccountAddress,
    account_config,
    account_config::association_address,
    event::{EventHandle, EventKey},
    language_storage::StructTag,
};
use anyhow::Result;
use libra_crypto::HashValue;
use move_core_types::identifier::{IdentStr, Identifier};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

/// Struct that will be persisted on chain to store the information of the current block.
///
/// The flow will look like following:
/// 1. The executor will pass this struct to VM at the end of a block proposal.
/// 2. The VM will use this struct to create a special system transaction that will emit an event
///    represents the information of the current block. This transaction can't
///    be emitted by regular users and is generated by each of the validators on the fly. Such
///    transaction will be executed before all of the user-submitted transactions in the blocks.
/// 3. Once that special resource is modified, the other user transactions can read the consensus
///    info by calling into the read method of that resource, which would thus give users the
///    information such as the current leader.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockMetadata {
    id: HashValue,
    round: u64,
    timestamp_usecs: u64,
    // The vector has to be sorted to ensure consistent result among all nodes
    previous_block_votes: Vec<AccountAddress>,
    proposer: AccountAddress,
}

impl BlockMetadata {
    pub fn new(
        id: HashValue,
        round: u64,
        timestamp_usecs: u64,
        previous_block_votes: Vec<AccountAddress>,
        proposer: AccountAddress,
    ) -> Self {
        Self {
            id,
            round,
            timestamp_usecs,
            previous_block_votes,
            proposer,
        }
    }

    pub fn id(&self) -> HashValue {
        self.id
    }

    pub fn into_inner(self) -> Result<(u64, u64, Vec<AccountAddress>, AccountAddress)> {
        Ok((
            self.round,
            self.timestamp_usecs,
            self.previous_block_votes.clone(),
            self.proposer,
        ))
    }

    pub fn proposer(&self) -> AccountAddress {
        self.proposer
    }

    pub fn voters(&self) -> Vec<AccountAddress> {
        self.previous_block_votes.clone()
    }
}

pub fn new_block_event_key() -> EventKey {
    EventKey::new_from_address(&association_address(), 2)
}

static LIBRA_BLOCK_MODULE_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("LibraBlock").unwrap());
static BLOCK_STRUCT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("BlockMetadata").unwrap());

pub fn libra_block_module_name() -> &'static IdentStr {
    &*LIBRA_BLOCK_MODULE_NAME
}

pub fn block_struct_name() -> &'static IdentStr {
    &*BLOCK_STRUCT_NAME
}

pub fn libra_block_tag() -> StructTag {
    StructTag {
        address: account_config::CORE_CODE_ADDRESS,
        name: block_struct_name().to_owned(),
        module: libra_block_module_name().to_owned(),
        type_params: vec![],
    }
}

/// The access path where the BlockMetadata resource is stored.
pub static LIBRA_BLOCK_RESOURCE_PATH: Lazy<Vec<u8>> =
    Lazy::new(|| AccessPath::resource_access_vec(&libra_block_tag(), &Accesses::empty()));

/// The path to the new block event handle under a LibraBlock::BlockMetadata resource.
pub static NEW_BLOCK_EVENT_PATH: Lazy<Vec<u8>> = Lazy::new(|| {
    let mut path = LIBRA_BLOCK_RESOURCE_PATH.to_vec();
    // it can be anything as long as it's referenced in AccountState::get_event_handle_by_query_path
    path.extend_from_slice(b"/new_block_event/");
    path
});

#[derive(Deserialize, Serialize)]
pub struct LibraBlockResource {
    height: u64,
    new_block_events: EventHandle,
}

impl LibraBlockResource {
    pub fn new_block_events(&self) -> &EventHandle {
        &self.new_block_events
    }
}

#[derive(Deserialize, Serialize)]
pub struct NewBlockEvent {
    round: u64,
    proposer: AccountAddress,
    votes: Vec<AccountAddress>,
    timestamp: u64,
}
