use std::{
    io,
    io::{BufReader, BufWriter, Read, Write},
    time::{SystemTime, UNIX_EPOCH},
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use indy_api_types::{
    domain::wallet::{KeyDerivationMethod, Record},
    errors::prelude::*,
};

use indy_utils::crypto::{
    chacha20poly1305_ietf,
    hash::{hash, HASHBYTES},
    pwhash_argon2i13,
};

use serde::{Deserialize, Serialize};

use crate::{encryption::KeyDerivationData, Wallet, WalletRecord};
use std::sync::Arc;

const CHUNK_SIZE: usize = 1024;

#[derive(Debug, Serialize, Deserialize)]
pub enum EncryptionMethod {
    // **ChaCha20-Poly1305-IETF** cypher in blocks per chunk_size bytes
    ChaCha20Poly1305IETF {
        // pwhash_argon2i13::Salt as bytes. Random salt used for deriving of key from passphrase
        salt: Vec<u8>,
        // chacha20poly1305_ietf::Nonce as bytes. Random start nonce. We increment nonce for each chunk to be sure in export file consistency
        nonce: Vec<u8>,
        // size of encrypted chunk
        chunk_size: usize,
    },
    // **ChaCha20-Poly1305-IETF interactive key derivation** cypher in blocks per chunk_size bytes
    ChaCha20Poly1305IETFInteractive {
        // pwhash_argon2i13::Salt as bytes. Random salt used for deriving of key from passphrase
        salt: Vec<u8>,
        // chacha20poly1305_ietf::Nonce as bytes. Random start nonce. We increment nonce for each chunk to be sure in export file consistency
        nonce: Vec<u8>,
        // size of encrypted chunk
        chunk_size: usize,
    },
    // **ChaCha20-Poly1305-IETF raw key** cypher in blocks per chunk_size bytes
    ChaCha20Poly1305IETFRaw {
        // chacha20poly1305_ietf::Nonce as bytes. Random start nonce. We increment nonce for each chunk to be sure in export file consistency
        nonce: Vec<u8>,
        // size of encrypted chunk
        chunk_size: usize,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Header {
    // Method of encryption for encrypted stream
    pub encryption_method: EncryptionMethod,
    // Export time in seconds from UNIX Epoch
    pub time: u64,
    // Version of header
    pub version: u32,
}

// Note that we use externally tagged enum serialization and header will be represented as:
//
// {
//   "encryption_method": {
//     "ChaCha20Poly1305IETF": {
//       "salt": ..,
//       "nonce": ..,
//       "chunk_size": ..,
//     },
//   },
//   "time": ..,
//   "version": ..,
// }

pub(super) async fn export_continue(
    wallet: Arc<Wallet>,
    writer: &mut (dyn Write + Send + Sync),
    version: u32,
    key: chacha20poly1305_ietf::Key,
    key_data: &KeyDerivationData,
) -> IndyResult<()> {
    let nonce = chacha20poly1305_ietf::gen_nonce();
    let chunk_size = CHUNK_SIZE;

    let encryption_method = match key_data {
        KeyDerivationData::Argon2iMod(_, salt) => EncryptionMethod::ChaCha20Poly1305IETF {
            salt: salt[..].to_vec(),
            nonce: nonce[..].to_vec(),
            chunk_size,
        },
        KeyDerivationData::Argon2iInt(_, salt) => {
            EncryptionMethod::ChaCha20Poly1305IETFInteractive {
                salt: salt[..].to_vec(),
                nonce: nonce[..].to_vec(),
                chunk_size,
            }
        }
        KeyDerivationData::Raw(_) => EncryptionMethod::ChaCha20Poly1305IETFRaw {
            nonce: nonce[..].to_vec(),
            chunk_size,
        },
    };

    let header = Header {
        encryption_method,
        time: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        version,
    };

    let header = rmp_serde::to_vec(&header).to_indy(
        IndyErrorKind::InvalidState,
        "Can't serialize wallet export file header",
    )?;

    // Write plain
    let mut writer = BufWriter::new(writer);
    writer.write_u32::<LittleEndian>(header.len() as u32)?;
    writer.write_all(&header)?;

    // Write ecnrypted
    let mut writer = chacha20poly1305_ietf::Writer::new(writer, key, nonce, chunk_size);

    writer.write_all(&hash(&header)?)?;

    let mut records = wallet.get_all().await?;

    while let Some(WalletRecord {
        type_,
        id,
        value,
        tags,
    }) = records.next().await?
    {
        let record = Record {
            type_: type_.ok_or_else(|| {
                err_msg(
                    IndyErrorKind::InvalidState,
                    "No type fetched for exported record",
                )
            })?,
            id,
            value: value.ok_or_else(|| {
                err_msg(
                    IndyErrorKind::InvalidState,
                    "No value fetched for exported record",
                )
            })?,
            tags: tags.ok_or_else(|| {
                err_msg(
                    IndyErrorKind::InvalidState,
                    "No tags fetched for exported record",
                )
            })?,
        };

        let record = rmp_serde::to_vec(&record)
            .to_indy(IndyErrorKind::InvalidState, "Can't serialize record")?;

        writer.write_u32::<LittleEndian>(record.len() as u32)?;
        writer.write_all(&record)?;
    }

    writer.write_u32::<LittleEndian>(0)?; // END message
    writer.flush()?;
    Ok(())
}

pub(super) fn preparse_file_to_import<T>(
    reader: T,
    passphrase: &str,
) -> IndyResult<(
    BufReader<T>,
    KeyDerivationData,
    chacha20poly1305_ietf::Nonce,
    usize,
    Vec<u8>,
)>
where
    T: Read,
{
    // Reads plain
    let mut reader = BufReader::new(reader);

    let header_len = reader.read_u32::<LittleEndian>().map_err(_map_io_err)? as usize;

    if header_len == 0 {
        return Err(err_msg(
            IndyErrorKind::InvalidStructure,
            "Invalid header length",
        ));
    }

    let mut header_bytes = vec![0u8; header_len];
    reader.read_exact(&mut header_bytes).map_err(_map_io_err)?;

    let header: Header = rmp_serde::from_slice(&header_bytes)
        .to_indy(IndyErrorKind::InvalidStructure, "Header is malformed json")?;

    if header.version != 0 {
        return Err(err_msg(
            IndyErrorKind::InvalidStructure,
            "Unsupported version",
        ));
    }

    let key_derivation_method = match header.encryption_method {
        EncryptionMethod::ChaCha20Poly1305IETF { .. } => KeyDerivationMethod::ARGON2I_MOD,
        EncryptionMethod::ChaCha20Poly1305IETFInteractive { .. } => {
            KeyDerivationMethod::ARGON2I_INT
        }
        EncryptionMethod::ChaCha20Poly1305IETFRaw { .. } => KeyDerivationMethod::RAW,
    };

    let (import_key_derivation_data, nonce, chunk_size) = match header.encryption_method {
        EncryptionMethod::ChaCha20Poly1305IETF {
            salt,
            nonce,
            chunk_size,
        }
        | EncryptionMethod::ChaCha20Poly1305IETFInteractive {
            salt,
            nonce,
            chunk_size,
        } => {
            let salt = pwhash_argon2i13::Salt::from_slice(&salt)
                .to_indy(IndyErrorKind::InvalidStructure, "Invalid salt")?;

            let nonce = chacha20poly1305_ietf::Nonce::from_slice(&nonce)
                .to_indy(IndyErrorKind::InvalidStructure, "Invalid nonce")?;

            let passphrase = passphrase.to_owned();

            let key_data = match key_derivation_method {
                KeyDerivationMethod::ARGON2I_INT => KeyDerivationData::Argon2iInt(passphrase, salt),
                KeyDerivationMethod::ARGON2I_MOD => KeyDerivationData::Argon2iMod(passphrase, salt),
                _ => unimplemented!("FIXME"), //FIXME
            };

            (key_data, nonce, chunk_size)
        }
        EncryptionMethod::ChaCha20Poly1305IETFRaw { nonce, chunk_size } => {
            let nonce = chacha20poly1305_ietf::Nonce::from_slice(&nonce)
                .to_indy(IndyErrorKind::InvalidStructure, "Invalid nonce")?;

            let key_data = KeyDerivationData::Raw(passphrase.to_owned());

            (key_data, nonce, chunk_size)
        }
    };

    Ok((
        reader,
        import_key_derivation_data,
        nonce,
        chunk_size,
        header_bytes,
    ))
}

pub(super) async fn finish_import<T>(
    wallet: &Wallet,
    reader: BufReader<T>,
    key: chacha20poly1305_ietf::Key,
    nonce: chacha20poly1305_ietf::Nonce,
    chunk_size: usize,
    header_bytes: Vec<u8>,
) -> IndyResult<()>
where
    T: Read,
{
    // Reads encrypted
    let mut reader = chacha20poly1305_ietf::Reader::new(reader, key, nonce, chunk_size);

    let mut header_hash = vec![0u8; HASHBYTES];
    reader.read_exact(&mut header_hash).map_err(_map_io_err)?;

    if hash(&header_bytes)? != header_hash {
        return Err(err_msg(
            IndyErrorKind::InvalidStructure,
            "Invalid header hash",
        ));
    }

    loop {
        let record_len = reader.read_u32::<LittleEndian>().map_err(_map_io_err)? as usize;

        if record_len == 0 {
            break;
        }

        let mut record = vec![0u8; record_len];
        reader.read_exact(&mut record).map_err(_map_io_err)?;

        let record: Record = rmp_serde::from_slice(&record).to_indy(
            IndyErrorKind::InvalidStructure,
            "Record is malformed msgpack",
        )?;

        wallet
            .add(&record.type_, &record.id, &record.value, &record.tags)
            .await?;
    }

    Ok(())
}

fn _map_io_err(e: io::Error) -> IndyError {
    match e {
        ref e
            if e.kind() == io::ErrorKind::UnexpectedEof
                || e.kind() == io::ErrorKind::InvalidData =>
        {
            err_msg(
                IndyErrorKind::InvalidStructure,
                "Invalid export file format",
            )
        }
        e => e.to_indy(IndyErrorKind::IOError, "Can't read export file"),
    }
}
