use actix_files::NamedFile;
use actix_web::{
    get,
    http::header::{self, ContentType},
    web, App, HttpRequest, HttpResponse, HttpServer, Responder, Result,
};
use std::{
    io::{BufReader, Read, Seek, SeekFrom},
    path::PathBuf,
};
use videoflix::RangeHeader;

#[get("/")]
async fn index() -> Result<NamedFile> {
    let path: PathBuf = "./index.html".parse().unwrap();
    Ok(NamedFile::open(path)?)
}

#[get("/{name}")]
async fn hello(name: web::Path<String>, request: HttpRequest) -> impl Responder {
    if name.to_string() == "test" {
        let header_map = request.headers();
        if !header_map.contains_key(header::RANGE) {
            return HttpResponse::BadRequest().finish();
        }
        let size = std::fs::metadata("test.mp4").unwrap().len();
        let range = header_map.get(header::RANGE).unwrap();
        let range_header = RangeHeader::parse(range, size).unwrap();
        if !(range_header.unit == "bytes") {
            return HttpResponse::BadRequest().finish();
        }
        let content_length = range_header.end - range_header.start + 1;
        let file = std::fs::File::open("test.mp4").unwrap();
        let mut buf_reader = BufReader::new(file);
        let _position = buf_reader
            .seek(SeekFrom::Start(range_header.start))
            .unwrap();
        let mut content = Vec::<u8>::new();
        let mut buffer = [0; videoflix::CHUNK_SIZE];
        let size_read = buf_reader.read(&mut buffer[..]).unwrap();
        if size_read > videoflix::CHUNK_SIZE {
            return HttpResponse::InternalServerError().finish();
        }
        content.extend_from_slice(&buffer[..size_read]);

        let response = HttpResponse::PartialContent()
            .content_type(ContentType("video/mp4".parse::<mime::Mime>().unwrap()))
            .append_header(("Content-Length", content_length))
            .append_header((
                "Content-Range",
                format!("bytes {}-{}/{}", range_header.start, range_header.end, size),
            ))
            .append_header(("Accept-Range", "bytes")).body(content);
        return response;
    }
    HttpResponse::NotFound().finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(index).service(hello))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
