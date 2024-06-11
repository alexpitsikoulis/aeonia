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
    let nonce = blockchain.proof_of_work().unwrap();
    blockchain.add_block(nonce)?;
    println!("{}", blockchain);
    Ok(())
}
