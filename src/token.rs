use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Token {
    pub kind: String,
    pub value: String,
}

#[derive(Deserialize, Debug)]
pub struct Tokens {
    pub tokens: Vec<Token>,
}