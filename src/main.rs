#[macro_use] extern crate rocket;
use rocket::State;
use rocket::figment::Figment;
use rocket::data::{Data, ToByteUnit};
#[cfg(debug_assertions)]
use rocket::fs::NamedFile;
#[cfg(not(debug_assertions))]
use rocket::http::ContentType;

struct EditorConfig {
    path: String
}

#[cfg(debug_assertions)]
#[get("/")]
async fn index() -> Option<NamedFile> {
    println!("debug.");
    NamedFile::open("src/index.html").await.ok()
}
#[cfg(not(debug_assertions))]
#[get("/")]
async fn index() -> (ContentType, &'static str) {
    println!("release.");
    (ContentType::HTML, include_str!("index.html"))
}

#[post("/api/file/load")]
async fn file_load(config: &State<EditorConfig>) -> String {
    println!("read {}", &(config.path));
    if std::path::Path::new(&(config.path)).exists() {
        std::fs::read_to_string(&(config.path)).expect("")
    } else {
        String::new()
    }
}

#[post("/api/file/save", data = "<data>")]
async fn file_save(config: &State<EditorConfig>, data: Data<'_>) -> String {
    println!("wite {}", &(config.path));
    // read request
    let stream = data.open(2.megabytes());
    let text = stream.into_string().await.expect("request decode error").into_inner();
    match std::fs::write(&(config.path), text) {
        Ok(_) => String::from("ok"),
        Err(_) => String::from("error")
    }
}

#[rocket::main]
async fn main() {
    let file_path: String = std::env::args().nth(1).expect("no file path given");
    let figment = Figment::from(rocket::Config::default())
        // .merge(("log_level", "critical"))
        .merge(("port", 18080))
        .merge(("address", "0.0.0.0"));
    rocket::custom(figment)
        .manage(EditorConfig {path: file_path})
        .mount("/", routes![
            index,
            file_load,
            file_save,
        ])
        .launch()
        .await.ok();
}
