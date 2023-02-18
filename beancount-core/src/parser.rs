use anyhow::{anyhow, Result};
use chrono::prelude::Local;

use crate::settings::Settings;
use pest::Parser;

#[derive(Parser)]
#[grammar = "transaction.pest"]
pub struct TransactionParser;

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

impl Default for Transaction {
    fn default() -> Self {
        Transaction {
            date: Local::now().format("%Y-%m-%d").to_string(),
            payee: String::default(),
            narration: String::default(),
            amount: 0.0,
            currency: "AUD".to_string(),
            from_account: String::default(),
            to_account: String::default(),
        }
    }
}

impl Transaction {
    pub fn year(&self) -> String {
        self.date.split('-').next().unwrap().into()
    }
}

impl From<Transaction> for String {
    fn from(transaction: Transaction) -> Self {
        format!(
            "{} * \"{}\" \"{}\"\n  {}        -{:.2} {}\n  {}        {:.2} {}\n",
            transaction.date,
            transaction.payee,
            transaction.narration,
            transaction.from_account,
            transaction.amount,
            transaction.currency,
            transaction.to_account,
            transaction.amount,
            transaction.currency
        )
    }
}

pub struct BeancountParser {
    settings: Settings,
}

impl BeancountParser {
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }

    pub fn parse(&self, input: &str) -> Result<Transaction> {
        if let Some(pairs) = TransactionParser::parse(Rule::transaction, input)?.next() {
            let mut transaction = Transaction::default();
            for pair in pairs.into_inner() {
                match pair.as_rule() {
                    Rule::date => transaction.date = pair.as_str().into(),
                    Rule::payee => transaction.payee = pair.as_str().trim_matches('@').into(),
                    Rule::narration => transaction.narration = pair.as_str().into(),
                    Rule::amount => transaction.amount = pair.as_str().parse::<f32>()?,
                    Rule::currency => transaction.currency = pair.as_str().into(),
                    Rule::from_account => {
                        transaction.from_account = self.parse_account(pair.as_str())?
                    }
                    Rule::to_account => {
                        transaction.to_account = self.parse_account(pair.as_str())?
                    }
                    Rule::EOI => break,
                    _ => unreachable!("Unexpected rule {:?}", pair.as_rule()),
                }
            }
            return Ok(transaction);
        }

        Err(anyhow!("Invalid input"))
    }

    fn parse_account(&self, matched: &str) -> Result<String> {
        match self.settings.accounts.get(matched) {
            Some(account) => Ok(account.to_string()),
            None => Err(anyhow!(format!(
                "account {} doesn't exist in current setting",
                matched
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lazy_static::lazy_static;
    use regex::Regex;

    lazy_static! {
        static ref DATE_RE: Regex = Regex::new("^\\d{4}-\\d{2}-\\d{2}$").unwrap();
    }

    fn create_parser() -> BeancountParser {
        let accounts = [
            ("cba".into(), "Assets:MasterCard:CBA".into()),
            ("amex".into(), "Liabilities:CreditCard:AMEX:Liang".into()),
            ("food".into(), "Expense:Food".into()),
        ]
        .iter()
        .cloned()
        .collect();
        let settings = Settings::new("AUD".into(), accounts);

        BeancountParser::new(settings)
    }

    #[test]
    fn parser_can_parse_standard_input_date_payee_narration_amount_currency_from_to() {
        let parser = create_parser();
        let result = parser.parse("2021-09-08 @KFC hamburger 12.40 AUD cba > food");
        assert!(result.is_ok());
        let transaction = result.unwrap();
        assert_eq!(transaction.year(), "2021");
        let actual_text: String = transaction.into();
        assert_eq!("2021-09-08 * \"KFC\" \"hamburger\"\n  Assets:MasterCard:CBA        -12.40 AUD\n  Expense:Food        12.40 AUD\n", actual_text);
    }

    #[test]
    fn parser_can_parse_standard_input_with_multi_space_in_between() {
        let parser = create_parser();
        let result = parser.parse("2021-09-08    @KFC    hamburger   12.40   AUD   cba >   food");
        assert!(result.is_ok());
        let transaction = result.unwrap();
        assert_eq!(transaction.year(), "2021");
        let actual_text: String = transaction.into();
        assert_eq!("2021-09-08 * \"KFC\" \"hamburger\"\n  Assets:MasterCard:CBA        -12.40 AUD\n  Expense:Food        12.40 AUD\n", actual_text);
    }

    #[test]
    fn parser_can_parse_input_without_date() {
        let parser = create_parser();
        let result = parser.parse("@KFC hamburger 12.40 AUD cba > food");
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
    fn parser_can_parse_amount_in_integer() {
        let parser = create_parser();
        let result = parser.parse("@KFC hamburger 12 AUD cba > food");
        assert!(result.is_ok());
        let transaction = result.unwrap();
        assert!(DATE_RE.is_match(&transaction.date));
        assert_eq!(transaction.payee, "KFC");
        assert_eq!(transaction.narration, "hamburger");
        assert_eq!(transaction.amount, 12.0);
        assert_eq!(transaction.currency, "AUD");
        assert_eq!(transaction.from_account, "Assets:MasterCard:CBA");
        assert_eq!(transaction.to_account, "Expense:Food");
    }

    #[test]
    fn parser_support_input_without_space_before_right_arrow() {
        let parser = create_parser();
        let result = parser.parse("@Costco lunch 8.97 cba>food");
        assert!(result.is_ok());
        let transaction = result.unwrap();
        assert!(DATE_RE.is_match(&transaction.date));
        assert_eq!(transaction.payee, "Costco");
        assert_eq!(transaction.narration, "lunch");
        assert_eq!(transaction.amount, 8.97);
        assert_eq!(transaction.currency, "AUD");
        assert_eq!(transaction.from_account, "Assets:MasterCard:CBA");
        assert_eq!(transaction.to_account, "Expense:Food");
    }

    #[test]
    fn parser_can_parse_input_in_payee_amount_from_account_to_account_format() {
        let parser = create_parser();
        let result = parser.parse("@KFL 22.34 cba > food");
        assert!(result.is_ok());
        let transaction = result.unwrap();
        assert!(DATE_RE.is_match(&transaction.date));
        assert_eq!(transaction.payee, "KFL");
        assert_eq!(transaction.narration, "");
        assert_eq!(transaction.amount, 22.34);
        assert_eq!(transaction.currency, "AUD");
        assert_eq!(transaction.from_account, "Assets:MasterCard:CBA");
        assert_eq!(transaction.to_account, "Expense:Food");
    }

    #[test]
    fn parser_can_parse_input_in_payee_amount_currency_from_account_to_account_format() {
        let parser = create_parser();
        let result = parser.parse("@KFL 22.34 USD cba > food");
        assert!(result.is_ok());
        let transaction = result.unwrap();
        assert!(DATE_RE.is_match(&transaction.date));
        assert_eq!(transaction.payee, "KFL");
        assert_eq!(transaction.narration, "");
        assert_eq!(transaction.amount, 22.34);
        assert_eq!(transaction.currency, "USD");
        assert_eq!(transaction.from_account, "Assets:MasterCard:CBA");
        assert_eq!(transaction.to_account, "Expense:Food");
    }

    #[test]
    fn parser_can_parse_input_in_date_amount_payee_from_account_to_account_format() {
        let parser = create_parser();
        let result = parser.parse("2021-11-23 @KFL 22.34 cba > food");
        assert!(result.is_ok());
        let transaction = result.unwrap();
        assert!(DATE_RE.is_match(&transaction.date));
        assert_eq!(transaction.payee, "KFL");
        assert_eq!(transaction.narration, "");
        assert_eq!(transaction.amount, 22.34);
        assert_eq!(transaction.currency, "AUD");
        assert_eq!(transaction.from_account, "Assets:MasterCard:CBA");
        assert_eq!(transaction.to_account, "Expense:Food");
    }

    #[test]
    fn parser_return_error_for_invalid_input() {
        let parser = create_parser();
        let result = parser.parse("I am testing here");
        assert!(result.is_err());

        let result = parser
            .parse("2021-09-08 @KFC hamburger 12.40 AUDE Assets:MasterCard:CBA > Expense:Food");
        assert!(result.is_err());

        let result = parser
            .parse("2021-09-08 KFC hamburger 12.40 AUDE Assets:MasterCard:CBA > Expense:Food");
        assert!(result.is_err());
    }

    #[test]
    fn parser_return_error_if_pay_account_not_exist() {
        let parser = create_parser();
        let result = parser.parse("2022-08-14 @MelbourneZoo 33.7 abc > home");
        assert!(result.is_err());
    }

    #[test]
    fn parser_can_parse_multi_words_narration() {
        let parser = create_parser();
        let result = parser.parse("@KFC beef hamburger and french fries 12 AUD cba > food");
        assert!(result.is_ok());
        let transaction = result.unwrap();
        assert!(DATE_RE.is_match(&transaction.date));
        assert_eq!(transaction.payee, "KFC");
        assert_eq!(transaction.narration, "beef hamburger and french fries");
        assert_eq!(transaction.amount, 12.0);
        assert_eq!(transaction.currency, "AUD");
        assert_eq!(transaction.from_account, "Assets:MasterCard:CBA");
        assert_eq!(transaction.to_account, "Expense:Food");
    }
}
