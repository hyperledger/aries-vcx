package com.evernym.sdk.vcx.token;


import com.evernym.sdk.vcx.LibVcx;
import com.evernym.sdk.vcx.VcxException;
import com.evernym.sdk.vcx.VcxJava;
import com.sun.jna.Callback;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.concurrent.CompletableFuture;

public class TokenApi extends VcxJava.API {

    private TokenApi() {
    }

    private static final Logger logger = LoggerFactory.getLogger("TokenApi");
    private static Callback vcxTokenCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err, String tokenInfo) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], tokenInfo = [****]");
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;

            future.complete(tokenInfo);
        }
    };
}
