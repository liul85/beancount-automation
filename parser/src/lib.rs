use anyhow::{anyhow, Result};
use chrono::prelude::Local;
use lazy_static::lazy_static;
use regex::Regex;
mod settings;
use settings::Settings;

lazy_static! {
    static ref DATE_RE: Regex = Regex::new("^\\d{4}-\\d{2}-\\d{2}$").unwrap();
    static ref PAYEE_RE: Regex = Regex::new("^@\\w+").unwrap();
    static ref CURRENCY_RE: Regex = Regex::new("^[A-Z]{3}$").unwrap();
}

#[derive(Debug)]
pub struct Transaction {
    date: String,
    payee: String,
    narration: String,
    amount: f32,
    currency: String,
    from_account: String,
    to_account: String,
}

impl Transaction {
    pub fn to_beancount(&self) -> String {
        format!(
            "{} * \"{}\" \"{}\"\n  {}        -{:.2} {}\n  {}        {:.2} {}\n",
            self.date,
            self.payee,
            self.narration,
            self.from_account,
            self.amount,
            self.currency,
            self.to_account,
            self.amount,
            self.currency
        )
    }
}

impl From<Transaction> for String {
    fn from(transaction: Transaction) -> Self {
        format!(
            "✅\n==============================\n{}",
            transaction.to_beancount()
        )
    }
}

pub struct Parser {
    settings: Settings,
}

impl Parser {
    pub fn new() -> Result<Self> {
        let settings = Settings::new()?;
        Ok(Self { settings })
    }

    pub fn parse(self, input: &str) -> Result<Transaction> {
        let mut date_vec: Vec<&str> = vec![];
        let mut payee_vec: Vec<&str> = vec![];
        let mut amount_vec: Vec<f32> = vec![];
        let mut currency_vec: Vec<&str> = vec![];
        let mut others_vec: Vec<&str> = vec![];

        let split: Vec<&str> = input.split(' ').collect();

        for value in split {
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

            others_vec.push(value);
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

        let right_arrow_index = match others_vec.iter().position(|r| *r == ">") {
            Some(index) => index,
            None => return Err(anyhow!("Could not find > in input.")),
        };

        let from = String::from(others_vec[right_arrow_index - 1]);
        let from_account = match self.settings.accounts.get(from.as_str()) {
            Some(v) => v.into(),
            None => from,
        };

        let to = String::from(others_vec[right_arrow_index + 1]);
        let to_account = match self.settings.accounts.get(to.as_str()) {
            Some(v) => v.into(),
            None => to,
        };

        for _ in 0..3 {
            others_vec.remove(right_arrow_index - 1);
        }

        let currency = match currency_vec.len() {
            0 => self.settings.currency,
            _ => currency_vec[0].into(),
        };

        return Ok(Transaction {
            date,
            payee,
            narration: others_vec.join(" "),
            amount,
            currency,
            from_account,
            to_account,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_can_parse_standard_input() {
        let parser = Parser {
            settings: Settings {
                currency: "AUD".into(),
                accounts: [
                    ("cba".into(), "Assets:MasterCard:CBA".into()),
                    ("amex".into(), "Liabilities:CreditCard:AMEX:Liang".into()),
                    ("food".into(), "Expenses:Food".into()),
                ]
                .iter()
                .cloned()
                .collect(),
            },
        };
        let result = parser
            .parse("2021-09-08 @KFC hamburger 12.40 AUD Assets:MasterCard:CBA > Expense:Food");
        assert!(result.is_ok());
        let transaction = result.unwrap();
        assert_eq!(transaction.to_beancount(), "2021-09-08 * \"KFC\" \"hamburger\"\n  Assets:MasterCard:CBA        -12.40 AUD\n  Expense:Food        12.40 AUD\n");
    }

    #[test]
    fn parser_can_parse_input_without_date() {
        let parser = Parser {
            settings: Settings {
                currency: "AUD".into(),
                accounts: [
                    ("cba".into(), "Assets:MasterCard:CBA".into()),
                    ("amex".into(), "Liabilities:CreditCard:AMEX:Liang".into()),
                    ("food".into(), "Expenses:Food".into()),
                ]
                .iter()
                .cloned()
                .collect(),
            },
        };
        let result = parser.parse("@KFC hamburger 12.40 AUD Assets:MasterCard:CBA > Expense:Food");
        assert!(result.is_ok());
        let transaction = result.unwrap();
        assert!(DATE_RE.is_match(&transaction.date));
        assert_eq!(transaction.payee, "KFC");
        assert_eq!(transaction.narration, "hamburger");
        assert_eq!(transaction.amount, 12.40);
        assert_eq!(transaction.currency, "AUD");
        assert_eq!(transaction.from_account, "Assets:MasterCard:CBA");
        assert_eq!(transaction.to_account, "Expense:Food");
    }

    #[test]
    fn parser_return_error_if_input_without_space_before_right_arrow() {
        let parser = Parser {
            settings: Settings {
                currency: "AUD".into(),
                accounts: [
                    ("cba".into(), "Assets:MasterCard:CBA".into()),
                    ("amex".into(), "Liabilities:CreditCard:AMEX:Liang".into()),
                    ("food".into(), "Expenses:Food".into()),
                ]
                .iter()
                .cloned()
                .collect(),
            },
        };
        let result = parser.parse("@Costco lunch 8.97 cba>food");
        assert!(result.is_err());
    }

    #[test]
    fn parser_can_parse_input_in_amount_payee_from_account_to_account_format() {
        let parser = Parser {
            settings: Settings {
                currency: "AUD".into(),
                accounts: [
                    ("cba".into(), "Assets:MasterCard:CBA".into()),
                    ("amex".into(), "Liabilities:CreditCard:AMEX:Liang".into()),
                    ("food".into(), "Expenses:Food".into()),
                ]
                .iter()
                .cloned()
                .collect(),
            },
        };
        let result = parser.parse("22.34 @KFL cba > food");
        assert!(result.is_ok());
        let transaction = result.unwrap();
        assert!(DATE_RE.is_match(&transaction.date));
        assert_eq!(transaction.payee, "KFL");
        assert_eq!(transaction.narration, "");
        assert_eq!(transaction.amount, 22.34);
        assert_eq!(transaction.currency, "AUD");
        assert_eq!(transaction.from_account, "Assets:MasterCard:CBA");
        assert_eq!(transaction.to_account, "Expenses:Food");
    }

    #[test]
    fn parser_can_parse_input_in_payee_amount_from_account_to_account_format() {
        let parser = Parser {
            settings: Settings {
                currency: "AUD".into(),
                accounts: [
                    ("cba".into(), "Assets:MasterCard:CBA".into()),
                    ("amex".into(), "Liabilities:CreditCard:AMEX:Liang".into()),
                    ("food".into(), "Expenses:Food".into()),
                ]
                .iter()
                .cloned()
                .collect(),
            },
        };
        let result = parser.parse("@KFL 22.34 cba > food");
        assert!(result.is_ok());
        let transaction = result.unwrap();
        assert!(DATE_RE.is_match(&transaction.date));
        assert_eq!(transaction.payee, "KFL");
        assert_eq!(transaction.narration, "");
        assert_eq!(transaction.amount, 22.34);
        assert_eq!(transaction.currency, "AUD");
        assert_eq!(transaction.from_account, "Assets:MasterCard:CBA");
        assert_eq!(transaction.to_account, "Expenses:Food");
    }

    #[test]
    fn parser_can_parse_input_in_amount_currency_payee_from_account_to_account_format() {
        let parser = Parser {
            settings: Settings {
                currency: "AUD".into(),
                accounts: [
                    ("cba".into(), "Assets:MasterCard:CBA".into()),
                    ("amex".into(), "Liabilities:CreditCard:AMEX:Liang".into()),
                    ("food".into(), "Expenses:Food".into()),
                ]
                .iter()
                .cloned()
                .collect(),
            },
        };
        let result = parser.parse("22.34 USD @KFL cba > food");
        assert!(result.is_ok());
        let transaction = result.unwrap();
        assert!(DATE_RE.is_match(&transaction.date));
        assert_eq!(transaction.payee, "KFL");
        assert_eq!(transaction.narration, "");
        assert_eq!(transaction.amount, 22.34);
        assert_eq!(transaction.currency, "USD");
        assert_eq!(transaction.from_account, "Assets:MasterCard:CBA");
        assert_eq!(transaction.to_account, "Expenses:Food");
    }

    #[test]
    fn parser_can_parse_input_in_date_amount_payee_from_account_to_account_format() {
        let parser = Parser {
            settings: Settings {
                currency: "AUD".into(),
                accounts: [
                    ("cba".into(), "Assets:MasterCard:CBA".into()),
                    ("amex".into(), "Liabilities:CreditCard:AMEX:Liang".into()),
                    ("food".into(), "Expenses:Food".into()),
                ]
                .iter()
                .cloned()
                .collect(),
            },
        };
        let result = parser.parse("2021-11-23 22.34 @KFL cba > food");
        assert!(result.is_ok());
        let transaction = result.unwrap();
        assert!(DATE_RE.is_match(&transaction.date));
        assert_eq!(transaction.payee, "KFL");
        assert_eq!(transaction.narration, "");
        assert_eq!(transaction.amount, 22.34);
        assert_eq!(transaction.currency, "AUD");
        assert_eq!(transaction.from_account, "Assets:MasterCard:CBA");
        assert_eq!(transaction.to_account, "Expenses:Food");
    }
}
