package utils;

public enum ProofState {

    Undefined(0),
    Validated(1),
    Invalid(2);

    private int value;
    ProofState(int value) {
        this.value = value;
    }
    public int getValue() {
        return value;
    }

}
