package com.evernym.sdk.vcx;

import com.evernym.sdk.vcx.utils.UtilsApi;
import com.evernym.sdk.vcx.vcx.VcxApi;
import com.evernym.sdk.vcx.wallet.WalletApi;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import java.util.concurrent.ExecutionException;

public class VcxApiTest {
    @BeforeEach
    void setup() throws Exception {
        System.setProperty(org.slf4j.impl.SimpleLogger.DEFAULT_LOG_LEVEL_KEY, "DEBUG");
        if (!TestHelper.vcxInitialized) {
            VcxApi.vcxInitCore(TestHelper.VCX_CONFIG_TEST_MODE);
            TestHelper.vcxInitialized = true;
        }
    }

    @Test
    @DisplayName("error message")
    void vcxErrorMessage() throws VcxException, ExecutionException, InterruptedException {
        String errorCMessage = VcxApi.vcxErrorCMessage(0);
        assert (errorCMessage.equals("Success"));
    }

    @Test
    @DisplayName("error message 1")
    void vcxUnknownErrorMessage() throws VcxException, ExecutionException, InterruptedException {
        String errorCMessage = VcxApi.vcxErrorCMessage(1001);
        assert (errorCMessage.equals("Unknown Error"));
    }


}
