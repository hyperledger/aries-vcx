use did_parser_nom::Did;

fn main() {
    // parse a string into DID
    let did = Did::parse("did:web:w3c-ccg.github.io".into()).unwrap();
    println!("{:?}", did.did());
    println!("{:?}", did.method());
    println!("{:?}", did.id());
}
