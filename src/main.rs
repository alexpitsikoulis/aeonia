use blockchain::Blockchain;

mod blockchain;

fn main() -> std::io::Result<()> {
    let mut blockchain = Blockchain::new().unwrap();
    blockchain
        .add_transation_to_pool("alex".into(), "blanche".into(), 100.0)
        .unwrap();
    println!("{}", blockchain);
    blockchain.add_block(21)?;
    println!("{}", blockchain);
    Ok(())
}
