use anyhow::{anyhow, Result};
use chrono::prelude::Local;
use lazy_static::lazy_static;
use regex::{Captures, Regex};
mod settings;
use settings::Settings;

lazy_static! {
    static ref TRANSACTION_REGEX: Regex =
        Regex::new(r"((?P<date>\d{4}-\d{2}-\d{2})\s+)?@(?P<payee>\w+)\s+((?P<narration>\w+)\s+)?(?P<amount>\d+\.\d+)(\s+(?P<currency>[A-Z]{3}))?\s+(?P<from>[a-zA-Z:]+)\s*>\s*(?P<to>[a-zA-Z:]+)")
            .unwrap();
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

pub struct Parser {
    settings: Settings,
}

impl Parser {
    pub fn new() -> Result<Self> {
        let settings = Settings::new()?;
        Ok(Self { settings })
    }

    pub fn parse(&self, input: &str) -> Result<Transaction> {
        if !TRANSACTION_REGEX.is_match(input) {
            return Err(anyhow!("Invalid input format, please follow examples here:\n* 2021-09-08 @KFC hamburger 12.40 AUD Assets:MasterCard:CBA > Expense:Food\n* @KFC hamburger 12.40 AUD Assets:MasterCard:CBA > Expense:Food\n* @Costco lunch 8.97 cba > food\n* @KFL 22.34 cba > food*\n"));
        }

        match TRANSACTION_REGEX.captures(input) {
            None => Err(anyhow!("Invalid input.")),
            Some(caps) => self.parse_caps(caps),
        }
    }

    fn parse_caps(&self, caps: Captures) -> Result<Transaction> {
        let date: String = caps
            .name("date")
            .map_or(Local::now().format("%Y-%m-%d").to_string(), |d| {
                d.as_str().to_string()
            });

        let payee = match caps.name("payee") {
            Some(payee) => payee.as_str().to_string(),
            None => return Err(anyhow!("Could not get payee from input")),
        };

        let narration = caps
            .name("narration")
            .map_or("".to_string(), |n| n.as_str().to_string());

        let amount = match caps.name("amount") {
            Some(amount) => amount.as_str().parse::<f32>()?,
            None => return Err(anyhow!("Could not get amount from input")),
        };

        let currency = caps
            .name("currency")
            .map_or("AUD".to_string(), |c| c.as_str().to_string());

        let from_account = match caps.name("from") {
            Some(from) => self.parse_account(from.as_str())?,
            None => return Err(anyhow!("Could not get from_account from input")),
        };

        let to_account = match caps.name("to") {
            Some(to) => self.parse_account(to.as_str())?,
            None => return Err(anyhow!("Could not get to_account from input")),
        };

        Ok(Transaction {
            date,
            payee,
            narration,
            amount,
            currency,
            from_account,
            to_account,
        })
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
    lazy_static! {
        static ref DATE_RE: Regex = Regex::new("^\\d{4}-\\d{2}-\\d{2}$").unwrap();
    }

    #[test]
    fn parser_can_parse_standard_input_date_payee_narration_amount_currency_from_to() {
        let parser = Parser {
            settings: Settings {
                currency: "AUD".into(),
                accounts: [
                    ("cba".into(), "Assets:MasterCard:CBA".into()),
                    ("amex".into(), "Liabilities:CreditCard:AMEX:Liang".into()),
                    ("food".into(), "Expense:Food".into()),
                ]
                .iter()
                .cloned()
                .collect(),
            },
        };
        let result = parser.parse("2021-09-08 @KFC hamburger 12.40 AUD cba > food");
        assert!(result.is_ok());
        let transaction = result.unwrap();
        assert_eq!(transaction.year(), "2021");
        let actual_text: String = transaction.into();
        assert_eq!("2021-09-08 * \"KFC\" \"hamburger\"\n  Assets:MasterCard:CBA        -12.40 AUD\n  Expense:Food        12.40 AUD\n", actual_text);
    }

    #[test]
    fn parser_can_parse_standard_input_with_multi_space_in_between() {
        let parser = Parser {
            settings: Settings {
                currency: "AUD".into(),
                accounts: [
                    ("cba".into(), "Assets:MasterCard:CBA".into()),
                    ("amex".into(), "Liabilities:CreditCard:AMEX:Liang".into()),
                    ("food".into(), "Expense:Food".into()),
                ]
                .iter()
                .cloned()
                .collect(),
            },
        };
        let result = parser.parse("2021-09-08    @KFC    hamburger   12.40   AUD   cba >   food");
        assert!(result.is_ok());
        let transaction = result.unwrap();
        assert_eq!(transaction.year(), "2021");
        let actual_text: String = transaction.into();
        assert_eq!("2021-09-08 * \"KFC\" \"hamburger\"\n  Assets:MasterCard:CBA        -12.40 AUD\n  Expense:Food        12.40 AUD\n", actual_text);
    }
    #[test]
    fn parser_can_parse_input_without_date() {
        let parser = Parser {
            settings: Settings {
                currency: "AUD".into(),
                accounts: [
                    ("cba".into(), "Assets:MasterCard:CBA".into()),
                    ("amex".into(), "Liabilities:CreditCard:AMEX:Liang".into()),
                    ("food".into(), "Expense:Food".into()),
                ]
                .iter()
                .cloned()
                .collect(),
            },
        };
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
    fn parser_support_input_without_space_before_right_arrow() {
        let parser = Parser {
            settings: Settings {
                currency: "AUD".into(),
                accounts: [
                    ("cba".into(), "Assets:MasterCard:CBA".into()),
                    ("amex".into(), "Liabilities:CreditCard:AMEX:Liang".into()),
                    ("food".into(), "Expense:Food".into()),
                ]
                .iter()
                .cloned()
                .collect(),
            },
        };
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
        let parser = Parser {
            settings: Settings {
                currency: "AUD".into(),
                accounts: [
                    ("cba".into(), "Assets:MasterCard:CBA".into()),
                    ("amex".into(), "Liabilities:CreditCard:AMEX:Liang".into()),
                    ("food".into(), "Expense:Food".into()),
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
        assert_eq!(transaction.to_account, "Expense:Food");
    }

    #[test]
    fn parser_can_parse_input_in_payee_amount_currency_from_account_to_account_format() {
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
        let result = parser.parse("@KFL 22.34 USD cba > food");
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
        let result = parser.parse("2021-11-23 @KFL 22.34 cba > food");
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
    fn parser_return_error_for_invalid_input() {
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

        let result = parser.parse("2022-08-14 @MelbourneZoo 33.7 abc > home");
        assert!(result.is_err());
    }
}
