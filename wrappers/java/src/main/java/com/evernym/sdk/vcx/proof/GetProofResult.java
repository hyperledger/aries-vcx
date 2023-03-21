package com.evernym.sdk.vcx.proof;

/**
 * Created by abdussami on 05/06/18.
 */

public class GetProofResult {
    public GetProofResult(String response_data) {
        this.response_data = response_data;
    }

    private String response_data;

    public String getResponse_data() {
        return response_data;
    }

    public void setResponse_data(String response_data) {
        this.response_data = response_data;
    }
}
