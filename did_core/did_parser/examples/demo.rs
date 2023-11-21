use did_parser::{Did, DidUrl};

fn main() {
    // parse a string into DID
    let did = Did::parse("did:web:w3c-ccg.github.io".into()).unwrap();
    println!("{:?}", did.did());
    println!("{:?}", did.method());
    println!("{:?}", did.id());

    // parse a string into DID URL
    let did_url =
        DidUrl::parse("did:example:123456789abcdefghi/foo;param=value?query=value".into()).unwrap();
    println!("{:?}", did_url.did());
    println!("{:?}", did_url.did_url());
    println!("{:?}", did_url.method());
    println!("{:?}", did_url.id());
    println!("{:?}", did_url.path());
    println!("{:?}", did_url.queries());
}
