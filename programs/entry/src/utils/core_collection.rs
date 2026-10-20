use anchor_lang::prelude::*;

use crate::errors::EntryError;

pub fn validate_collection(data: &[u8], event_authority: Pubkey) -> Result<()> {
    let collection =
        mpl_core::Collection::deserialize(data).map_err(|_| error!(EntryError::InvalidAssetId))?;
    require_keys_eq!(
        collection.base.update_authority,
        event_authority,
        EntryError::InvalidAssetId
    );

    let freeze = collection
        .plugin_list
        .permanent_freeze_delegate
        .as_ref()
        .ok_or(EntryError::InvalidAssetId)?;
    let transfer = collection
        .plugin_list
        .permanent_transfer_delegate
        .as_ref()
        .ok_or(EntryError::InvalidAssetId)?;
    require!(
        collection.plugin_list.bubblegum_v2.is_some()
            && !freeze.permanent_freeze_delegate.frozen
            && freeze.base.authority.authority_type
                == mpl_core::advanced_types::AuthorityType::UpdateAuthority
            && transfer.base.authority.authority_type
                == mpl_core::advanced_types::AuthorityType::UpdateAuthority,
        EntryError::InvalidAssetId
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use borsh::BorshSerialize;
    use mpl_core::{
        accounts::{BaseCollectionV1, PluginHeaderV1, PluginRegistryV1},
        types::{
            BubblegumV2, Key, PermanentFreezeDelegate, PermanentTransferDelegate, Plugin,
            PluginAuthority, PluginType, RegistryRecord,
        },
    };

    fn collection_data(
        update_authority: Pubkey,
        freeze_authority: PluginAuthority,
        freeze_enabled: bool,
        include_transfer: bool,
        include_bubblegum: bool,
    ) -> Vec<u8> {
        let base = BaseCollectionV1 {
            key: Key::CollectionV1,
            update_authority,
            name: "ENTRY collection".to_owned(),
            uri: "https://example.invalid".to_owned(),
            num_minted: 0,
            current_size: 0,
        };
        let base_bytes = base.try_to_vec().unwrap();
        let mut plugin_bytes = Vec::new();
        let mut registry = Vec::new();
        let mut push_plugin = |plugin_type, plugin: Plugin, authority: PluginAuthority| {
            let offset = base_bytes.len() + PluginHeaderV1::LEN + plugin_bytes.len();
            let bytes = plugin.try_to_vec().unwrap();
            plugin_bytes.extend_from_slice(&bytes);
            registry.push(RegistryRecord {
                plugin_type,
                authority,
                offset: offset as u64,
            });
        };

        push_plugin(
            PluginType::PermanentFreezeDelegate,
            Plugin::PermanentFreezeDelegate(PermanentFreezeDelegate {
                frozen: freeze_enabled,
            }),
            freeze_authority,
        );
        if include_transfer {
            push_plugin(
                PluginType::PermanentTransferDelegate,
                Plugin::PermanentTransferDelegate(PermanentTransferDelegate {}),
                PluginAuthority::UpdateAuthority,
            );
        }
        if include_bubblegum {
            push_plugin(
                PluginType::BubblegumV2,
                Plugin::BubblegumV2(BubblegumV2 {}),
                PluginAuthority::UpdateAuthority,
            );
        }

        let registry_offset = base_bytes.len() + PluginHeaderV1::LEN + plugin_bytes.len();
        let header = PluginHeaderV1 {
            key: Key::PluginHeaderV1,
            plugin_registry_offset: registry_offset as u64,
        };
        let plugin_registry = PluginRegistryV1 {
            key: Key::PluginRegistryV1,
            registry,
            external_registry: Vec::new(),
        };
        let mut data = base_bytes;
        data.extend_from_slice(&header.try_to_vec().unwrap());
        data.extend_from_slice(&plugin_bytes);
        data.extend_from_slice(&plugin_registry.try_to_vec().unwrap());
        data
    }

    #[test]
    fn accepts_event_controlled_bubblegum_collection_and_permanent_delegates() {
        let event = Pubkey::new_unique();
        let data = collection_data(event, PluginAuthority::UpdateAuthority, false, true, true);
        validate_collection(&data, event).unwrap();
    }

    #[test]
    fn rejects_collection_without_required_plugin_or_with_wrong_authority() {
        let event = Pubkey::new_unique();
        let without_transfer =
            collection_data(event, PluginAuthority::UpdateAuthority, false, false, true);
        assert!(validate_collection(&without_transfer, event).is_err());

        let wrong_authority = collection_data(
            event,
            PluginAuthority::Address {
                address: Pubkey::new_unique(),
            },
            false,
            true,
            true,
        );
        assert!(validate_collection(&wrong_authority, event).is_err());
    }

    #[test]
    fn rejects_frozen_collection_plugin_and_wrong_update_authority() {
        let event = Pubkey::new_unique();
        let frozen = collection_data(event, PluginAuthority::UpdateAuthority, true, true, true);
        assert!(validate_collection(&frozen, event).is_err());

        let wrong_update_authority = collection_data(
            Pubkey::new_unique(),
            PluginAuthority::UpdateAuthority,
            false,
            true,
            true,
        );
        assert!(validate_collection(&wrong_update_authority, event).is_err());
    }
}
