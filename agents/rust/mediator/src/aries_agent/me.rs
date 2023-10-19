trait Foo {}
trait Faa {}

struct Obar {}

impl Foo for Obar {}
impl Faa for Obar {}

fn trat() {
    let a = Obar {};
    tak(a);
}
fn tak(name: &(dyn Foo + Faa)) {}
