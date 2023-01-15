use roli;

fn main() {
    let roli_client = roli::ClientBuilder::new().build();
    dbg!(roli_client.contains_roli_verification());
}
