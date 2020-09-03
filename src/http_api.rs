use actix_web::{web, App, HttpResponse, HttpServer};

use crate::iiif::IiifGenerator;
use crate::image::source::ImageSource;

#[actix_rt::main]
pub async fn start(
    iiif_generator: IiifGenerator,
    image_source: ImageSource,
    bind: String,
) -> std::io::Result<()> {
    println!("Starting iiif-presenter on http://{}", bind);
    let iiif_generator_ref = web::Data::new(iiif_generator);
    let image_source_ref = web::Data::new(image_source);
    HttpServer::new(move || {
        App::new()
            .app_data(iiif_generator_ref.clone())
            .app_data(image_source_ref.clone())
            .service(index)
            .service(collection)
    })
    .bind(bind)?
    .run()
    .await
}

#[get("/{id:.*}/manifest")]
async fn index(
    image_source: web::Data<ImageSource>,
    iiif_generator: web::Data<IiifGenerator>,
    path: web::Path<String>,
) -> HttpResponse {
    println!("Url-Path (Manifest): {}", path.to_string());
    let id = path.to_string();
    let images = match image_source.load(&id) {
        Some(images) => images,
        None => return HttpResponse::NotFound().body(id),
    };
    println!("Images: {}", images.len());
    match iiif_generator.get_ref().manifest_for(&id, images) {
        Ok(manifest) => HttpResponse::Ok().json(manifest),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

#[get("/{id:.*}/collection")]
async fn collection(
    iiif_generator: web::Data<IiifGenerator>,
    path: web::Path<String>,
) -> HttpResponse {
    println!("Url-Path (Collection): {}", path.to_string());
    let id = path.to_string();
    match iiif_generator.get_ref().collection_for(&id) {
        Ok(manifest) => HttpResponse::Ok().json(manifest),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
