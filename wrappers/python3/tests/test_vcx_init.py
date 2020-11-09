import pytest
from vcx.error import ErrorCode, VcxError, error_message
from vcx.api.wallet import Wallet
from vcx.api.connection import Connection


@pytest.mark.asyncio
@pytest.mark.usefixtures('vcx_init_test_mode')
async def test_vcx_init():
    pass


@pytest.mark.asyncio
async def test_error_message(vcx_init_test_mode):
    assert error_message(ErrorCode.NotReady) == 'Object not ready for specified action'


