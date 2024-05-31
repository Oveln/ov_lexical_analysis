use nfa::NFA;
use token::{Token, Tokens};

mod nfa;
mod token;

fn main() {
    // let file = std::fs::read_to_string("tokens.toml").unwrap();
    // let tokens: Tokens = toml::from_str(file.as_str()).unwrap();
    // println!("{:?}", tokens);

    let nfa = NFA::new_from_token(&Token {
        value: "aaaa|bbbd|cc".to_string(),
        kind: "char".to_string(),
    });
    print!("{}", nfa);
}

#[test]
fn test() {
    use std::str::FromStr;
    let file = String::from_str(
        r#"
        tokens = [
            { kind = "INT", value = "[0-9]+" },
            { kind = "ID", value = "[a-zA-Z_][a-zA-Z0-9_]*" },
            { kind = "OP", value = "[+\\-*/]" },
            { kind = "EQ", value = "=" },
            { kind = "WS", value = "[ \t\n]+" },
        ]
        "#,
    )
    .unwrap();
    let tokens: Tokens = toml::from_str(file.as_str()).unwrap();
    println!("{:?}", tokens);
    assert_eq!(tokens.tokens.len(), 5);
}
