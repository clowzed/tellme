use actix_web::{get, post, web, App, HttpServer, HttpResponse, Result, http};
use std::collections::HashMap;
use std::io::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};


macro_rules! hashme
{
    ($hashme:expr) =>
    {
        {
            let mut hasher = DefaultHasher::new();
            $hashme.hash(&mut hasher);
            hasher.finish()
        }
    };
}


macro_rules! open_file
{
    ($filename:expr) =>
    {
        {
            std::fs::OpenOptions::new()
                                 .write(true)
                                 .create(true)
                                 .truncate(true)
                                 .open($filename)?
        }
    }
}




#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct Service
{
    service_type         : String,
    available            : bool,
    healthcheck_endpoint : String,
    is_accepted          : bool,
    identifier           : String,
}


#[derive(serde::Deserialize, serde::Serialize)]
pub struct ServiceForm
{
    service_type         : String,
    access_token         : String,
    healthcheck_endpoint : String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct NewTokenForm
{
    login    : String,
    password : String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Token
{
    token: String,
}


#[derive(serde::Deserialize, serde::Serialize)]
pub struct FindForm
{
    service_type : Option<String>,
    limit        : Option<usize>,
    available    : Option<bool>,
}



#[derive(serde::Deserialize, serde::Serialize)]
pub struct AdminCredentials
{
    login_hash    : u64,
    password_hash : u64,
}


pub struct AppState
{
    pub access_tokens_hashes : Vec<u64>,
    pub services             : HashMap<String, Service>,
    pub admin_credentials    : AdminCredentials,
}


#[derive(serde::Deserialize, serde::Serialize)]
pub struct AcceptForm
{
    login      : String,
    password   : String,
    identifier : String,
}


#[post("/newtoken")]
async fn new_token(data: web::Data<std::sync::Mutex<AppState>>, form : web::Form<NewTokenForm>) -> Result<HttpResponse>
{
    let mut app_data = data.lock().unwrap();

    if hashme!(form.login)    != app_data.admin_credentials.login_hash    ||
       hashme!(form.password) != app_data.admin_credentials.password_hash
    {
        return Err(actix_web::error::ErrorUnauthorized("Bad credentials!"));
    }

    let new_token = uuid::Uuid::new_v4().to_string();
    app_data.access_tokens_hashes.push(hashme!(new_token));

    Ok(HttpResponse::Ok()
                        .set_header(actix_web::http::header::CONTENT_TYPE, 
                                    "application/json")
                        .json(Token{token: new_token}))

}


#[post("/accept_service")]
async fn accept_service(data: web::Data<std::sync::Mutex<AppState>>, form:web::Form<AcceptForm>) -> Result<HttpResponse>
{
    let mut app_data = data.lock().unwrap();

    if hashme!(form.login)    != app_data.admin_credentials.login_hash    ||
       hashme!(form.password) != app_data.admin_credentials.password_hash
    {
        return Err(actix_web::error::ErrorUnauthorized("Bad credentials!"));
    }

    if !app_data.services.contains_key(&form.identifier)
    {
        return  Err(actix_web::error::ErrorNotFound("Identifier was not found!"));
    }

    app_data.services.get_mut(&form.identifier).unwrap().is_accepted = true;

    Ok(HttpResponse::Accepted().into())
}

#[post("/disable_service")]
async fn disable_service(data: web::Data<std::sync::Mutex<AppState>>, form:web::Form<AcceptForm>) -> Result<HttpResponse>
{
    let mut app_data = data.lock().unwrap();

    if hashme!(form.login)    != app_data.admin_credentials.login_hash    ||
       hashme!(form.password) != app_data.admin_credentials.password_hash
    {
        return Err(actix_web::error::ErrorUnauthorized("Bad credentials!"));
    }

    if !app_data.services.contains_key(&form.identifier)
    {
        return  Err(actix_web::error::ErrorNotFound("Identifier was not found!"));
    }

    app_data.services.get_mut(&form.identifier).unwrap().is_accepted = false;

    Ok(HttpResponse::Accepted().into())
}

#[get("/find")]
async fn find(data: web::Data<std::sync::Mutex<AppState>>, query: web::Query<FindForm>) -> Result<HttpResponse>
{
    let app_data = data.lock().unwrap();

    let mut all_services = app_data.services
                                    .values()
                                    .cloned()
                                    .filter(|service|
                                             service.is_accepted)
                                    .collect::<Vec<Service>>();


    if let Some(service_type) = &query.service_type
    {
        all_services = all_services.into_iter()
                                   .filter(|service|
                                            service.service_type.eq(service_type))
                                   .collect::<Vec<Service>>();
    }

    if let Some(is_available) = query.available
    {
        all_services = all_services
                           .into_iter()
                           .filter(|service|
                                    service.available == is_available)
                           .collect::<Vec<Service>>();
    }

    if let Some(limit_size) = query.limit
    {
        all_services.truncate(limit_size);
    }

    Ok(HttpResponse::Ok()
                        .set_header(http::header::CONTENT_TYPE,
                                    "application/json")
                        .json(all_services))
}


#[post("/me")]
async fn register_service(data: web::Data<std::sync::Mutex<AppState>>, form:web::Form<ServiceForm>) -> Result<HttpResponse>
{
    let access_token = &form.access_token;

    let token_hash   = hashme!(access_token);

    let mut app_data = data.lock().unwrap();


    if app_data.access_tokens_hashes.contains(&token_hash)
    {
        let unique_identifier = uuid::Uuid::new_v4().to_string();

        let service = Service {
                                service_type         : form.service_type.clone(),
                                available            : false,
                                is_accepted          : false,
                                healthcheck_endpoint : form.healthcheck_endpoint.clone(),
                                identifier           : unique_identifier.clone(),
                            };

        app_data.services.insert(unique_identifier, service.clone());

        return Ok(HttpResponse::Ok()
                                    .set_header(http::header::CONTENT_TYPE,
                                                "application/json")
                                    .json(service))
    }

    Err(actix_web::error::ErrorUnauthorized("Incorrect access token!"))
}





#[actix_web::main]
async fn main() -> std::io::Result<()>
{

    let mut healthcheck_interval_seconds :u64 = 30;
    let mut port                         :u32 = 5000;
    let mut credentials_filename         : String = String::new();


    {
        let mut parser = argparse::ArgumentParser::new();

        parser.set_description("Tellme is a tiny service registry and health checker");

        parser.refer(&mut healthcheck_interval_seconds)
              .add_option(
            &vec!["-i", "--interval"],
            argparse::Store,
            "Set healthcheck interval",
        );

        parser.refer(&mut port)
              .add_option(&vec!["-p", "--port"],
                         argparse::Store,
                          "Set port");

        parser.refer(&mut credentials_filename)
              .add_option(&vec!["-c", "--creds"],
                         argparse::Store,
                          "Set credentials filename");


        parser.parse_args_or_exit();

    }

    let credentials_filename = match credentials_filename.len()
    {
        0 => String::from("tellme.creds"),
        _ => credentials_filename,
    };


    let mut admin_credentials_file = open_file!(credentials_filename);

    let login    = uuid::Uuid::new_v4().to_string();
    let password = uuid::Uuid::new_v4().to_string();

    match writeln!(&mut admin_credentials_file, "login : {}", login)
    {
        Ok(()) => {},
        Err(e) => {
            println!("Failed to write login to file! Reason: {}", e.to_string());
            return Ok(()); // Hmm why ok?
        }
    }

    match writeln!(&mut admin_credentials_file, "password : {}", password)
    {
        Ok(()) => {},
        Err(e) => {
            println!("Failed to write password to file! Reason: {}", e.to_string());
            return Ok(()); // Hmm why ok?
        }
    }


    let app_data = web::Data::new(std::sync::Mutex::new(AppState{
        access_tokens_hashes : Vec::new(),
        services             : HashMap::new(),
        admin_credentials    : AdminCredentials{ login_hash     : hashme!(login),
                                                  password_hash : hashme!(password)
                                                }
    }));



    let cloned_data = app_data.clone();


    actix_web::rt::spawn(async move
        {
            let mut interval = actix_web::rt::time::interval(std::time::Duration::from_secs(healthcheck_interval_seconds));

            let connector = actix_web::client::Connector::new()
                                                                .timeout(std::time::Duration::from_secs(100))
                                                                .finish();
            let client = actix_web::client::ClientBuilder::new()
                                                                .connector(connector)
                                                                .timeout(std::time::Duration::from_secs(100)) // Why 100? Well first connect requires too long time som Timeout is recieved. Look at actix-web issues
                                                                .finish();
            loop
                {
                    interval.tick().await;

                    let services = &cloned_data.lock().unwrap().services.values()
                                                                            .cloned()
                                                                            .filter(|service|
                                                                                     service.is_accepted)
                                                                            .collect::<Vec<Service>>();
                    for service in services.iter()
                    {
                        let status:bool =
                        {
                            let response = client.get(&service.healthcheck_endpoint)
                                                                                  .send()
                                                                                  .await;
                            match response
                            {
                                Ok(response) => response.status().is_success(),
                                Err(_) => false,
                            }
                        };

                        cloned_data.lock().unwrap().services.get_mut(&service.identifier)
                                                            .unwrap()
                                                            .available = status;
                    }

                }

        }
    );

    HttpServer::new(
    move ||
         App::new()
         .app_data(app_data.clone())
         .service(register_service)
         .service(find)
         .service(accept_service)
         .service(disable_service)
         .service(new_token)
    )
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await

}
