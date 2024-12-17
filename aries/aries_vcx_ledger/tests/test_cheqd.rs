#[cfg(feature = "cheqd")]
mod test_cheqd {
    use std::sync::Arc;

    use anoncreds_types::data_types::identifiers::{
        cred_def_id::CredentialDefinitionId, rev_reg_def_id::RevocationRegistryDefinitionId,
        schema_id::SchemaId,
    };
    use aries_vcx_ledger::ledger::{
        base_ledger::AnoncredsLedgerRead, cheqd::CheqdAnoncredsLedgerRead,
    };
    use chrono::{DateTime, Utc};
    use did_cheqd::resolution::resolver::DidCheqdResolver;
    use serde_json::json;

    #[tokio::test]
    async fn test_resolve_schema_vector() {
        let id = "did:cheqd:testnet:d37eba59-513d-42d3-8f9f-d1df0548b675/resources/\
                  a7e2fc0b-5f6c-466d-911f-3ed9909f98a0";

        let reader =
            CheqdAnoncredsLedgerRead::new(Arc::new(DidCheqdResolver::new(Default::default())));
        let schema = reader
            .get_schema(&SchemaId::new_unchecked(id), None)
            .await
            .unwrap();

        assert_eq!(
            schema.id.0,
            "did:cheqd:testnet:d37eba59-513d-42d3-8f9f-d1df0548b675/resources/\
             a7e2fc0b-5f6c-466d-911f-3ed9909f98a0"
        );
        assert!(schema.seq_no.is_none());
        assert_eq!(
            schema.name,
            "Faber College221a463c-9160-41bd-839c-26c0154e64b4"
        );
        assert_eq!(schema.version, "1.0.0");
        assert_eq!(
            schema.attr_names.0,
            vec!["name".to_string(), "degree".to_string(), "date".to_string()]
        );
        assert_eq!(
            schema.issuer_id.0,
            "did:cheqd:testnet:d37eba59-513d-42d3-8f9f-d1df0548b675"
        );
    }

    #[tokio::test]
    async fn test_resolve_cred_def_vector() {
        let id = "did:cheqd:testnet:e5d13e49-9f5d-4ec1-b0f6-43e43e211fdc/resources/\
                  796f4d32-ceb2-4549-ac2f-5270442066ee";

        let reader =
            CheqdAnoncredsLedgerRead::new(Arc::new(DidCheqdResolver::new(Default::default())));
        let cred_def = reader
            .get_cred_def(&CredentialDefinitionId::new_unchecked(id), None)
            .await
            .unwrap();

        let expected_cred_def = json!({
            "id": "did:cheqd:testnet:e5d13e49-9f5d-4ec1-b0f6-43e43e211fdc/resources/796f4d32-ceb2-4549-ac2f-5270442066ee",
            "schemaId": "did:cheqd:testnet:e5d13e49-9f5d-4ec1-b0f6-43e43e211fdc/resources/441dd8ac-5132-4f64-a899-b95e6631861e",
            "issuerId": "did:cheqd:testnet:e5d13e49-9f5d-4ec1-b0f6-43e43e211fdc",
            "type": "CL",
            "tag": "default",
            "value": {
            "primary": {
            "n": "101775425686705322446321729042513323885366249575341827532852151406029439141945446540758073370857140650173543806620665009319990704696336867406228341102050525062347572470125847326218665243292282264450928863478570405137509031729993280909749863406879170040618211281523178059425360274769809374009133917080596452472404002929533556061638640203000980751755775872098282217341230637597866343991247907713086791452810236342936381540330526648988208357641862302546515645514361066795780511377402835905056549804798920667891392124801399775162573084938362756949887199195057995844386465801420665147294058855188546320799079663835174965161",
            "s": "87624888798698822624423723118022809956578348937987561814948126500576219529138836392203224198856904274580002516694066594875873709134352028429835540535481267491635062762312964551217817275494607969045880749427396603031296648191143611156088073589185938269488488710758969627092198856856923016111781105026554515570693238770759473619839075955094865571664244739744514364819888950198503617844415579864953624998989292124086306326904837728507294638098267220789662137529137088453854836926570472177996704875536555330577801472880881494686752967061366433608898978631273502532992063649958617359359105975313298611541375812686478449038",
            "r": {
            "master_secret": "91924390643616684447850600547636799126754942750676961814085841416767517190041327602185387580658201961018441581408151006589910605450989653472590754296936606411587277500591300185962955561382125964973024308169242022915814560288197996510864084499323589139928149142636050230482921777834206450457769957179088070681863843269743734589083210580654397696548923614457471055030224454430850991434040684872112181784382757277123431379517419634053875223449800478697799219665093330299855414452721993787344846427989123844376930340324838760730841881922239261134345165306574342224223006710126470962389247658692615899692622733364593917564",
            "dataload": "67714311012680607861506009159005649926100729749085079545683454286626632138688065577440485138428200490485338821059098371694307470028480026620243200576189622077496672555428629091259952610415973355058680447309063025743992477107070451623444621247413013233746035427316025697312475570466580668335703497887313077562889740624862997672438829468032595482449521331150586223865869041877654993640507137080181293240650234816519778512756819260970205819993241324592879273813227162717013131055606974830594578099412351827306325727807837670155477487273346541222802392212114521431844422126972523175992819888243089660896279345668836709183"
            },
            "rctxt": "70939857802453506951149531957606930306640909143475371737027498474152925628494791068427574134203017421399121411176717498176846791145767680818780201808144435771494206471213311901071561885391866584823165735626586292923926605780832222900819531483444405585980754893162270863536237119860353096313485759974542267053904367917010014776300492094349532540865655521444795825149399229035168301897753439893554059797022750502266578483363311220307405821402958792359030164217593034199227560018630509640528678991350608730838863727066178052927862093157207477972326979317508513953451471067387162273207269626177777770178388199904693271885",
            "z": "67232321822071084762251502223976923452971987672236221455852097322998038231254751227728590284858878856391984973291870462921522030038401971062122863827666305436738444365691249161806642192223615405177957760215302017704093487843885193856291620515859197624091514138527124658905269978674424356277491558952327833769860308310713639320922734643110516571614031976998124656051686500162012298658320291610287606636134513132238361082981123202624198501889516057149568201642936231925672511435865393828765935813568402464860650327397205857299165873490962876370815478186692229961439123671741775783729284710421491763990499547934996243081"
            }
            }
        });
        assert_eq!(serde_json::to_value(cred_def).unwrap(), expected_cred_def);
    }

    // https://testnet-explorer.cheqd.io/transactions/92C31ED20512FEE73EA4D8A6C8E63E652AA61A14D4F8C00203312EA185419CB9
    #[tokio::test]
    async fn test_resolve_rev_reg_def_vector() {
        let id = "did:cheqd:testnet:8bbd2026-03f5-42c7-bf80-09f46fc4d67b/resources/\
                  4f265d83-4657-4c37-ba80-c66cc399457e";

        let reader =
            CheqdAnoncredsLedgerRead::new(Arc::new(DidCheqdResolver::new(Default::default())));
        let (rev_reg_def, meta) = reader
            .get_rev_reg_def_json(&RevocationRegistryDefinitionId::new_unchecked(id))
            .await
            .unwrap();

        let expected_rev_reg_def = json!({
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
        });
        assert_eq!(
            serde_json::to_value(rev_reg_def).unwrap(),
            expected_rev_reg_def
        );

        assert_eq!(
            meta.resource_name,
            "275990cc056b46176a7122cfd888f46a2bd8e3d45a71d5ff20764a874ed02edd"
        );
    }

    // test status list resolution from credo-ts uploaded vectors
    // https://resolver.cheqd.net/1.0/identifiers/did:cheqd:testnet:8bbd2026-03f5-42c7-bf80-09f46fc4d67b?resourceName=275990cc056b46176a7122cfd888f46a2bd8e3d45a71d5ff20764a874ed02edd&resourceType=anonCredsStatusList&resourceMetadata=true
    // reset:    https://testnet-explorer.cheqd.io/transactions/356F65125E585B9F439C423F2AD7DE73ADF4DC9A0811AA8EE5D03D63B1872DC0
    // 2024-12-04T22:14:55Z
    // update 1: https://testnet-explorer.cheqd.io/transactions/ADF7D562A5005576FA6EF8DC864DAA306EB62C40911FEB5B30C8F98968AE7B51
    // 2024-12-04T22:15:07Z
    // update 2: https://testnet-explorer.cheqd.io/transactions/222FF2D023C2C9A097BB38F3875F072DF8DEC7B0CBD46AC3459C9B4C3C74382F
    // 2024-12-04T22:15:18Z
    // update 3: https://testnet-explorer.cheqd.io/transactions/791D57B8C49C270B3EDA0E9E7E00811CA828190C2D6517FDE8E40CD8FE445E1C
    // 2024-12-04T22:15:30Z
    #[tokio::test]
    async fn test_resolve_rev_status_list_versions() {
        let def_id = "did:cheqd:testnet:8bbd2026-03f5-42c7-bf80-09f46fc4d67b/resources/\
                      4f265d83-4657-4c37-ba80-c66cc399457e";
        let def_id = RevocationRegistryDefinitionId::new_unchecked(def_id);

        let init_time = DateTime::parse_from_rfc3339("2024-12-04T22:14:55Z")
            .unwrap()
            .timestamp() as u64;
        let update1_time = DateTime::parse_from_rfc3339("2024-12-04T22:15:07Z")
            .unwrap()
            .timestamp() as u64;
        let update2_time = DateTime::parse_from_rfc3339("2024-12-04T22:15:18Z")
            .unwrap()
            .timestamp() as u64;
        let update3_time = DateTime::parse_from_rfc3339("2024-12-04T22:15:30Z")
            .unwrap()
            .timestamp() as u64;

        let reader =
            CheqdAnoncredsLedgerRead::new(Arc::new(DidCheqdResolver::new(Default::default())));

        let def_meta = reader.get_rev_reg_def_json(&def_id).await.unwrap().1;

        // scenario 1: get most recent
        let now = Utc::now().timestamp() as u64;
        let (status_list, update_time) = reader
            .get_rev_status_list(&def_id, now, Some(&def_meta))
            .await
            .unwrap();
        assert_eq!(update_time, update3_time);
        assert_eq!(
            serde_json::to_value(status_list).unwrap(),
            json!({
            "issuerId": "did:cheqd:testnet:8bbd2026-03f5-42c7-bf80-09f46fc4d67b",
            "revRegDefId": "did:cheqd:testnet:8bbd2026-03f5-42c7-bf80-09f46fc4d67b/resources/4f265d83-4657-4c37-ba80-c66cc399457e",
            "revocationList": [1,1,1,1,0],
            "currentAccumulator": "21 114BE4F2BBAAF18F07E994D74B28347FA0BEC500A616B47F57F2E0B0864F7602E 21 12AB68E307C5F2AA30F34A03ADB298C7F4C02555649E510919979C2AEB49CCDF1 6 5FB9FB957339A842130C84FC98240A163E56DC58B96423F1EFD53E9106671B94 4 28F2F8297E345FFF55CDEE87C83DE471486826C91EBBA2C39570A46013B5BFBA 6 565A830A4358E1F6F21A10804C23E36D739B5630C6A188D760F4B6F434D1311D 4 14F87165B42A780974AC70669DC3CF629F1103DF73AE15AC11A1151883A91941",
            "timestamp": update3_time
            })
        );

        // scenario 2: between update 2 & 3
        let (status_list, update_time) = reader
            .get_rev_status_list(&def_id, update2_time + 3, Some(&def_meta))
            .await
            .unwrap();
        assert_eq!(update_time, update2_time);
        assert_eq!(
            serde_json::to_value(status_list).unwrap(),
            json!({
            "issuerId": "did:cheqd:testnet:8bbd2026-03f5-42c7-bf80-09f46fc4d67b",
            "currentAccumulator": "21 125DF938B3B772619CB43E561D69004CF09667376E9CD53C818D84860BAE3D1D9 21 11ECFC5F9B469AC74E2A0E329F86C6E60B423A53CAC5AE7A4DBE7A978BFFC0DA1 6 6FAD628FED470FF640BF2C5DB57C2C18D009645DBEF15D4AF710739D2AD93E2D 4 22093A3300411B059B8BB7A8C3296A2ED9C4C8E00106C3B2BAD76E25AC792063 6 71D70ECA81BCE610D1C22CADE688AF4A122C8258E8B306635A111D0A35A7238A 4 1E80F38ABA3A966B8657D722D4E956F076BB2F5CCF36AA8942E65500F8898FF3",
            "revRegDefId": "did:cheqd:testnet:8bbd2026-03f5-42c7-bf80-09f46fc4d67b/resources/4f265d83-4657-4c37-ba80-c66cc399457e",
            "revocationList": [1,1,1,0,0],
            "timestamp": update2_time
            })
        );

        // scenario 3: between update 1 & 2
        let (status_list, update_time) = reader
            .get_rev_status_list(&def_id, update1_time + 3, Some(&def_meta))
            .await
            .unwrap();
        assert_eq!(update_time, update1_time);
        assert_eq!(
            serde_json::to_value(status_list).unwrap(),
            json!({
            "issuerId": "did:cheqd:testnet:8bbd2026-03f5-42c7-bf80-09f46fc4d67b",
            "currentAccumulator": "21 136FA865B5CD0AEA1DA05BE412C6E06C23066C338D39C5B79C5E1AE1D5BA20AAA 21 124182E098BE418B9DBECF600EEA3D070EDB85D6B412EE75B4B43C440FEA2E631 6 669D66FB3BC245B4EF892B8DB5A330ACA6A4CE6706FB58D9B487C0487DBB5C04 4 2C5C9551DFE2A4AE71D355DD3A981F155F51B9BCF8E2ED8B8263726DDF60D09C 6 7243CF31A80313C254F51D2B0A3573320B885178F36F4AE1E8FF4A520EF9CDCA 4 1B8DBE9563FAD9FBF8B75BCE41C9425E1D15EE0B3D195D0A86AD8A2C91D5BB73",
            "revRegDefId": "did:cheqd:testnet:8bbd2026-03f5-42c7-bf80-09f46fc4d67b/resources/4f265d83-4657-4c37-ba80-c66cc399457e",
            "revocationList": [1,1,0,0,0],
            "timestamp": update1_time
            })
        );

        // scenario 4: between init & update 1
        let (status_list, update_time) = reader
            .get_rev_status_list(&def_id, init_time + 3, Some(&def_meta))
            .await
            .unwrap();
        assert_eq!(update_time, init_time);
        assert_eq!(
            serde_json::to_value(status_list).unwrap(),
            json!({
            "issuerId": "did:cheqd:testnet:8bbd2026-03f5-42c7-bf80-09f46fc4d67b",
            "currentAccumulator": "1 0443A0BC791EE82B8F34066404B36E81E0CE68B64BD2A48A55587E4585B16CCA 1 0343A5D644B28DCC0EAF9C6D3E104DC0F61FCD711AFE93DB67031905DAA5F654 1 02CE577295DF112BB2C7F16250D4593FC922B074436EC0F4F124E2409EF99785 1 1692EE5DFE9885809DA503A2EEDC4EECDA5D7D415C743E3931576EFD72FB51AC 2 095E45DDF417D05FB10933FFC63D474548B7FFFF7888802F07FFFFFF7D07A8A8 1 0000000000000000000000000000000000000000000000000000000000000000",
            "revRegDefId": "did:cheqd:testnet:8bbd2026-03f5-42c7-bf80-09f46fc4d67b/resources/4f265d83-4657-4c37-ba80-c66cc399457e",
            "revocationList": [0,0,0,0,0],
            "timestamp": init_time
            })
        );
    }
}
