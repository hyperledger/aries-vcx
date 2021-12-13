use vdrtoolsrs::{future::Future, cheqd_ledger, IndyError};

pub fn build_msg_send(
    from: &str,
    to: &str,
    amount: &str,
    denom: &str,
) -> Result<Vec<u8>, IndyError> {
    cheqd_ledger::bank::build_msg_send(from, to, amount, denom).wait()
}

pub fn parse_msg_send_resp(commit_resp: &str) -> Result<String, IndyError> {
    cheqd_ledger::bank::parse_msg_send_resp(commit_resp).wait()
}

pub fn bank_build_query_balance(
    address: &str,
    denom: &str,
) -> Result<String, IndyError> {
    cheqd_ledger::bank::build_query_balance(address, denom).wait()
}

pub fn parse_query_balance_resp(commit_resp: &str) -> Result<String, IndyError> {
    cheqd_ledger::bank::parse_query_balance_resp(commit_resp).wait()
}
