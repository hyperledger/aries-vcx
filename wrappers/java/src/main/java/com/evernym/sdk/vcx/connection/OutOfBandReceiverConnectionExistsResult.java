package com.evernym.sdk.vcx.connection;

public class OutOfBandReceiverConnectionExistsResult {
    public int conn_handle;
    public boolean found_one;

    public OutOfBandReceiverConnectionExistsResult(int conn_handle, boolean found_one) {
        this.conn_handle = conn_handle;
        this.found_one = found_one;
    }
}
