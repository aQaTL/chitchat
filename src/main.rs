use std::io;
use std::sync::Mutex;

use crate::pagination::Paginate;
use actix_session::CookieSession;
use actix_web::web::Data;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use diesel::prelude::*;
use serde::Deserialize;

use crate::chat::Broadcaster;
use actix_web::middleware::Logger;
use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use rand::Rng;
use std::path::PathBuf;
use std::time::Duration;

#[macro_use]
extern crate diesel; //Needed for ORM macros

mod chat;
mod get_paste;
mod models;
mod pagination;
mod schema;

#[derive(Deserialize)]
struct Config {
    ip: String,
    port: u16,
}

fn _try_ffsend_upload() -> io::Result<()> {
    use ffsend_api::{
        action::{params::*, upload::*},
        api::Version,
        config::*,
    };

    openssl_probe::init_ssl_cert_env_vars();
    let client_config = ffsend_api::client::ClientConfigBuilder::default()
        .timeout(Some(Duration::from_secs(10)))
        .transfer_timeout(Some(Duration::from_secs(10)))
        .basic_auth(None)
        .build()
        .expect("Failed to build client");
    let client = client_config.clone().client(true);

    let params = ParamsDataBuilder::default()
        .download_limit(Some(100))
        .expiry_time(None)
        .build()
        .unwrap();

    let file = Upload::new(
        Version::V3,
        url::Url::parse(SEND_DEFAULT_HOST).unwrap(),
        PathBuf::from("img.jpg"),
        None,
        None,
        Some(params),
    )
    .invoke(&client, None);
    let file = match file {
        Ok(f) => f,
        Err(e) => {
            println!("Err: {:?}", e);
            return Ok(());
        }
    };
    let url = file.download_url(true);
    println!("Upload successful");
    println!("Url: {}", url);

    Ok(())
}

#[actix_rt::main]
async fn main() -> io::Result<()> {
    let config = {
        let data = match std::fs::read("config.toml") {
            Ok(data) => data,
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                println!("config.toml not found, using config_template.toml");
                std::fs::read("config_template.toml")?
            }
            Err(e) => return Err(e),
        };
        toml::from_slice::<Config>(data.as_slice())?
    };

    dotenv::dotenv()
        .or_else(|_| {
            println!(".env not found, using .env_template");
            dotenv::from_filename(".env_template")
        })
        .expect("Failed to load .env");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let conn_manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .max_size(4)
        .build(conn_manager)
        .expect("Failed to create Pool");

    let broadcaster = chat::Broadcaster::new();

    let bind_addr = format!("{}:{}", config.ip, config.port);

    let mut gen = rand::thread_rng();
    let cookie_key = (0..64)
        .into_iter()
        .map(|_| gen.gen::<u8>())
        .collect::<Vec<u8>>();

    let server = HttpServer::new(move || {
        use handlers::*;

        App::new()
            .wrap(Logger::default())
            .data(pool.clone())
            .wrap(CookieSession::signed(&cookie_key).secure(false))
            .app_data(broadcaster.clone())
            .route("/events", web::get().to(new_client))
            .route("/send_msg", web::post().to(send_msg))
            .route("/send_paste", web::post().to(send_paste))
            .route("/get_pastes", web::get().to(get_pastes))
            .route("/raw/{id}", web::get().to(get_paste_raw))
            .route("/paste/{id}", web::get().to(get_paste))
            .route("/send_cmd", web::post().to(chat_command))
            .service(actix_files::Files::new("/", "frontend/dist").index_file("index.html"))
    })
    .bind(&bind_addr)?
    .run();

    println!("Running on: {}", bind_addr);

    server.await
}

mod handlers {
    type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

    use crate::*;
    use actix_session::Session;
    use actix_web::web;
    use diesel::r2d2::ConnectionManager;
    use diesel::PgConnection;

    #[derive(Deserialize)]
    pub struct NewClientQueryParams {
        nick: String,
        color: Option<String>,
    }

    pub async fn new_client(
        params: web::Query<NewClientQueryParams>,
        broadcaster: Data<Mutex<Broadcaster>>,
        session: Session,
    ) -> Result<impl Responder, actix_web::Error> {
        let mut broadcaster = broadcaster.lock().unwrap();
        session.set("nick", &params.nick).unwrap();

        let (rx, new_user) = broadcaster.new_user(&params.nick);

        session.set("id", new_user.id).unwrap();

        new_user.color = params.color.clone();

        new_user
            .sender
            .clone()
            .try_send(chat::event_data(chat::Msg::connected(&broadcaster.history)))
            .unwrap();

        Ok(HttpResponse::Ok()
            .header("content-type", "text/event-stream")
            .no_chunking()
            .streaming(rx))
    }

    pub async fn send_msg(
        msg: web::Json<String>,
        broadcaster: Data<Mutex<Broadcaster>>,
        session: Session,
    ) -> Result<impl Responder, actix_web::Error> {
        let id = match session.get::<u64>("id")? {
            Some(id) => id,
            None => return Ok(HttpResponse::Unauthorized().body("")),
        };
        broadcaster.lock().unwrap().send(id, msg.0.clone());

        Ok(HttpResponse::Ok().body(""))
    }

    #[derive(Deserialize)]
    pub struct NewPaste {
        filename: String,
        content: String,
    }

    pub async fn send_paste(
        new_paste: web::Json<NewPaste>,
        broadcaster: Data<Mutex<Broadcaster>>,
        session: Session,
        pool: Data<Pool>,
    ) -> Result<impl Responder, actix_web::Error> {
        if let None = session.get::<String>("nick")? {
            return Ok(HttpResponse::Unauthorized().body(""));
        }

        let new_paste = models::Paste {
            id: 0,
            filename: Some(new_paste.0.filename),
            content: Some(new_paste.0.content),
            creation_date: now(),
        };

        use crate::schema::pastes::dsl::pastes;
        let paste = match diesel::insert_into(pastes)
            .values(new_paste)
            .get_result::<models::Paste>(&pool.get().unwrap())
        {
            Ok(paste) => paste,
            Err(e) => {
                println!("Error inserting new paste: {}", e);
                return Ok(HttpResponse::InternalServerError().body(""));
            }
        };

        broadcaster.lock().unwrap().send_paste(paste);

        Ok(HttpResponse::Ok().body(""))
    }

    #[derive(Deserialize)]
    pub struct GetPastesQuery {
        page: Option<i64>,
        per_page: Option<i64>,
    }

    pub async fn get_pastes(
        query: web::Query<GetPastesQuery>,
        session: Session,
        pool: Data<Pool>,
    ) -> impl Responder {
        if let None = session.get::<String>("nick").unwrap() {
            return HttpResponse::Unauthorized().body("");
        }

        let db_conn = match pool.get() {
            Ok(conn) => conn,
            Err(e) => {
                println!("Failed to get connection to the database: {}", e);
                return HttpResponse::InternalServerError().body("");
            }
        };
        let pastes = {
            use crate::schema::pastes::dsl::*;

            pastes
                .order(id.desc())
                .paginate(query.page.unwrap_or(1), query.per_page.unwrap_or(10))
                .load_and_count_pages::<models::Paste>(&db_conn)
        };
        let pastes = match pastes {
            Ok(pastes) => pastes,
            Err(e) => {
                println!("Error getting pastes: {}", e);
                return HttpResponse::InternalServerError().body("");
            }
        };
        HttpResponse::Ok()
            .content_type("application/json")
            .body(serde_json::to_string(&pastes).unwrap())
    }

    pub async fn get_paste_raw(path: web::Path<i64>, pool: Data<Pool>) -> impl Responder {
        let requested_id = *path;

        let db_conn = match pool.get() {
            Ok(conn) => conn,
            Err(e) => {
                println!("Failed to get connection to the database: {}", e);
                return HttpResponse::InternalServerError().body("");
            }
        };

        let paste = {
            use crate::schema::pastes::dsl::*;

            pastes
                .filter(id.eq(requested_id))
                .first::<models::Paste>(&db_conn)
                .expect("Database error")
        };

        HttpResponse::Ok()
            .content_type("text/plain charset=UTF-8")
            .body(paste.content.unwrap_or_default())
    }

    pub async fn get_paste(path: web::Path<i64>, pool: Data<Pool>) -> impl Responder {
        let requested_id = *path;

        let db_conn = match pool.get() {
            Ok(conn) => conn,
            Err(e) => {
                println!("Failed to get connection to the database: {}", e);
                return HttpResponse::InternalServerError().body("");
            }
        };

        let paste: models::Paste = {
            use crate::schema::pastes::dsl::*;

            pastes
                .filter(id.eq(requested_id))
                .first::<models::Paste>(&db_conn)
                .expect("Database error")
        };

        HttpResponse::Ok()
            .content_type("text/html charset=UTF-8")
            .streaming(
                get_paste::PasteRenderer::new(&[
                    paste
                        .filename
                        .clone()
                        .unwrap_or(paste.id.to_string())
                        .into_bytes(),
                    paste.content.unwrap_or_default().into_bytes(),
                ])
                .expect("io failed :("),
            )
    }

    #[derive(Deserialize)]
    pub enum ChatCommand {
        Color(String),
        Nick(String),
    }

    pub async fn chat_command(
        cmd: web::Json<ChatCommand>,
        session: Session,
        broadcaster: Data<Mutex<Broadcaster>>,
    ) -> Result<impl Responder, actix_web::Error> {
        let id = match session.get::<u64>("id")? {
            Some(id) => id,
            None => return Ok(HttpResponse::Unauthorized().body("")),
        };
        let mut broadcaster = broadcaster.lock().unwrap();
        let user = &mut broadcaster.users.iter_mut().find(|u| u.id == id).unwrap();

        match cmd.0 {
            ChatCommand::Color(color) => {
                user.sender
                    .try_send(chat::event_data(chat::Msg::color_change_msg(&color)))
                    .unwrap();
                user.color = Some(color);
            }
            ChatCommand::Nick(nick) => {
                session.set("nick", &nick).expect("Failed to change nick");
                user.sender
                    .try_send(chat::event_data(chat::Msg::nick_change_msg(&nick)))
                    .unwrap();
                user.nick = nick;
            }
        }
        Ok(HttpResponse::Ok().body(""))
    }

    pub fn now() -> chrono::NaiveDateTime {
        let since_unix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards");
        chrono::NaiveDateTime::from_timestamp(since_unix.as_secs() as i64, 0)
    }
}
