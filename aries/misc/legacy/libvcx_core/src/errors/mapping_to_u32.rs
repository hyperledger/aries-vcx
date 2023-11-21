use crate::errors::error::{LibvcxError, LibvcxErrorKind};

impl From<LibvcxError> for u32 {
    fn from(error: LibvcxError) -> u32 {
        error.kind().into()
    }
}

static UNKNOWN_ERROR_CODE: u32 = 1001;

lazy_static! {
    static ref ERROR_KINDS: Vec<(LibvcxErrorKind, u32)> = vec![
        (LibvcxErrorKind::InvalidConnectionHandle, 1003),
        (LibvcxErrorKind::InvalidConfiguration, 1004),
        (LibvcxErrorKind::NotReady, 1005),
        (LibvcxErrorKind::InvalidOption, 1007),
        (LibvcxErrorKind::InvalidDid, 1008),
        (LibvcxErrorKind::InvalidVerkey, 1009),
        (LibvcxErrorKind::PostMessageFailed, 1010),
        (LibvcxErrorKind::InvalidNonce, 1011),
        (LibvcxErrorKind::InvalidUrl, 1013),
        (LibvcxErrorKind::NotBase58, 1014),
        (LibvcxErrorKind::InvalidIssuerCredentialHandle, 1015),
        (LibvcxErrorKind::InvalidJson, 1016),
        (LibvcxErrorKind::InvalidProofHandle, 1017),
        (LibvcxErrorKind::InvalidMessagePack, 1019),
        (LibvcxErrorKind::InvalidMessages, 1020),
        (LibvcxErrorKind::InvalidAttributesStructure, 1021),
        (LibvcxErrorKind::EncodeError, 1022),
        (LibvcxErrorKind::InvalidProof, 1023),
        (LibvcxErrorKind::InvalidGenesisTxnPath, 1024),
        (LibvcxErrorKind::PoolLedgerConnect, 1025),
        (LibvcxErrorKind::CreatePoolConfig, 1026),
        (LibvcxErrorKind::InvalidProofCredentialData, 1027),
        (LibvcxErrorKind::NoPoolOpen, 1030),
        (LibvcxErrorKind::InvalidSchema, 1031),
        (LibvcxErrorKind::InvalidHttpResponse, 1033),
        (LibvcxErrorKind::CreateCredDef, 1034),
        (LibvcxErrorKind::UnknownLibndyError, 1035),
        (LibvcxErrorKind::InvalidCredDefHandle, 1037),
        (LibvcxErrorKind::CredDefAlreadyCreated, 1039),
        (LibvcxErrorKind::InvalidSchemaSeqNo, 1040),
        (LibvcxErrorKind::CreateSchema, 1041),
        (LibvcxErrorKind::InvalidSchemaHandle, 1042),
        (LibvcxErrorKind::AlreadyInitialized, 1044),
        (LibvcxErrorKind::InvalidHandle, 1048),
        (LibvcxErrorKind::InvalidDisclosedProofHandle, 1049),
        (LibvcxErrorKind::SerializationError, 1050),
        (LibvcxErrorKind::DuplicationWallet, 1051),
        (LibvcxErrorKind::WalletAlreadyOpen, 1052),
        (LibvcxErrorKind::InvalidCredentialHandle, 1053),
        (LibvcxErrorKind::InvalidWalletHandle, 1057),
        (LibvcxErrorKind::WalletCreate, 1058),
        (LibvcxErrorKind::CreateConnection, 1061),
        (LibvcxErrorKind::InvalidLibindyParam, 1067),
        (LibvcxErrorKind::IOError, 1074),
        (LibvcxErrorKind::WalletAccessFailed, 1075),
        (LibvcxErrorKind::WalletRecordNotFound, 1073),
        (LibvcxErrorKind::DuplicationWalletRecord, 1072),
        (LibvcxErrorKind::WalletNotFound, 1079),
        (LibvcxErrorKind::LibindyInvalidStructure, 1080),
        (LibvcxErrorKind::InvalidState, 1081),
        (LibvcxErrorKind::InvalidLedgerResponse, 1082),
        (LibvcxErrorKind::DuplicationDid, 1083),
        (LibvcxErrorKind::DuplicationMasterSecret, 1084),
        (LibvcxErrorKind::InvalidProofRequest, 1086),
        (LibvcxErrorKind::DuplicationSchema, 1088),
        (LibvcxErrorKind::LoggingError, 1090),
        (LibvcxErrorKind::InvalidRevocationDetails, 1091),
        (LibvcxErrorKind::InvalidRevocationEntry, 1092),
        (LibvcxErrorKind::InvalidRevocationTimestamp, 1093),
        (LibvcxErrorKind::UnknownSchemaRejection, 1094),
        (LibvcxErrorKind::CreateRevRegDef, 1095),
        (LibvcxErrorKind::ActionNotSupported, 1103),
        (LibvcxErrorKind::NoAgentInformation, 1106),
        (LibvcxErrorKind::RevRegDefNotFound, 1107),
        (LibvcxErrorKind::RevDeltaNotFound, 1108),
        (LibvcxErrorKind::PoisonedLock, 1109),
        (LibvcxErrorKind::ObjectAccessError, 1110),
        (LibvcxErrorKind::InvalidMessageFormat, 1111),
        (LibvcxErrorKind::CreateOutOfBand, 1112),
        (LibvcxErrorKind::RevDeltaFailedToClear, 1114),
        (LibvcxErrorKind::InvalidInput, 1115),
        (LibvcxErrorKind::ParsingError, 1116),
        (LibvcxErrorKind::UnimplementedFeature, 1117),
        (LibvcxErrorKind::LedgerItemNotFound, 1118),
        (LibvcxErrorKind::UnknownError, UNKNOWN_ERROR_CODE),
    ];
}

// note: iterating few tens of values in case of error should not have much impact, but it surely
// can be optimized. The implementation is optimizing on easy of adding new errors and minimizing
// duplication of error code pairs.
impl From<LibvcxErrorKind> for u32 {
    fn from(kind: LibvcxErrorKind) -> u32 {
        match kind {
            LibvcxErrorKind::LibndyError(code) => code,
            _ => match ERROR_KINDS
                .iter()
                .find(|(mapping_kind, _)| *mapping_kind == kind)
            {
                Some((_, mapping_code)) => *mapping_code,
                None => UNKNOWN_ERROR_CODE,
            },
        }
    }
}

impl From<u32> for LibvcxErrorKind {
    fn from(code: u32) -> LibvcxErrorKind {
        match ERROR_KINDS.iter().find(|(_, n)| *n == code) {
            Some((kind, _)) => *kind,
            None => LibvcxErrorKind::UnknownError,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::errors::error::LibvcxErrorKind;

    #[test]
    fn it_should_map_error_kinds_to_codes() {
        assert_eq!(u32::from(LibvcxErrorKind::InvalidState), 1081);
        assert_eq!(u32::from(LibvcxErrorKind::InvalidConfiguration), 1004);
        assert_eq!(u32::from(LibvcxErrorKind::InvalidHandle), 1048);
        assert_eq!(u32::from(LibvcxErrorKind::InvalidJson), 1016);
        assert_eq!(u32::from(LibvcxErrorKind::InvalidOption), 1007);
        assert_eq!(u32::from(LibvcxErrorKind::InvalidMessagePack), 1019);
        assert_eq!(u32::from(LibvcxErrorKind::NotReady), 1005);
        assert_eq!(u32::from(LibvcxErrorKind::InvalidRevocationDetails), 1091);
        assert_eq!(u32::from(LibvcxErrorKind::IOError), 1074);
        assert_eq!(u32::from(LibvcxErrorKind::LibindyInvalidStructure), 1080);
        assert_eq!(u32::from(LibvcxErrorKind::InvalidLibindyParam), 1067);
        assert_eq!(u32::from(LibvcxErrorKind::AlreadyInitialized), 1044);
        assert_eq!(u32::from(LibvcxErrorKind::CreateConnection), 1061);
        assert_eq!(u32::from(LibvcxErrorKind::InvalidConnectionHandle), 1003);
        assert_eq!(u32::from(LibvcxErrorKind::CreateCredDef), 1034);
        assert_eq!(u32::from(LibvcxErrorKind::CredDefAlreadyCreated), 1039);
        assert_eq!(u32::from(LibvcxErrorKind::InvalidCredDefHandle), 1037);
        assert_eq!(u32::from(LibvcxErrorKind::InvalidRevocationEntry), 1092);
        assert_eq!(u32::from(LibvcxErrorKind::CreateRevRegDef), 1095);
        assert_eq!(u32::from(LibvcxErrorKind::InvalidCredentialHandle), 1053);
        assert_eq!(
            u32::from(LibvcxErrorKind::InvalidIssuerCredentialHandle),
            1015
        );
        assert_eq!(u32::from(LibvcxErrorKind::InvalidProofHandle), 1017);
        assert_eq!(
            u32::from(LibvcxErrorKind::InvalidDisclosedProofHandle),
            1049
        );
        assert_eq!(u32::from(LibvcxErrorKind::InvalidProof), 1023);
        assert_eq!(u32::from(LibvcxErrorKind::InvalidSchema), 1031);
        assert_eq!(u32::from(LibvcxErrorKind::InvalidProofCredentialData), 1027);
        assert_eq!(u32::from(LibvcxErrorKind::InvalidRevocationTimestamp), 1093);
        assert_eq!(u32::from(LibvcxErrorKind::CreateSchema), 1041);
        assert_eq!(u32::from(LibvcxErrorKind::InvalidSchemaHandle), 1042);
        assert_eq!(u32::from(LibvcxErrorKind::InvalidSchemaSeqNo), 1040);
        assert_eq!(u32::from(LibvcxErrorKind::DuplicationSchema), 1088);
        assert_eq!(u32::from(LibvcxErrorKind::UnknownSchemaRejection), 1094);
        assert_eq!(u32::from(LibvcxErrorKind::WalletCreate), 1058);
        assert_eq!(u32::from(LibvcxErrorKind::WalletAccessFailed), 1075);
        assert_eq!(u32::from(LibvcxErrorKind::InvalidWalletHandle), 1057);
        assert_eq!(u32::from(LibvcxErrorKind::DuplicationWallet), 1051);
        assert_eq!(u32::from(LibvcxErrorKind::WalletNotFound), 1079);
        assert_eq!(u32::from(LibvcxErrorKind::WalletRecordNotFound), 1073);
        assert_eq!(u32::from(LibvcxErrorKind::PoolLedgerConnect), 1025);
        assert_eq!(u32::from(LibvcxErrorKind::InvalidGenesisTxnPath), 1024);
        assert_eq!(u32::from(LibvcxErrorKind::CreatePoolConfig), 1026);
        assert_eq!(u32::from(LibvcxErrorKind::DuplicationWalletRecord), 1072);
        assert_eq!(u32::from(LibvcxErrorKind::WalletAlreadyOpen), 1052);
        assert_eq!(u32::from(LibvcxErrorKind::DuplicationMasterSecret), 1084);
        assert_eq!(u32::from(LibvcxErrorKind::DuplicationDid), 1083);
        assert_eq!(u32::from(LibvcxErrorKind::InvalidLedgerResponse), 1082);
        assert_eq!(u32::from(LibvcxErrorKind::InvalidAttributesStructure), 1021);
        assert_eq!(u32::from(LibvcxErrorKind::InvalidProofRequest), 1086);
        assert_eq!(u32::from(LibvcxErrorKind::NoPoolOpen), 1030);
        assert_eq!(u32::from(LibvcxErrorKind::PostMessageFailed), 1010);
        assert_eq!(u32::from(LibvcxErrorKind::LoggingError), 1090);
        assert_eq!(u32::from(LibvcxErrorKind::EncodeError), 1022);
        assert_eq!(u32::from(LibvcxErrorKind::UnknownError), 1001);
        assert_eq!(u32::from(LibvcxErrorKind::InvalidDid), 1008);
        assert_eq!(u32::from(LibvcxErrorKind::InvalidVerkey), 1009);
        assert_eq!(u32::from(LibvcxErrorKind::InvalidNonce), 1011);
        assert_eq!(u32::from(LibvcxErrorKind::InvalidUrl), 1013);
        assert_eq!(u32::from(LibvcxErrorKind::SerializationError), 1050);
        assert_eq!(u32::from(LibvcxErrorKind::NotBase58), 1014);
        assert_eq!(u32::from(LibvcxErrorKind::InvalidHttpResponse), 1033);
        assert_eq!(u32::from(LibvcxErrorKind::InvalidMessages), 1020);
        assert_eq!(u32::from(LibvcxErrorKind::UnknownLibndyError), 1035);
        assert_eq!(u32::from(LibvcxErrorKind::ActionNotSupported), 1103);
        assert_eq!(u32::from(LibvcxErrorKind::LibndyError(22222)), 22222);
        assert_eq!(u32::from(LibvcxErrorKind::NoAgentInformation), 1106);
        assert_eq!(u32::from(LibvcxErrorKind::RevRegDefNotFound), 1107);
        assert_eq!(u32::from(LibvcxErrorKind::RevDeltaNotFound), 1108);
        assert_eq!(u32::from(LibvcxErrorKind::RevDeltaFailedToClear), 1114);
        assert_eq!(u32::from(LibvcxErrorKind::PoisonedLock), 1109);
        assert_eq!(u32::from(LibvcxErrorKind::ObjectAccessError), 1110);
        assert_eq!(u32::from(LibvcxErrorKind::InvalidMessageFormat), 1111);
        assert_eq!(u32::from(LibvcxErrorKind::CreateOutOfBand), 1112);
        assert_eq!(u32::from(LibvcxErrorKind::InvalidInput), 1115);
        assert_eq!(u32::from(LibvcxErrorKind::ParsingError), 1116);
        assert_eq!(u32::from(LibvcxErrorKind::UnimplementedFeature), 1117);
        assert_eq!(u32::from(LibvcxErrorKind::LedgerItemNotFound), 1118);
    }

    #[test]
    fn it_should_map_error_codes_to_error_kinds() {
        assert_eq!(LibvcxErrorKind::from(1081), LibvcxErrorKind::InvalidState);
        assert_eq!(
            LibvcxErrorKind::from(1004),
            LibvcxErrorKind::InvalidConfiguration
        );
        assert_eq!(LibvcxErrorKind::from(1048), LibvcxErrorKind::InvalidHandle);
        assert_eq!(LibvcxErrorKind::from(1016), LibvcxErrorKind::InvalidJson);
        assert_eq!(LibvcxErrorKind::from(1007), LibvcxErrorKind::InvalidOption);
        assert_eq!(
            LibvcxErrorKind::from(1019),
            LibvcxErrorKind::InvalidMessagePack
        );
        assert_eq!(LibvcxErrorKind::from(1005), LibvcxErrorKind::NotReady);
        assert_eq!(
            LibvcxErrorKind::from(1091),
            LibvcxErrorKind::InvalidRevocationDetails
        );
        assert_eq!(LibvcxErrorKind::from(1074), LibvcxErrorKind::IOError);
        assert_eq!(
            LibvcxErrorKind::from(1080),
            LibvcxErrorKind::LibindyInvalidStructure
        );
        assert_eq!(
            LibvcxErrorKind::from(1067),
            LibvcxErrorKind::InvalidLibindyParam
        );
        assert_eq!(
            LibvcxErrorKind::from(1044),
            LibvcxErrorKind::AlreadyInitialized
        );
        assert_eq!(
            LibvcxErrorKind::from(1061),
            LibvcxErrorKind::CreateConnection
        );
        assert_eq!(
            LibvcxErrorKind::from(1003),
            LibvcxErrorKind::InvalidConnectionHandle
        );
        assert_eq!(LibvcxErrorKind::from(1034), LibvcxErrorKind::CreateCredDef);
        assert_eq!(
            LibvcxErrorKind::from(1039),
            LibvcxErrorKind::CredDefAlreadyCreated
        );
        assert_eq!(
            LibvcxErrorKind::from(1037),
            LibvcxErrorKind::InvalidCredDefHandle
        );
        assert_eq!(
            LibvcxErrorKind::from(1092),
            LibvcxErrorKind::InvalidRevocationEntry
        );
        assert_eq!(
            LibvcxErrorKind::from(1095),
            LibvcxErrorKind::CreateRevRegDef
        );
        assert_eq!(
            LibvcxErrorKind::from(1053),
            LibvcxErrorKind::InvalidCredentialHandle
        );
        assert_eq!(
            LibvcxErrorKind::from(1015),
            LibvcxErrorKind::InvalidIssuerCredentialHandle
        );
        assert_eq!(
            LibvcxErrorKind::from(1017),
            LibvcxErrorKind::InvalidProofHandle
        );
        assert_eq!(
            LibvcxErrorKind::from(1049),
            LibvcxErrorKind::InvalidDisclosedProofHandle
        );
        assert_eq!(LibvcxErrorKind::from(1023), LibvcxErrorKind::InvalidProof);
        assert_eq!(LibvcxErrorKind::from(1031), LibvcxErrorKind::InvalidSchema);
        assert_eq!(
            LibvcxErrorKind::from(1027),
            LibvcxErrorKind::InvalidProofCredentialData
        );
        assert_eq!(
            LibvcxErrorKind::from(1093),
            LibvcxErrorKind::InvalidRevocationTimestamp
        );
        assert_eq!(LibvcxErrorKind::from(1041), LibvcxErrorKind::CreateSchema);
        assert_eq!(
            LibvcxErrorKind::from(1042),
            LibvcxErrorKind::InvalidSchemaHandle
        );
        assert_eq!(
            LibvcxErrorKind::from(1040),
            LibvcxErrorKind::InvalidSchemaSeqNo
        );
        assert_eq!(
            LibvcxErrorKind::from(1088),
            LibvcxErrorKind::DuplicationSchema
        );
        assert_eq!(
            LibvcxErrorKind::from(1094),
            LibvcxErrorKind::UnknownSchemaRejection
        );
        assert_eq!(LibvcxErrorKind::from(1058), LibvcxErrorKind::WalletCreate);
        assert_eq!(
            LibvcxErrorKind::from(1075),
            LibvcxErrorKind::WalletAccessFailed
        );
        assert_eq!(
            LibvcxErrorKind::from(1057),
            LibvcxErrorKind::InvalidWalletHandle
        );
        assert_eq!(
            LibvcxErrorKind::from(1051),
            LibvcxErrorKind::DuplicationWallet
        );
        assert_eq!(LibvcxErrorKind::from(1079), LibvcxErrorKind::WalletNotFound);
        assert_eq!(
            LibvcxErrorKind::from(1073),
            LibvcxErrorKind::WalletRecordNotFound
        );
        assert_eq!(
            LibvcxErrorKind::from(1025),
            LibvcxErrorKind::PoolLedgerConnect
        );
        assert_eq!(
            LibvcxErrorKind::from(1024),
            LibvcxErrorKind::InvalidGenesisTxnPath
        );
        assert_eq!(
            LibvcxErrorKind::from(1026),
            LibvcxErrorKind::CreatePoolConfig
        );
        assert_eq!(
            LibvcxErrorKind::from(1072),
            LibvcxErrorKind::DuplicationWalletRecord
        );
        assert_eq!(
            LibvcxErrorKind::from(1052),
            LibvcxErrorKind::WalletAlreadyOpen
        );
        assert_eq!(
            LibvcxErrorKind::from(1084),
            LibvcxErrorKind::DuplicationMasterSecret
        );
        assert_eq!(LibvcxErrorKind::from(1083), LibvcxErrorKind::DuplicationDid);
        assert_eq!(
            LibvcxErrorKind::from(1082),
            LibvcxErrorKind::InvalidLedgerResponse
        );
        assert_eq!(
            LibvcxErrorKind::from(1021),
            LibvcxErrorKind::InvalidAttributesStructure
        );
        assert_eq!(
            LibvcxErrorKind::from(1086),
            LibvcxErrorKind::InvalidProofRequest
        );
        assert_eq!(LibvcxErrorKind::from(1030), LibvcxErrorKind::NoPoolOpen);
        assert_eq!(
            LibvcxErrorKind::from(1010),
            LibvcxErrorKind::PostMessageFailed
        );
        assert_eq!(LibvcxErrorKind::from(1090), LibvcxErrorKind::LoggingError);
        assert_eq!(LibvcxErrorKind::from(1022), LibvcxErrorKind::EncodeError);
        assert_eq!(LibvcxErrorKind::from(1001), LibvcxErrorKind::UnknownError);
        assert_eq!(LibvcxErrorKind::from(1008), LibvcxErrorKind::InvalidDid);
        assert_eq!(LibvcxErrorKind::from(1009), LibvcxErrorKind::InvalidVerkey);
        assert_eq!(LibvcxErrorKind::from(1011), LibvcxErrorKind::InvalidNonce);
        assert_eq!(LibvcxErrorKind::from(1013), LibvcxErrorKind::InvalidUrl);
        assert_eq!(
            LibvcxErrorKind::from(1050),
            LibvcxErrorKind::SerializationError
        );
        assert_eq!(LibvcxErrorKind::from(1014), LibvcxErrorKind::NotBase58);
        assert_eq!(
            LibvcxErrorKind::from(1033),
            LibvcxErrorKind::InvalidHttpResponse
        );
        assert_eq!(
            LibvcxErrorKind::from(1020),
            LibvcxErrorKind::InvalidMessages
        );
        assert_eq!(
            LibvcxErrorKind::from(1035),
            LibvcxErrorKind::UnknownLibndyError
        );
        assert_eq!(
            LibvcxErrorKind::from(1103),
            LibvcxErrorKind::ActionNotSupported
        );
        assert_eq!(
            LibvcxErrorKind::from(1106),
            LibvcxErrorKind::NoAgentInformation
        );
        assert_eq!(
            LibvcxErrorKind::from(1107),
            LibvcxErrorKind::RevRegDefNotFound
        );
        assert_eq!(
            LibvcxErrorKind::from(1108),
            LibvcxErrorKind::RevDeltaNotFound
        );
        assert_eq!(
            LibvcxErrorKind::from(1114),
            LibvcxErrorKind::RevDeltaFailedToClear
        );
        assert_eq!(LibvcxErrorKind::from(1109), LibvcxErrorKind::PoisonedLock);
        assert_eq!(
            LibvcxErrorKind::from(1110),
            LibvcxErrorKind::ObjectAccessError
        );
        assert_eq!(
            LibvcxErrorKind::from(1111),
            LibvcxErrorKind::InvalidMessageFormat
        );
        assert_eq!(
            LibvcxErrorKind::from(1112),
            LibvcxErrorKind::CreateOutOfBand
        );
        assert_eq!(LibvcxErrorKind::from(1115), LibvcxErrorKind::InvalidInput);
        assert_eq!(LibvcxErrorKind::from(1116), LibvcxErrorKind::ParsingError);
        assert_eq!(
            LibvcxErrorKind::from(1117),
            LibvcxErrorKind::UnimplementedFeature
        );
        assert_eq!(
            LibvcxErrorKind::from(1118),
            LibvcxErrorKind::LedgerItemNotFound
        );
        assert_eq!(LibvcxErrorKind::from(9999), LibvcxErrorKind::UnknownError);
    }
}
