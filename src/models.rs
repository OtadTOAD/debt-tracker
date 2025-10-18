use chrono::{NaiveDate, NaiveDateTime};
use egui::ahash::HashSet;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Person {
    pub name: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MoneyType {
    GEL,
    USD,
    EUR,
    GBP,
    RUB,
    Other,
}

impl MoneyType {
    pub fn symbol(&self) -> &str {
        match self {
            MoneyType::GEL => "₾",
            MoneyType::USD => "$",
            MoneyType::EUR => "€",
            MoneyType::GBP => "£",
            MoneyType::RUB => "₽",
            MoneyType::Other => "¤",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Direction {
    Lent,
    Borrowed,
    Returned,
    Repaid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub person: Person,
    pub amount: f64,
    pub money_type: MoneyType,
    pub direction: Direction,
    pub datetime: NaiveDateTime,
    pub expected_return_date: Option<NaiveDate>,
    pub attachment_path: Option<String>,
}

#[derive(Default)]
pub struct PersonStats {
    pub lent: f64,
    pub borrowed: f64,
    pub returned: f64,
    pub repaid: f64,
    pub outstanding: f64,
    pub lent_transactions: Vec<Transaction>,
    pub return_transactions: Vec<Transaction>,
    pub currencies: HashSet<MoneyType>,
}

#[derive(PartialEq, Clone, Copy)]
pub enum SortBy {
    DateNewest,
    DateOldest,
    AmountHighest,
    AmountLowest,
    Person,
}
