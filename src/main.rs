use blockchain::Blockchain;
use wallet::Wallet;

mod blockchain;
mod wallet;

fn main() -> std::io::Result<()> {
    let mut blockchain = Blockchain::new(0x00).unwrap();
    let mut wallet = Wallet::new(0x01).unwrap();
    let mut wallet2 = Wallet::new(0x01).unwrap();
    let (mut transaction, mut signature, mut v_key) = wallet
        .sign_transaction("harry's wallet".into(), 1.0)
        .unwrap();
    blockchain
        .add_transation_to_pool(transaction, signature, v_key)
        .unwrap();
    (transaction, signature, v_key) = wallet2
        .sign_transaction("alex's wallet".into(), 1.0)
        .unwrap();
    blockchain
        .add_transation_to_pool(transaction, signature, v_key)
        .unwrap();
    blockchain.mining();
    println!("{}", blockchain);

    Ok(())
}
