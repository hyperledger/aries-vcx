use std::str::FromStr;

use crate::cl::{
    CredentialPrimaryPublicKey, CredentialPrivateKey, CredentialPublicKey,
    CredentialRevocationPublicKey,
};
use crate::{error::ConversionError, impl_anoncreds_object_identifier};

use super::{issuer_id::IssuerId, schema::SchemaId};

pub const CL_SIGNATURE_TYPE: &str = "CL";

impl_anoncreds_object_identifier!(CredentialDefinitionId);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignatureType {
    CL,
}

impl FromStr for SignatureType {
    type Err = ConversionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            CL_SIGNATURE_TYPE => Ok(Self::CL),
            _ => Err(ConversionError::from_msg("Invalid signature type")),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CredentialDefinitionData {
    pub primary: CredentialPrimaryPublicKey,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revocation: Option<CredentialRevocationPublicKey>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialDefinition {
    pub schema_id: SchemaId,
    #[serde(rename = "type")]
    pub signature_type: SignatureType,
    pub tag: String,
    pub value: CredentialDefinitionData,
    pub issuer_id: IssuerId,
}

impl CredentialDefinition {
    pub fn get_public_key(&self) -> Result<CredentialPublicKey, ConversionError> {
        let key = CredentialPublicKey::build_from_parts(
            &self.value.primary,
            self.value.revocation.as_ref(),
        )
        .map_err(|e| e.to_string())?;
        Ok(key)
    }

    pub fn try_clone(&self) -> Result<Self, crate::Error> {
        let cred_data = CredentialDefinitionData {
            primary: self.value.primary.try_clone()?,
            revocation: self.value.revocation.clone(),
        };

        Ok(Self {
            schema_id: self.schema_id.clone(),
            signature_type: self.signature_type,
            tag: self.tag.clone(),
            value: cred_data,
            issuer_id: self.issuer_id.clone(),
        })
    }
}

impl Validatable for CredentialDefinition {
    fn validate(&self) -> Result<(), ValidationError> {
        self.schema_id.validate()?;
        self.issuer_id.validate()?;

        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CredentialDefinitionPrivate {
    pub value: CredentialPrivateKey,
}

// #[derive(Debug, Deserialize, Serialize)]
// #[serde(transparent)]
// pub struct CredentialKeyCorrectnessProof {
//     pub value: CryptoCredentialKeyCorrectnessProof,
// }
//
// impl CredentialKeyCorrectnessProof {
//     pub fn try_clone(&self) -> Result<Self, ConversionError> {
//         Ok(Self {
//             value: self.value.try_clone().map_err(|e| e.to_string())?,
//         })
//     }
// }
//
// #[cfg(test)]
// mod test_cred_def {
//     use super::*;
//     use crate::data_types::schema::Schema;
//     use crate::issuer;
//     use crate::types::CredentialDefinitionConfig;
//
//     fn schema() -> Schema {
//         issuer::create_schema(
//             "name",
//             "1.0",
//             "did:example".try_into().unwrap(),
//             vec!["name".to_owned(), "age".to_owned()].into(),
//         )
//         .expect("Unable to create Schema")
//     }
//
//     fn cred_def() -> (
//         CredentialDefinition,
//         CredentialDefinitionPrivate,
//         CredentialKeyCorrectnessProof,
//     ) {
//         let schema = schema();
//         issuer::create_credential_definition(
//             "did:example/schema".try_into().unwrap(),
//             &schema,
//             "did:exampple".try_into().unwrap(),
//             "default-tag",
//             SignatureType::CL,
//             CredentialDefinitionConfig::default(),
//         )
//         .expect("Unable to create credential Definition")
//     }
//
//     #[test]
//     fn should_create_credential_definition() {
//         let schema = schema();
//         let result = issuer::create_credential_definition(
//             "did:example/schema".try_into().unwrap(),
//             &schema,
//             "did:exampple".try_into().unwrap(),
//             "default-tag",
//             SignatureType::CL,
//             CredentialDefinitionConfig::default(),
//         );
//
//         assert!(result.is_ok());
//     }
//
//     #[test]
//     fn should_validate_credential_definition() {
//         let (cred_def, _, _) = cred_def();
//
//         assert!(cred_def.validate().is_ok());
//     }
//
//     #[test]
//     fn should_get_public_key() {
//         let (cred_def, _, _) = cred_def();
//
//         assert!(cred_def.get_public_key().is_ok());
//     }
//
//     #[test]
//     fn should_clone_key_correctness_proof() {
//         let (_, _, key_correctness_proof) = cred_def();
//
//         assert!(key_correctness_proof.try_clone().is_ok());
//     }
//
//     #[test]
//     fn should_create_cred_def_from_json() {
//         let json = serde_json::json!({
//            "schemaId":"did:example/schema",
//            "type":"CL",
//            "tag":"default-tag",
//            "value":{
//               "primary":{
//                  "n":"935979291220971862704 592165681658743368502355970787648706395000638708831646449277673914639455890162280171218526289357653559011217921 381179234240544289719662699577985360925146920144825220975695113600719509513661409854581145180857024389772444364 215170162846206911976335827475379172617595717376676318672666375261477842319431759753015820984511999723117700036 314156565134414047850028080765217651779320384904770355272647015925363502950950601967012240776102949493254736500 515153812951926834578804113175853917041556235839879128315128246679052177250401037299314415358777660652228175998 52068898927329685677780318700495028384397",
//                  "s":"599062104246467631868210219081301893864225866912453729750290792 332429108055185332093174863006105031494958634315779188819023633200248484514454314009116953949113622021543786109 359508266625905343316635835987334769868701399231224257049458849742283023407661890746594947755353385167624628579 252788875899276963218332181681735590325601688483079838161559885088636339437248017314173468683518400771359664526 211039335170271655419060096254465064304850945631904970274305455093356320660601505151649652302097605712265427103 35196392918580221426160672658614917634675227198720634585626786331302034690664585299318960517604204906470635473",
//                  "r":{
//                     "name":"6762882845831465228877062814346915205760727038299169865376306921982996240152914497029857603658631 623983235865539404733972503432485527427494090208675903311247126861833158589871593344150952438845967333014320478 228751861198549753559430436514014792653250701560781344373533113377575688696867591873034290154002855776063379095 994150607140551285871960700619328544583792394527098612619695454025618539880995541013070109279660052563574602566 706841331737415408195583316287920201010299710222089395604154980134268680322266083387524772701260415836394753496 1223346295451460071601495031068594958289570281060897305275504114273882976121",
//                     "master_secret":"834327501130721383 922647227893893485688183039838682877013748897169553486150098565316645019470001227808850532176097786850582344387 253822685644183179653911947409283114698973727298477082684432358179394761329093468743528076966149330841589065783 281451347352327230146162773906448836821181634455300391216430198940441494236628280198931812072731822567658500373 697054814965252638400739781272327698100052752660389510928973789421699280591255410954366143149910532414967638399 025619862351479498702029793455313863803374387182033401617684461829020160630117460855921152743119919239488258375 54855484708539430700207247687104287527548998",
//                     "age":"7709301379926011936017071957575272040339133639808781450430 871576223756227740626140652545287050010703595495181086072651128911753760065970396605199140917897621650930872506 614616152858349223304531775718418731677664092524067060432494575984575083396483361494688392327138858588063800364 309816312624259788265738531776682315131821330930299590221058428511875137251487269190722835539844615961605983997 643905243896672657074861294488013801913431870539044148276035511857781969699740240742530848362897241488134464688 945870580181720333933577797589290917500895085245819966959231524717256473214562044613845001674777032808741936182 4416"
//                  },
//                  "rctxt":"73656301303273199382665699395760051281018648729663651033265967081645925670861933395931479550648 701153114243620636591380967855818339769397176656069791245704739859120788149089447734165825501546786231177638207 359718953666070869270193623575870839462327632242015951855441562153274132485536883814223446596709848575961919049 419346226705564963602056856745047811888573542586255687935416902676613425309672743530867940973634942972115000403 587763366560036690083919848411570824415227712237514070904904299511939785342370611619043119010883514435772025313 673357681692202910574272142277362962045625845272577932678287165804590362500998",
//                  "z":"42600522970105141673255939 583349956013043591940524446116593861845406891338239994189638834889833797519799020554388634882308685728371670047 230870144299790205810338083621050348793010255785256748425833840958191572495035192159310364108499046681117306785 706367854325599885532135471594430505660301732514741067898521871623800410416367416726887431226889314185229547538 284212444112324122107450676305474896548674970518951558210235902571885028080712543559018363771092069510133831271 951909156799458230350889723790955968591857776367846136496461612630446043267301372429625254336623655724485011672 580454949711118061757671747473160528"
//               }
//            },
//            "issuerId":"did:exampple"
//         });
//
//         let cred_def: Result<CredentialDefinition, _> = serde_json::from_value(json);
//
//         assert!(cred_def.is_ok());
//     }
// }
