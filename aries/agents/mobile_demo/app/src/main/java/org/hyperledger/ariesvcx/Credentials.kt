package org.hyperledger.ariesvcx

data class Credential(
    val attrs: Map<String, String>,
    val cred_def_id: String,
    val cred_rev_id: Any?,
    val referent: String,
    val rev_reg_id: Any?,
    val schema_id: String
)
