use chrono::prelude::Local;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref DATE_RE: Regex = Regex::new("^\\d{4}-\\d{2}-\\d{2}$").unwrap();
    static ref PAYEE_RE: Regex = Regex::new("^@\\w+").unwrap();
    static ref CURRENCY_RE: Regex = Regex::new("^[A-Z]{3}$").unwrap();
}

pub struct Transaction {
    date: String,
    payee: String,
    narration: String,
    amount: f32,
    currency: String,
    from_account: String,
    to_account: String,
}

pub fn parse(input: &str) -> Result<Transaction, String> {
    let mut date_vec: Vec<&str> = vec![];
    let mut payee_vec: Vec<&str> = vec![];
    let mut amount_vec: Vec<f32> = vec![];
    let mut currency_vec: Vec<&str> = vec![];
    let mut accounts_vec: Vec<&str> = vec![];

    let split: Vec<&str> = input.split(' ').collect();

    for value in &split {
        println!("{}", value);
        if DATE_RE.is_match(value) {
            date_vec.push(value);
            continue;
        }

        if PAYEE_RE.is_match(value) {
            payee_vec.push(value);
            continue;
        }

        match value.parse::<f32>() {
            Ok(v) => {
                amount_vec.push(v);
                continue;
            }
            Err(_) => (),
        }

        if CURRENCY_RE.is_match(value) {
            currency_vec.push(value);
            continue;
        }

        accounts_vec.push(value);
    }

    let date = match date_vec.first() {
        Some(v) => v.to_string(),
        None => Local::now().format("%Y-%m-%d").to_string(),
    };

    let payee = match payee_vec.first() {
        Some(v) => v[1..].to_string(),
        None => "".into(),
    };

    let amount = match amount_vec.first() {
        Some(v) => *v,
        None => 0.0,
    };

    println!("accounts_vec {:?}", accounts_vec);

    let right_arrow_index = accounts_vec
        .iter()
        .position(|r| *r == ">")
        .expect("No > provided");

    let from_account = String::from(accounts_vec[right_arrow_index - 1]);
    let to_account = String::from(accounts_vec[right_arrow_index + 1]);
    for _ in 0..3 {
        accounts_vec.remove(right_arrow_index - 1);
    }

    return Ok(Transaction {
        date,
        payee,
        narration: accounts_vec.join(" "),
        amount,
        currency: String::from("AUD"),
        from_account,
        to_account,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_can_parse_standard_input() {
        let result =
            parse("2021-09-08 @KFC hamburger 12.40 AUD Assets:MasterCard:CBA > Expense:Food");
        assert!(result.is_ok());
        let transaction = result.unwrap();
        assert_eq!(transaction.date, "2021-09-08");
        assert_eq!(transaction.payee, "KFC");
        assert_eq!(transaction.narration, "hamburger");
        assert_eq!(transaction.amount, 12.40);
        assert_eq!(transaction.currency, "AUD");
        assert_eq!(transaction.from_account, "Assets:MasterCard:CBA");
        assert_eq!(transaction.to_account, "Expense:Food");
    }

    #[test]
    fn parser_can_parse_input_without_date() {
        let result = parse("@KFC hamburger 12.40 AUD Assets:MasterCard:CBA > Expense:Food");
        assert!(result.is_ok());
        let transaction = result.unwrap();
        assert!(DATE_RE.is_match(&transaction.date[..]));
        assert_eq!(transaction.payee, "KFC");
        assert_eq!(transaction.narration, "hamburger");
        assert_eq!(transaction.amount, 12.40);
        assert_eq!(transaction.currency, "AUD");
        assert_eq!(transaction.from_account, "Assets:MasterCard:CBA");
        assert_eq!(transaction.to_account, "Expense:Food");
    }
}
