use indy_api_types::IndyError;
use indy_api_types::errors::{IndyErrorKind, IndyResult, IndyResultExt};
use cosmrs::rpc;
use prost::Message;

pub fn check_proofs(
    result: &rpc::endpoint::abci_query::Response,
) -> IndyResult<()> {
    // Decode state proofs

    // Decode proof for inner ival tree
    let proof_op_0 = &result.response.proof.as_ref().ok_or(
        IndyError::from_msg(
            IndyErrorKind::InvalidStructure,
            "The proof for inner ival tree is absent but should be placed"
        ))?;
    let proof_op_0 = &proof_op_0.ops[0].clone();
    let proof_0_data_decoded =
        ics23::CommitmentProof::decode(proof_op_0.data.as_slice()).to_indy(
                IndyErrorKind::InvalidStructure,
                "The proof for inner ival tree cannot be decoded into ics23::CommitmentProof"
        )?;

    // Decode proof for outer `ics23:simple` tendermint tree)
    let proof_op_1 = result.response.proof.as_ref().ok_or(
        IndyError::from_msg(
            IndyErrorKind::InvalidStructure,
            "The proof for outer ics23:simple is absent but should be placed"
        ))?;
    let proof_op_1 = &proof_op_1.ops[1].clone();
    let proof_1_data_decoded =
        ics23::CommitmentProof::decode(proof_op_1.data.as_slice()).to_indy(
                IndyErrorKind::InvalidStructure,
                "The proof for outer ics23:simple cannot be decoded into ics23::CommitmentProof"
        )?;

    // Get a root hash for the inner ival tree from the outer tree proof
    let proof_1_existence = if let Some(ics23::commitment_proof::Proof::Exist(ex)) =
    proof_1_data_decoded.proof.clone()
    {
        ex
    } else {
        let proof_op_1_str = serde_json::to_string(proof_op_1).to_indy(
            IndyErrorKind::InvalidState,
            "Cannot serialize object with proof for outer `ics23:simple` tendermint tree"
        )?;
        return Err(IndyError::from_msg(
            IndyErrorKind::InvalidStructure,
            format!(
                "Commitment proof has an incorrect format {}",
                proof_op_1_str
            ),
        ));
    };
    let proof_0_root = proof_1_existence.clone().value;

    // Check state proofs 0 (inner iavl tree)
    let is_proof_correct = match proof_0_data_decoded.proof {
        Some(ics23::commitment_proof::Proof::Exist(_)) => {
            ics23::verify_membership(
                &proof_0_data_decoded, // proof for verification
                &ics23::iavl_spec(), // tree specification
                &proof_0_root, // value root hash in the inner ival tree (value for outer tree)
                &proof_op_0.key, // key for the inner ival tree
                &result.response.value, // received value
            )
        }
        Some(ics23::commitment_proof::Proof::Nonexist(_)) => {
            ics23::verify_non_membership(
                &proof_0_data_decoded, // proof for verification
                &ics23::iavl_spec(), // tree specification
                &proof_0_root, // value root hash in the inner ival tree
                &proof_op_0.key // key for the inner ival tree
            )
        }
        _ => {false}
    };

    if !is_proof_correct {
        let proof_op_0_str = serde_json::to_string(proof_op_0).to_indy(
            IndyErrorKind::InvalidState,
            "Cannot serialize object with proof for inner ival tree"
        )?;
        return Err(IndyError::from_msg(
            IndyErrorKind::InvalidStructure,
            format!(
                "Commitment proof 0 is incorrect {}",
                proof_op_0_str
            ),
        ));
    }

    // Should be output from light client
    // Calculate a root hash for the outer tree
    let proof_1_root = ics23::calculate_existence_root(&proof_1_existence.clone())
        .map_err(|er | IndyError::from_msg(
        IndyErrorKind::InvalidStructure,
        format!("Commitment proof has an incorrect format {}", er)))?;

    // Check state proofs 1 (outer `ics23:simple` tendermint tree)
    if !ics23::verify_membership(
        &proof_1_data_decoded, // proof for verification
        &ics23::tendermint_spec(), // tree specification
        &proof_1_root,  // root hash for the outer tree
        &proof_op_1.key, // key for the outer tree
        &proof_0_root, // inner tree root hash in the outer tree (should exist)
    ) {
        let proof_op_1_str = serde_json::to_string(proof_op_1).to_indy(
            IndyErrorKind::InvalidState,
            "Cannot serialize object with proof for outer `ics23:simple` tendermint tree"
        )?;
        return Err(IndyError::from_msg(
            IndyErrorKind::InvalidStructure,
            format!(
                "Commitment proof 1 is incorrect {}",
                proof_op_1_str
            ),
        ));
    }

    Ok(())
}
