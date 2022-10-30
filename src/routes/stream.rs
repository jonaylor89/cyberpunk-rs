use actix_files::NamedFile;
use actix_web::{Result, HttpRequest};

pub async fn stream(req: HttpRequest) -> Result<NamedFile> {
    let filename: String = req.match_info().get("filename").unwrap().parse().unwrap();
    tracing::info!("the filename: {}", filename);
    Ok(NamedFile::open(format!(
        "/Users/johannes/Repos/cyberpunk-rs/testdata/{}",
        filename
    ))
    ?.set_content_type("audio/mpeg3".parse::<mime::Mime>().unwrap()))
}
