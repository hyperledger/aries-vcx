package com.evernym.sdk.vcx.credential;

import com.evernym.sdk.vcx.LibVcx;
import com.evernym.sdk.vcx.ParamGuard;
import com.evernym.sdk.vcx.VcxException;
import com.evernym.sdk.vcx.VcxJava;
import com.sun.jna.Callback;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.concurrent.CompletableFuture;

public class CredentialApi extends VcxJava.API {

    private static final Logger logger = LoggerFactory.getLogger("CredentialApi");
    private CredentialApi() {
    }

    private static Callback vcxCredentialCreateWithMsgidCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int command_handle, int err, int credentialHandle, String offer) {
            logger.debug("callback() called with: command_handle = [" + command_handle + "], err = [" + err + "], credentialHandle = [" + credentialHandle + "], offer = [****]");
            CompletableFuture<GetCredentialCreateMsgidResult> future = (CompletableFuture<GetCredentialCreateMsgidResult>) removeFuture(command_handle);
            if (!checkCallback(future, err)) return;
            GetCredentialCreateMsgidResult result = new GetCredentialCreateMsgidResult(credentialHandle, offer);
            future.complete(result);
        }
    };

    public static CompletableFuture<GetCredentialCreateMsgidResult> credentialCreateWithMsgid(
            String sourceId,
            int connectionHandle,
            String msgId
    ) throws VcxException {
        ParamGuard.notNullOrWhiteSpace(sourceId, "sourceId");
        ParamGuard.notNullOrWhiteSpace(msgId, "msgId");
        logger.debug("credentialCreateWithMsgid() called with: sourceId = [" + sourceId + "], connectionHandle = [" + connectionHandle + "], msgId = [" + msgId + "]");
        CompletableFuture<GetCredentialCreateMsgidResult> future = new CompletableFuture<GetCredentialCreateMsgidResult>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_credential_create_with_msgid(
                commandHandle,
                sourceId,
                connectionHandle,
                msgId,
                vcxCredentialCreateWithMsgidCB);
        checkResult(result);

        return future;

    }

    private static Callback vcxCredentialSendRequestCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int command_handle, int err) {
            logger.debug("callback() called with: command_handle = [" + command_handle + "], err = [" + err + "]");
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(command_handle);
            if (!checkCallback(future,err)) return;
            // returning empty string from here because we don't want to complete future with null
            future.complete("");
        }
    };

    public static CompletableFuture<String> credentialSendRequest(
            int credentialHandle,
            int connectionHandle,
            int paymentHandle
    ) throws VcxException {
        logger.debug("credentialSendRequest() called with: credentialHandle = [" + credentialHandle + "], connectionHandle = [" + connectionHandle + "], paymentHandle = [" + paymentHandle + "]");
        CompletableFuture<String> future = new CompletableFuture<String>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_credential_send_request(
                commandHandle,
                credentialHandle,
                connectionHandle,
                paymentHandle,
                vcxCredentialSendRequestCB);
        checkResult(result);

        return future;

    }

    public static CompletableFuture<String> credentialGetRequestMsg(
            int credentialHandle,
            String myPwDid,
            String theirPwDid,
            int paymentHandle
    ) throws VcxException {
        logger.debug("credentialGetRequestMsg() called with: credentialHandle = [" + credentialHandle + "], myPwDid = [" + myPwDid + "], theirPwDid = [" + theirPwDid + "], paymentHandle = [" + paymentHandle + "]");
        CompletableFuture<String> future = new CompletableFuture<String>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_credential_get_request_msg(
                commandHandle,
                credentialHandle,
                myPwDid,
                theirPwDid,
                paymentHandle,
                vcxCredentialStringCB);
        checkResult(result);

        return future;

    }

    private static Callback vcxCredentialStringCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int command_handle, int err, String stringData) {
            logger.debug("callback() called with: command_handle = [" + command_handle + "], err = [" + err + "], string = [" + stringData + "]");
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(command_handle);
            if (!checkCallback(future, err)) return;
            future.complete(stringData);
        }
    };

    public static CompletableFuture<String> credentialSerialize(
            int credentailHandle
    ) throws VcxException {
        logger.debug("credentialSerialize() called with: credentailHandle = [" + credentailHandle + "]");
        CompletableFuture<String> future = new CompletableFuture<String>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_credential_serialize(commandHandle,
                credentailHandle,
                vcxCredentialStringCB);
        checkResult(result);

        return future;

    }

    private static Callback vcxCredentialDeserializeCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int command_handle, int err, int credentialHandle) {
            logger.debug("callback() called with: command_handle = [" + command_handle + "], err = [" + err + "], credentialHandle = [" + credentialHandle + "]");
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(command_handle);
            if (!checkCallback(future, err)) return;
            Integer result = credentialHandle;
            future.complete(result);
        }
    };

    public static CompletableFuture<Integer> credentialDeserialize(
            String serializedCredential
    ) throws VcxException {
        ParamGuard.notNull(serializedCredential, "serializedCredential");
        logger.debug("credentialDeserialize() called with: serializedCredential = [****]");
        CompletableFuture<Integer> future = new CompletableFuture<Integer>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_credential_deserialize(commandHandle,
                serializedCredential,
                vcxCredentialDeserializeCB);
        checkResult(result);

        return future;

    }

    private static Callback vcxGetCredentialCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int command_handle, int err, String credential) {
            logger.debug("callback() called with: command_handle = [" + command_handle + "], err = [" + err + "], credential = [****]");
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(command_handle);
            if (!checkCallback(future, err)) return;
            future.complete(credential);
        }
    };

    public static CompletableFuture<String> getCredential(
            int credentialHandle
    ) throws VcxException {
        ParamGuard.notNull(credentialHandle, "credentialHandle");
        logger.debug("getCredential() called with: credentialHandle = [" + credentialHandle + "]");
        CompletableFuture<String> future = new CompletableFuture<String>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_get_credential(commandHandle, credentialHandle, vcxGetCredentialCB);
        checkResult(result);

        return future;
    }

    private static Callback vcxDeleteCredentialCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int command_handle, int err) {
            logger.debug("callback() called with: command_handle = [" + command_handle + "], err = [" + err + "]");
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(command_handle);
            if (!checkCallback(future,err)) return;
            // returning empty string from here because we don't want to complete future with null
            future.complete("");
        }
    };

    public static CompletableFuture<String> deleteCredential(
            int credentialHandle
    ) throws VcxException {
        ParamGuard.notNull(credentialHandle, "credentialHandle");
        logger.debug("deleteCredential() called with: credentialHandle = [" + credentialHandle + "]");
        CompletableFuture<String> future = new CompletableFuture<String>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_delete_credential(commandHandle, credentialHandle, vcxDeleteCredentialCB);
        checkResult(result);

        return future;
    }

    private static Callback vcxCredentialUpdateStateCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int command_handle, int err, int state) {
            logger.debug("callback() called with: command_handle = [" + command_handle + "], err = [" + err + "], state = [" + state + "]");
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(command_handle);
            if (!checkCallback(future, err)) return;
            Integer result = state;
            future.complete(result);
        }
    };

    public static CompletableFuture<Integer> credentialUpdateStateV2(
            int credentialHandle,
            int connectionHandle
    ) throws VcxException {
        ParamGuard.notNull(credentialHandle, "credentialHandle");
        logger.debug("credentialUpdateStateV2() called with: credentialHandle = [" + credentialHandle + "], connectionHandle = [" + connectionHandle + "]");
        CompletableFuture<Integer> future = new CompletableFuture<Integer>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_v2_credential_update_state(commandHandle, credentialHandle, connectionHandle, vcxCredentialUpdateStateCB);
        checkResult(result);

        return future;
    }

    public static CompletableFuture<Integer> credentialUpdateStateWithMessageV2(
            int credentialHandle,
            int connectionHandle,
            String message
    ) throws VcxException {
        ParamGuard.notNull(credentialHandle, "credentialHandle");
        ParamGuard.notNull(connectionHandle, "credentialHandle");
        logger.debug("credentialUpdateStateWithMessageV2() called with: credentialHandle = [" + credentialHandle + "], connectionHandle = [" + connectionHandle + "]");
        CompletableFuture<Integer> future = new CompletableFuture<Integer>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_v2_credential_update_state_with_message(commandHandle, credentialHandle, connectionHandle, message, vcxCredentialUpdateStateCB);
        checkResult(result);

        return future;
    }

    private static Callback vcxCredentialGetStateCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int command_handle, int err, int state) {
            logger.debug("callback() called with: command_handle = [" + command_handle + "], err = [" + err + "], state = [" + state + "]");
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(command_handle);
            if (!checkCallback(future, err)) return;
            Integer result = state;
            future.complete(result);
        }
    };

    public static CompletableFuture<Integer> credentialGetState(
            int credentialHandle
    ) throws VcxException {
        ParamGuard.notNull(credentialHandle, "credentialHandle");
        logger.debug("credentialGetState() called with: credentialHandle = [" + credentialHandle + "]");
        CompletableFuture<Integer> future = new CompletableFuture<Integer>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_credential_get_state(commandHandle, credentialHandle, vcxCredentialGetStateCB);
        checkResult(result);

        return future;
    }

    private static Callback vcxCredentialGetAttributesCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int command_handle, int err, String attributes) {
            logger.debug("vcxCredentialGetAttributesCB() called with: command_handle = [" + command_handle + "], err = [" + err + "], attributes = [" + attributes + "]");
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(command_handle);
            if (!checkCallback(future, err)) return;
            future.complete(attributes);
        }
    };

    public static CompletableFuture<String> credentialGetAttributes(
            int credentialHandle
    ) throws VcxException {
        ParamGuard.notNull(credentialHandle, "credentialHandle");
        logger.debug("getAttributes() called with: credentialHandle = [" + credentialHandle + "]");
        CompletableFuture<String> future = new CompletableFuture<String>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_credential_get_attributes(commandHandle, credentialHandle, vcxCredentialGetAttributesCB);
        checkResult(result);

        return future;
    }

    private static Callback vcxCredentialGetAttachmentCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int command_handle, int err, String attachment) {
            logger.debug("vcxCredentialGetAttachmentCB() called with: command_handle = [" + command_handle + "], err = [" + err + "], attachment = [" + attachment + "]");
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(command_handle);
            if (!checkCallback(future, err)) return;
            future.complete(attachment);
        }
    };

    public static CompletableFuture<String> credentialGetAttachment(
            int credentialHandle
    ) throws VcxException {
        ParamGuard.notNull(credentialHandle, "credentialHandle");
        logger.debug("getAttachment() called with: credentialHandle = [" + credentialHandle + "]");
        CompletableFuture<String> future = new CompletableFuture<String>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_credential_get_attachment(commandHandle, credentialHandle, vcxCredentialGetAttachmentCB);
        checkResult(result);

        return future;
    }

    private static Callback credentialGetTailsLocationCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int command_handle, int err, String tailsLocation) {
            logger.debug("credentialGetTailsLocationCB() called with: command_handle = [" + command_handle + "], err = [" + err + "], tailsLocation = [" + tailsLocation + "]");
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(command_handle);
            if (!checkCallback(future, err)) return;
            future.complete(tailsLocation);
        }
    };

    public static CompletableFuture<String> credentialGetTailsLocation(
            int credentialHandle
    ) throws VcxException {
        ParamGuard.notNull(credentialHandle, "credentialHandle");
        logger.debug("credentialGetTailsLocation() called with: credentialHandle = [" + credentialHandle + "]");
        CompletableFuture<String> future = new CompletableFuture<String>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_credential_get_tails_location(commandHandle, credentialHandle, credentialGetTailsLocationCB);
        checkResult(result);

        return future;
    }

    private static Callback credentialGetTailsHashCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int command_handle, int err, String tailsHash) {
            logger.debug("credentialGetTailsHashCB() called with: command_handle = [" + command_handle + "], err = [" + err + "], tailsHash = [" + tailsHash + "]");
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(command_handle);
            if (!checkCallback(future, err)) return;
            future.complete(tailsHash);
        }
    };

    public static CompletableFuture<String> credentialGetTailsHash(
            int credentialHandle
    ) throws VcxException {
        ParamGuard.notNull(credentialHandle, "credentialHandle");
        logger.debug("credentialGetTailsHash() called with: credentialHandle = [" + credentialHandle + "]");
        CompletableFuture<String> future = new CompletableFuture<String>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_credential_get_tails_hash(commandHandle, credentialHandle, credentialGetTailsHashCB);
        checkResult(result);

        return future;
    }

    private static Callback credentialGetRevRegIdCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int command_handle, int err, String revRegId) {
            logger.debug("credentialGetRevRegIdCB() called with: command_handle = [" + command_handle + "], err = [" + err + "], revRegId = [" + revRegId + "]");
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(command_handle);
            if (!checkCallback(future, err)) return;
            future.complete(revRegId);
        }
    };

    public static CompletableFuture<String> credentialGetRevRegId(
            int credentialHandle
    ) throws VcxException {
        ParamGuard.notNull(credentialHandle, "credentialHandle");
        logger.debug("credentialGetRevRegId() called with: credentialHandle = [" + credentialHandle + "]");
        CompletableFuture<String> future = new CompletableFuture<String>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_credential_get_rev_reg_id(commandHandle, credentialHandle, credentialGetRevRegIdCB);
        checkResult(result);

        return future;
    }

    private static Callback credentialIsRevokableCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int command_handle, int err, boolean revokable) {
            logger.debug("credentialIsRevokableCB() called with: command_handle = [" + command_handle + "], err = [" + err + "], revokable = [" + revokable + "]");
            CompletableFuture<Boolean> future = (CompletableFuture<Boolean>) removeFuture(command_handle);
            if (!checkCallback(future, err)) return;
            future.complete(revokable);
        }
    };

    public static CompletableFuture<Boolean> credentialIsRevokable(
            int credentialHandle
    ) throws VcxException {
        ParamGuard.notNull(credentialHandle, "credentialHandle");
        logger.debug("credentialIsRevokable() called with: credentialHandle = [" + credentialHandle + "]");
        CompletableFuture<Boolean> future = new CompletableFuture<Boolean>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_credential_is_revokable(commandHandle, credentialHandle, credentialIsRevokableCB);
        checkResult(result);

        return future;
    }

    public static int credentialRelease(int credentialHandle) throws VcxException {
        ParamGuard.notNull(credentialHandle, "credentialHandle");
        logger.debug("credentialRelease() called with: credentialHandle = [" + credentialHandle + "]");

        int result = LibVcx.api.vcx_credential_release(credentialHandle);
        checkResult(result);

        return result;
    }

    private static Callback vcxCredentialGetOffersCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int command_handle, int err, String credential_offers) {
            logger.debug("callback() called with: command_handle = [" + command_handle + "], err = [" + err + "], credential_offers = [****]");
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(command_handle);
            if (!checkCallback(future, err)) return;
            future.complete(credential_offers);
        }
    };

    public static CompletableFuture<String> credentialGetOffers(
            int connectionHandle
    ) throws VcxException {
        ParamGuard.notNull(connectionHandle, "connectionHandle");
        logger.debug("credentialGetOffers() called with: connectionHandle = [" + connectionHandle + "]");
        CompletableFuture<String> future = new CompletableFuture<String>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_credential_get_offers(commandHandle, connectionHandle, vcxCredentialGetOffersCB);
        checkResult(result);

        return future;
    }

    private static Callback vcxCredentialCreateWithOfferCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int command_handle, int err, int credential_handle) {
            logger.debug("callback() called with: command_handle = [" + command_handle + "], err = [" + err + "], credential_handle = [" + credential_handle + "]");
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(command_handle);
            if (!checkCallback(future, err)) return;
            Integer result = credential_handle;
            future.complete(result);
        }
    };

    public static CompletableFuture<Integer> credentialCreateWithOffer(
            String sourceId,
            String credentialOffer
    ) throws VcxException {
        ParamGuard.notNull(sourceId, "sourceId");
        ParamGuard.notNull(credentialOffer, "credentialOffer");
        logger.debug("credentialCreateWithOffer() called with: sourceId = [" + sourceId + "], credentialOffer = [****]");
        CompletableFuture<Integer> future = new CompletableFuture<Integer>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_credential_create_with_offer(commandHandle, sourceId, credentialOffer, vcxCredentialCreateWithOfferCB);
        checkResult(result);

        return future;
    }
}
