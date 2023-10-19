use mediation::didcomm_types::PickupMsgEnum;

use super::utils::prelude::*;

pub async fn handle_pickup_protocol(
    _agent: ArcAgent<impl BaseWallet + 'static, impl MediatorPersistence>,
    _pickup_msg: PickupMsgEnum,
) -> Result<EncryptionEnvelope, String> {
    todo!()
}
