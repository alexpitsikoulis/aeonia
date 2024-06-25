use blockchain::Blockchain;
use wallet::Wallet;

mod blockchain;
mod wallet;

fn main() -> std::io::Result<()> {
    let mut blockchain = Blockchain::new(0x00).unwrap();
    let mut wallet = Wallet::new(0x01).unwrap();
    blockchain.deposit_to_wallet(wallet.address(), 100.0).unwrap();
    let mut wallet2 = Wallet::new(0x01).unwrap();
    blockchain.deposit_to_wallet(wallet2.address(), 100.0).unwrap();
    println!("{}", blockchain);
    let (mut transaction, mut signature, mut v_key) = wallet
        .sign_transaction(wallet2.address(), 1.0)
        .unwrap();
    blockchain
        .add_transation_to_pool(transaction, signature, v_key)
        .unwrap();
    (transaction, signature, v_key) = wallet2
        .sign_transaction(wallet.address(), 1.0)
        .unwrap();
    blockchain
        .add_transation_to_pool(transaction, signature, v_key)
        .unwrap();
    blockchain.mining(wallet.address());


    // should fail on balance exceeded
    (transaction, signature, v_key) = wallet.sign_transaction(wallet2.address(), 1000.0).unwrap();
    blockchain.add_transation_to_pool(transaction, signature, v_key).unwrap();

    Ok(())
}
