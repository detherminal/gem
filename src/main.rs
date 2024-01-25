use std::ops::Div;
use std::thread;

use chrono::NaiveDate;
use clipboard::ClipboardProvider;
use eframe::egui;
use image::{io::Reader as ImageReader, DynamicImage};
use image::{EncodableLayout, GenericImage, GenericImageView, Luma, Rgba};
use imageproc::drawing::{draw_line_segment_mut, draw_text_mut};
use libmonero::{derive_hex_seed, derive_priv_keys, derive_pub_key, generate_seed};
use qrcode::QrCode;
use rusttype::{Font, Scale};
use serde_json::json;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_resizable(false)
            .with_fullscreen(false)
            .with_title("Gem - Gift Easily Monero")
            .with_inner_size([1100.0, 650.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Gem - Gift Easily Monero",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::<GemApp>::default()
        }),
    )
}

struct GemApp {
    mnemonic: String,
    description: String,
    amount: f32,
    value_xmr: f32,
    address: String,
    qr_main: DynamicImage,
    qr_addr: DynamicImage,
    auto_wallet: bool,
    block_height: u64,
    date: NaiveDate,
    gifter: String,
    contact: String,
    booted: bool,
}

impl Default for GemApp {
    fn default() -> Self {
        let date = chrono::Local::now();
        let date = date.format("%d/%m/%Y").to_string();
        Self {
            mnemonic: "".to_string(),
            description: "".to_string(),
            amount: 1.0,
            value_xmr: 150.0,
            address: String::new(),
            qr_main: DynamicImage::new_rgb8(1, 1),
            qr_addr: DynamicImage::new_rgb8(1, 1),
            auto_wallet: true,
            block_height: 3000000,
            date: NaiveDate::parse_from_str(date.as_str(), "%d/%m/%Y").unwrap(),
            booted: false,
            gifter: "".to_string(),
            contact: "".to_string(),
        }
    }
}

fn auto_fill(self_app: &mut GemApp) {
    // Get block height via ureq
    let url = "http://xmr-node.cakewallet.com:18081/json_rpc";
    let resp = ureq::post(url)
        .set("Content-Type", "application/json")
        .send_json(json!({
            "jsonrpc": "2.0",
            "id": "0",
            "method": "get_block_count"
        }))
        .unwrap();
    let resp = resp.into_string().unwrap();
    let resp: serde_json::Value = serde_json::from_str(resp.as_str()).unwrap();
    let block_height = resp["result"]["count"].as_u64().unwrap();
    self_app.block_height = block_height - 1000;
    // Get price via coingecko
    let url = "https://api.coingecko.com/api/v3/simple/price?ids=monero&vs_currencies=usd";
    let resp = ureq::get(url).call().unwrap();
    let resp = resp.into_string().unwrap();
    let resp: serde_json::Value = serde_json::from_str(resp.as_str()).unwrap();
    let price = resp["monero"]["usd"].as_f64().unwrap();
    self_app.value_xmr = price as f32;
    // Get date
    let date = chrono::Local::now();
    let date = date.format("%d/%m/%Y").to_string();
    self_app.date = NaiveDate::parse_from_str(date.as_str(), "%d/%m/%Y").unwrap();
    // Generate wallet
    let mnemonic = generate_seed("en", "original");
    let priv_keys = derive_priv_keys(derive_hex_seed(mnemonic.clone()));
    let priv_sk = priv_keys[0].to_string();
    let priv_vk = priv_keys[1].to_string();
    let pub_sk = derive_pub_key(priv_sk);
    let pub_vk = derive_pub_key(priv_vk);
    let address = libmonero::derive_address(pub_sk, pub_vk, 0);
    self_app.address = address.clone();
    self_app.mnemonic = mnemonic.join(" ");
    let mne_str_encoded = mnemonic.join("%20");
    let qr_code = QrCode::new(format!(
        "monero_wallet:{}?seed={}",
        address, mne_str_encoded
    ))
    .unwrap();
    let qr_img = qr_code.render::<Luma<u8>>().build();
    let qr_img = DynamicImage::ImageLuma8(qr_img);
    let qr_img = qr_img.resize_exact(350, 350, image::imageops::FilterType::Nearest);
    self_app.qr_main = qr_img;
    let qr_addr_code = QrCode::new(format!("{}", address)).unwrap();
    let qr_addr_img = qr_addr_code.render::<Luma<u8>>().build();
    let qr_addr_img = DynamicImage::ImageLuma8(qr_addr_img);
    let qr_addr_img = qr_addr_img.resize_exact(150, 150, image::imageops::FilterType::Nearest);
    self_app.qr_addr = qr_addr_img;
}

impl eframe::App for GemApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.booted {
            auto_fill(self);
            self.booted = true;
        }
        // Get date
        let date = chrono::Local::now();
        let date = date.format("%d/%m/%Y").to_string();
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut img = ImageReader::open("./assets/empty_card.png")
                .unwrap()
                .decode()
                .unwrap();
            let font_size = 20.0;
            let black = Rgba([0, 0, 0, 0]);
            draw_text_mut(
                &mut img,
                black,
                160,
                65,
                Scale { x: 60.0, y: 60.0 },
                &Font::try_from_vec(Vec::from(
                    include_bytes!("./../assets/MoneroGothic_v3.otf") as &[u8]
                ))
                .unwrap(),
                "MONERO GIFT",
            );
            draw_text_mut(
                &mut img,
                black,
                60,
                160,
                Scale {
                    x: font_size,
                    y: font_size,
                },
                &Font::try_from_vec(Vec::from(
                    include_bytes!("./../assets/MoneroGothic_v3.otf") as &[u8]
                ))
                .unwrap(),
                format!(
                    "Congratulations! You have been gifted {} XMR (~{:.2})",
                    self.amount,
                    self.value_xmr * self.amount
                )
                .as_str(),
            );
            draw_text_mut(
                &mut img,
                black,
                60,
                190,
                Scale {
                    x: font_size,
                    y: font_size,
                },
                &Font::try_from_vec(Vec::from(
                    include_bytes!("./../assets/MoneroGothic_v3.otf") as &[u8]
                ))
                .unwrap(),
                "You can redeem this gift at any time into a Monero wallet.",
            );
            draw_text_mut(
                &mut img,
                black,
                60,
                220,
                Scale {
                    x: font_size,
                    y: font_size,
                },
                &Font::try_from_vec(Vec::from(
                    include_bytes!("./../assets/MoneroGothic_v3.otf") as &[u8]
                ))
                .unwrap(),
                "For example, you can use the instructions below for",
            );
            draw_text_mut(
                &mut img,
                black,
                60,
                250,
                Scale {
                    x: font_size,
                    y: font_size,
                },
                &Font::try_from_vec(Vec::from(
                    include_bytes!("./../assets/MoneroGothic_v3.otf") as &[u8]
                ))
                .unwrap(),
                "redeeming this gift into the Cake Wallet app:",
            );
            draw_text_mut(
                &mut img,
                black,
                60,
                280,
                Scale {
                    x: font_size,
                    y: font_size,
                },
                &Font::try_from_vec(Vec::from(
                    include_bytes!("./../assets/MoneroGothic_v3.otf") as &[u8]
                ))
                .unwrap(),
                "1 - Install and open the Cake Wallet app on your phone.",
            );
            draw_text_mut(
                &mut img,
                black,
                60,
                310,
                Scale {
                    x: font_size,
                    y: font_size,
                },
                &Font::try_from_vec(Vec::from(
                    include_bytes!("./../assets/MoneroGothic_v3.otf") as &[u8]
                ))
                .unwrap(),
                "2 - Tap the 'Restore Wallet' button.",
            );
            draw_text_mut(
                &mut img,
                black,
                60,
                340,
                Scale {
                    x: font_size,
                    y: font_size,
                },
                &Font::try_from_vec(Vec::from(
                    include_bytes!("./../assets/MoneroGothic_v3.otf") as &[u8]
                ))
                .unwrap(),
                "3 - Tap the 'Scan QR Code' button.",
            );
            draw_text_mut(
                &mut img,
                black,
                60,
                370,
                Scale {
                    x: font_size,
                    y: font_size,
                },
                &Font::try_from_vec(Vec::from(
                    include_bytes!("./../assets/MoneroGothic_v3.otf") as &[u8]
                ))
                .unwrap(),
                "4 - Scan the big QR code on the side.",
            );
            draw_text_mut(
                &mut img,
                black,
                60,
                400,
                Scale {
                    x: font_size,
                    y: font_size,
                },
                &Font::try_from_vec(Vec::from(
                    include_bytes!("./../assets/MoneroGothic_v3.otf") as &[u8]
                ))
                .unwrap(),
                "After importing, you can use the XMR in the wallet as you wish.",
            );
            draw_text_mut(
                &mut img,
                black,
                60,
                430,
                Scale {
                    x: font_size,
                    y: font_size,
                },
                &Font::try_from_vec(Vec::from(
                    include_bytes!("./../assets/MoneroGothic_v3.otf") as &[u8]
                ))
                .unwrap(),
                "Message: ",
            );
            draw_text_mut(
                &mut img,
                black,
                60,
                460,
                Scale {
                    x: font_size,
                    y: font_size,
                },
                &Font::try_from_vec(Vec::from(
                    include_bytes!("./../assets/MoneroGothic_v3.otf") as &[u8]
                ))
                .unwrap(),
                format!("- {}", self.description).as_str(),
            );
            draw_text_mut(
                &mut img,
                black,
                60,
                490,
                Scale {
                    x: font_size,
                    y: font_size,
                },
                &Font::try_from_vec(Vec::from(
                    include_bytes!("./../assets/MoneroGothic_v3.otf") as &[u8]
                ))
                .unwrap(),
                "Contact:",
            );
            draw_text_mut(
                &mut img,
                black,
                60,
                520,
                Scale {
                    x: font_size,
                    y: font_size,
                },
                &Font::try_from_vec(Vec::from(
                    include_bytes!("./../assets/MoneroGothic_v3.otf") as &[u8]
                ))
                .unwrap(),
                format!("- {}", self.contact).as_str(),
            );
            draw_text_mut(
                &mut img,
                black,
                740,
                30,
                Scale { x: 30.0, y: 30.0 },
                &Font::try_from_vec(Vec::from(
                    include_bytes!("./../assets/MoneroGothic_v3.otf") as &[u8]
                ))
                .unwrap(),
                "WALLET",
            );
            draw_text_mut(
                &mut img,
                black,
                660,
                405,
                Scale {
                    x: font_size,
                    y: font_size,
                },
                &Font::try_from_vec(Vec::from(
                    include_bytes!("./../assets/MoneroGothic_v3.otf") as &[u8]
                ))
                .unwrap(),
                "ADDRESS",
            );
            draw_text_mut(
                &mut img,
                black,
                800,
                440,
                Scale {
                    x: font_size,
                    y: font_size,
                },
                &Font::try_from_vec(Vec::from(
                    include_bytes!("./../assets/MoneroGothic_v3.otf") as &[u8]
                ))
                .unwrap(),
                format!("Date: {}", self.date.format("%d/%m/%Y")).as_str(),
            );
            draw_text_mut(
                &mut img,
                black,
                800,
                470,
                Scale {
                    x: font_size,
                    y: font_size,
                },
                &Font::try_from_vec(Vec::from(
                    include_bytes!("./../assets/MoneroGothic_v3.otf") as &[u8]
                ))
                .unwrap(),
                format!("Height: {}", self.block_height).as_str(),
            );
            draw_text_mut(
                &mut img,
                black,
                800,
                500,
                Scale {
                    x: font_size,
                    y: font_size,
                },
                &Font::try_from_vec(Vec::from(
                    include_bytes!("./../assets/MoneroGothic_v3.otf") as &[u8]
                ))
                .unwrap(),
                format!("From {}", self.gifter).as_str(),
            );
            draw_line_segment_mut(&mut img, (575.0, 0.0), (575.0, 590.0), black);
            // ui.horizontal(|ui| {
            //     ui.heading("Enter amount of XMR to be gifted: ");
            //     ui.add(
            //         egui::DragValue::new(&mut self.amount)
            //             .speed(0.01)
            //             .min_decimals(4).max_decimals(4)
            //             .clamp_range(0.0..=1000000.0),
            //     );
            // });

            // egui::Grid::new("my_grid")
            //     .striped(true)
            //     .spacing([10.0, 10.0])
            //     .show(ui, |ui| {
            //         ui.label("Enter amount of XMR to be gifted: ");
            //         ui.add(
            //             egui::DragValue::new(&mut self.amount)
            //                 .speed(0.01)
            //                 .min_decimals(4)
            //                 .max_decimals(4)
            //                 .clamp_range(0.0..=1000000.0),
            //         );
            //     });
            ui.vertical_centered(|ui| {
                // Grid with width of entire ui
                egui::Grid::new("my_grid")
                    .striped(true)
                    .num_columns(4)
                    .min_col_width(1000.0 / 4.0)
                    .max_col_width(1000.0 / 4.0)
                    .show(ui, |ui| {
                        // First row
                        // label with big font
                        ui.heading("Gift Amount: ");
                        ui.add(
                            egui::DragValue::new(&mut self.amount)
                                .speed(0.01)
                                .fixed_decimals(4)
                                .clamp_range(0.0..=1000000.0),
                        );
                        ui.heading("Auto Fill (Might Be Slow): ");
                        if ui.checkbox(&mut self.auto_wallet, "").clicked() && self.auto_wallet {
                            auto_fill(self);
                        }
                        ui.end_row();
                        ui.heading("Mnemonic: ");
                        if self.auto_wallet {
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(&mut self.mnemonic).interactive(false));
                                if ui.button("Copy").clicked() {
                                    let mut ctx: clipboard::ClipboardContext =
                                        clipboard::ClipboardProvider::new().unwrap();
                                    ctx.set_contents(self.mnemonic.clone()).unwrap();
                                }
                            });
                        } else {
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(&mut self.mnemonic));
                                if ui.button("Copy").clicked() {
                                    let mut ctx: clipboard::ClipboardContext =
                                        clipboard::ClipboardProvider::new().unwrap();
                                    ctx.set_contents(self.mnemonic.clone()).unwrap();
                                }
                            });
                        }
                        ui.heading("Address: ");
                        if self.auto_wallet {
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(&mut self.address).interactive(false));
                                if ui.button("Copy").clicked() {
                                    let mut ctx: clipboard::ClipboardContext =
                                        clipboard::ClipboardProvider::new().unwrap();
                                    ctx.set_contents(self.address.clone()).unwrap();
                                }
                            });
                        } else {
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(&mut self.address));
                                if ui.button("Copy").clicked() {
                                    let mut ctx: clipboard::ClipboardContext =
                                        clipboard::ClipboardProvider::new().unwrap();
                                    ctx.set_contents(self.address.clone()).unwrap();
                                }
                            });
                        }
                        ui.end_row();
                        if self.auto_wallet {
                            ui.heading("Block Height (Current - 1k): ");
                            ui.label(self.block_height.to_string());
                        } else {
                            ui.heading("Block Height: ");
                            ui.add(
                                egui::DragValue::new(&mut self.block_height)
                                    .speed(100)
                                    .fixed_decimals(0)
                                    .clamp_range(0.0..=100000000.0),
                            );
                        }
                        ui.heading("Date: ");
                        if self.auto_wallet {
                            ui.label(self.date.format("%Y-%m-%d").to_string());
                        } else {
                            ui.add(egui_extras::DatePickerButton::new(&mut self.date));
                        }
                        ui.end_row();
                        ui.heading("Value Per XMR: ");
                        if self.auto_wallet {
                            ui.label(format!("${:.2}", self.value_xmr * self.amount));
                        } else {
                            ui.add(
                                egui::DragValue::new(&mut self.value_xmr)
                                    .speed(0.01)
                                    .fixed_decimals(2)
                                    .clamp_range(0.0..=1000000.0),
                            );
                        }
                        ui.heading("Message: ");
                        ui.add(egui::TextEdit::singleline(&mut self.description).char_limit(50));
                        ui.end_row();
                        ui.heading("Gifter: ");
                        ui.add(egui::TextEdit::singleline(&mut self.gifter).char_limit(15));
                        ui.heading("Contact: ");
                        ui.add(egui::TextEdit::singleline(&mut self.contact).char_limit(50));
                        ui.end_row();
                    });
                ui.add_space(10.0);
                if self.auto_wallet {
                    if ui.button("Generate New Wallet").clicked() {
                        // We have to do all deriving manually for now, libmonero will support generating directly a wallet soon
                        let mnemonic = generate_seed("en", "original");
                        let priv_keys = derive_priv_keys(derive_hex_seed(mnemonic.clone()));
                        let priv_sk = priv_keys[0].to_string();
                        let priv_vk = priv_keys[1].to_string();
                        let pub_sk = derive_pub_key(priv_sk);
                        let pub_vk = derive_pub_key(priv_vk);
                        let address = libmonero::derive_address(pub_sk, pub_vk, 0);
                        self.address = address.clone();
                        self.mnemonic = mnemonic.join(" ");
                        let mne_str_encoded = mnemonic.join("%20");
                        let qr_code = QrCode::new(format!(
                            "monero_wallet:{}?seed={}",
                            address, mne_str_encoded
                        ))
                        .unwrap();
                        let qr_img = qr_code.render::<Luma<u8>>().build();
                        let qr_img = DynamicImage::ImageLuma8(qr_img);
                        let qr_img =
                            qr_img.resize_exact(350, 350, image::imageops::FilterType::Nearest);
                        self.qr_main = qr_img;
                        let qr_addr_code = QrCode::new(format!("{}", address)).unwrap();
                        let qr_addr_img = qr_addr_code.render::<Luma<u8>>().build();
                        let qr_addr_img = DynamicImage::ImageLuma8(qr_addr_img);
                        let qr_addr_img = qr_addr_img.resize_exact(
                            150,
                            150,
                            image::imageops::FilterType::Nearest,
                        );
                        self.qr_addr = qr_addr_img;
                    }
                } else {
                    if ui.button("Update QR Codes").clicked() {
                        let mne_str_encoded = (self.mnemonic.split(" "))
                            .map(|x| x.to_string())
                            .collect::<Vec<String>>()
                            .join("%20");
                        let qr_code = QrCode::new(format!(
                            "monero_wallet:{}?seed={}&height={}",
                            self.address, mne_str_encoded, self.block_height
                        ))
                        .unwrap();
                        let qr_img = qr_code.render::<Luma<u8>>().build();
                        let qr_img = DynamicImage::ImageLuma8(qr_img);
                        let qr_img =
                            qr_img.resize_exact(350, 350, image::imageops::FilterType::Nearest);
                        self.qr_main = qr_img;
                        let qr_addr_code = QrCode::new(format!("{}", self.address)).unwrap();
                        let qr_addr_img = qr_addr_code.render::<Luma<u8>>().build();
                        let qr_addr_img = DynamicImage::ImageLuma8(qr_addr_img);
                        let qr_addr_img = qr_addr_img.resize_exact(
                            150,
                            150,
                            image::imageops::FilterType::Nearest,
                        );
                        self.qr_addr = qr_addr_img;
                    }
                }
                ui.add_space(10.0);
                // draw qr code
                for (x, y, pixel) in self.qr_main.pixels() {
                    let pixel = pixel.0[0];
                    let pixel = Rgba([pixel, pixel, pixel, 255]);
                    img.put_pixel(x + 615, y + 55, pixel);
                }
                // draw qr code
                for (x, y, pixel) in self.qr_addr.pixels() {
                    let pixel = pixel.0[0];
                    let pixel = Rgba([pixel, pixel, pixel, 255]);
                    img.put_pixel(x + 620, y + 425, pixel);
                }
                let color_image = match &img {
                    DynamicImage::ImageRgb8(image) => {
                        // common case optimization
                        egui::ColorImage::from_rgb(
                            [image.width() as usize, image.height() as usize],
                            image.as_bytes(),
                        )
                    }
                    other => {
                        let image = other.to_rgba8();
                        egui::ColorImage::from_rgba_unmultiplied(
                            [image.width() as usize, image.height() as usize],
                            image.as_bytes(),
                        )
                    }
                };
                // you must keep the handle, if the handle is destroyed so the texture will be destroyed as well
                let handle =
                    ctx.load_texture("gem", color_image.clone(), egui::TextureOptions::default());
                let sized_image = egui::load::SizedTexture::new(
                    handle.id(),
                    egui::vec2(
                        (color_image.size[0] as f32).div(1.25),
                        (color_image.size[1] as f32).div(1.25),
                    ),
                );
                let image = egui::Image::from_texture(sized_image);
                ui.add(image);
            });
        });
    }
}
