mod routes;
use actix_web::{App, HttpResponse, HttpServer, Responder,Result,web};
use std::sync::{Arc, Mutex};
use actix_cors::Cors;
use actix_web::http::header;
use tokio_postgres::{NoTls,Error,Client};
use tokio::spawn;
use std::env;

#[actix_web::main]//main function to run
async fn main() -> std::io::Result<()>{

let  (mut pool,conn)= tokio_postgres::connect("db_url",NoTls).await.unwrap();
    spawn(async move {
        if let Err(e) = conn.await {
            println!("connection error: {}", e);
        }
    });
   
    pool.execute("
    CREATE TABLE IF NOT EXISTS users (
        email TEXT PRIMARY KEY,
        password TEXT NOT NULL
        );
        ",&[]).await.unwrap();

    pool.execute("CREATE TABLE IF NOT EXISTS duels (
            duel_id TEXT PRIMARY KEY,
            duel_name TEXT NOT NULL UNIQUE,
            current_round INTEGER DEFAULT 1,
            rounds INTEGER NOT NULL,
            image TEXT,
            description TEXT NOT NULL,
            duel_admin TEXT,
            FOREIGN KEY(duel_admin) REFERENCES users(email)
            );
        ",&[]).await.unwrap();

    pool.execute("CREATE TABLE IF NOT EXISTS duel_judges (
            id BIGSERIAL PRIMARY KEY,
            judge_email TEXT NOT NULL,
            duel_id TEXT,
            FOREIGN KEY (duel_id) REFERENCES duels (duel_id)
        );
        ",&[]).await.unwrap();

   let np = Arc::new(Mutex::new(pool));
            println!("hello thereeee");
            
    let port = match std::env::var("PORT"){
                Ok(port)=>port,
                _=>String::from("8080")
            };

    let address = format!("0.0.0.0:{}",port);
    
    println!("{}",port);
    HttpServer::new(move|| {

        

        let cors = Cors::default()
        .allow_any_origin()
        .allowed_methods(vec!["GET", "POST"])
        .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
        .allowed_header(header::CONTENT_TYPE)
        .max_age(3600);

        App::new().wrap(cors).service(
            web::scope("/duel")// add /duel to all routes as prefix
            .app_data(web::Data::new(np.clone()))
            .service(routes::sign_up)
            .service(routes::create_duels)
            .service(routes::creator_login)
            .service(routes::get_all_duels)
            .service(routes::upload_url)
            .service(routes::add_judges_to_duel)
            .service(routes::update_duels)
        ) // register the route
    }).bind((
        address
    ))?
    .run()
    .await
}
