use base64::{
    alphabet,
    engine::{DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig},
};

/// A default [GeneralPurposeConfig] configuration with a [decode_padding_mode] of
/// [DecodePaddingMode::Indifferent]
pub const LENIENT_PAD: GeneralPurposeConfig =
    GeneralPurposeConfig::new().with_decode_padding_mode(DecodePaddingMode::Indifferent);

/// A [GeneralPurpose] engine using the [alphabet::URL_SAFE] base64 alphabet and
/// [DecodePaddingMode::Indifferent] config to decode both padded and unpadded.
pub const URL_SAFE_LENIENT: GeneralPurpose = GeneralPurpose::new(&alphabet::URL_SAFE, LENIENT_PAD);
