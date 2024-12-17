use std::fmt::Debug;

use anoncreds_types::data_types::{
    identifiers::{
        cred_def_id::CredentialDefinitionId, rev_reg_def_id::RevocationRegistryDefinitionId,
        schema_id::SchemaId,
    },
    ledger::{
        cred_def::CredentialDefinition, rev_reg::RevocationRegistry,
        rev_reg_def::RevocationRegistryDefinition, rev_reg_delta::RevocationRegistryDelta,
        rev_status_list::RevocationStatusList, schema::Schema,
    },
};
use async_trait::async_trait;
use did_parser_nom::Did;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::base_ledger::{AnoncredsLedgerRead, AnoncredsLedgerSupport};
use crate::errors::error::{VcxLedgerError, VcxLedgerResult};

// FUTURE - this multi-ledger anoncreds reader finds the first impl that supports the identifier
// and attempts to use it. This behaviour may need some enhancements in the future when
// considering coordination of unqualified object resolution, when multiple capable anoncreds
// readers are available (e.g. sovrin testnet & sovrin mainnet).
// Enhancements may include:
// * priority system - try A resolver before B if A & B both support the identifier
// * fallback/chain system - try A resolver, if it fails, try B resolver
// Alternatively these enhancements can be skipped if qualified DIDs/objects are used instead,
// e.g. did:indy:a:123, did:indy:b:123

/// Struct to aggregate multiple [AnoncredsLedgerRead] implementations into a single
/// [AnoncredsLedgerRead]. The child [AnoncredsLedgerRead] implementations are
/// utilized depending on whether or not they support resolution of the given object ID
/// (e.g. based on the DID Method).
#[derive(Default, Debug)]
pub struct MultiLedgerAnoncredsRead {
    readers: Vec<Box<dyn AnoncredsLedgerReadAdaptorTrait>>,
}

#[async_trait]
impl AnoncredsLedgerRead for MultiLedgerAnoncredsRead {
    type RevocationRegistryDefinitionAdditionalMetadata = Value;

    async fn get_schema(
        &self,
        schema_id: &SchemaId,
        submitter_did: Option<&Did>,
    ) -> VcxLedgerResult<Schema> {
        let reader = self
            .readers
            .iter()
            .find(|r| r.supports_schema(schema_id))
            .ok_or(VcxLedgerError::UnsupportedLedgerIdentifier(
                schema_id.to_string(),
            ))?;

        reader.get_schema(schema_id, submitter_did).await
    }

    async fn get_cred_def(
        &self,
        cred_def_id: &CredentialDefinitionId,
        submitter_did: Option<&Did>,
    ) -> VcxLedgerResult<CredentialDefinition> {
        let reader = self
            .readers
            .iter()
            .find(|r| r.supports_credential_definition(cred_def_id))
            .ok_or(VcxLedgerError::UnsupportedLedgerIdentifier(
                cred_def_id.to_string(),
            ))?;

        reader.get_cred_def(cred_def_id, submitter_did).await
    }

    async fn get_rev_reg_def_json(
        &self,
        rev_reg_id: &RevocationRegistryDefinitionId,
    ) -> VcxLedgerResult<(
        RevocationRegistryDefinition,
        Self::RevocationRegistryDefinitionAdditionalMetadata,
    )> {
        let reader = self
            .readers
            .iter()
            .find(|r| r.supports_revocation_registry(rev_reg_id))
            .ok_or(VcxLedgerError::UnsupportedLedgerIdentifier(
                rev_reg_id.to_string(),
            ))?;

        reader.get_rev_reg_def_json(rev_reg_id).await
    }
    async fn get_rev_reg_delta_json(
        &self,
        rev_reg_id: &RevocationRegistryDefinitionId,
        from: Option<u64>,
        to: Option<u64>,
    ) -> VcxLedgerResult<(RevocationRegistryDelta, u64)> {
        let reader = self
            .readers
            .iter()
            .find(|r| r.supports_revocation_registry(rev_reg_id))
            .ok_or(VcxLedgerError::UnsupportedLedgerIdentifier(
                rev_reg_id.to_string(),
            ))?;

        #[allow(deprecated)] // TODO - https://github.com/hyperledger/aries-vcx/issues/1309
        reader.get_rev_reg_delta_json(rev_reg_id, from, to).await
    }

    async fn get_rev_status_list(
        &self,
        rev_reg_id: &RevocationRegistryDefinitionId,
        timestamp: u64,
        rev_reg_def_meta: Option<&Self::RevocationRegistryDefinitionAdditionalMetadata>,
    ) -> VcxLedgerResult<(RevocationStatusList, u64)> {
        let reader = self
            .readers
            .iter()
            .find(|r| r.supports_revocation_registry(rev_reg_id))
            .ok_or(VcxLedgerError::UnsupportedLedgerIdentifier(
                rev_reg_id.to_string(),
            ))?;

        reader
            .get_rev_status_list(rev_reg_id, timestamp, rev_reg_def_meta)
            .await
    }

    async fn get_rev_reg(
        &self,
        rev_reg_id: &RevocationRegistryDefinitionId,
        timestamp: u64,
    ) -> VcxLedgerResult<(RevocationRegistry, u64)> {
        let reader = self
            .readers
            .iter()
            .find(|r| r.supports_revocation_registry(rev_reg_id))
            .ok_or(VcxLedgerError::UnsupportedLedgerIdentifier(
                rev_reg_id.to_string(),
            ))?;

        reader.get_rev_reg(rev_reg_id, timestamp).await
    }
}

impl MultiLedgerAnoncredsRead {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_reader<T>(mut self, reader: T) -> Self
    where
        T: AnoncredsLedgerRead + AnoncredsLedgerSupport + 'static,
        for<'de> <T as AnoncredsLedgerRead>::RevocationRegistryDefinitionAdditionalMetadata:
            Serialize + Deserialize<'de> + Send + Sync,
    {
        let adaptor = AnoncredsLedgerReadAdaptor { inner: reader };
        self.readers.push(Box::new(adaptor));
        self
    }
}

impl AnoncredsLedgerSupport for MultiLedgerAnoncredsRead {
    fn supports_schema(&self, id: &SchemaId) -> bool {
        self.readers.iter().any(|r| r.supports_schema(id))
    }

    fn supports_credential_definition(&self, id: &CredentialDefinitionId) -> bool {
        self.readers
            .iter()
            .any(|r| r.supports_credential_definition(id))
    }

    fn supports_revocation_registry(&self, id: &RevocationRegistryDefinitionId) -> bool {
        self.readers
            .iter()
            .any(|r| r.supports_revocation_registry(id))
    }
}

#[derive(Debug)]
pub struct AnoncredsLedgerReadAdaptor<T> {
    inner: T,
}

pub trait AnoncredsLedgerReadAdaptorTrait:
    AnoncredsLedgerRead<RevocationRegistryDefinitionAdditionalMetadata = Value> + AnoncredsLedgerSupport
{
}

impl<T> AnoncredsLedgerSupport for AnoncredsLedgerReadAdaptor<T>
where
    T: AnoncredsLedgerSupport,
{
    fn supports_schema(&self, id: &SchemaId) -> bool {
        self.inner.supports_schema(id)
    }
    fn supports_credential_definition(&self, id: &CredentialDefinitionId) -> bool {
        self.inner.supports_credential_definition(id)
    }
    fn supports_revocation_registry(&self, id: &RevocationRegistryDefinitionId) -> bool {
        self.inner.supports_revocation_registry(id)
    }
}

#[async_trait]
impl<T> AnoncredsLedgerRead for AnoncredsLedgerReadAdaptor<T>
where
    T: AnoncredsLedgerRead,
    T::RevocationRegistryDefinitionAdditionalMetadata:
        Serialize + for<'de> Deserialize<'de> + Send + Sync,
{
    type RevocationRegistryDefinitionAdditionalMetadata = Value;

    async fn get_schema(
        &self,
        schema_id: &SchemaId,
        submitter_did: Option<&Did>,
    ) -> VcxLedgerResult<Schema> {
        self.inner.get_schema(schema_id, submitter_did).await
    }

    async fn get_cred_def(
        &self,
        cred_def_id: &CredentialDefinitionId,
        submitter_did: Option<&Did>,
    ) -> VcxLedgerResult<CredentialDefinition> {
        self.inner.get_cred_def(cred_def_id, submitter_did).await
    }

    async fn get_rev_reg_def_json(
        &self,
        rev_reg_id: &RevocationRegistryDefinitionId,
    ) -> VcxLedgerResult<(
        RevocationRegistryDefinition,
        Self::RevocationRegistryDefinitionAdditionalMetadata,
    )> {
        let (reg, meta) = self.inner.get_rev_reg_def_json(rev_reg_id).await?;

        Ok((reg, serde_json::to_value(meta)?))
    }
    async fn get_rev_reg_delta_json(
        &self,
        rev_reg_id: &RevocationRegistryDefinitionId,
        from: Option<u64>,
        to: Option<u64>,
    ) -> VcxLedgerResult<(RevocationRegistryDelta, u64)> {
        #[allow(deprecated)] // TODO - https://github.com/hyperledger/aries-vcx/issues/1309
        self.inner
            .get_rev_reg_delta_json(rev_reg_id, from, to)
            .await
    }

    async fn get_rev_status_list(
        &self,
        rev_reg_id: &RevocationRegistryDefinitionId,
        timestamp: u64,
        rev_reg_def_meta: Option<&Self::RevocationRegistryDefinitionAdditionalMetadata>,
    ) -> VcxLedgerResult<(RevocationStatusList, u64)> {
        let meta = match rev_reg_def_meta {
            Some(v) => Some(serde_json::from_value(v.to_owned())?),
            None => None,
        };

        self.inner
            .get_rev_status_list(rev_reg_id, timestamp, meta.as_ref())
            .await
    }
    async fn get_rev_reg(
        &self,
        rev_reg_id: &RevocationRegistryDefinitionId,
        timestamp: u64,
    ) -> VcxLedgerResult<(RevocationRegistry, u64)> {
        self.inner.get_rev_reg(rev_reg_id, timestamp).await
    }
}

impl<T> AnoncredsLedgerReadAdaptorTrait for AnoncredsLedgerReadAdaptor<T>
where
    T: AnoncredsLedgerRead + AnoncredsLedgerSupport,
    T::RevocationRegistryDefinitionAdditionalMetadata:
        Serialize + for<'de> Deserialize<'de> + Send + Sync,
{
}

#[cfg(test)]
mod unit_tests {
    use async_trait::async_trait;
    use mockall::{mock, predicate::eq};
    use serde_json::json;

    use super::*;

    mock! {
        #[derive(Debug)]
        pub Reader {}
        #[async_trait]
        impl AnoncredsLedgerRead for Reader {
            type RevocationRegistryDefinitionAdditionalMetadata = Value;

            // NOTE: these method signatures were generated as a result of the expanded #[async_trait] form.
            //  this was needed to escape some #[async_trait] compiling issues
            fn get_schema<'life0,'life1,'life2,'async_trait>(&'life0 self,schema_id: &'life1 SchemaId,submitter_did:Option< &'life2 Did> ,) ->  ::core::pin::Pin<Box<dyn ::core::future::Future<Output = VcxLedgerResult<Schema> > + ::core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,Self:'async_trait;
            fn get_cred_def<'life0,'life1,'life2,'async_trait>(&'life0 self,cred_def_id: &'life1 CredentialDefinitionId,submitter_did:Option< &'life2 Did> ,) ->  ::core::pin::Pin<Box<dyn ::core::future::Future<Output = VcxLedgerResult<CredentialDefinition> > + ::core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,Self:'async_trait;
            async fn get_rev_reg_def_json(&self, rev_reg_id: &RevocationRegistryDefinitionId) -> VcxLedgerResult<(RevocationRegistryDefinition, Value)>;
            async fn get_rev_reg_delta_json(&self, rev_reg_id: &RevocationRegistryDefinitionId, from: Option<u64>, to: Option<u64>) -> VcxLedgerResult<(RevocationRegistryDelta, u64)>;
            #[allow(clippy::type_complexity)] // generated
            fn get_rev_status_list<'life0,'life1,'life2,'async_trait>(&'life0 self,rev_reg_id: &'life1 RevocationRegistryDefinitionId,timestamp:u64,rev_reg_def_meta:Option< &'life2 Value>) ->  ::core::pin::Pin<Box<dyn ::core::future::Future<Output = VcxLedgerResult<(RevocationStatusList,u64)> > + ::core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,Self:'async_trait;
            async fn get_rev_reg(&self, rev_reg_id: &RevocationRegistryDefinitionId, timestamp: u64) -> VcxLedgerResult<(RevocationRegistry, u64)>;
        }
        impl AnoncredsLedgerSupport for Reader {
            fn supports_schema(&self, id: &SchemaId) -> bool;
            fn supports_credential_definition(&self, id: &CredentialDefinitionId) -> bool;
            fn supports_revocation_registry(&self, id: &RevocationRegistryDefinitionId) -> bool;
        }
    }

    #[test]
    fn test_anoncreds_supports_schema_if_any_support() {
        let mut reader1 = MockReader::new();
        reader1.expect_supports_schema().return_const(false);
        let mut reader2 = MockReader::new();
        reader2.expect_supports_schema().return_const(false);
        let mut reader3 = MockReader::new();
        reader3.expect_supports_schema().return_const(true);

        // no readers
        let mut reader = MultiLedgerAnoncredsRead::new();
        assert!(!reader.supports_schema(&SchemaId::new_unchecked("")));

        // with reader 1
        reader = reader.register_reader(reader1);
        assert!(!reader.supports_schema(&SchemaId::new_unchecked("")));

        // with reader 1,2
        reader = reader.register_reader(reader2);
        assert!(!reader.supports_schema(&SchemaId::new_unchecked("")));

        // with reader 1,2,3
        reader = reader.register_reader(reader3);
        assert!(reader.supports_schema(&SchemaId::new_unchecked("")));
    }

    #[test]
    fn test_anoncreds_supports_cred_def_if_any_support() {
        let mut reader1 = MockReader::new();
        reader1
            .expect_supports_credential_definition()
            .return_const(false);
        let mut reader2 = MockReader::new();
        reader2
            .expect_supports_credential_definition()
            .return_const(false);
        let mut reader3 = MockReader::new();
        reader3
            .expect_supports_credential_definition()
            .return_const(true);

        // no readers
        let mut reader = MultiLedgerAnoncredsRead::new();
        assert!(!reader.supports_credential_definition(&CredentialDefinitionId::new_unchecked("")));

        // with reader 1
        reader = reader.register_reader(reader1);
        assert!(!reader.supports_credential_definition(&CredentialDefinitionId::new_unchecked("")));

        // with reader 1,2
        reader = reader.register_reader(reader2);
        assert!(!reader.supports_credential_definition(&CredentialDefinitionId::new_unchecked("")));

        // with reader 1,2,3
        reader = reader.register_reader(reader3);
        assert!(reader.supports_credential_definition(&CredentialDefinitionId::new_unchecked("")));
    }

    #[test]
    fn test_anoncreds_supports_rev_reg_if_any_support() {
        let mut reader1 = MockReader::new();
        reader1
            .expect_supports_revocation_registry()
            .return_const(false);
        let mut reader2 = MockReader::new();
        reader2
            .expect_supports_revocation_registry()
            .return_const(false);
        let mut reader3 = MockReader::new();
        reader3
            .expect_supports_revocation_registry()
            .return_const(true);

        // no readers
        let mut reader = MultiLedgerAnoncredsRead::new();
        assert!(!reader
            .supports_revocation_registry(&RevocationRegistryDefinitionId::new_unchecked("")));

        // with reader 1
        reader = reader.register_reader(reader1);
        assert!(!reader
            .supports_revocation_registry(&RevocationRegistryDefinitionId::new_unchecked("")));

        // with reader 1,2
        reader = reader.register_reader(reader2);
        assert!(!reader
            .supports_revocation_registry(&RevocationRegistryDefinitionId::new_unchecked("")));

        // with reader 1,2,3
        reader = reader.register_reader(reader3);
        assert!(
            reader.supports_revocation_registry(&RevocationRegistryDefinitionId::new_unchecked(""))
        );
    }

    #[tokio::test]
    async fn test_get_schema_proxy() {
        let id = SchemaId::new_unchecked(uuid::Uuid::new_v4().to_string());
        let schema: Schema = serde_json::from_value(json!({
            "id": "2hoqvcwupRTUNkXn6ArYzs:2:test-licence:4.4.4",
            "issuerId": "https://example.org/issuers/74acabe2-0edc-415e-ad3d-c259bac04c15",
            "name": "Example schema",
            "version": "0.0.1",
            "attrNames": ["name", "age", "vmax"]
        }))
        .unwrap();

        let mut reader1 = MockReader::new();
        reader1.expect_supports_schema().return_const(false);
        let mut reader2 = MockReader::new();
        reader2.expect_supports_schema().return_const(true);

        let return_schema = schema.clone();
        let expected_id = id.clone();
        reader2
            .expect_get_schema()
            .times(1)
            .withf(move |id, _| id == &expected_id)
            .return_once(move |_, _| {
                let schema = return_schema.clone();
                Box::pin(async { Ok(schema) })
            });

        let reader = MultiLedgerAnoncredsRead::new()
            .register_reader(reader1)
            .register_reader(reader2);

        let actual_schema = reader.get_schema(&id, None).await.unwrap();
        assert_eq!(actual_schema, schema);
    }

    #[tokio::test]
    async fn test_get_cred_def_proxy() {
        let id = CredentialDefinitionId::new_unchecked(uuid::Uuid::new_v4().to_string());
        let cred_def: CredentialDefinition = serde_json::from_value(json!({
            "issuerId":"2hoqvcwupRTUNkXn6ArYzs",
            "id":"V4SGRU86Z58d6TV7PBUe6f:3:CL:47:tag1",
            "schemaId":"47",
            "type":"CL",
            "tag":"tag1",
            "value":{"primary":{"n":"84315068910733942941538809865498212264459549000792879639194636554256493686411316918356364554212859153177297904254803226691261360163017568975126644984686667408908554649062870983454913573717132685671583694031717814141080679830350673819424952039249083306114176943232365022146274429759329160659813641353420711789491657407711667921406386914492317126931232096716361835185621871614892278526459017279649489759883446875710292582916862323180484745724279188511139859961558899492420881308319897939707149225665665750089691347075452237689762438550797003970061912118059748053097698671091262939106899007559343210815224380894488995113","s":"59945496099091529683010866609186265795684084593915879677365915314463441987443154552461559065352420479185095999059693986296256981985940684025427266913415378231498151267007158809467729823827861060943249985214295565954619712884813475078876243461567805408186530126430117091175878433493470477405243290350006589531746383498912735958116529589160259765518623375993634632483497463706818065357079852866706817198531892216731574724567574484572625164896490617904130456390333434781688685711531389963108606060826034467848677030185954836450146848456029587966315982491265726825962242361193382187200762212532433623355158921675478443655","r":{"date":"4530952538742033870264476947349521164360569832293598526413037147348444667070532579011380195228953804587892910083371419619205239227176615305655903664544843027114068926732016634797988559479373173420379032359085866159812396874183493337593923986352373367443489125306598512961937690050408637316178808308468698130916843726350286470498112647012795872536655274223930769557053638433983185163528151238000191689201476902211980373037910868763273992732059619082351273391415938130689371576452787378432526477108387174464183508295202268819673712009207498503854885022563424081101176507778361451123464434452487512804710607807387159128","age":"15503562420190113779412573443648670692955804417106499721072073114914016752313409982751974380400397358007517489487316569306402072592113120841927939614225596950574350717260042735712458756920802142037678828871321635736672163943850608100354921834523463955560137056697629609048487954091228011758465160630168342803951659774432090950114225268433943738713511960512519720523823132697152235977167573478225681125743108316237901888395175219199986619980202460002105538052202194307901021813863765169429019328213794814861353730831662815923471654084390063142965516688500592978949402225958824367010905689069418109693434050714606537583","master_secret":"14941959991861844203640254789113219741444241402376608919648803136983822447657869566295932734574583636024573117873598134469823095273660720727015649700465955361130129864938450014649937111357170711555934174503723640116145690157792143484339964157425981213977310483344564302214614951542098664609872435210449645226834832312148045998264085006307562873923691290268448463268834055489643805348568651181211925383052438130996893167066397253030164424601706641019876113890399331369874738474583032456630131756536716380809815371596958967704560484978381009830921031570414600773593753852611696648360844881738828836829723309887344258937","degree":"67311842668657630299931187112088054454211766880915366228670112262543717421860411017794917555864962789983330613927578732076546462151555711446970436129702520726176833537897538147071443776547628756352432432835899834656529545556299956904072273738406120215054506535933414063527222201017224487746551625599741110865905908376807007226016794285559868443574843566769079505217003255193711592800832900528743423878219697996298053773029816576222817010079313631823474213756143038164999157352990720891769630050634864109412199796030812795554794690666251581638175258059399898533412136730015599390155694930639754604259112806273788686507","name":"40219920836667226846980595968749841377080664270433982561850197901597771608457272395564987365558822976616985553706043669221173208652485993987057610703687751081160705282278075965587059598006922553243119778614657332904443210949022832267152413602769950017196241603290054807405460837019722951966510572795690156337466164027728801433988397914462240536123226066351251559900540164628048512242681757179461134506191038056117474598282470153495944204106741140156797129717000379741035610651103388047392025454843123388667794650989667119908702980140953817053781064048661940525040691050449058181790166453523432033421772638164421440685"},"rctxt":"82211384518226913414705883468527183492640036595481131419387459490531641219140075943570487048949702495964452189878216535351106872901004288800618738558792099255633906919332859771231971995964089510050225371749467514963769062274472125808020747444043470829002911273221886787637565057381007986495929400181408859710441856354570835196999403016572481694010972193591132652075787983469256801030377358546763251500115728919042442285672299538383497028946509014399210152856456151424115321673276206158701693427269192935107281015760959764697283756967072538997471422184132520456634304294669685041869025373789168601981777301930702142717","z":"597630352445077967631363625241310742422278192985165396805410934637833378871905757950406233560527227666603560941363695352019306999881059381981398160064772044355803827657704204341958693819520622597159046224481967205282164000142449330594352761829061889177257816365267888924702641013893644925126158726833651215929952524830180400844722456288706452674387695005404773156271263201912761375013255052598817079064887267420056304601888709527771529363399401864219168554878211931445812982845627741042319366707757767391990264005995464401655063121998656818808854159318505715730088491626077711079755220554530286586303737320102790443"},"revocation":{"g":"1 163FAAD4EB9AA0DF02C19EAC9E91DAFF5C9EEC50B13D59613BB03AC57A864724 1 091121B3A92D2C48A12FB2B6904AFDA9708EAC9CBDF4E9BF988C9071BB4CFEC2 2 095E45DDF417D05FB10933FFC63D474548B7FFFF7888802F07FFFFFF7D07A8A8","g_dash":"1 165E30BED89E08D23FC61685E5F38A65A74342EDF75283BE2E3D7A84D036AC1F 1 07D4894035E05420317656B39B0104E2E6CF372024C8DA9B5E69C05D073DEE17 1 2408458D7F1B790AC27A05055FB7DB562BD51E2597BC3CA5713589716A128647 1 1965D421EA07B38C9C287348BC6AAC53B7FF6E44DE6AC3202F4E62B147019FB3 2 095E45DDF417D05FB10933FFC63D474548B7FFFF7888802F07FFFFFF7D07A8A8 1 0000000000000000000000000000000000000000000000000000000000000000","h":"1 1EC742963B128F781DEC49BF60E9D7D627BE75EE6DB6FC7EC0A4388EB6EDDD5E 1 0A98F72733982BF22B83E40FB03AA339C990424498DFF7D227B75F442F089E71 2 095E45DDF417D05FB10933FFC63D474548B7FFFF7888802F07FFFFFF7D07A8A8","h0":"1 1EBE1D3B82473D8435B13E1B22B9B8A3FFD8251F3FACF423CE3CF0A63AF81D6B 1 10890876E36CCB96308ED4C284CDC4B2B014AE67404207E73F287EC86ACFE809 2 095E45DDF417D05FB10933FFC63D474548B7FFFF7888802F07FFFFFF7D07A8A8","h1":"1 21F6A9DA5A812DB4840340A817788CC84EB3C079E07C9908E9637C0A00F2DD56 1 1B1A0005E895B479500E818FC2280B6D06C088788CCF44C07E94B55941EE85F6 2 095E45DDF417D05FB10933FFC63D474548B7FFFF7888802F07FFFFFF7D07A8A8","h2":"1 180ADD04BFF577BCC49D09F97A9C11347C7A0359A0347DE9F138393CAF5F1F93 1 1044FFDF4AC72BBD8B6CC38D918A7C64A441E53D4561A7F5799B68D48E355294 2 095E45DDF417D05FB10933FFC63D474548B7FFFF7888802F07FFFFFF7D07A8A8","htilde":"1 031D6DDE2A7B05F29EFB574973E6D54AE36B79EBDD0599CD5AD2DF93BDBD0661 1 23A358FEC4883CE1EF44EEC1640B4D4C27FF5C7D64E9798BBF2C5A0D414D1AB5 2 095E45DDF417D05FB10933FFC63D474548B7FFFF7888802F07FFFFFF7D07A8A8","h_cap":"1 1536F787B8F6676E31B769543085CC12E484D6B9A136A29E05723DE993E52C78 1 05EF3C2E5AC1F62132E1F62AC715588203902BCBA8D40203606E6834F9065BB5 1 09878859092CA40C7D5AB4D42F6AFC16987CC90C361F161F9383BCD70F0BD7F0 1 2472E732278D393032B33DEDD2F38F84C3D05E109819E97D462D55822FD14DAA 2 095E45DDF417D05FB10933FFC63D474548B7FFFF7888802F07FFFFFF7D07A8A8 1 0000000000000000000000000000000000000000000000000000000000000000","u":"1 11523B940E760339BBDA36AE6B2DDA570E9CCC2E9314744FCB6C767DF022C5CF 1 1DADE6A6EBFFB2D329A691DB51C3A862F5FBD7D6BD5E594216E613BE882DBC02 1 0E4DE16A4C7514B7F1E09D1253F79B1D3127FD45AB2E535717BA2912F048D587 1 14A1436619A0C1B02302D66D78CE66027A1AAF44FC6FA0BA605E045526A76B76 2 095E45DDF417D05FB10933FFC63D474548B7FFFF7888802F07FFFFFF7D07A8A8 1 0000000000000000000000000000000000000000000000000000000000000000","pk":"1 0CBF9F57DD1607305F2D6297E85CA9B1D71BCBDCE26E10329224C4EC3C0299D6 1 01EE49B4D07D933E518E9647105408758B1D7E977E66E976E4FE4A2E66F8E734 2 095E45DDF417D05FB10933FFC63D474548B7FFFF7888802F07FFFFFF7D07A8A8","y":"1 1D2618B8EA3B4E1C5C8D0382940E34DA19425E3CE69C2F6A55F10352ABDF7BD9 1 1F45619B4247A65FDFE577B6AE40474F53E94A83551622859795E71B44920FA0 1 21324A71042C04555C2C89881F297E6E4FB10BA3949B0C3C345B4E5EE4C48100 1 0FAF04961F119E50C72FF39E7E7198EBE46C2217A87A47C6A6A5BFAB6D39E1EE 2 095E45DDF417D05FB10933FFC63D474548B7FFFF7888802F07FFFFFF7D07A8A8 1 0000000000000000000000000000000000000000000000000000000000000000"}}
        }))
        .unwrap();

        let mut reader1 = MockReader::new();
        reader1
            .expect_supports_credential_definition()
            .return_const(false);
        let mut reader2 = MockReader::new();
        reader2
            .expect_supports_credential_definition()
            .return_const(true);

        let return_cred_def = cred_def.try_clone().unwrap();
        let expected_id = id.clone();
        reader2
            .expect_get_cred_def()
            .times(1)
            .withf(move |id, _| id == &expected_id)
            .return_once(move |_, _| {
                let cred_def = return_cred_def.try_clone().unwrap();
                Box::pin(async { Ok(cred_def) })
            });

        let reader = MultiLedgerAnoncredsRead::new()
            .register_reader(reader1)
            .register_reader(reader2);

        let actual_cred_def = reader.get_cred_def(&id, None).await.unwrap();
        assert_eq!(
            serde_json::to_value(actual_cred_def).unwrap(),
            serde_json::to_value(cred_def).unwrap()
        );
    }

    #[tokio::test]
    async fn test_get_rev_reg_def_proxy() {
        let id = RevocationRegistryDefinitionId::new_unchecked(uuid::Uuid::new_v4().to_string());
        let rev_reg_def: RevocationRegistryDefinition = serde_json::from_value(json!({
            "id": id,
            "issuerId": "did:cheqd:testnet:8bbd2026-03f5-42c7-bf80-09f46fc4d67b",
            "credDefId": "did:cheqd:testnet:8bbd2026-03f5-42c7-bf80-09f46fc4d67b/resources/8372c1bc-907d-44a9-86be-ac3672b26e2e",
            "revocDefType": "CL_ACCUM",
            "tag": "1.0",
            "value": {
                "maxCredNum": 5,
                "publicKeys": {
                "accumKey": {
                "z": "1 10D3560CAE0591EEA7D7A63E1A362FC31448EF321E04FD75F248BBAF02DE9749 1 118C4B0C7F3D86D46C22D62BAE7E613B137A879B50EFDFC56451AB9012BA57A0 1 23D6952F9D058744D4930D1DE6D79548BDCA3EE8BAAF64B712668E52A1290547 1 14C4C4389D92A99C4DA7E6CC2BD0C82E2809D3CD202CD2F0AD6C33D75AA39049 1 174EACBC7981492A791A72D57C6CB9FE488A679C4A5674E4F3C247D73A827384 1 0172B8961122D4D825B282CA1CD1BBC3B8DC459994C9FE2827CDF74B3AB08D38 1 181159044E453DC59FF320E9E08C666176F6B9309E162E2DA4FC1DB3156F7B1F 1 2323CEBFB26C6D28CBAF5F87F155362C6FA14AFA0EBA7DE2B4154FE4082E30FD 1 2354CB1624B42A284B41E5B3B4489C2795DBA9B88A725005555FB698AFF97260 1 07EEEF48EF52E5B15FD4AC28F0DAEDE0A259A27500855992307518A0DBE29A83 1 00FE73BCDB27D1DAD37E4F0E424372CA9548F11B4EC977DCCCC53D99A5C66F36 1 07E9DC0DD2163A66EDA84CD6BF282C7E18CB821762B6047CA1AB9FBE94DC6546"
                }
                },
                "tailsHash": "GW1bmjcMmtHnLwbWrabX4sWYVopJMEvQWgYMAEDmbJS3",
                "tailsLocation": "GW1bmjcMmtHnLwbWrabX4sWYVopJMEvQWgYMAEDmbJS3"
            }
        }))
        .unwrap();
        let meta = json!({
            "foo": "bar",
            uuid::Uuid::new_v4().to_string(): [uuid::Uuid::new_v4().to_string()],
        });

        let mut reader1 = MockReader::new();
        reader1
            .expect_supports_revocation_registry()
            .return_const(false);
        let mut reader2 = MockReader::new();
        reader2
            .expect_supports_revocation_registry()
            .return_const(true);

        let return_def = rev_reg_def.clone();
        let return_meta = meta.clone();
        reader2
            .expect_get_rev_reg_def_json()
            .times(1)
            .with(eq(id.clone()))
            .return_once(move |_| Ok((return_def, return_meta)));

        let reader = MultiLedgerAnoncredsRead::new()
            .register_reader(reader1)
            .register_reader(reader2);

        let (actual_def, actual_meta) = reader.get_rev_reg_def_json(&id).await.unwrap();
        assert_eq!(
            serde_json::to_value(actual_def).unwrap(),
            serde_json::to_value(rev_reg_def).unwrap()
        );
        assert_eq!(
            serde_json::to_value(actual_meta).unwrap(),
            serde_json::to_value(meta).unwrap()
        );
    }

    #[tokio::test]
    async fn test_get_rev_status_list_proxy() {
        let id = RevocationRegistryDefinitionId::new_unchecked(uuid::Uuid::new_v4().to_string());
        let input_timestamp = 978;
        let meta = json!({
            "foo": "bar",
            uuid::Uuid::new_v4().to_string(): [uuid::Uuid::new_v4().to_string()],
        });
        let rev_status_list: RevocationStatusList = serde_json::from_value(json!({
            "issuerId": "did:cheqd:testnet:8bbd2026-03f5-42c7-bf80-09f46fc4d67b",
            "revRegDefId": "4xE68b6S5VRFrKMMG1U95M:4:4xE68b6S5VRFrKMMG1U95M:3:CL:59232:default:CL_ACCUM:4ae1cc6c-f6bd-486c-8057-88f2ce74e960",
            "revocationList": [0, 1, 1, 0],
            "currentAccumulator": "21 124C594B6B20E41B681E92B2C43FD165EA9E68BC3C9D63A82C8893124983CAE94 21 124C5341937827427B0A3A32113BD5E64FB7AB39BD3E5ABDD7970874501CA4897 6 5438CB6F442E2F807812FD9DC0C39AFF4A86B1E6766DBB5359E86A4D70401B0F 4 39D1CA5C4716FFC4FE0853C4FF7F081DFD8DF8D2C2CA79705211680AC77BF3A1 6 70504A5493F89C97C225B68310811A41AD9CD889301F238E93C95AD085E84191 4 39582252194D756D5D86D0EED02BF1B95CE12AED2FA5CD3C53260747D891993C",
            "timestamp": 1669640864
        }))
        .unwrap();
        let output_timestamp = 876;

        let mut reader1 = MockReader::new();
        reader1
            .expect_supports_revocation_registry()
            .return_const(false);
        let mut reader2 = MockReader::new();
        reader2
            .expect_supports_revocation_registry()
            .return_const(true);

        let return_list = rev_status_list.clone();
        let expected_id = id.clone();
        let expected_meta = meta.clone();
        reader2
            .expect_get_rev_status_list()
            .times(1)
            .withf(move |id, ts, meta| {
                id == &expected_id && ts == &input_timestamp && meta == &Some(&expected_meta)
            })
            .return_once(move |_, _, _| {
                Box::pin(async move { Ok((return_list, output_timestamp)) })
            });

        let reader = MultiLedgerAnoncredsRead::new()
            .register_reader(reader1)
            .register_reader(reader2);

        let (actual_list, actual_timestamp) = reader
            .get_rev_status_list(&id, input_timestamp, Some(&meta))
            .await
            .unwrap();
        assert_eq!(
            serde_json::to_value(actual_list).unwrap(),
            serde_json::to_value(rev_status_list).unwrap()
        );
        assert_eq!(
            serde_json::to_value(actual_timestamp).unwrap(),
            serde_json::to_value(output_timestamp).unwrap()
        );
    }

    #[allow(deprecated)] // TODO - https://github.com/hyperledger/aries-vcx/issues/1309
    #[tokio::test]
    async fn test_get_rev_reg_delta_proxy() {
        let id = RevocationRegistryDefinitionId::new_unchecked(uuid::Uuid::new_v4().to_string());
        let rev_reg_delta: RevocationRegistryDelta = serde_json::from_value(json!({
            "value":{"accum":"2 0A0752AD393CCA8E840459E79BCF48F16ECEF17C00E9B639AC6CE2CCC93954C9 2 242D07E4AE3284C1E499D98E4EDF65ACFC0392E64C2BFF55192AC3AE51C3657C 2 165A2D44CAEE9717F1F52CC1BA6F72F39B21F969B3C4CDCA4FB501880F7AD297 2 1B08C9BB4876353F70E4A639F3B41593488B9964D4A56B61B0E1FF8B0FB0A1E7 2 095E45DDF417D05FB10933FFC63D474548B7FFFF7888802F07FFFFFF7D07A8A8 1 0000000000000000000000000000000000000000000000000000000000000000"}
        }))
        .unwrap();
        let from = 123;
        let to = 345;
        let timestamp = 678;

        let mut reader1 = MockReader::new();
        reader1
            .expect_supports_revocation_registry()
            .return_const(false);
        let mut reader2 = MockReader::new();
        reader2
            .expect_supports_revocation_registry()
            .return_const(true);

        let return_delta = rev_reg_delta.clone();
        reader2
            .expect_get_rev_reg_delta_json()
            .times(1)
            .with(eq(id.clone()), eq(Some(from)), eq(Some(to)))
            .return_once(move |_, _, _| Ok((return_delta, timestamp)));

        let reader = MultiLedgerAnoncredsRead::new()
            .register_reader(reader1)
            .register_reader(reader2);

        let (actual_delta, actual_timestamp) = reader
            .get_rev_reg_delta_json(&id, Some(from), Some(to))
            .await
            .unwrap();
        assert_eq!(actual_delta, rev_reg_delta);
        assert_eq!(actual_timestamp, timestamp);
    }

    #[tokio::test]
    async fn test_get_rev_reg_proxy() {
        let id = RevocationRegistryDefinitionId::new_unchecked(uuid::Uuid::new_v4().to_string());
        let rev_reg: RevocationRegistry = serde_json::from_value(json!({
            "value":{"accum":"2 0A0752AD393CCA8E840459E79BCF48F16ECEF17C00E9B639AC6CE2CCC93954C9 2 242D07E4AE3284C1E499D98E4EDF65ACFC0392E64C2BFF55192AC3AE51C3657C 2 165A2D44CAEE9717F1F52CC1BA6F72F39B21F969B3C4CDCA4FB501880F7AD297 2 1B08C9BB4876353F70E4A639F3B41593488B9964D4A56B61B0E1FF8B0FB0A1E7 2 095E45DDF417D05FB10933FFC63D474548B7FFFF7888802F07FFFFFF7D07A8A8 1 0000000000000000000000000000000000000000000000000000000000000000"}
        }))
        .unwrap();
        let to = 345;
        let timestamp = 678;

        let mut reader1 = MockReader::new();
        reader1
            .expect_supports_revocation_registry()
            .return_const(false);
        let mut reader2 = MockReader::new();
        reader2
            .expect_supports_revocation_registry()
            .return_const(true);

        let return_reg = rev_reg.clone();
        reader2
            .expect_get_rev_reg()
            .times(1)
            .with(eq(id.clone()), eq(to))
            .return_once(move |_, _| Ok((return_reg, timestamp)));

        let reader = MultiLedgerAnoncredsRead::new()
            .register_reader(reader1)
            .register_reader(reader2);

        let (actual_reg, actual_timestamp) = reader.get_rev_reg(&id, to).await.unwrap();
        assert_eq!(actual_reg, rev_reg);
        assert_eq!(actual_timestamp, timestamp);
    }
}
