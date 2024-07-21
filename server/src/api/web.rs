use std::path::PathBuf;

use rocket::{fs::NamedFile, get, routes, Route};

#[get("/<path>")]
pub async fn serve(path: PathBuf) -> Option<NamedFile> {
    let path = PathBuf::from("web").join(path);

    if path.is_dir() {
        NamedFile::open(path.join("index.html")).await.ok()
    } else {
        NamedFile::open(path).await.ok()
    }
}

#[get("/")]
pub async fn index() -> Option<NamedFile> {
    NamedFile::open("web/index.html").await.ok()
}

pub fn exports() -> Vec<Route> {
    routes![index, serve]
}
