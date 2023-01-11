use std::sync::Arc;

use indy_api_types::errors::prelude::*;
use log::trace;

use ursa::{
    cl::{RevocationTailsAccessor, RevocationTailsGenerator, Tail},
    errors::prelude::{UrsaCryptoError, UrsaCryptoErrorKind},
};

use crate::utils::crypto::base58::{FromBase58, ToBase58};

use crate::{
    domain::anoncreds::revocation_registry_definition::RevocationRegistryDefinitionV1,
    services::BlobStorageService,
};

const TAILS_BLOB_TAG_SZ: u8 = 2;
const TAIL_SIZE: usize = Tail::BYTES_REPR_SIZE;

pub(crate) struct SDKTailsAccessor {
    tails_service: Arc<BlobStorageService>,
    tails_reader_handle: i32,
}

impl SDKTailsAccessor {
    pub(crate) async fn new(
        tails_service: Arc<BlobStorageService>,
        tails_reader_handle: i32,
        rev_reg_def: &RevocationRegistryDefinitionV1,
    ) -> IndyResult<SDKTailsAccessor> {
        let tails_hash =
            rev_reg_def.value.tails_hash.from_base58().map_err(|_| {
                err_msg(IndyErrorKind::InvalidState, "Invalid base58 for Tails hash")
            })?;

        let tails_reader_handle = tails_service
            .open_blob(
                tails_reader_handle,
                &rev_reg_def.value.tails_location,
                tails_hash.as_slice(),
            )
            .await?;

        Ok(SDKTailsAccessor {
            tails_service,
            tails_reader_handle,
        })
    }
}

impl Drop for SDKTailsAccessor {
    fn drop(&mut self) {
        #[allow(unused_must_use)] //TODO
        {
            self.tails_service.close(self.tails_reader_handle)
                .map_err(map_err_err!());
        }
    }
}

impl RevocationTailsAccessor for SDKTailsAccessor {
    fn access_tail(
        &self,
        tail_id: u32,
        accessor: &mut dyn FnMut(&Tail),
    ) -> Result<(), UrsaCryptoError> {
        trace!("access_tail > tail_id {:?}", tail_id);

        // FIXME: Potentially it is significant lock
        let tail_bytes = self.tails_service.read(
            self.tails_reader_handle,
            TAIL_SIZE,
            TAIL_SIZE * tail_id as usize + TAILS_BLOB_TAG_SZ as usize,
        )
        .map_err(|_| {
            UrsaCryptoError::from_msg(
                UrsaCryptoErrorKind::InvalidState,
                "Can't read tail bytes from blob storage",
            )
        })?; // FIXME: IO error should be returned

        let tail = Tail::from_bytes(tail_bytes.as_slice())?;
        accessor(&tail);

        let res = Ok(());
        trace!("access_tail < {:?}", res);
        res
    }
}

pub(crate) async fn store_tails_from_generator(
    service: Arc<BlobStorageService>,
    writer_handle: i32,
    rtg: &mut RevocationTailsGenerator,
) -> IndyResult<(String, String)> {
    trace!(
        "store_tails_from_generator > writer_handle {:?}",
        writer_handle
    );

    let blob_handle = service.create_blob(writer_handle).await?;

    let version = vec![0u8, TAILS_BLOB_TAG_SZ];
    service.append(blob_handle, version.as_slice()).await?;

    while let Some(tail) = rtg.try_next()? {
        let tail_bytes = tail.to_bytes()?;
        service.append(blob_handle, tail_bytes.as_slice()).await?;
    }

    let tails_info = service
        .finalize(blob_handle)
        .await
        .map(|(location, hash)| (location, hash.to_base58()))?;

    let res = Ok(tails_info);
    trace!("store_tails_from_generator < {:?}", res);
    res
}
