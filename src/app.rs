use chrono::{Local, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use eframe::egui;
use egui::Image;
use egui_plot::{Bar, BarChart, Legend, Line, Plot};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::{
    database::Database,
    models::{DeadlineChange, Direction, MoneyType, Person, PersonStats, SortBy, Transaction},
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
    attachment_path: Option<String>,

    current_tab: Tab,
    status_message: String,

    pub logo_texture: Option<egui::TextureHandle>,

    search_query: String,
    sort_by: SortBy,

    edit_transaction_index: Option<usize>,
    attachment_textures: HashMap<String, egui::TextureHandle>,
    viewing_attachment: Option<String>,

    editing_deadline_for: Option<usize>,
    temp_new_deadline: NaiveDate,
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
            attachment_path: None,
            current_tab: Tab::AddTransaction,
            status_message: String::new(),
            logo_texture: None,
            search_query: String::new(),
            sort_by: SortBy::DateNewest,
            edit_transaction_index: None,
            attachment_textures: HashMap::new(),
            viewing_attachment: None,
            editing_deadline_for: None,
            temp_new_deadline: now.date_naive(),
        }
    }
}

impl eframe::App for BankingApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(ref path) = self.viewing_attachment.clone() {
            egui::Window::new("📷 Attachment Viewer")
                .collapsible(false)
                .resizable(true)
                .default_width(600.0)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        if let Some(texture) = self.attachment_textures.get(path) {
                            let max_size = egui::vec2(800.0, 600.0);
                            let img_size = texture.size_vec2();
                            let scale = (max_size.x / img_size.x)
                                .min(max_size.y / img_size.y)
                                .min(1.0);
                            let display_size = img_size * scale;

                            ui.add(Image::new(texture).fit_to_exact_size(display_size));
                        } else {
                            ui.label("Failed to load image");
                        }

                        if ui.button("Close").clicked() {
                            self.viewing_attachment = None;
                        }
                    });
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    if let Some(tex) = &self.logo_texture {
                        ui.add(Image::new(tex).fit_to_exact_size(egui::vec2(40.0, 40.0)));
                    } else {
                        ui.heading("💰");
                    }
                    ui.heading(
                        egui::RichText::new("Shalom, let's track some goyim!")
                            .size(28.0)
                            .strong(),
                    );
                    ui.heading("💰💸💰");
                });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    let available = ui.available_width();
                    let total_btn_width = 450.0;
                    let margin = (available - total_btn_width) / 2.0;
                    ui.add_space(margin);

                    let btn_style = |selected: bool| -> egui::Button {
                        let color = if selected {
                            egui::Color32::from_rgb(100, 150, 255)
                        } else {
                            egui::Color32::from_rgb(60, 80, 120)
                        };
                        let text_color = egui::Color32::WHITE;

                        egui::Button::new(
                            egui::RichText::new("➕ Add Transaction")
                                .size(14.0)
                                .color(text_color)
                                .strong(),
                        )
                        .fill(color)
                        .min_size([140.0, 40.0].into())
                    };

                    if ui
                        .add(btn_style(self.current_tab == Tab::AddTransaction))
                        .clicked()
                    {
                        self.current_tab = Tab::AddTransaction;
                    }

                    let btn_style = |selected: bool| -> egui::Button {
                        let color = if selected {
                            egui::Color32::from_rgb(100, 200, 100)
                        } else {
                            egui::Color32::from_rgb(60, 120, 60)
                        };
                        let text_color = egui::Color32::WHITE;

                        egui::Button::new(
                            egui::RichText::new("📊 Analysis")
                                .size(14.0)
                                .color(text_color)
                                .strong(),
                        )
                        .fill(color)
                        .min_size([140.0, 40.0].into())
                    };

                    if ui
                        .add(btn_style(self.current_tab == Tab::Analysis))
                        .clicked()
                    {
                        self.current_tab = Tab::Analysis;
                    }

                    let btn_style = |selected: bool| -> egui::Button {
                        let color = if selected {
                            egui::Color32::from_rgb(255, 150, 100)
                        } else {
                            egui::Color32::from_rgb(150, 90, 60)
                        };
                        let text_color = egui::Color32::WHITE;

                        egui::Button::new(
                            egui::RichText::new("📜 History")
                                .size(14.0)
                                .color(text_color)
                                .strong(),
                        )
                        .fill(color)
                        .min_size([140.0, 40.0].into())
                    };

                    if ui
                        .add(btn_style(self.current_tab == Tab::Transactions))
                        .clicked()
                    {
                        self.current_tab = Tab::Transactions;
                    }
                });

                ui.add_space(15.0);
                ui.separator();
                ui.add_space(15.0);

                match self.current_tab {
                    Tab::AddTransaction => self.show_add_transaction(ui),
                    Tab::Analysis => self.show_analysis(ui),
                    Tab::Transactions => self.show_transactions(ui, ctx),
                }
            });
        });
    }
}

impl BankingApp {
    fn show_add_transaction(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading(
                egui::RichText::new("Add New Transaction")
                    .size(24.0)
                    .strong(),
            );
            ui.add_space(20.0);

            let max_width = 500.0;
            ui.allocate_ui_with_layout(
                egui::vec2(max_width, ui.available_height()),
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    egui::Grid::new("transaction_form")
                        .num_columns(2)
                        .spacing([40.0, 15.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("👤 Person:").size(14.0));
                            ui.text_edit_singleline(&mut self.person_name);
                            ui.end_row();

                            ui.label(egui::RichText::new("💵 Amount:").size(14.0));
                            ui.text_edit_singleline(&mut self.amount);
                            ui.end_row();

                            ui.label(egui::RichText::new("💱 Currency:").size(14.0));
                            egui::ComboBox::from_id_source("money_type")
                                .selected_text(format!("{:?}", self.money_type))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.money_type,
                                        MoneyType::GEL,
                                        "GEL",
                                    );
                                    ui.selectable_value(
                                        &mut self.money_type,
                                        MoneyType::USD,
                                        "USD",
                                    );
                                    ui.selectable_value(
                                        &mut self.money_type,
                                        MoneyType::EUR,
                                        "EUR",
                                    );
                                    ui.selectable_value(
                                        &mut self.money_type,
                                        MoneyType::GBP,
                                        "GBP",
                                    );
                                    ui.selectable_value(
                                        &mut self.money_type,
                                        MoneyType::RUB,
                                        "RUB",
                                    );
                                    ui.selectable_value(
                                        &mut self.money_type,
                                        MoneyType::Other,
                                        "Other",
                                    );
                                });
                            ui.end_row();

                            ui.label(egui::RichText::new("🔄 Direction:").size(14.0));
                            egui::ComboBox::from_id_source("direction")
                                .selected_text(format!("{:?}", self.direction))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.direction,
                                        Direction::Lent,
                                        "Lent (I gave)",
                                    );
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

                            ui.label(egui::RichText::new("📅 Date:").size(14.0));
                            ui.add(egui_extras::DatePickerButton::new(&mut self.selected_date));
                            ui.end_row();

                            ui.label(egui::RichText::new("⏰ Time:").size(14.0));
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::DragValue::new(&mut self.selected_hour)
                                        .clamp_range(0..=23),
                                );
                                ui.label(":");
                                ui.add(
                                    egui::DragValue::new(&mut self.selected_minute)
                                        .clamp_range(0..=59),
                                );
                            });
                            ui.end_row();

                            ui.label(egui::RichText::new("📖 Expected Return:").size(14.0));
                            ui.horizontal(|ui| {
                                let can_set_expected =
                                    matches!(self.direction, Direction::Lent | Direction::Borrowed);

                                if can_set_expected {
                                    ui.checkbox(&mut self.has_expected_return, "Set date");
                                    if self.has_expected_return {
                                        ui.add(egui_extras::DatePickerButton::new(
                                            &mut self.expected_return_date,
                                        ));
                                    }
                                } else {
                                    ui.label(egui::RichText::new("(N/A)").weak());
                                    self.has_expected_return = false;
                                }
                            });
                            ui.end_row();

                            ui.label(egui::RichText::new("📎 Attachment:").size(14.0));
                            ui.horizontal(|ui| {
                                if ui.button("📁 Browse...").clicked() {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .add_filter("Images", &["png", "jpg", "jpeg", "gif", "bmp"])
                                        .pick_file()
                                    {
                                        self.attachment_path =
                                            Some(path.to_string_lossy().to_string());
                                    }
                                }

                                if let Some(ref path) = self.attachment_path {
                                    ui.label(
                                        PathBuf::from(path)
                                            .file_name()
                                            .and_then(|n| n.to_str())
                                            .unwrap_or("file"),
                                    );
                                    if ui.small_button("❌").clicked() {
                                        self.attachment_path = None;
                                    }
                                } else {
                                    ui.label(egui::RichText::new("None").weak());
                                }
                            });
                            ui.end_row();
                        });
                },
            );

            ui.add_space(20.0);

            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new("✅ Add Transaction")
                            .size(16.0)
                            .strong()
                            .color(egui::Color32::WHITE),
                    )
                    .fill(egui::Color32::from_rgb(100, 200, 100))
                    .min_size([150.0, 45.0].into()),
                )
                .clicked()
            {
                if let Ok(amount) = self.amount.parse::<f64>() {
                    if !self.person_name.trim().is_empty() && amount > 0.0 {
                        let time =
                            NaiveTime::from_hms_opt(self.selected_hour, self.selected_minute, 0)
                                .unwrap_or_else(|| NaiveTime::from_hms_opt(0, 0, 0).unwrap());
                        let datetime = NaiveDateTime::new(self.selected_date, time);

                        let stored_attachment = if let Some(ref path) = self.attachment_path {
                            match Database::copy_attachment_to_storage(path) {
                                Ok(stored_path) => Some(stored_path),
                                Err(e) => {
                                    self.status_message =
                                        format!("⚠️ Failed to copy attachment: {}", e);
                                    None
                                }
                            }
                        } else {
                            None
                        };

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
                            attachment_path: stored_attachment,
                            deadline_changes: Vec::new(),
                        };

                        self.db.add_transaction(transaction);
                        if let Err(e) = self.db.save() {
                            self.status_message = format!("❌ Error saving: {}", e);
                        } else {
                            self.status_message = "✅ Transaction added successfully!".to_string();
                            self.person_name.clear();
                            self.amount.clear();
                            self.has_expected_return = false;
                            self.attachment_path = None;
                        }
                    } else {
                        self.status_message =
                            "⚠️ Invalid input: name required and amount must be positive"
                                .to_string();
                    }
                } else {
                    self.status_message = "⚠️ Invalid amount".to_string();
                }
            }

            if !self.status_message.is_empty() {
                ui.add_space(15.0);
                let color = if self.status_message.starts_with("✅") {
                    egui::Color32::GREEN
                } else if self.status_message.starts_with("❌") {
                    egui::Color32::RED
                } else {
                    egui::Color32::YELLOW
                };
                ui.colored_label(color, egui::RichText::new(&self.status_message).size(14.0));
            }
        });
    }

    fn show_analysis(&mut self, ui: &mut egui::Ui) {
        if self.db.transactions.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.heading("🔭 No transactions yet");
                ui.label("Add some transactions to see detailed analysis!");
            });
            return;
        }

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                let available_width = ui.available_width();
                let content_width = (available_width - 40.0).min(1200.0);
                let margin = (available_width - content_width) / 2.0;

                ui.add_space(15.0);

                ui.vertical_centered(|ui| {
                    ui.heading(
                        egui::RichText::new("Financial Analysis Dashboard")
                            .size(24.0)
                            .strong(),
                    );
                });

                ui.add_space(20.0);

                let mut balances_by_currency: HashMap<MoneyType, f64> = HashMap::new();
                let mut total_lent = 0.0;
                let mut total_borrowed = 0.0;
                let mut total_returned = 0.0;
                let mut total_repaid = 0.0;

                for currency in [MoneyType::GEL, MoneyType::USD, MoneyType::EUR] {
                    balances_by_currency.insert(currency, 0.0);
                }

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

                ui.horizontal(|ui| {
                    ui.add_space(margin);
                    ui.vertical(|ui| {
                        ui.set_width(content_width);

                        let card_min_width = 160.0;
                        let card_spacing = 10.0;
                        let cards_per_row = ((content_width + card_spacing)
                            / (card_min_width + card_spacing))
                            .floor()
                            .max(1.0) as usize;

                        egui::Grid::new("stats_grid")
                            .spacing([card_spacing, card_spacing])
                            .num_columns(cards_per_row)
                            .show(ui, |ui| {
                                let stats = [
                                    (
                                        "💸 Total Lent",
                                        total_lent,
                                        egui::Color32::from_rgb(255, 130, 130),
                                    ),
                                    (
                                        "🔥 Total Borrowed",
                                        total_borrowed,
                                        egui::Color32::from_rgb(120, 160, 255),
                                    ),
                                    (
                                        "✅ Total Returned",
                                        total_returned,
                                        egui::Color32::from_rgb(120, 220, 120),
                                    ),
                                    (
                                        "💳 Total Repaid",
                                        total_repaid,
                                        egui::Color32::from_rgb(200, 255, 150),
                                    ),
                                ];

                                for (idx, (label, value, color)) in stats.iter().enumerate() {
                                    ui.group(|ui| {
                                        ui.set_min_width(card_min_width);
                                        ui.vertical_centered(|ui| {
                                            ui.colored_label(
                                                *color,
                                                egui::RichText::new(*label).size(14.0).strong(),
                                            );
                                            ui.label(
                                                egui::RichText::new(format!("{:.2}", value))
                                                    .size(20.0)
                                                    .strong(),
                                            );
                                        });
                                    });

                                    if (idx + 1) % cards_per_row == 0 {
                                        ui.end_row();
                                    }
                                }
                            });
                    });
                });

                ui.add_space(25.0);

                ui.horizontal(|ui| {
                    ui.add_space(margin);
                    ui.group(|ui| {
                        ui.set_width(content_width);
                        ui.vertical(|ui| {
                            ui.heading(
                                egui::RichText::new("💼 Current Net Balance by Currency")
                                    .size(18.0)
                                    .strong(),
                            );
                            ui.add_space(10.0);

                            let mut currencies: Vec<_> = balances_by_currency.iter().collect();
                            currencies.sort_by(|a, b| {
                                let abs_cmp =
                                    b.1.abs()
                                        .partial_cmp(&a.1.abs())
                                        .unwrap_or(std::cmp::Ordering::Equal);
                                if abs_cmp == std::cmp::Ordering::Equal {
                                    format!("{:?}", a.0).cmp(&format!("{:?}", b.0))
                                } else {
                                    abs_cmp
                                }
                            });

                            let card_width = 140.0;
                            let card_spacing = 10.0;
                            let num_cards = currencies.len() as f32;
                            let total_width =
                                (card_width * num_cards) + (card_spacing * (num_cards - 1.0));
                            let left_padding = ((content_width - total_width) / 2.0).max(0.0);

                            ui.horizontal(|ui| {
                                ui.add_space(left_padding);
                                for (idx, (currency, balance)) in currencies.iter().enumerate() {
                                    let color = if **balance > 0.0 {
                                        egui::Color32::from_rgb(100, 200, 100)
                                    } else if **balance < 0.0 {
                                        egui::Color32::from_rgb(255, 120, 120)
                                    } else {
                                        egui::Color32::GRAY
                                    };

                                    ui.group(|ui| {
                                        ui.set_width(card_width);
                                        ui.vertical_centered(|ui| {
                                            ui.label(format!("{:?}", currency));
                                            ui.colored_label(
                                                color,
                                                egui::RichText::new(format!(
                                                    "{}{:.2}",
                                                    currency.symbol(),
                                                    balance
                                                ))
                                                .size(18.0)
                                                .strong(),
                                            );
                                        });
                                    });

                                    if idx < currencies.len() - 1 {
                                        ui.add_space(card_spacing);
                                    }
                                }
                            });
                        });
                    });
                });

                ui.add_space(30.0);

                ui.horizontal(|ui| {
                    ui.add_space(margin);
                    ui.group(|ui| {
                        ui.set_width(content_width);
                        ui.vertical(|ui| {
                            ui.heading(
                                egui::RichText::new("📊 Balance Timeline")
                                    .size(20.0)
                                    .strong(),
                            );
                            ui.add_space(15.0);

                            let timeline = self.generate_balance_timeline();

                            Plot::new("balance_timeline")
                                .legend(Legend::default().position(egui_plot::Corner::LeftTop))
                                .show_axes([true, true])
                                .height(400.0)
                                .allow_scroll(false)
                                .allow_zoom(true)
                                .allow_drag(true)
                                .width(content_width - 40.0)
                                .show(ui, |plot_ui| {
                                    for (idx, (currency, points)) in timeline.iter().enumerate() {
                                        let color = match idx {
                                            0 => egui::Color32::from_rgb(255, 100, 100),
                                            1 => egui::Color32::from_rgb(100, 150, 255),
                                            2 => egui::Color32::from_rgb(100, 220, 100),
                                            3 => egui::Color32::from_rgb(255, 180, 50),
                                            4 => egui::Color32::from_rgb(200, 100, 255),
                                            _ => egui::Color32::GRAY,
                                        };

                                        let line = Line::new(points.clone())
                                            .name(format!("{:?}", currency))
                                            .stroke(egui::Stroke::new(3.0, color));
                                        plot_ui.line(line);
                                    }
                                });
                        });
                    });
                });

                ui.add_space(30.0);

                ui.horizontal(|ui| {
                    ui.add_space(margin);
                    ui.vertical(|ui| {
                        ui.set_width(content_width);

                        let chart_min_width = 300.0;
                        let use_single_column = content_width < (chart_min_width * 2.0 + 20.0);

                        if use_single_column {
                            self.draw_outstanding_chart(ui, content_width);
                            ui.add_space(20.0);
                            self.draw_return_rate_chart(ui, content_width);
                        } else {
                            let chart_width = (content_width - 20.0) / 2.0;
                            ui.columns(2, |columns| {
                                self.draw_outstanding_chart(&mut columns[0], chart_width);
                                self.draw_return_rate_chart(&mut columns[1], chart_width);
                            });
                        }
                    });
                });

                ui.add_space(30.0);

                ui.horizontal(|ui| {
                    ui.add_space(margin);
                    ui.separator();
                });

                ui.add_space(20.0);

                ui.vertical_centered(|ui| {
                    ui.heading(
                        egui::RichText::new("👥 Individual Statistics")
                            .size(20.0)
                            .strong(),
                    );
                });

                ui.add_space(15.0);

                ui.horizontal(|ui| {
                    ui.add_space(margin);
                    ui.vertical(|ui| {
                        ui.set_width(content_width);
                        ui.horizontal(|ui| {
                            ui.label("🔍 Search person:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.search_query)
                                    .desired_width(200.0),
                            );
                        });
                    });
                });

                ui.add_space(15.0);

                let person_data = self.calculate_person_stats();
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

                people.retain(|(name, _)| {
                    name.to_lowercase()
                        .contains(&self.search_query.to_lowercase())
                });

                ui.horizontal(|ui| {
                    ui.add_space(margin);
                    ui.vertical(|ui| {
                        ui.set_width(content_width);

                        let card_width = 340.0;
                        let card_spacing = 15.0;
                        let columns = ((content_width + card_spacing) / (card_width + card_spacing))
                            .floor()
                            .max(1.0) as usize;

                        egui::Grid::new("people_grid")
                            .spacing([card_spacing, card_spacing])
                            .num_columns(columns)
                            .show(ui, |ui| {
                                for (idx, (name, stats)) in people.iter().enumerate() {
                                    self.draw_person_card(ui, name, stats);
                                    if (idx + 1) % columns == 0 {
                                        ui.end_row();
                                    }
                                }
                            });
                    });
                });

                ui.add_space(30.0);
            });
    }

    fn draw_outstanding_chart(&self, ui: &mut egui::Ui, width: f32) {
        ui.group(|ui| {
            ui.set_width(width);
            ui.vertical(|ui| {
                ui.heading(
                    egui::RichText::new("👥 Outstanding by Person")
                        .size(16.0)
                        .strong(),
                );
                ui.add_space(10.0);

                let person_data = self.calculate_person_stats();
                let mut people: Vec<_> = person_data
                    .iter()
                    .filter(|(_, stats)| stats.outstanding.abs() > 0.01)
                    .collect();
                people.sort_by(|a, b| {
                    b.1.outstanding
                        .abs()
                        .partial_cmp(&a.1.outstanding.abs())
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                if !people.is_empty() {
                    Plot::new("outstanding_balances")
                        .legend(Legend::default())
                        .show_axes([true, true])
                        .height(300.0)
                        .allow_scroll(false)
                        .width(width - 40.0)
                        .show(ui, |plot_ui| {
                            let bars: Vec<Bar> = people
                                .iter()
                                .enumerate()
                                .map(|(i, (name, stats))| {
                                    let color = if stats.outstanding > 0.0 {
                                        egui::Color32::from_rgb(255, 130, 130)
                                    } else {
                                        egui::Color32::from_rgb(130, 220, 130)
                                    };
                                    Bar::new(i as f64, stats.outstanding)
                                        .name(name.as_str())
                                        .fill(color)
                                })
                                .collect();
                            plot_ui.bar_chart(BarChart::new(bars).width(0.7));
                        });
                } else {
                    ui.label("No outstanding balances");
                }
            });
        });
    }

    fn draw_return_rate_chart(&self, ui: &mut egui::Ui, width: f32) {
        ui.group(|ui| {
            ui.set_width(width);
            ui.vertical(|ui| {
                ui.heading(
                    egui::RichText::new("📈 Return Rate by Person")
                        .size(16.0)
                        .strong(),
                );
                ui.add_space(10.0);

                let person_data = self.calculate_person_stats();
                let mut people: Vec<_> = person_data
                    .iter()
                    .filter(|(_, stats)| stats.lent > 0.0)
                    .map(|(name, stats)| (name, (stats.returned / stats.lent) * 100.0))
                    .collect();
                people.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

                if !people.is_empty() {
                    Plot::new("return_rates")
                        .legend(Legend::default())
                        .show_axes([true, true])
                        .height(300.0)
                        .allow_scroll(false)
                        .width(width - 40.0)
                        .show(ui, |plot_ui| {
                            let bars: Vec<Bar> = people
                                .iter()
                                .enumerate()
                                .map(|(i, (name, rate))| {
                                    let color = if *rate >= 80.0 {
                                        egui::Color32::from_rgb(100, 220, 100)
                                    } else if *rate >= 50.0 {
                                        egui::Color32::from_rgb(255, 200, 100)
                                    } else {
                                        egui::Color32::from_rgb(255, 130, 130)
                                    };
                                    Bar::new(i as f64, *rate).name(name.as_str()).fill(color)
                                })
                                .collect();
                            plot_ui.bar_chart(BarChart::new(bars).width(0.7));
                        });
                } else {
                    ui.label("No lending history");
                }
            });
        });
    }

    fn calculate_person_stats(&self) -> HashMap<String, PersonStats> {
        let mut person_data: HashMap<String, PersonStats> = HashMap::new();

        for t in &self.db.transactions {
            let stats = person_data
                .entry(t.person.name.clone())
                .or_insert(PersonStats::default());

            stats.currencies.insert(t.money_type);

            stats.deadline_changes_count += t.deadline_changes.len();

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

        person_data
    }

    fn draw_person_card(&self, ui: &mut egui::Ui, name: &str, stats: &PersonStats) {
        ui.vertical(|ui| {
            ui.set_width(360.0);
            ui.group(|ui| {
                ui.vertical_centered(|ui| {
                    ui.set_min_height(250.0);
                    ui.set_width(340.0);

                    ui.label(egui::RichText::new(name).strong().size(16.0));
                    ui.separator();

                    let color = if stats.outstanding > 0.0 {
                        egui::Color32::from_rgb(255, 130, 130)
                    } else if stats.outstanding < 0.0 {
                        egui::Color32::from_rgb(130, 220, 130)
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
                        egui::RichText::new(format!(
                            "{}{:.2}",
                            currency_symbol.symbol(),
                            stats.outstanding
                        ))
                        .strong()
                        .size(18.0),
                    );

                    ui.add_space(10.0);
                    ui.label(format!(
                        "📤 Lent: {}{:.2}",
                        currency_symbol.symbol(),
                        stats.lent
                    ));
                    ui.label(format!(
                        "🔥 Borrowed: {}{:.2}",
                        currency_symbol.symbol(),
                        stats.borrowed
                    ));
                    ui.label(format!(
                        "✅ Returned: {}{:.2}",
                        currency_symbol.symbol(),
                        stats.returned
                    ));
                    ui.label(format!(
                        "💳 Repaid: {}{:.2}",
                        currency_symbol.symbol(),
                        stats.repaid
                    ));

                    if stats.lent > 0.0 {
                        ui.add_space(12.0);
                        let return_rate = (stats.returned / stats.lent) * 100.0;
                        ui.colored_label(
                            egui::Color32::LIGHT_BLUE,
                            format!("Return Rate: {:.1}%", return_rate),
                        );

                        if let Some(avg_days) = calculate_avg_return_time(
                            &stats.lent_transactions,
                            &stats.return_transactions,
                        ) {
                            ui.label(format!("⏱ Avg Return: {:.0} days", avg_days));
                        } else {
                            ui.label("⏱ Avg Return: N/A");
                        }

                        if let Some((kept, total)) = calculate_promise_keeping_rate(
                            &stats.lent_transactions,
                            &stats.return_transactions,
                        ) {
                            let rate = (kept as f64 / total as f64) * 100.0;
                            ui.colored_label(
                                if rate >= 80.0 {
                                    egui::Color32::GREEN
                                } else {
                                    egui::Color32::YELLOW
                                },
                                format!("💌 Promises Kept: {:.1}% ({}/{})", rate, kept, total),
                            );
                        } else {
                            ui.label("💌 Promises Kept: N/A");
                        }

                        if stats.deadline_changes_count > 0 {
                            ui.colored_label(
                                egui::Color32::YELLOW,
                                format!("🔄 Deadline Changes: {}", stats.deadline_changes_count),
                            );
                        }
                    } else {
                        ui.add_space(12.0);
                        ui.label("Return Rate: N/A");
                        ui.label("⏱ Avg Return: N/A");
                        ui.label("💌 Promises Kept: N/A");
                    }
                });
            });
        });
    }

    fn generate_balance_timeline(&self) -> HashMap<MoneyType, Vec<[f64; 2]>> {
        let mut result: HashMap<MoneyType, Vec<[f64; 2]>> = HashMap::new();
        let mut balances: HashMap<MoneyType, f64> = HashMap::new();

        let mut sorted_tx = self.db.transactions.clone();
        sorted_tx.sort_by_key(|t| t.datetime);

        for (idx, t) in sorted_tx.iter().enumerate() {
            let balance = balances.entry(t.money_type).or_insert(0.0);

            match t.direction {
                Direction::Lent => *balance -= t.amount,
                Direction::Borrowed => *balance += t.amount,
                Direction::Returned => *balance += t.amount,
                Direction::Repaid => *balance -= t.amount,
            }

            result
                .entry(t.money_type)
                .or_insert_with(Vec::new)
                .push([idx as f64, *balance]);
        }

        result
    }

    fn show_transactions(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.vertical_centered(|ui| {
            ui.add_space(15.0);
            ui.heading(
                egui::RichText::new("Transaction History")
                    .size(24.0)
                    .strong(),
            );
            ui.add_space(15.0);

            ui.horizontal(|ui| {
                let margin = (ui.available_width() - 700.0) / 2.0;
                ui.add_space(margin.max(0.0));

                ui.label("🔍 Search:");
                ui.text_edit_singleline(&mut self.search_query);

                ui.separator();

                ui.label("📌 Sort:");
                egui::ComboBox::from_id_source("sort_by")
                    .selected_text(match self.sort_by {
                        SortBy::DateNewest => "📅 Date (Newest)",
                        SortBy::DateOldest => "📅 Date (Oldest)",
                        SortBy::AmountHighest => "💰 Amount (High)",
                        SortBy::AmountLowest => "💰 Amount (Low)",
                        SortBy::Person => "👤 Person",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.sort_by,
                            SortBy::DateNewest,
                            "📅 Date (Newest)",
                        );
                        ui.selectable_value(
                            &mut self.sort_by,
                            SortBy::DateOldest,
                            "📅 Date (Oldest)",
                        );
                        ui.selectable_value(
                            &mut self.sort_by,
                            SortBy::AmountHighest,
                            "💰 Amount (High)",
                        );
                        ui.selectable_value(
                            &mut self.sort_by,
                            SortBy::AmountLowest,
                            "💰 Amount (Low)",
                        );
                        ui.selectable_value(&mut self.sort_by, SortBy::Person, "👤 Person");
                    });
            });

            ui.add_space(10.0);
        });

        if self.db.transactions.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.heading("🔭 No transactions yet");
            });
            return;
        }

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    let paid_back_indices = self.calculate_paid_back_transactions();

                    let mut filtered_txs: Vec<(usize, &Transaction)> = self
                        .db
                        .transactions
                        .iter()
                        .enumerate()
                        .filter(|(_, t)| {
                            let search_lower = self.search_query.to_lowercase();
                            t.person.name.to_lowercase().contains(&search_lower)
                                || format!("{:.2}", t.amount).contains(&search_lower)
                                || format!("{:?}", t.direction)
                                    .to_lowercase()
                                    .contains(&search_lower)
                        })
                        .collect();

                    match self.sort_by {
                        SortBy::DateNewest => {
                            filtered_txs.sort_by_key(|(_, t)| std::cmp::Reverse(t.datetime))
                        }
                        SortBy::DateOldest => filtered_txs.sort_by_key(|(_, t)| t.datetime),
                        SortBy::AmountHighest => filtered_txs.sort_by(|a, b| {
                            b.1.amount
                                .partial_cmp(&a.1.amount)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        }),
                        SortBy::AmountLowest => filtered_txs.sort_by(|a, b| {
                            a.1.amount
                                .partial_cmp(&b.1.amount)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        }),
                        SortBy::Person => {
                            filtered_txs.sort_by(|a, b| a.1.person.name.cmp(&b.1.person.name))
                        }
                    }

                    for (i, t) in filtered_txs.iter() {
                        let color = match t.direction {
                            Direction::Lent => egui::Color32::from_rgb(255, 130, 130),
                            Direction::Borrowed => egui::Color32::from_rgb(120, 160, 255),
                            Direction::Returned => egui::Color32::from_rgb(120, 220, 120),
                            Direction::Repaid => egui::Color32::from_rgb(200, 255, 150),
                        };

                        let is_paid_back = paid_back_indices.contains(i);

                        ui.group(|ui| {
                            ui.set_width(850.0);
                            ui.horizontal(|ui| {
                                ui.colored_label(egui::Color32::GRAY, format!("#{}", i + 1));
                                ui.separator();

                                let name_text = if is_paid_back {
                                    egui::RichText::new(&t.person.name).strong().strikethrough()
                                } else {
                                    egui::RichText::new(&t.person.name).strong()
                                };
                                ui.label(name_text);
                                ui.separator();

                                let amount_text = if is_paid_back {
                                    egui::RichText::new(format!(
                                        "{}{:.2}",
                                        t.money_type.symbol(),
                                        t.amount
                                    ))
                                    .strong()
                                    .strikethrough()
                                } else {
                                    egui::RichText::new(format!(
                                        "{}{:.2}",
                                        t.money_type.symbol(),
                                        t.amount
                                    ))
                                    .strong()
                                };
                                ui.colored_label(color, amount_text);
                                ui.separator();

                                let direction_text = if is_paid_back {
                                    egui::RichText::new(format!("{:?}", t.direction))
                                        .strikethrough()
                                } else {
                                    egui::RichText::new(format!("{:?}", t.direction))
                                };
                                ui.label(direction_text);
                                ui.separator();

                                ui.label(
                                    egui::RichText::new(
                                        t.datetime.format("%Y-%m-%d %H:%M").to_string(),
                                    )
                                    .weak(),
                                );

                                if let Some(expected) = t.expected_return_date {
                                    ui.separator();

                                    let deadline_color = if !t.deadline_changes.is_empty() {
                                        egui::Color32::YELLOW
                                    } else {
                                        egui::Color32::LIGHT_BLUE
                                    };

                                    let deadline_text = if !t.deadline_changes.is_empty() {
                                        format!(
                                            "📅 Expected: {} ({}×)",
                                            expected.format("%Y-%m-%d"),
                                            t.deadline_changes.len()
                                        )
                                    } else {
                                        format!("📅 Expected: {}", expected.format("%Y-%m-%d"))
                                    };

                                    ui.colored_label(deadline_color, deadline_text);

                                    if matches!(t.direction, Direction::Lent | Direction::Borrowed)
                                    {
                                        if ui.small_button("📝").clicked() {
                                            self.editing_deadline_for = Some(*i);
                                            self.temp_new_deadline = expected;
                                        }
                                    }
                                }

                                if t.attachment_path.is_some() {
                                    ui.separator();
                                    if ui.small_button("📷").clicked() {
                                        if let Some(ref path) = t.attachment_path {
                                            if !self.attachment_textures.contains_key(path) {
                                                if let Ok(img) = image::open(path) {
                                                    let img = img.to_rgba8();
                                                    let (w, h) = img.dimensions();
                                                    let pixels = img.into_raw();
                                                    let color_img =
                                                        egui::ColorImage::from_rgba_premultiplied(
                                                            [w as usize, h as usize],
                                                            &pixels,
                                                        );
                                                    let texture = ctx.load_texture(
                                                        path,
                                                        color_img,
                                                        egui::TextureOptions::LINEAR,
                                                    );
                                                    self.attachment_textures
                                                        .insert(path.clone(), texture);
                                                }
                                            }
                                            self.viewing_attachment = Some(path.clone());
                                        }
                                    }
                                }

                                ui.separator();
                                if ui.small_button("✏").clicked() {
                                    self.edit_transaction_index = Some(*i);
                                }
                            });
                        });
                    }
                });
            });

        if let Some(edit_idx) = self.editing_deadline_for {
            let mut should_close = false;
            let mut should_save = false;

            egui::Window::new("📝 Change Deadline")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    if let Some(t) = self.db.transactions.get(edit_idx) {
                        ui.label(format!("Changing deadline for: {}", t.person.name));
                        ui.label(format!("Transaction #{}", edit_idx + 1));
                        ui.separator();

                        ui.add_space(10.0);

                        if let Some(current_deadline) = t.expected_return_date {
                            ui.label(format!(
                                "Current deadline: {}",
                                current_deadline.format("%Y-%m-%d")
                            ));

                            if !t.deadline_changes.is_empty() {
                                ui.label(format!(
                                    "Previously changed {} time(s)",
                                    t.deadline_changes.len()
                                ));
                                ui.add_space(5.0);

                                egui::CollapsingHeader::new("📜 Change History").show(ui, |ui| {
                                    for (idx, change) in t.deadline_changes.iter().enumerate() {
                                        ui.label(format!(
                                            "{}. {} ➡ {} (changed on {})",
                                            idx + 1,
                                            change.old_date.format("%Y-%m-%d"),
                                            change.new_date.format("%Y-%m-%d"),
                                            change.changed_at.format("%Y-%m-%d %H:%M")
                                        ));
                                    }
                                });
                            }
                        }

                        ui.add_space(15.0);
                        ui.label("New deadline:");
                        ui.add(egui_extras::DatePickerButton::new(
                            &mut self.temp_new_deadline,
                        ));

                        ui.add_space(15.0);
                        ui.horizontal(|ui| {
                            if ui.button("💾 Save").clicked() {
                                should_save = true;
                                should_close = true;
                            }
                            if ui.button("❌ Cancel").clicked() {
                                should_close = true;
                            }
                        });
                    }
                });

            if should_save {
                if let Some(t) = self.db.transactions.get_mut(edit_idx) {
                    if let Some(old_deadline) = t.expected_return_date {
                        if old_deadline != self.temp_new_deadline {
                            let change = DeadlineChange {
                                old_date: old_deadline,
                                new_date: self.temp_new_deadline,
                                changed_at: Local::now().naive_local(),
                            };
                            t.deadline_changes.push(change);
                            t.expected_return_date = Some(self.temp_new_deadline);

                            if let Err(e) = self.db.save() {
                                self.status_message = format!("❌ Error saving: {}", e);
                            } else {
                                self.status_message = "✅ Deadline updated!".to_string();
                            }
                        }
                    }
                }
            }

            if should_close {
                self.editing_deadline_for = None;
            }
        }

        if let Some(edit_idx) = self.edit_transaction_index {
            let mut should_close = false;
            let mut should_save = false;
            let mut new_attachment: Option<Option<String>> = None;

            egui::Window::new("✏ Edit Transaction")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    if let Some(t) = self.db.transactions.get(edit_idx) {
                        ui.label(format!("Editing transaction #{}", edit_idx + 1));
                        ui.separator();

                        ui.horizontal(|ui| {
                            ui.label("📎 Attachment:");
                            if ui.button("📁 Change...").clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("Images", &["png", "jpg", "jpeg", "gif", "bmp"])
                                    .pick_file()
                                {
                                    match Database::copy_attachment_to_storage(
                                        &path.to_string_lossy().to_string(),
                                    ) {
                                        Ok(stored_path) => {
                                            new_attachment = Some(Some(stored_path));
                                        }
                                        Err(e) => {
                                            self.status_message =
                                                format!("⚠️ Failed to copy attachment: {}", e);
                                        }
                                    }
                                }
                            }

                            if let Some(ref path) = t.attachment_path {
                                ui.label(
                                    PathBuf::from(path)
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("file"),
                                );
                                if ui.small_button("❌").clicked() {
                                    new_attachment = Some(None);
                                }
                            } else {
                                ui.label(egui::RichText::new("None").weak());
                            }
                        });

                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            if ui.button("💾 Save").clicked() {
                                should_save = true;
                                should_close = true;
                            }
                            if ui.button("❌ Cancel").clicked() {
                                should_close = true;
                            }
                        });
                    }
                });

            if let Some(new_att) = new_attachment {
                if let Some(t) = self.db.transactions.get_mut(edit_idx) {
                    t.attachment_path = new_att;
                }
            }

            if should_save {
                let _ = self.db.save();
                self.status_message = "✅ Transaction updated!".to_string();
            }

            if should_close {
                self.edit_transaction_index = None;
            }
        }
    }

    fn calculate_paid_back_transactions(&self) -> std::collections::HashSet<usize> {
        use std::collections::HashSet;
        let mut paid_back = HashSet::new();

        let mut person_debts: HashMap<(String, MoneyType), Vec<(usize, f64, Direction)>> =
            HashMap::new();

        for (idx, t) in self.db.transactions.iter().enumerate() {
            let key = (t.person.name.clone(), t.money_type);
            person_debts
                .entry(key)
                .or_insert_with(Vec::new)
                .push((idx, t.amount, t.direction));
        }

        for debts in person_debts.values() {
            let lent_borrowed: Vec<(usize, f64, Direction)> = debts
                .iter()
                .filter(|(_, _, dir)| matches!(dir, Direction::Lent | Direction::Borrowed))
                .copied()
                .collect();

            let returns: Vec<(usize, f64, Direction)> = debts
                .iter()
                .filter(|(_, _, dir)| matches!(dir, Direction::Returned | Direction::Repaid))
                .copied()
                .collect();

            let mut remaining_returns = returns.iter().map(|(_, amount, _)| *amount).sum::<f64>();

            for (idx, amount, _) in lent_borrowed.iter() {
                if remaining_returns >= *amount {
                    paid_back.insert(*idx);
                    remaining_returns -= amount;
                } else if remaining_returns > 0.0 {
                    break;
                } else {
                    break;
                }
            }
        }

        paid_back
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
