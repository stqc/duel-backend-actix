use actix_web::{get,post,web,HttpResponse,Result,Responder};
use serde::{Deserialize,Serialize};
use std::sync::{Arc, Mutex};
use bcrypt::{hash,verify};
use std::time::SystemTime;
use tokio_postgres::Client;
use google_cloud_storage::client;
use google_cloud_storage::sign::SignedURLOptions;
use google_cloud_storage::sign::SignedURLMethod;
use google_cloud_storage::http::objects::upload::{Media, UploadObjectRequest, UploadType};
use google_cloud_auth::credentials::CredentialsFile;

#[derive(Deserialize)]
struct User{
    email: String,
    password: String,
}

#[derive(Deserialize,Serialize)]
struct duel{
    id:String,
    name:String,
    description:String,
    rounds:i32,
    // round_start:, add in final
    // round_end,
    image: String,
    email:String
}

#[derive(Serialize)]
struct Status{
    stat: bool,
}

#[derive(Deserialize)]
struct Email{
    email:String
}

#[derive(Deserialize)]
struct ImageData{
    email:String,
    duel_name:String,
}

#[derive(Serialize)]
struct Duel_array{
    duels:Vec<duel>
}

#[derive(Serialize,Deserialize)]
struct Message{
    message:String
}

#[derive(Serialize,Deserialize)]
struct DuelJudge{
    email:String,
    duel_id:String,
}

type DbConnection =Arc<Mutex<Client>>;

#[post("/sign_up")]
async fn sign_up(inf: web::Json<User>, pool:web::Data<DbConnection>)-> Result<impl Responder>{
    
    let mut conn = pool.lock().unwrap();
    
    let mut status=false;
    
    let encryptedPass = hash(&inf.password,12).unwrap();

    match conn.execute("INSERT INTO users (email,password) values ($1,$2)",&[&inf.email,&encryptedPass]).await.unwrap(){
        1=>{
            status=true;

        },
        _=>{
            status=false
        }
    }

    Ok(web::Json(Message{message:"Sign up successful".to_string(),}))
}

//login route to send a jwt
#[post("/creator_login")]
async fn creator_login(pool:web::Data<DbConnection>,data:web::Json<User>)-> Result<impl Responder>{
    
    let mut conn = pool.lock().unwrap();

    let query = ["select * from users where email='",&data.email,"'"].join("");

    let mut statement=conn.prepare(&query).await.unwrap();
      
    let  mut row = conn.query(&query,&[]).await.unwrap();

    let mut found=false;

    let mut pass_hash:String;

    let mut matches=false;
    let mut count =0;

    //generate a JWT here in future
    for ro in row{
            
            let x:String= ro.get(0);
            count+=1;
            pass_hash=ro.get(1);
            matches=verify(&data.password,&pass_hash).unwrap();
    }

    Ok(
        web::Json(Status{stat:matches})
    )
}


//add authentication mechanism only the invited emails can make a new duel
#[post("/create_duels")]
async fn create_duels(pool:web::Data<DbConnection>,data:web::Json<duel>)->Result<impl Responder>{
    
    let mut conn = pool.lock().unwrap();

    let q = ["SELECT * FROM users WHERE email='",&data.email,"'"].join("");
    // let mut statement = conn.prepare(&q).unwrap();
    let mut row = conn.query(&q,&[]).await.unwrap();

    // let  Some(rows) = row.unwrap() else {return Ok(web::Json(Status{stat:false}))};
    if row.len()==0{
        return Ok(web::Json(Message{message:"User Not Found".to_string(),}));
    }

    let mut expected:User=User{email:String::new(),password:String::new()};
    for r in row{
        expected = User{email:r.get(0),password:r.get(1)};
    }
     
    
    let mut stat1=false;
    let mut status=false;
    let email:String = data.email.clone();

    match expected.email{
       ref email =>{
            println!("{}={}",email,expected.email);
            stat1=true;
           
        }
        _=>{
            println!("ererere");

            stat1=false;
        }
    }

    if stat1{
        println!("yay");

        let credFile = CredentialsFile::new_from_file("upload.json".to_string()).await.unwrap();
    
        let config = client::ClientConfig::default().with_credentials(credFile).await.unwrap();

        let client = client::Client::new(config);

        let current_time_stamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();

        let id  = ["duel", &current_time_stamp.as_secs().to_string()].join("_");

        let name = format!("{}_{}",data.email,data.name);
        println!("{}",name);

        // let image_url = client.signed_url("production.duel.cool/test",&name, None, None,SignedURLOptions::default()).await.unwrap();
        
        match conn.execute("INSERT INTO duels (duel_id,duel_name,rounds,image,description,duel_admin) values ($1,$2,$3,$4,$5,$6)",&[&id,&data.name,&data.rounds,&name,&data.description,&data.email]).await.unwrap(){
            1=>{
                println!("yay2");
                status=true;
            }
            _=>{
                println!("err");
                return Ok(web::Json(Message{message:"An Error Occured".to_string(),}));
            }
    }

    return Ok(web::Json(Message{message:"Duel Created".to_string(),}));
}
println!("nay?");
return Ok(web::Json(Message{message:"Wrong Email".to_string(),}))
    
}


#[post("/update_duels")]
async fn update_duels(pool:web::Data<DbConnection>, duel:web::Json<duel>)->Result<impl Responder>{
    
    let mut conn = pool.lock().unwrap();

    let q = ["SELECT * FROM users WHERE email='",&duel.email,"'"].join("");
    let mut row = conn.query(&q,&[]).await.unwrap();
    
    let d = ["SELECT * FROM duels WHERE duel_id='",&duel.id,"'"].join("");
    let mut row2 = conn.query(&d,&[]).await.unwrap();

    if row.len()==0{
        return Ok(web::Json(Message{message:"User Not Found".to_string(),}));
    }

    if row2.len()==0{
        return Ok(web::Json(Message{message:"Duel not found".to_string(),}));
    }

    let mut expected:User=User{email:String::new(),password:String::new()};
    for r in row2{
        expected = User{email:r.get(6),password:r.get(1)};
    }
     
    
    let mut stat1=false;
    let mut status=false;
    let email:String = duel.email.clone();
    println!("{}",email);
    println!("{}",expected.email);

    if(expected.email==email){
        stat1=true;
    }

    if stat1{
        println!("yay");
        
        match conn.execute("UPDATE duels SET rounds=$1, description=$2 WHERE duel_id=$3",&[&duel.rounds,&duel.description,&duel.id]).await.unwrap(){
            1=>{
                println!("yay2");
                status=true;
            }
            _=>{
                println!("err");
                status=false;
                return Ok(web::Json(Message{message:"An Error Occured".to_string(),}));
            }
        }
        return Ok(web::Json(Message{message:"Duel Updated".to_string(),}));

    }
    return Ok(web::Json(Message{message:"Not The Owner of The Duel".to_string(),}))



}


//add a jwt to this route
#[post("/add_judges_to_duel")]
async fn add_judges_to_duel(pool:web::Data<DbConnection>, judges:web::Json<DuelJudge>)->Result<impl Responder>{

    let mut conn = pool.lock().unwrap();
    let mut status=false;
    
    //future upgrades ensure only admin can add new duels, ensure the user created exists with us, if it does not send them an email to sign up with us

    match conn.execute("INSERT INTO duel_judges(judge_email,duel_id) VALUES ($1,$2)",&[&judges.email,&judges.duel_id]).await.unwrap(){
        1=>{
            println!("Added");
            status=true;
        }
        _=>{
            status=false;
        }
    }

    Ok(web::Json(Status{stat:status}))



}

#[get("/get_all_duels_of_user")]
async fn get_all_duels(pool:web::Data<DbConnection>, creator:web::Query<Email>)->Result<impl Responder>{

    let mut conn = pool.lock().unwrap();
    
    let email = creator.email.clone();

    let query = ["SELECT * FROM duels where duel_admin='",&email,"'"].join("");

    // let mut statement = conn.prepare(&query).unwrap();
    let mut rows = conn.query(&query,&[]).await.unwrap();

    let mut duels_vec:Vec<duel>=vec![];

    let credFile = CredentialsFile::new_from_file("upload.json".to_string()).await.unwrap();
    
    let config = client::ClientConfig::default().with_credentials(credFile).await.unwrap();

    let client = client::Client::new(config);

    for row in rows{
        
                let image_url = client.signed_url("production.duel.cool/test",row.get(4), None, None,SignedURLOptions::default()).await.unwrap();
                duels_vec.push(duel{
                    id:row.get(0),
                    name:row.get(1),
                    description:row.get(5),
                    rounds:row.get(3),
                    image:image_url,
                    email:row.get(6)
                });
    }

    
    Ok(web::Json(Duel_array{duels:duels_vec}))

}

#[get("/upload_image")]
async fn upload_url(user: web::Query<ImageData>)->Result<impl Responder>{

    let credFile = CredentialsFile::new_from_file("upload.json".to_string()).await.unwrap();
    
    let config = client::ClientConfig::default().with_credentials(credFile).await.unwrap();
    let client = client::Client::new(config);

    let name = format!("{}_{}",user.email,user.duel_name);

    let upload = client.signed_url("production.duel.cool/test",&name, None, None,SignedURLOptions{
        method:SignedURLMethod::PUT,
        ..Default::default()
    }).await.unwrap();

    Ok(web::Json(Message{message:upload}))
}

