use chrono::{Local, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use eframe::egui;
use egui::Image;
use egui_plot::{Bar, BarChart, Legend, Plot};
use std::collections::HashMap;

use crate::{
    database::Database,
    models::{Direction, MoneyType, Person, PersonStats, Transaction},
};

#[derive(PartialEq)]
enum Tab {
    AddTransaction,
    Analysis,
    Transactions,
}

impl Default for Tab {
    fn default() -> Self {
        Tab::AddTransaction
    }
}

pub struct BankingApp {
    db: Database,

    person_name: String,
    amount: String,
    money_type: MoneyType,
    direction: Direction,
    selected_date: NaiveDate,
    selected_hour: u32,
    selected_minute: u32,
    has_expected_return: bool,
    expected_return_date: NaiveDate,

    current_tab: Tab,
    status_message: String,

    pub logo_texture: Option<egui::TextureHandle>,
}

impl Default for BankingApp {
    fn default() -> Self {
        let now = Local::now();
        Self {
            db: Database::load(),
            person_name: String::new(),
            amount: String::new(),
            money_type: MoneyType::GEL,
            direction: Direction::Lent,
            selected_date: now.date_naive(),
            selected_hour: now.hour(),
            selected_minute: now.minute(),
            has_expected_return: false,
            expected_return_date: now.date_naive(),
            current_tab: Tab::AddTransaction,
            status_message: String::new(),
            logo_texture: None,
        }
    }
}

impl eframe::App for BankingApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if let Some(tex) = &self.logo_texture {
                    ui.add(Image::new(tex).fit_to_exact_size(egui::vec2(32.0, 32.0)));
                } else {
                    ui.heading("💰");
                }
                ui.heading("Shalom, let's track some goyim!");
                ui.heading("💰👃💰");
            });
            ui.separator();

            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut self.current_tab,
                    Tab::AddTransaction,
                    "Add Transaction",
                );
                ui.selectable_value(&mut self.current_tab, Tab::Analysis, "Analysis");
                ui.selectable_value(
                    &mut self.current_tab,
                    Tab::Transactions,
                    "View Transactions",
                );
            });

            ui.separator();

            match self.current_tab {
                Tab::AddTransaction => self.show_add_transaction(ui),
                Tab::Analysis => self.show_analysis(ui),
                Tab::Transactions => self.show_transactions(ui),
            }
        });
    }
}

impl BankingApp {
    fn show_add_transaction(&mut self, ui: &mut egui::Ui) {
        ui.heading("Add New Transaction");
        ui.add_space(10.0);

        egui::Grid::new("transaction_form")
            .num_columns(2)
            .spacing([40.0, 8.0])
            .show(ui, |ui| {
                ui.label("Person:");
                ui.text_edit_singleline(&mut self.person_name);
                ui.end_row();

                ui.label("Amount:");
                ui.text_edit_singleline(&mut self.amount);
                ui.end_row();

                ui.label("Currency:");
                egui::ComboBox::from_id_source("money_type")
                    .selected_text(format!("{:?}", self.money_type))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.money_type, MoneyType::GEL, "GEL");
                        ui.selectable_value(&mut self.money_type, MoneyType::USD, "USD");
                        ui.selectable_value(&mut self.money_type, MoneyType::EUR, "EUR");
                        ui.selectable_value(&mut self.money_type, MoneyType::GBP, "GBP");
                        ui.selectable_value(&mut self.money_type, MoneyType::RUB, "RUB");
                        ui.selectable_value(&mut self.money_type, MoneyType::Other, "Other");
                    });
                ui.end_row();

                ui.label("Direction:");
                egui::ComboBox::from_id_source("direction")
                    .selected_text(format!("{:?}", self.direction))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.direction, Direction::Lent, "Lent (I gave)");
                        ui.selectable_value(
                            &mut self.direction,
                            Direction::Borrowed,
                            "Borrowed (I received)",
                        );
                        ui.selectable_value(
                            &mut self.direction,
                            Direction::Returned,
                            "Returned (They gave back)",
                        );
                        ui.selectable_value(
                            &mut self.direction,
                            Direction::Repaid,
                            "Repaid (I gave back)",
                        );
                    });
                ui.end_row();

                ui.label("Date:");
                ui.add(egui_extras::DatePickerButton::new(&mut self.selected_date));
                ui.end_row();

                ui.label("Time:");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut self.selected_hour).clamp_range(0..=23));
                    ui.label(":");
                    ui.add(egui::DragValue::new(&mut self.selected_minute).clamp_range(0..=59));
                });
                ui.end_row();

                ui.label("Expected Return:");
                ui.horizontal(|ui| {
                    let can_set_expected =
                        matches!(self.direction, Direction::Lent | Direction::Borrowed);

                    if can_set_expected {
                        ui.checkbox(&mut self.has_expected_return, "Set expected date");
                        if self.has_expected_return {
                            ui.add(egui_extras::DatePickerButton::new(
                                &mut self.expected_return_date,
                            ));
                        }
                    } else {
                        ui.label("(Not applicable for returns/repayments)");
                        self.has_expected_return = false;
                    }
                });
                ui.end_row();
            });

        ui.add_space(10.0);

        if ui.button("Add Transaction").clicked() {
            if let Ok(amount) = self.amount.parse::<f64>() {
                if !self.person_name.trim().is_empty() && amount > 0.0 {
                    let time = NaiveTime::from_hms_opt(self.selected_hour, self.selected_minute, 0)
                        .unwrap_or_else(|| NaiveTime::from_hms_opt(0, 0, 0).unwrap());
                    let datetime = NaiveDateTime::new(self.selected_date, time);

                    let transaction = Transaction {
                        person: Person {
                            name: self.person_name.trim().to_string(),
                        },
                        amount,
                        money_type: self.money_type,
                        direction: self.direction,
                        datetime,
                        expected_return_date: if self.has_expected_return {
                            Some(self.expected_return_date)
                        } else {
                            None
                        },
                    };

                    self.db.add_transaction(transaction);
                    if let Err(e) = self.db.save() {
                        self.status_message = format!("Error saving: {}", e);
                    } else {
                        self.status_message = "Transaction added successfully!".to_string();
                        self.person_name.clear();
                        self.amount.clear();
                        self.has_expected_return = false;
                    }
                } else {
                    self.status_message =
                        "Invalid input: name required and amount must be positive".to_string();
                }
            } else {
                self.status_message = "Invalid amount".to_string();
            }
        }

        if !self.status_message.is_empty() {
            ui.add_space(10.0);
            ui.colored_label(egui::Color32::GREEN, &self.status_message);
        }
    }

    fn show_analysis(&mut self, ui: &mut egui::Ui) {
        ui.heading("Financial Analysis");
        ui.add_space(10.0);

        if self.db.transactions.is_empty() {
            ui.label("No transactions yet. Add some to see analysis!");
            return;
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut balances_by_currency: HashMap<MoneyType, f64> = HashMap::new();
            let mut total_lent = 0.0;
            let mut total_borrowed = 0.0;
            let mut total_returned = 0.0;
            let mut total_repaid = 0.0;

            for t in &self.db.transactions {
                let balance = balances_by_currency.entry(t.money_type).or_insert(0.0);

                match t.direction {
                    Direction::Lent => {
                        *balance -= t.amount;
                        total_lent += t.amount;
                    }
                    Direction::Borrowed => {
                        *balance += t.amount;
                        total_borrowed += t.amount;
                    }
                    Direction::Returned => {
                        *balance += t.amount;
                        total_returned += t.amount;
                    }
                    Direction::Repaid => {
                        *balance -= t.amount;
                        total_repaid += t.amount;
                    }
                }
            }

            ui.group(|ui| {
                ui.strong("Net Balance by Currency:");
                for (currency, balance) in &balances_by_currency {
                    let color = if *balance > 0.0 {
                        egui::Color32::GREEN
                    } else if *balance < 0.0 {
                        egui::Color32::RED
                    } else {
                        egui::Color32::GRAY
                    };
                    ui.colored_label(color, format!("{}{:.2}", currency.symbol(), balance));
                }
            });

            ui.add_space(15.0);

            ui.collapsing("Transaction Distribution Chart", |ui| {
                ui.add_space(5.0);
                Plot::new("transaction_dist")
                    .legend(Legend::default())
                    .show_axes([false, true])
                    .height(200.0)
                    .show(ui, |plot_ui| {
                        let bars = vec![
                            Bar::new(0.0, total_lent)
                                .name("Lent")
                                .fill(egui::Color32::from_rgb(255, 100, 100)),
                            Bar::new(1.0, total_borrowed)
                                .name("Borrowed")
                                .fill(egui::Color32::from_rgb(100, 150, 255)),
                            Bar::new(2.0, total_returned)
                                .name("Returned")
                                .fill(egui::Color32::from_rgb(100, 200, 100)),
                            Bar::new(3.0, total_repaid)
                                .name("Repaid")
                                .fill(egui::Color32::from_rgb(150, 255, 150)),
                        ];
                        plot_ui.bar_chart(BarChart::new(bars).width(0.7));
                    });
            });

            ui.add_space(15.0);
            ui.separator();

            ui.heading("Per-Person Analysis");

            let mut person_data: HashMap<String, PersonStats> = HashMap::new();

            for t in &self.db.transactions {
                let stats = person_data
                    .entry(t.person.name.clone())
                    .or_insert(PersonStats::default());

                stats.currencies.insert(t.money_type);

                match t.direction {
                    Direction::Lent => {
                        stats.lent += t.amount;
                        stats.outstanding += t.amount;
                        stats.lent_transactions.push(t.clone());
                    }
                    Direction::Borrowed => {
                        stats.borrowed += t.amount;
                        stats.outstanding -= t.amount;
                    }
                    Direction::Returned => {
                        stats.returned += t.amount;
                        stats.outstanding -= t.amount;
                        stats.return_transactions.push(t.clone());
                    }
                    Direction::Repaid => {
                        stats.repaid += t.amount;
                        stats.outstanding += t.amount;
                    }
                }
            }

            let mut people: Vec<_> = person_data.iter().collect();
            people.sort_by(|a, b| {
                let a_reliability = if a.1.lent > 0.0 {
                    a.1.returned / a.1.lent
                } else {
                    0.0
                };
                let b_reliability = if b.1.lent > 0.0 {
                    b.1.returned / b.1.lent
                } else {
                    0.0
                };

                match b_reliability
                    .partial_cmp(&a_reliability)
                    .unwrap_or(std::cmp::Ordering::Equal)
                {
                    std::cmp::Ordering::Equal => a.0.cmp(b.0),
                    other => other,
                }
            });

            if !people.is_empty() {
                ui.collapsing("Outstanding Balances by Person", |ui| {
                    ui.add_space(5.0);
                    Plot::new("outstanding_balances")
                        .legend(Legend::default())
                        .show_axes([false, true])
                        .height(200.0)
                        .show(ui, |plot_ui| {
                            let bars: Vec<Bar> = people
                                .iter()
                                .enumerate()
                                .map(|(i, (name, stats))| {
                                    let color = if stats.outstanding > 0.0 {
                                        egui::Color32::from_rgb(255, 100, 100)
                                    } else if stats.outstanding < 0.0 {
                                        egui::Color32::from_rgb(100, 200, 100)
                                    } else {
                                        egui::Color32::GRAY
                                    };
                                    Bar::new(i as f64, stats.outstanding)
                                        .name(name.as_str())
                                        .fill(color)
                                })
                                .collect();
                            plot_ui.bar_chart(BarChart::new(bars).width(0.7));
                        });
                });

                ui.add_space(15.0);
            }

            ui.heading("Individual Statistics");
            ui.add_space(10.0);

            let available_width = ui.available_width();
            let card_width = 300.0;
            let columns = (available_width / card_width).floor().max(1.0) as usize;

            egui::Grid::new("people_grid")
                .spacing([10.0, 10.0])
                .num_columns(columns)
                .show(ui, |ui| {
                    for (idx, (name, stats)) in people.iter().enumerate() {
                        ui.vertical(|ui| {
                            ui.set_width(card_width - 20.0);
                            ui.group(|ui| {
                                ui.set_width(card_width - 30.0);
                                ui.label(egui::RichText::new(name.as_str()).strong().size(16.0));
                                ui.separator();

                                let color = if stats.outstanding > 0.0 {
                                    egui::Color32::RED
                                } else if stats.outstanding < 0.0 {
                                    egui::Color32::GREEN
                                } else {
                                    egui::Color32::GRAY
                                };

                                let currency_symbol = if stats.currencies.len() == 1 {
                                    *stats.currencies.iter().next().unwrap()
                                } else {
                                    MoneyType::USD
                                };
                                ui.colored_label(
                                    color,
                                    format!("{}{:.2}", currency_symbol.symbol(), stats.outstanding),
                                );
                                ui.label(format!(
                                    "Total Lent: {}{:.2}",
                                    currency_symbol.symbol(),
                                    stats.lent
                                ));
                                ui.label(format!(
                                    "Total Returned: {}{:.2}",
                                    currency_symbol.symbol(),
                                    stats.returned
                                ));

                                if stats.lent > 0.0 {
                                    let return_rate = (stats.returned / stats.lent) * 100.0;
                                    ui.label(format!("Return Rate: {:.1}%", return_rate));

                                    if let Some(avg_days) = calculate_avg_return_time(
                                        &stats.lent_transactions,
                                        &stats.return_transactions,
                                    ) {
                                        ui.label(format!("Avg Return Time: {:.0} days", avg_days));
                                    }

                                    if let Some((kept, total)) = calculate_promise_keeping_rate(
                                        &stats.lent_transactions,
                                        &stats.return_transactions,
                                    ) {
                                        let rate = (kept as f64 / total as f64) * 100.0;
                                        ui.label(format!(
                                            "Promises Kept: {:.1}% ({}/{})",
                                            rate, kept, total
                                        ));
                                    }
                                }
                            });
                        });

                        if (idx + 1) % columns == 0 {
                            ui.end_row();
                        }
                    }
                });
        });
    }

    fn show_transactions(&mut self, ui: &mut egui::Ui) {
        ui.heading("Transaction History");
        ui.add_space(10.0);

        if self.db.transactions.is_empty() {
            ui.label("No transactions yet.");
            return;
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            for (i, t) in self.db.transactions.iter().enumerate().rev() {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(format!("#{}", self.db.transactions.len() - i));
                        ui.separator();
                        ui.label(&t.person.name);
                        ui.separator();
                        ui.label(format!("{}{:.2}", t.money_type.symbol(), t.amount));
                        ui.separator();
                        ui.label(format!("{:?}", t.direction));
                        ui.separator();
                        ui.label(format!("{:?}", t.money_type));
                        ui.separator();
                        ui.label(t.datetime.format("%Y-%m-%d %H:%M").to_string());
                        if let Some(expected) = t.expected_return_date {
                            ui.separator();
                            ui.colored_label(
                                egui::Color32::LIGHT_BLUE,
                                format!("Expected: {}", expected.format("%Y-%m-%d")),
                            );
                        }
                    });
                });
            }
        });
    }
}

fn calculate_avg_return_time(lent: &[Transaction], returned: &[Transaction]) -> Option<f64> {
    if lent.is_empty() || returned.is_empty() {
        return None;
    }

    let mut total_days = 0i64;
    let mut count = 0;

    for ret in returned {
        if let Some(lent_tx) = lent
            .iter()
            .filter(|l| l.datetime <= ret.datetime)
            .max_by_key(|l| l.datetime)
        {
            let days = (ret.datetime.date() - lent_tx.datetime.date()).num_days();
            total_days += days;
            count += 1;
        }
    }

    if count > 0 {
        Some(total_days as f64 / count as f64)
    } else {
        None
    }
}

fn calculate_promise_keeping_rate(
    lent: &[Transaction],
    returned: &[Transaction],
) -> Option<(usize, usize)> {
    let lent_with_expected: Vec<_> = lent
        .iter()
        .filter(|t| t.expected_return_date.is_some())
        .collect();

    if lent_with_expected.is_empty() {
        return None;
    }

    let mut promises_kept = 0;
    let mut total_promises = 0;

    for lent_tx in lent_with_expected {
        if let Some(expected_date) = lent_tx.expected_return_date {
            total_promises += 1;

            if let Some(return_tx) = returned
                .iter()
                .filter(|r| r.datetime >= lent_tx.datetime)
                .min_by_key(|r| r.datetime)
            {
                if return_tx.datetime.date() <= expected_date {
                    promises_kept += 1;
                }
            }
        }
    }

    if total_promises > 0 {
        Some((promises_kept, total_promises))
    } else {
        None
    }
}
