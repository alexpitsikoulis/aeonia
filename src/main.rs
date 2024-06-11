use blockchain::Blockchain;

mod blockchain;

fn main() -> std::io::Result<()> {
    let mut blockchain = Blockchain::default();
    blockchain
        .add_transation_to_pool("alex".into(), "blanche".into(), 100.0)
        .unwrap();
    blockchain
        .add_transation_to_pool("harry".into(), "gus".into(), 100.0)
        .unwrap();
    blockchain.mining();
    println!("{}", blockchain);
    Ok(())
}
