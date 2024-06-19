use base64::{
    alphabet,
    engine::{DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig},
};

/// A default [GeneralPurposeConfig] configuration with a [decode_padding_mode] of
/// [DecodePaddingMode::Indifferent], and
const LENIENT_PAD: GeneralPurposeConfig = GeneralPurposeConfig::new()
    .with_encode_padding(false)
    .with_decode_padding_mode(DecodePaddingMode::Indifferent);

/// A [GeneralPurpose] engine using the [alphabet::URL_SAFE] base64 alphabet and
/// [DecodePaddingMode::Indifferent] config to decode both padded and unpadded.
/// It will encode with NO padding.
/// In alignment with RFC 0017 https://github.com/hyperledger/aries-rfcs/tree/main/concepts/0017-attachments#base64url
pub const URL_SAFE_LENIENT: GeneralPurpose = GeneralPurpose::new(&alphabet::URL_SAFE, LENIENT_PAD);
