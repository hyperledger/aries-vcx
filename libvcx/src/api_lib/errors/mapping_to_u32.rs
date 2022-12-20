use crate::api_lib::errors::error::{ErrorLibvcx, ErrorKindLibvcx};
use crate::api_lib::errors::error;

impl From<ErrorLibvcx> for u32 {
    fn from(error: ErrorLibvcx) -> u32 {
        error.kind().into()
    }
}

lazy_static! {
    static ref ERROR_KINDS: Vec<(ErrorKindLibvcx, u32)> = vec![
            (ErrorKindLibvcx::InvalidState, 1081),
            (ErrorKindLibvcx::InvalidConfiguration, 1004),
            (ErrorKindLibvcx::InvalidHandle, 1048),
            (ErrorKindLibvcx::InvalidJson, 1016),
            (ErrorKindLibvcx::InvalidOption, 1007),
            (ErrorKindLibvcx::InvalidMessagePack, 1019),
            (ErrorKindLibvcx::NotReady, 1005),
            (ErrorKindLibvcx::InvalidRevocationDetails, 1091),
            (ErrorKindLibvcx::IOError, 1074),
            (ErrorKindLibvcx::LibindyInvalidStructure, 1080),
            (ErrorKindLibvcx::InvalidLibindyParam, 1067),
            (ErrorKindLibvcx::AlreadyInitialized, 1044),
            (ErrorKindLibvcx::CreateConnection, 1061),
            (ErrorKindLibvcx::InvalidConnectionHandle, 1003),
            (ErrorKindLibvcx::CreateCredDef, 1034),
            (ErrorKindLibvcx::CredDefAlreadyCreated, 1039),
            (ErrorKindLibvcx::InvalidCredDefHandle, 1037),
            (ErrorKindLibvcx::InvalidRevocationEntry, 1092),
            (ErrorKindLibvcx::CreateRevRegDef, 1095),
            (ErrorKindLibvcx::InvalidCredentialHandle, 1053),
            (ErrorKindLibvcx::InvalidIssuerCredentialHandle, 1015),
            (ErrorKindLibvcx::InvalidProofHandle, 1017),
            (ErrorKindLibvcx::InvalidDisclosedProofHandle, 1049),
            (ErrorKindLibvcx::InvalidProof, 1023),
            (ErrorKindLibvcx::InvalidSchema, 1031),
            (ErrorKindLibvcx::InvalidProofCredentialData, 1027),
            (ErrorKindLibvcx::InvalidRevocationTimestamp, 1093),
            (ErrorKindLibvcx::CreateSchema, 1041),
            (ErrorKindLibvcx::InvalidSchemaHandle, 1042),
            (ErrorKindLibvcx::InvalidSchemaSeqNo, 1040),
            (ErrorKindLibvcx::DuplicationSchema, 1088),
            (ErrorKindLibvcx::UnknownSchemaRejection, 1094),
            (ErrorKindLibvcx::WalletCreate, 1058),
            (ErrorKindLibvcx::WalletAccessFailed, 1075),
            (ErrorKindLibvcx::InvalidWalletHandle, 1057),
            (ErrorKindLibvcx::DuplicationWallet, 1051),
            (ErrorKindLibvcx::WalletNotFound, 1079),
            (ErrorKindLibvcx::WalletRecordNotFound, 1073),
            (ErrorKindLibvcx::PoolLedgerConnect, 1025),
            (ErrorKindLibvcx::InvalidGenesisTxnPath, 1024),
            (ErrorKindLibvcx::CreatePoolConfig, 1026),
            (ErrorKindLibvcx::DuplicationWalletRecord, 1072),
            (ErrorKindLibvcx::WalletAlreadyOpen, 1052),
            (ErrorKindLibvcx::DuplicationMasterSecret, 1084),
            (ErrorKindLibvcx::DuplicationDid, 1083),
            (ErrorKindLibvcx::InvalidLedgerResponse, 1082),
            (ErrorKindLibvcx::InvalidAttributesStructure, 1021),
            (ErrorKindLibvcx::InvalidProofRequest, 1086),
            (ErrorKindLibvcx::NoPoolOpen, 1030),
            (ErrorKindLibvcx::PostMessageFailed, 1010),
            (ErrorKindLibvcx::LoggingError, 1090),
            (ErrorKindLibvcx::EncodeError, 1022),
            (ErrorKindLibvcx::UnknownError, 1001),
            (ErrorKindLibvcx::InvalidDid, 1008),
            (ErrorKindLibvcx::InvalidVerkey, 1009),
            (ErrorKindLibvcx::InvalidNonce, 1011),
            (ErrorKindLibvcx::InvalidUrl, 1013),
            (ErrorKindLibvcx::SerializationError, 1050),
            (ErrorKindLibvcx::NotBase58, 1014),
            (ErrorKindLibvcx::InvalidHttpResponse, 1033),
            (ErrorKindLibvcx::InvalidMessages, 1020),
            (ErrorKindLibvcx::UnknownLibndyError, 1035),
            (ErrorKindLibvcx::ActionNotSupported, 1103),
            (ErrorKindLibvcx::NoAgentInformation, 1106),
            (ErrorKindLibvcx::RevRegDefNotFound, 1107),
            (ErrorKindLibvcx::RevDeltaNotFound, 1108),
            (ErrorKindLibvcx::RevDeltaFailedToClear, 1114),
            (ErrorKindLibvcx::PoisonedLock, 1109),
            (ErrorKindLibvcx::InvalidMessageFormat, 1111),
            (ErrorKindLibvcx::CreatePublicAgent, 1110),
            (ErrorKindLibvcx::CreateOutOfBand, 1112),
            (ErrorKindLibvcx::InvalidInput, 1115),
            (ErrorKindLibvcx::ParsingError, 1116),
            (ErrorKindLibvcx::UnimplementedFeature, 1117)
        ];
}

// note: iterating few tens of values in case of error should not have much impact, but it surely
// can be optimized. The implementation is optimizing on easy of adding new errors and minimizing
// duplication of error code pairs.
impl From<ErrorKindLibvcx> for u32 {
    fn from(kind: ErrorKindLibvcx) -> u32 {
        match kind {
            ErrorKindLibvcx::Common(code) => code,
            ErrorKindLibvcx::LibndyError(code) => code,
            _ => {
                match ERROR_KINDS.iter().find(|(k, _)| *k == kind) {
                    Some((_, num)) => *num,
                    None => 1001
                }
            }
        }
    }
}

impl From<u32> for ErrorKindLibvcx {
    fn from(code: u32) -> ErrorKindLibvcx {
        match ERROR_KINDS.iter().find(|(_, n)| *n == code) {
            Some((kind, _)) => *kind,
            None => ErrorKindLibvcx::UnknownError
        }
    }
}


#[cfg(test)]
mod tests {
    use serde_json;
    use crate::api_lib::errors::error::ErrorKindLibvcx;

    #[test]
    #[cfg(feature = "general_test")]
    fn it_should_map_error_kinds_to_codes() {
        assert_eq!(u32::from(ErrorKindLibvcx::InvalidState), 1081);
        assert_eq!(u32::from(ErrorKindLibvcx::InvalidConfiguration), 1004);
        assert_eq!(u32::from(ErrorKindLibvcx::InvalidHandle), 1048);
        assert_eq!(u32::from(ErrorKindLibvcx::InvalidJson), 1016);
        assert_eq!(u32::from(ErrorKindLibvcx::InvalidOption), 1007);
        assert_eq!(u32::from(ErrorKindLibvcx::InvalidMessagePack), 1019);
        assert_eq!(u32::from(ErrorKindLibvcx::NotReady), 1005);
        assert_eq!(u32::from(ErrorKindLibvcx::InvalidRevocationDetails), 1091);
        assert_eq!(u32::from(ErrorKindLibvcx::IOError), 1074);
        assert_eq!(u32::from(ErrorKindLibvcx::LibindyInvalidStructure), 1080);
        assert_eq!(u32::from(ErrorKindLibvcx::InvalidLibindyParam), 1067);
        assert_eq!(u32::from(ErrorKindLibvcx::AlreadyInitialized), 1044);
        assert_eq!(u32::from(ErrorKindLibvcx::CreateConnection), 1061);
        assert_eq!(u32::from(ErrorKindLibvcx::InvalidConnectionHandle), 1003);
        assert_eq!(u32::from(ErrorKindLibvcx::CreateCredDef), 1034);
        assert_eq!(u32::from(ErrorKindLibvcx::CredDefAlreadyCreated), 1039);
        assert_eq!(u32::from(ErrorKindLibvcx::InvalidCredDefHandle), 1037);
        assert_eq!(u32::from(ErrorKindLibvcx::InvalidRevocationEntry), 1092);
        assert_eq!(u32::from(ErrorKindLibvcx::CreateRevRegDef), 1095);
        assert_eq!(u32::from(ErrorKindLibvcx::InvalidCredentialHandle), 1053);
        assert_eq!(u32::from(ErrorKindLibvcx::InvalidIssuerCredentialHandle), 1015);
        assert_eq!(u32::from(ErrorKindLibvcx::InvalidProofHandle), 1017);
        assert_eq!(u32::from(ErrorKindLibvcx::InvalidDisclosedProofHandle), 1049);
        assert_eq!(u32::from(ErrorKindLibvcx::InvalidProof), 1023);
        assert_eq!(u32::from(ErrorKindLibvcx::InvalidSchema), 1031);
        assert_eq!(u32::from(ErrorKindLibvcx::InvalidProofCredentialData), 1027);
        assert_eq!(u32::from(ErrorKindLibvcx::InvalidRevocationTimestamp), 1093);
        assert_eq!(u32::from(ErrorKindLibvcx::CreateSchema), 1041);
        assert_eq!(u32::from(ErrorKindLibvcx::InvalidSchemaHandle), 1042);
        assert_eq!(u32::from(ErrorKindLibvcx::InvalidSchemaSeqNo), 1040);
        assert_eq!(u32::from(ErrorKindLibvcx::DuplicationSchema), 1088);
        assert_eq!(u32::from(ErrorKindLibvcx::UnknownSchemaRejection), 1094);
        assert_eq!(u32::from(ErrorKindLibvcx::WalletCreate), 1058);
        assert_eq!(u32::from(ErrorKindLibvcx::WalletAccessFailed), 1075);
        assert_eq!(u32::from(ErrorKindLibvcx::InvalidWalletHandle), 1057);
        assert_eq!(u32::from(ErrorKindLibvcx::DuplicationWallet), 1051);
        assert_eq!(u32::from(ErrorKindLibvcx::WalletNotFound), 1079);
        assert_eq!(u32::from(ErrorKindLibvcx::WalletRecordNotFound), 1073);
        assert_eq!(u32::from(ErrorKindLibvcx::PoolLedgerConnect), 1025);
        assert_eq!(u32::from(ErrorKindLibvcx::InvalidGenesisTxnPath), 1024);
        assert_eq!(u32::from(ErrorKindLibvcx::CreatePoolConfig), 1026);
        assert_eq!(u32::from(ErrorKindLibvcx::DuplicationWalletRecord), 1072);
        assert_eq!(u32::from(ErrorKindLibvcx::WalletAlreadyOpen), 1052);
        assert_eq!(u32::from(ErrorKindLibvcx::DuplicationMasterSecret), 1084);
        assert_eq!(u32::from(ErrorKindLibvcx::DuplicationDid), 1083);
        assert_eq!(u32::from(ErrorKindLibvcx::InvalidLedgerResponse), 1082);
        assert_eq!(u32::from(ErrorKindLibvcx::InvalidAttributesStructure), 1021);
        assert_eq!(u32::from(ErrorKindLibvcx::InvalidProofRequest), 1086);
        assert_eq!(u32::from(ErrorKindLibvcx::NoPoolOpen), 1030);
        assert_eq!(u32::from(ErrorKindLibvcx::PostMessageFailed), 1010);
        assert_eq!(u32::from(ErrorKindLibvcx::LoggingError), 1090);
        assert_eq!(u32::from(ErrorKindLibvcx::EncodeError), 1022);
        assert_eq!(u32::from(ErrorKindLibvcx::UnknownError), 1001);
        assert_eq!(u32::from(ErrorKindLibvcx::InvalidDid), 1008);
        assert_eq!(u32::from(ErrorKindLibvcx::InvalidVerkey), 1009);
        assert_eq!(u32::from(ErrorKindLibvcx::InvalidNonce), 1011);
        assert_eq!(u32::from(ErrorKindLibvcx::InvalidUrl), 1013);
        assert_eq!(u32::from(ErrorKindLibvcx::SerializationError), 1050);
        assert_eq!(u32::from(ErrorKindLibvcx::NotBase58), 1014);
        assert_eq!(u32::from(ErrorKindLibvcx::InvalidHttpResponse), 1033);
        assert_eq!(u32::from(ErrorKindLibvcx::InvalidMessages), 1020);
        assert_eq!(u32::from(ErrorKindLibvcx::UnknownLibndyError), 1035);
        assert_eq!(u32::from(ErrorKindLibvcx::ActionNotSupported), 1103);
        assert_eq!(u32::from(ErrorKindLibvcx::Common(11111)), 11111);
        assert_eq!(u32::from(ErrorKindLibvcx::LibndyError(22222)), 22222);
        assert_eq!(u32::from(ErrorKindLibvcx::NoAgentInformation), 1106);
        assert_eq!(u32::from(ErrorKindLibvcx::RevRegDefNotFound), 1107);
        assert_eq!(u32::from(ErrorKindLibvcx::RevDeltaNotFound), 1108);
        assert_eq!(u32::from(ErrorKindLibvcx::RevDeltaFailedToClear), 1114);
        assert_eq!(u32::from(ErrorKindLibvcx::PoisonedLock), 1109);
        assert_eq!(u32::from(ErrorKindLibvcx::InvalidMessageFormat), 1111);
        assert_eq!(u32::from(ErrorKindLibvcx::CreatePublicAgent), 1110);
        assert_eq!(u32::from(ErrorKindLibvcx::CreateOutOfBand), 1112);
        assert_eq!(u32::from(ErrorKindLibvcx::InvalidInput), 1115);
        assert_eq!(u32::from(ErrorKindLibvcx::ParsingError), 1116);
        assert_eq!(u32::from(ErrorKindLibvcx::UnimplementedFeature), 1117);
    }


    #[test]
    #[cfg(feature = "general_test")]
    fn it_should_map_error_codes_to_error_kinds() {
        assert_eq!(ErrorKindLibvcx::from(1081), ErrorKindLibvcx::InvalidState);
        assert_eq!(ErrorKindLibvcx::from(1004), ErrorKindLibvcx::InvalidConfiguration);
        assert_eq!(ErrorKindLibvcx::from(1048), ErrorKindLibvcx::InvalidHandle);
        assert_eq!(ErrorKindLibvcx::from(1016), ErrorKindLibvcx::InvalidJson);
        assert_eq!(ErrorKindLibvcx::from(1007), ErrorKindLibvcx::InvalidOption);
        assert_eq!(ErrorKindLibvcx::from(1019), ErrorKindLibvcx::InvalidMessagePack);
        assert_eq!(ErrorKindLibvcx::from(1005), ErrorKindLibvcx::NotReady);
        assert_eq!(ErrorKindLibvcx::from(1091), ErrorKindLibvcx::InvalidRevocationDetails);
        assert_eq!(ErrorKindLibvcx::from(1074), ErrorKindLibvcx::IOError);
        assert_eq!(ErrorKindLibvcx::from(1080), ErrorKindLibvcx::LibindyInvalidStructure);
        assert_eq!(ErrorKindLibvcx::from(1067), ErrorKindLibvcx::InvalidLibindyParam);
        assert_eq!(ErrorKindLibvcx::from(1044), ErrorKindLibvcx::AlreadyInitialized);
        assert_eq!(ErrorKindLibvcx::from(1061), ErrorKindLibvcx::CreateConnection);
        assert_eq!(ErrorKindLibvcx::from(1003), ErrorKindLibvcx::InvalidConnectionHandle);
        assert_eq!(ErrorKindLibvcx::from(1034), ErrorKindLibvcx::CreateCredDef);
        assert_eq!(ErrorKindLibvcx::from(1039), ErrorKindLibvcx::CredDefAlreadyCreated);
        assert_eq!(ErrorKindLibvcx::from(1037), ErrorKindLibvcx::InvalidCredDefHandle);
        assert_eq!(ErrorKindLibvcx::from(1092), ErrorKindLibvcx::InvalidRevocationEntry);
        assert_eq!(ErrorKindLibvcx::from(1095), ErrorKindLibvcx::CreateRevRegDef);
        assert_eq!(ErrorKindLibvcx::from(1053), ErrorKindLibvcx::InvalidCredentialHandle);
        assert_eq!(ErrorKindLibvcx::from(1015), ErrorKindLibvcx::InvalidIssuerCredentialHandle);
        assert_eq!(ErrorKindLibvcx::from(1017), ErrorKindLibvcx::InvalidProofHandle);
        assert_eq!(ErrorKindLibvcx::from(1049), ErrorKindLibvcx::InvalidDisclosedProofHandle);
        assert_eq!(ErrorKindLibvcx::from(1023), ErrorKindLibvcx::InvalidProof);
        assert_eq!(ErrorKindLibvcx::from(1031), ErrorKindLibvcx::InvalidSchema);
        assert_eq!(ErrorKindLibvcx::from(1027), ErrorKindLibvcx::InvalidProofCredentialData);
        assert_eq!(ErrorKindLibvcx::from(1093), ErrorKindLibvcx::InvalidRevocationTimestamp);
        assert_eq!(ErrorKindLibvcx::from(1041), ErrorKindLibvcx::CreateSchema);
        assert_eq!(ErrorKindLibvcx::from(1042), ErrorKindLibvcx::InvalidSchemaHandle);
        assert_eq!(ErrorKindLibvcx::from(1040), ErrorKindLibvcx::InvalidSchemaSeqNo);
        assert_eq!(ErrorKindLibvcx::from(1088), ErrorKindLibvcx::DuplicationSchema);
        assert_eq!(ErrorKindLibvcx::from(1094), ErrorKindLibvcx::UnknownSchemaRejection);
        assert_eq!(ErrorKindLibvcx::from(1058), ErrorKindLibvcx::WalletCreate);
        assert_eq!(ErrorKindLibvcx::from(1075), ErrorKindLibvcx::WalletAccessFailed);
        assert_eq!(ErrorKindLibvcx::from(1057), ErrorKindLibvcx::InvalidWalletHandle);
        assert_eq!(ErrorKindLibvcx::from(1051), ErrorKindLibvcx::DuplicationWallet);
        assert_eq!(ErrorKindLibvcx::from(1079), ErrorKindLibvcx::WalletNotFound);
        assert_eq!(ErrorKindLibvcx::from(1073), ErrorKindLibvcx::WalletRecordNotFound);
        assert_eq!(ErrorKindLibvcx::from(1025), ErrorKindLibvcx::PoolLedgerConnect);
        assert_eq!(ErrorKindLibvcx::from(1024), ErrorKindLibvcx::InvalidGenesisTxnPath);
        assert_eq!(ErrorKindLibvcx::from(1026), ErrorKindLibvcx::CreatePoolConfig);
        assert_eq!(ErrorKindLibvcx::from(1072), ErrorKindLibvcx::DuplicationWalletRecord);
        assert_eq!(ErrorKindLibvcx::from(1052), ErrorKindLibvcx::WalletAlreadyOpen);
        assert_eq!(ErrorKindLibvcx::from(1084), ErrorKindLibvcx::DuplicationMasterSecret);
        assert_eq!(ErrorKindLibvcx::from(1083), ErrorKindLibvcx::DuplicationDid);
        assert_eq!(ErrorKindLibvcx::from(1082), ErrorKindLibvcx::InvalidLedgerResponse);
        assert_eq!(ErrorKindLibvcx::from(1021), ErrorKindLibvcx::InvalidAttributesStructure);
        assert_eq!(ErrorKindLibvcx::from(1086), ErrorKindLibvcx::InvalidProofRequest);
        assert_eq!(ErrorKindLibvcx::from(1030), ErrorKindLibvcx::NoPoolOpen);
        assert_eq!(ErrorKindLibvcx::from(1010), ErrorKindLibvcx::PostMessageFailed);
        assert_eq!(ErrorKindLibvcx::from(1090), ErrorKindLibvcx::LoggingError);
        assert_eq!(ErrorKindLibvcx::from(1022), ErrorKindLibvcx::EncodeError);
        assert_eq!(ErrorKindLibvcx::from(1001), ErrorKindLibvcx::UnknownError);
        assert_eq!(ErrorKindLibvcx::from(1008), ErrorKindLibvcx::InvalidDid);
        assert_eq!(ErrorKindLibvcx::from(1009), ErrorKindLibvcx::InvalidVerkey);
        assert_eq!(ErrorKindLibvcx::from(1011), ErrorKindLibvcx::InvalidNonce);
        assert_eq!(ErrorKindLibvcx::from(1013), ErrorKindLibvcx::InvalidUrl);
        assert_eq!(ErrorKindLibvcx::from(1050), ErrorKindLibvcx::SerializationError);
        assert_eq!(ErrorKindLibvcx::from(1014), ErrorKindLibvcx::NotBase58);
        assert_eq!(ErrorKindLibvcx::from(1033), ErrorKindLibvcx::InvalidHttpResponse);
        assert_eq!(ErrorKindLibvcx::from(1020), ErrorKindLibvcx::InvalidMessages);
        assert_eq!(ErrorKindLibvcx::from(1035), ErrorKindLibvcx::UnknownLibndyError);
        assert_eq!(ErrorKindLibvcx::from(1103), ErrorKindLibvcx::ActionNotSupported);
        assert_eq!(ErrorKindLibvcx::from(1106), ErrorKindLibvcx::NoAgentInformation);
        assert_eq!(ErrorKindLibvcx::from(1107), ErrorKindLibvcx::RevRegDefNotFound);
        assert_eq!(ErrorKindLibvcx::from(1108), ErrorKindLibvcx::RevDeltaNotFound);
        assert_eq!(ErrorKindLibvcx::from(1114), ErrorKindLibvcx::RevDeltaFailedToClear);
        assert_eq!(ErrorKindLibvcx::from(1109), ErrorKindLibvcx::PoisonedLock);
        assert_eq!(ErrorKindLibvcx::from(1111), ErrorKindLibvcx::InvalidMessageFormat);
        assert_eq!(ErrorKindLibvcx::from(1110), ErrorKindLibvcx::CreatePublicAgent);
        assert_eq!(ErrorKindLibvcx::from(1112), ErrorKindLibvcx::CreateOutOfBand);
        assert_eq!(ErrorKindLibvcx::from(1115), ErrorKindLibvcx::InvalidInput);
        assert_eq!(ErrorKindLibvcx::from(1116), ErrorKindLibvcx::ParsingError);
        assert_eq!(ErrorKindLibvcx::from(1117), ErrorKindLibvcx::UnimplementedFeature);
        assert_eq!(ErrorKindLibvcx::from(9999), ErrorKindLibvcx::UnknownError);
    }
}
