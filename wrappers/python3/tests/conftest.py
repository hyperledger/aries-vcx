import logging
import pytest
import time
from vcx.common import shutdown as vcx_shutdown

flag = False

logging.basicConfig(level=logging.DEBUG)


@pytest.mark.asyncio
@pytest.fixture
async def vcx_init_test_mode():
    global flag

    if not flag:
        # todo: should make FFI call to set up aries-vcx into testing mode
        flag = True


@pytest.fixture
async def cleanup():

    def _shutdown(erase):
        global flag
        vcx_shutdown(erase)
        if flag:
            flag = False

    return _shutdown


@pytest.fixture(scope='session', autouse=True)
def wait_libindy():
    yield
    time.sleep(1) # FIXME IS-1060

