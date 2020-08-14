package utils;

public enum VcxState {

    None(0),
    Initialized(1),
    OfferSent(2),
    RequestReceived(3),
    Accepted(4),
    Unfulfilled(5),
    Expired(6),
    Revoked(7),
    Redirected(8),
    Rejected(9);

    private int value;
    VcxState(int value) {
        this.value = value;
    }
    public int getValue() {
        return value;
    }

}
