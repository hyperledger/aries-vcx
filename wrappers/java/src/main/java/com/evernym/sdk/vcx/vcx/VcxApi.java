package com.evernym.sdk.vcx.vcx;


import com.evernym.sdk.vcx.LibVcx;
import com.evernym.sdk.vcx.ParamGuard;
import com.evernym.sdk.vcx.VcxException;
import com.evernym.sdk.vcx.VcxJava;
import com.sun.jna.Callback;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.concurrent.CompletableFuture;

public class VcxApi extends VcxJava.API {
    private static final Logger logger = LoggerFactory.getLogger("VcxApi");
    private VcxApi() {
    }


    private static Callback cmdHandleErrCodeCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "]");
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            Integer result = err;
            future.complete(result);
        }
    };

    public static void vcxInitCore(String configJson) throws VcxException {
        ParamGuard.notNullOrWhiteSpace(configJson, "configJson");
        logger.debug("vcxInitCore() called with: configJson = [****]");
        int result = LibVcx.api.vcx_init_core(configJson);
        checkResult(result);
    }

    public static CompletableFuture<Integer> vcxOpenPool() throws VcxException {
        logger.debug("vcxOpenPool()");
        CompletableFuture<Integer> future = new CompletableFuture<Integer>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_open_pool(
                commandHandle,
                cmdHandleErrCodeCB);
        checkResult(result);

        return future;
    }

    public static CompletableFuture<Integer> vcxOpenWallet() throws VcxException {
        logger.debug("vcxOpenWallet()");
        CompletableFuture<Integer> future = new CompletableFuture<Integer>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_open_wallet(
                commandHandle,
                cmdHandleErrCodeCB);
        checkResult(result);

        return future;
    }

    public static CompletableFuture<Integer> vcxUpdateWebhookUrl(String notificationWebhookUrl) throws VcxException {
        ParamGuard.notNullOrWhiteSpace(notificationWebhookUrl, "notificationWebhookUrl");
        logger.debug("vcxUpdateWebhookUrl() called with: notificationWebhookUrl = [" + notificationWebhookUrl + "]");
        CompletableFuture<Integer> future = new CompletableFuture<Integer>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_update_webhook_url(
                commandHandle,
                notificationWebhookUrl,
                cmdHandleErrCodeCB);
        checkResult(result);

        return future;
    }

    public static int vcxShutdown(Boolean deleteWallet) throws VcxException {
        logger.debug("vcxShutdown() called with: deleteWallet = [" + deleteWallet + "]");
        int result = LibVcx.api.vcx_shutdown(deleteWallet);
        checkResult(result);
        return result;
    }

    public static String vcxVersion() throws VcxException {
        logger.debug("vcxVersion()");
        return LibVcx.api.vcx_version();
    }

    public static String vcxErrorCMessage(int errorCode) {
        logger.debug("vcxErrorCMessage() called with: errorCode = [" + errorCode + "]");
        return LibVcx.api.vcx_error_c_message(errorCode);

    }

}