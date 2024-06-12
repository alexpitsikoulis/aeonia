use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub sender: String,
    pub recipient: String,
    pub amount: f64,
}

impl Transaction {
    pub fn new(sender: String, recipient: String, amount: f64) -> Self {
        Transaction {
            sender,
            recipient,
            amount,
        }
    }
}

impl ToString for Transaction {
    fn to_string(&self) -> String {
        format!(
            r#"
        {{
            "sender": "{}",
            "recipient": "{}",
            "amount": {}
        }}
        "#,
            self.sender, self.recipient, self.amount
        )
    }
}
