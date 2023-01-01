use chrono::Datelike;
// #![windows_subsystem = "windows"]
use clap::Parser;
use color_eyre::eyre::{ContextCompat, bail};
use dotenvy::dotenv;
use eframe::IconData;
use egui::TextBuffer;

use itertools::Itertools;
use notify_rust::Notification;
use poll_promise::Promise;
use rspotify::{
    model::TimeRange,
    prelude::*,
    prelude::{BaseClient, OAuthClient},
    scopes, AuthCodeSpotify, Config, Credentials, OAuth,
};
use serde::{Deserialize, Serialize};
use spotify_oauth::{SpotifyAuth, SpotifyCallback, SpotifyScope, SpotifyToken};
use std::{
    env, fs,
    io::{stdin, BufRead, BufReader, Write},
    net::TcpListener,
    str::FromStr,
};
use url::Url;

fn main() -> color_eyre::Result<()> {
    env_logger::init();
    dotenv().ok();

    let args = Args::parse();

    let logo = include_bytes!("../assets/BOTM Logo.jpg");

    let logo = image::load_from_memory_with_format(logo, image::ImageFormat::Jpeg)?;
    // let logo = dbg!(logo);

    let icon = IconData {
        rgba: logo.to_rgba8().to_vec(),
        width: logo.width(),
        height: logo.height(),
    };
    // simple_logger::from().ok();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(300.0, 215.0)),
        resizable: false,
        initial_window_pos: Some(egui::pos2(800.0, 400.0)),
        icon_data: Some(icon),
        ..Default::default()
    };

    if args.no_gui {
        if !std::path::Path::new("config.json").exists() {
            eprintln!("NO CONFIG FILE FOUND!\nplease set config in gui");
            bail!("config.json could not be found");
        }
        let config: Configuration =
            serde_json::from_str(&fs::read_to_string("config.json").unwrap_or_default())
                .unwrap_or_default();
        let creds = Credentials::new(&config.client_id, &config.client_secret);
        generate_botm(args, Some(creds), Some(config.port))?;
        Notification::new()
            .summary("Nice BOTM you got there!")
            .body("Generated a new BOTM")
            .appname("BOTM")
            .show()?;
    } else {
        eframe::run_native(
            "BOTM",
            options,
            Box::new(|_cc| {
                Box::new(MyApp {
                    config: serde_json::from_str(
                        &fs::read_to_string("config.json").unwrap_or_default(),
                    )
                    .unwrap_or_default(),
                    ..Default::default()
                })
            }),
        );
    }

    Ok(())
    // dotenv().ok();
}

#[derive(Default)]
struct MyApp {
    last_month: bool,
    promise: Option<Promise<String>>,
    config: Configuration,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct ClientId(String);

impl TextBuffer for ClientId {
    fn is_mutable(&self) -> bool {
        self.0.is_mutable()
    }

    fn as_str(&self) -> &str {
        self.0.as_str()
    }

    fn insert_text(&mut self, text: &str, char_index: usize) -> usize {
        self.0.insert_text(text, char_index)
    }

    fn delete_char_range(&mut self, char_range: std::ops::Range<usize>) {
        self.0.delete_char_range(char_range)
    }
}

impl From<String> for ClientId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.config.client_id = self.config.client_id.trim().to_owned();
        self.config.client_secret = self.config.client_secret.trim().to_owned();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("BOTM");
                ui.add_enabled_ui(!self.config.client_id.is_empty(), |ui| {
                    if ui
                        .button("Generate BOTM")
                        .on_disabled_hover_text("Configuration needed")
                        .on_hover_text(format!(
                            "{}\n\n{}",
                            "Generates the playlist.",
                            "Opens the login page,\nif you are not already authenticated."
                        ))
                        .clicked()
                    {
                        // generate_botm();
                        self.promise.get_or_insert_with(|| {
                            let last_month = self.last_month;
                            let config = self.config.clone();

                            Promise::spawn_thread("BOTM", move || {
                                let out = generate_botm(
                                    Args {
                                        last_month,
                                        ..Default::default()
                                    },
                                    Some(Credentials {
                                        id: config.client_id.clone(),
                                        secret: Some(config.client_secret.clone()),
                                    }),
                                    Some(config.port),
                                );
                                if out.is_err() {
                                    format!("Error generating playlist. ({})", out.unwrap_err())
                                } else {
                                    "Done generating playlist. Check Spotify".to_owned()
                                }
                            })
                        });
                    }
                });

                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.last_month, "Generate for last month")
                        .on_hover_text("Use last month in the playlist name");

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // TODO: Check if task already exists
                        if ui
                            .button("Schedule Run")
                            .on_hover_text(format!(
                                "{}\n\n{}\n{}\n\n{}",
                                "Creates a playlist every month automatically.",
                                "Runs at the start of the month,",
                                "using the 'Generate for last month' option.",
                                "Uses Windows Task Scheduler."
                            ))
                            .clicked()
                        {
                            log::info!("Generate Schedule Task");
                            let create_task_script = include_str!("../add_task.ps1");
                            log::trace!(
                                "Current dir {}",
                                env::current_dir()
                                    .unwrap_or_default()
                                    .as_os_str()
                                    .to_str()
                                    .unwrap_or_default()
                            );
                            let script = format!(
                                "$location=\"{}\"\r\n{}",
                                env::current_dir()
                                    .unwrap_or_default()
                                    .as_os_str()
                                    .to_str()
                                    .unwrap_or_default(),
                                create_task_script
                            );
                            let output = powershell_script::run(&script);
                            log::trace!("{:#?}", output);

                            if let Ok(output) = output {
                                if output.success() {
                                    log::info!("Successfully created Task");
                                } else {
                                    log::warn!("Could not successfully create Task");
                                    log::warn!("Got error: {:?}", output);
                                }
                            }
                        }
                    });
                });
                if let Some(prom) = &self.promise {
                    if let Some(out) = prom.ready() {
                        ui.label(out);
                    } else {
                        ui.spinner();
                    }
                }
            });
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.heading("Configuration");
                if ui.link("?").on_hover_text("Get help").clicked() {
                    open::that("https://github.com/gaweringo/BOTM#-getting-started-").ok();
                };
            });
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    let name_label = ui.label("Client ID");

                    ui.text_edit_singleline(&mut self.config.client_id)
                        .labelled_by(name_label.id);
                });

                ui.horizontal(|ui| {
                    let name_label = ui.label("Client Secret");
                    ui.text_edit_singleline(&mut self.config.client_secret)
                        .labelled_by(name_label.id);
                });

                ui.horizontal(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Port");

                        ui.add(
                            egui::DragValue::new(&mut self.config.port).clamp_range(8080..=9000), // .prefix("Port"),
                        );
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Test config").clicked() {
                            log::info!("Test config pressed");
                        }
                    });
                });
            });
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        fs::write(
            "config.json",
            serde_json::to_vec_pretty(&self.config).unwrap(),
        );
    }
}

fn generate_botm(
    args: Args,
    creds: Option<Credentials>,
    port: Option<i32>,
) -> color_eyre::Result<()> {
    log::info!("Running generate_botm()");

    log::debug!("Got args: {args:?}");
    log::debug!("Got creds: {creds:?}");
    log::debug!("Got port: {port:?}");

    let scopes = scopes!(
        "playlist-modify-private",
        "playlist-modify-public",
        "user-top-read"
    );

    // May require the `env-file` feature enabled if the environment variables
    // aren't configured manually.
    let config = Config {
        token_refreshing: true,
        token_cached: true,
        ..Default::default()
    };

    log::debug!("Setting up spotify struct");

    // let creds = creds.unwrap_or_else(|| Credentials::from_env().wrap_err("Could not get creds from env")?);
    let creds = if let Some(creds) = creds {
        creds
    } else {
        log::debug!("No creds specified, reading from env");
        Credentials::from_env()
            .wrap_err("Got no credentials (id, secret), tried reading from env")?
    };

    // let oauth = OAuth::from_env(scopes).wrap_err("Could not get OAuth from env")?;
    let port = port.unwrap_or(8081);
    let oauth = OAuth {
        redirect_uri: format!("http://localhost:{}", port),
        scopes,
        ..Default::default()
    };

    log::debug!("OAuth: {oauth:?}");

    let mut spotify = AuthCodeSpotify::with_config(creds, oauth, config);

    // Obtaining the access token
    let url = spotify.get_authorize_url(false)?;

    {
        // let ref mut this = spotify;
        let url: &str = &url;
        match spotify.read_token_cache(true) {
            Ok(Some(new_token)) => {
                let expired = new_token.is_expired();
                *spotify.get_token().lock().unwrap() = Some(new_token);
                if expired {
                    match spotify.refetch_token()? {
                        Some(refreshed_token) => {
                            log::info!("Successfully refreshed expired token from token cache");
                            *spotify.get_token().lock().unwrap() = Some(refreshed_token)
                        }
                        None => {
                            log::info!("Unable to refresh expired token from token cache");
                            let code = spotify.get_code_from_user(url)?;
                            spotify.request_token(&code)?;
                        }
                    }
                }
            }
            _ => {
                let code = prompt_user(&spotify, url, port)?;
                spotify.request_token(&code)?;
            }
        }
        spotify.write_token_cache()
    }?;

    log::info!("Getting top tracks");
    let top_tracks = spotify
        .current_user_top_tracks_manual(Some(&TimeRange::ShortTerm), Some(50), None)?
        .items;
    // .collect_vec();
    // .collect_vec();

    log::info!("Got {} top tracks", top_tracks.len());
    // println!("Request: {res:?}");

    log::info!("Getting user id");
    let id = spotify.me()?.id;

    let mut now = chrono::Local::now();
    if args.last_month {
        now = now.with_day(1).unwrap_or(now);
        if now.month() == 1 {
            now = now.with_year(now.year() - 1).unwrap_or(now);
            now = now.with_month(12).unwrap_or(now);
        } else {
            now = now.with_month(now.month() - 1).unwrap_or(now);
        }
    }

    let playlist_name = now.format("%Y-%m (%b) BOTM").to_string();
    let description = now.format("Bangers of the month for %B %Y").to_string();
    let description = format!(
        "{}, (generated on {})",
        description,
        chrono::Local::now().format("%F")
    );

    log::info!("Creating Playlist");
    log::debug!("Playlist name: {}", playlist_name);
    log::debug!("Playlist description: {}", description);
    let playlist =
        spotify.user_playlist_create(&id, &playlist_name, None, None, Some(&description))?;

    let tracks = top_tracks
        .iter()
        // .flatten()
        .filter_map(|t| t.id.clone())
        .collect_vec();

    // WHYYYYYYYY DID MY THING NOT WORK. IT WAS PRACTICALLY THE SAME
    // https://github.com/ramsayleung/rspotify/pull/305#issue-1164923492
    let playable = tracks.iter().map(|id| id as &dyn PlayableId).collect_vec();

    log::info!("Adding songs to playlist");
    spotify.playlist_add_items(&playlist.id, playable, None)?;

    log::info!("Done");
    // let mut buf = String::new();
    // stdin().read_line(&mut buf);
    Ok(())
}

fn prompt_user(
    spotify: &AuthCodeSpotify,
    url: &str,
    port: i32,
) -> Result<String, color_eyre::Report> {
    let _this = spotify;
    let url = url;
    use rspotify::ClientError;
    log::info!("Opening brower with auth URL");
    match open::that(url) {
        Ok(_) => println!("Opened {} in your browser.", url),
        Err(why) => eprintln!(
            "Error when trying to open an URL in your browser: {:?}. \
                 Please navigate here manually: {}",
            why, url
        ),
    }
    log::info!("Prompting user for code");

    // let local = Url::parse(env::var("RSPOTIFY_REDIRECT_URI")?.as_str())?;
    let local = Url::parse(&format!("http://localhost:{}", port))?;
    dbg!(&local.to_string());

    // let loc_v4: SocketAddrV4 = local[0];
    // let local_v4 = SocketAddrV4::(local);
    // let listener = TcpListener::bind(local);
    let listener = TcpListener::bind(format!("127.0.0.1:{}", local.port().unwrap_or(8080)));
    // let listener = TcpListener::bind("localhost:8080");
    let code = if let Ok(listener) = listener {
        log::info!("got listener: {:?}", listener);
        let (mut stream, _sock_add) = listener.accept()?;

        log::info!("Accepted stream: {:?}", stream);

        let mut reader = BufReader::new(&stream);

        let mut request_line = String::new();
        reader.read_line(&mut request_line).unwrap();

        log::info!("request_line: {}", request_line);

        let redirect_url = request_line.split_whitespace().nth(1).unwrap();
        let url = Url::parse(&("http://localhost".to_string() + redirect_url)).unwrap();

        log::info!("TcpStream url: {}", url.as_str());

        let response = format!("HTTP/1.1 200 OK\r\n\r\n{}", include_str!("response.html"));

        stream.write_all(response.as_bytes())?;
        stream.flush()?;

        let url = url.to_string();
        spotify
            .parse_response_code(&url)
            .ok_or_else(|| ClientError::Cli("unable to parse the response code".to_string()))?
    } else {
        log::warn!(
            "Could not create TcpListener at {local}. Got error: {}",
            listener.err().unwrap()
        );
        println!("Please enter the URL you were redirected to: ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        spotify
            .parse_response_code(&input)
            .ok_or_else(|| ClientError::Cli("unable to parse the response code".to_string()))?
    };

    Ok(code)
}

async fn spotify_auth() -> color_eyre::Result<SpotifyToken> {
    // Setup Spotify Auth URL
    let auth = SpotifyAuth::new_from_env("code".into(), vec![SpotifyScope::UserTopRead], false);
    let auth_url = auth.authorize_url()?;

    // Open the auth URL in the default browser of the user.
    open::that(auth_url)?;

    println!("Input callback URL:");
    let mut buffer = String::new();
    stdin().read_line(&mut buffer)?;

    // Convert the given callback URL into a token.
    let token = SpotifyCallback::from_str(buffer.trim())?
        .convert_into_token(auth.client_id, auth.client_secret, auth.redirect_uri)
        .await?;

    println!("Token: {:#?}", token);

    Ok(token)
}

#[derive(clap::Parser, Debug, Default)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = false)]
    pub no_gui: bool,
    #[arg(short, long, default_value_t = false)]
    pub last_month: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Configuration {
    client_id: String,
    client_secret: String,
    port: i32,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            client_id: Default::default(),
            client_secret: Default::default(),
            port: 8081,
        }
    }
}
