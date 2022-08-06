package com.evernym.sdk.vcx.connection;

import com.evernym.sdk.vcx.LibVcx;
import com.evernym.sdk.vcx.ParamGuard;
import com.evernym.sdk.vcx.VcxException;
import com.evernym.sdk.vcx.VcxJava;
import com.sun.jna.Callback;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.concurrent.CompletableFuture;

public class OutOfBandApi extends VcxJava.API {

	private static final Logger logger = LoggerFactory.getLogger("OutOfBandApi");

    private static Callback vcxOutOfBandSenderCreateCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err, int handle) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], handle = [" + handle + "]");
			CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
			if (! checkCallback(future, err)) return;
			future.complete(handle);
		}
	};
	public static CompletableFuture<Integer> vcxOutOfBandSenderCreate(String config) throws VcxException {
		ParamGuard.notNullOrWhiteSpace(config, "config");
		logger.debug("vcxOutOfBandSenderCreate() called with: config = [ {} ]", config);
		CompletableFuture<Integer> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_out_of_band_sender_create(
				commandHandle,
				config,
				vcxOutOfBandSenderCreateCB
		);
		checkResult(result);
		return future;
	}

    private static Callback vcxOutOfBandReceiverCreateCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err, int handle) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], handle = [" + handle + "]");
			CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
			if (! checkCallback(future, err)) return;
			future.complete(handle);
		}
	};
	public static CompletableFuture<Integer> vcxOutOfBandReceiverCreate(String message) throws VcxException {
		ParamGuard.notNullOrWhiteSpace(message, "message");
		logger.debug("vcxOutOfBandReceiverCreate() called with: message = [ {} ]", message);
		CompletableFuture<Integer> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_out_of_band_receiver_create(
				commandHandle,
				message,
				vcxOutOfBandReceiverCreateCB
		);
		checkResult(result);
		return future;
	} 

    private static Callback vcxOutOfBandSenderGetThreadIdCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err, String thid) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], thid = [" + thid + "]");
			CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
			if (! checkCallback(future, err)) return;
			future.complete(thid);
		}
	};
	public static CompletableFuture<String> vcxOutOfBandSenderGetThreadId(int handle) throws VcxException {
		ParamGuard.notNull(handle, "handle");
		logger.debug("vcxOutOfBandSenderGetThreadId() called with: handle = [ {} ]", handle);
		CompletableFuture<String> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_out_of_band_sender_get_thread_id(
				commandHandle,
                handle,
				vcxOutOfBandSenderGetThreadIdCB
		);
		checkResult(result);
		return future;
	}

    private static Callback vcxOutOfBandReceiverGetThreadIdCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err, String thid) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], thid = [" + thid + "]");
			CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
			if (! checkCallback(future, err)) return;
			future.complete(thid);
		}
	};
	public static CompletableFuture<String> vcxOutOfBandReceiverGetThreadId(int handle) throws VcxException {
		ParamGuard.notNull(handle, "handle");
		logger.debug("vcxOutOfBandReceiverGetThreadId() called with: handle = [ {} ]", handle);
		CompletableFuture<String> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_out_of_band_receiver_get_thread_id(
				commandHandle,
                handle,
				vcxOutOfBandReceiverGetThreadIdCB
		);
		checkResult(result);
		return future;
	}

    private static Callback vcxOutOfBandSenderAppendMessageCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "]");
			CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
			if (! checkCallback(future, err)) return;
			future.complete(0);
		}
	};
	public static CompletableFuture<Integer> vcxOutOfBandSenderAppendMessage(int handle, String message) throws VcxException {
		ParamGuard.notNull(handle, "handle");
        ParamGuard.notNullOrWhiteSpace(message, "message");
		logger.debug("vcxOutOfBandSenderAppendMessage() called with: handle = [ {} ]", handle, ", message = [ {} ] ",message);
		CompletableFuture<Integer> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_out_of_band_sender_append_message(
				commandHandle,
                handle,
                message,
				vcxOutOfBandSenderAppendMessageCB
		);
		checkResult(result);
		return future;
	}

    private static Callback vcxOutOfBandSenderAppendServiceCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "]");
			CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
			if (! checkCallback(future, err)) return;
			future.complete(0);
		}
	};
	public static CompletableFuture<Integer> vcxOutOfBandSenderAppendService(int handle, String service) throws VcxException {
		ParamGuard.notNull(handle, "handle");
        ParamGuard.notNullOrWhiteSpace(service, "service");
		logger.debug("vcxOutOfBandSenderAppendService() called with: handle = [ {} ]", handle, ", service = [ {} ] ",service);
		CompletableFuture<Integer> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_out_of_band_sender_append_service(
				commandHandle,
                handle,
                service,
				vcxOutOfBandSenderAppendServiceCB
		);
		checkResult(result);
		return future;
	}

    private static Callback vcxOutOfBandSenderAppendServiceDidCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "]");
			CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
			if (! checkCallback(future, err)) return;
			future.complete(0);
		}
	};
	public static CompletableFuture<Integer> vcxOutOfBandSenderAppendServiceDid(int handle, String did) throws VcxException {
		ParamGuard.notNull(handle, "handle");
        ParamGuard.notNullOrWhiteSpace(did, "did");
		logger.debug("vcxOutOfBandSenderAppendServiceDid() called with: handle = [ {} ]", handle, ", did = [ {} ] ",did);
		CompletableFuture<Integer> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_out_of_band_sender_append_service_did(
				commandHandle,
                handle,
                did,
				vcxOutOfBandSenderAppendServiceDidCB
		);
		checkResult(result);
		return future;
	}

    private static Callback vcxOutOfBandReceiverExtractMessageCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err, String message) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], message = [" + message + "]");
			CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
			if (! checkCallback(future, err)) return;
			future.complete(message);
		}
	};
	public static CompletableFuture<String> vcxOutOfBandReceiverExtractMessage(int handle) throws VcxException {
		ParamGuard.notNull(handle, "handle");
		logger.debug("vcxOutOfBandReceiverExtractMessage() called with: handle = [ {} ]", handle);
		CompletableFuture<String> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_out_of_band_receiver_extract_message(
				commandHandle,
                handle,
				vcxOutOfBandReceiverExtractMessageCB
		);
		checkResult(result);
		return future;
	}

    private static Callback vcxOutOfBandToMessageCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err, String message) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], message = [" + message + "]");
			CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
			if (! checkCallback(future, err)) return;
			future.complete(message);
		}
	};
	public static CompletableFuture<String> vcxOutOfBandToMessage(int handle) throws VcxException {
		ParamGuard.notNull(handle, "handle");
		logger.debug("vcxOutOfBandToMessage() called with: handle = [ {} ]", handle);
		CompletableFuture<String> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_out_of_band_to_message(
				commandHandle,
                handle,
				vcxOutOfBandToMessageCB
		);
		checkResult(result);
		return future;
	}

    private static Callback vcxOutOfReceiverConnectionExistsCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err, int conn_handle, boolean found_one) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], conn_handle = [" + conn_handle + "] , found_one = [" + found_one + "]");
			CompletableFuture<OutOfBandReceiverConnectionExistsResult> future = (CompletableFuture<OutOfBandReceiverConnectionExistsResult>) removeFuture(commandHandle);
			if (! checkCallback(future, err)) return;
            OutOfBandReceiverConnectionExistsResult result = new OutOfBandReceiverConnectionExistsResult(conn_handle,found_one);
			future.complete(result);
		}
	};
	public static CompletableFuture<OutOfBandReceiverConnectionExistsResult> vcxOutOfReceiverConnectionExists(int handle, String conn_handles) throws VcxException {
        ParamGuard.notNull(handle, "handle");
        ParamGuard.notNullOrWhiteSpace(conn_handles, "conn_handles");
		logger.debug("vcxOutOfReceiverConnectionExists() called with: handle = [ {} ]", handle, ", conn_handles = [ {} ]", conn_handles);
		CompletableFuture<OutOfBandReceiverConnectionExistsResult> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_out_of_band_receiver_connection_exists(
				commandHandle,
                handle,
                conn_handles,
				vcxOutOfReceiverConnectionExistsCB
		);
		checkResult(result);
		return future;
	}

    private static Callback vcxOutOfBandReceiverBuildConnectionCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err, String connection) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], connection = [" + connection + "]");
			CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
			if (! checkCallback(future, err)) return;
			future.complete(connection);
		}
	};
	public static CompletableFuture<String> vcxOutOfBandReceiverBuildConnection(int handle) throws VcxException {
		ParamGuard.notNull(handle, "handle");
		logger.debug("vcxOutOfBandReceiverBuildConnection() called with: handle = [ {} ]", handle);
		CompletableFuture<String> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_out_of_band_receiver_build_connection(
				commandHandle,
                handle,
				vcxOutOfBandReceiverBuildConnectionCB
		);
		checkResult(result);
		return future;
	}

    private static Callback vcxOutOfBandSenderSerializeCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err, String oob_json) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], oob_json = [" + oob_json + "]");
			CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
			if (! checkCallback(future, err)) return;
			future.complete(oob_json);
		}
	};
	public static CompletableFuture<String> vcxOutOfBandSenderSerialize(int handle) throws VcxException {
		ParamGuard.notNull(handle, "handle");
		logger.debug("vcxOutOfBandSenderSerialize() called with: handle = [ {} ]", handle);
		CompletableFuture<String> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_out_of_band_sender_serialize(
				commandHandle,
                handle,
				vcxOutOfBandSenderSerializeCB
		);
		checkResult(result);
		return future;
	}

    private static Callback vcxOutOfBandReceiverSerializeCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err, String oob_json) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], oob_json = [" + oob_json + "]");
			CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
			if (! checkCallback(future, err)) return;
			future.complete(oob_json);
		}
	};
	public static CompletableFuture<String> vcxOutOfBandReceiverSerialize(int handle) throws VcxException {
		ParamGuard.notNull(handle, "handle");
		logger.debug("vcxOutOfBandReceiverSerialize() called with: handle = [ {} ]", handle);
		CompletableFuture<String> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_out_of_band_receiver_serialize(
				commandHandle,
                handle,
				vcxOutOfBandReceiverSerializeCB
		);
		checkResult(result);
		return future;
	}

    private static Callback vcxOutOfBandSenderDeserializeCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err, int handle) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], handle = [" + handle + "]");
			CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
			if (! checkCallback(future, err)) return;
			future.complete(handle);
		}
	};
	public static CompletableFuture<Integer> vcxOutOfBandSenderDeserialize(String oob_json) throws VcxException {
		ParamGuard.notNullOrWhiteSpace(oob_json, "oob_json");
		logger.debug("vcxOutOfBandSenderDeserialize() called with: oob_json = [ {} ]", oob_json);
		CompletableFuture<Integer> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_out_of_band_sender_deserialize(
				commandHandle,
				oob_json,
				vcxOutOfBandSenderDeserializeCB
		);
		checkResult(result);
		return future;
	}

    private static Callback vcxOutOfBandReceiverDeserializeCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err, int handle) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], handle = [" + handle + "]");
			CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
			if (! checkCallback(future, err)) return;
			future.complete(handle);
		}
	};
	public static CompletableFuture<Integer> vcxOutOfBandReceiverDeserialize(String oob_json) throws VcxException {
		ParamGuard.notNullOrWhiteSpace(oob_json, "oob_json");
		logger.debug("vcxOutOfBandReceiverDeserialize() called with: oob_json = [ {} ]", oob_json);
		CompletableFuture<Integer> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_out_of_band_receiver_deserialize(
				commandHandle,
				oob_json,
				vcxOutOfBandReceiverDeserializeCB
		);
		checkResult(result);
		return future;
	}

    public static int vcxOutOfBandSenderRelease(int handle) throws VcxException {
		logger.debug("vcxOutOfBandSenderRelease() called with: handle = [" + handle + "]");
		ParamGuard.notNull(handle, "handle");
		int result = LibVcx.api.vcx_out_of_band_sender_release(handle);
		checkResult(result);

		return result;
	}

    public static int vcxOutOfBandReceiverRelease(int handle) throws VcxException {
		logger.debug("vcxOutOfBandReceiverRelease() called with: handle = [" + handle + "]");
		ParamGuard.notNull(handle, "handle");
		int result = LibVcx.api.vcx_out_of_band_receiver_release(handle);
		checkResult(result);

		return result;
	}

}