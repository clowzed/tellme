#![feature(let_chains)]

mod config;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;

macro_rules! hashme {
    ($hashme:expr) => {{
        let mut hasher = DefaultHasher::new();
        $hashme.hash(&mut hasher);
        hasher.finish()
    }};
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct Service {
    service_type         : String,
    available            : bool,
    healthcheck_endpoint : String,
    is_accepted          : bool,
    identifier           : String,
    ip                   : url::Url,
}

impl Service{
    pub async fn ping(&mut self){
        self.available = {
            match self.ip.join(&self.healthcheck_endpoint)
            {
                Ok(endpoint) => match reqwest::get(endpoint).await{
                    Ok(response) => response.status().is_success(),
                    Err(_) => false
                },
                Err(_) => false
            }
        };
    }

    pub async fn notify(&self, data: &Self, endpoint: String){
        match self.ip.join(&endpoint) {
            Ok(endpoint) => {
                let client = reqwest::Client::new();
                client.post(endpoint.to_owned()).form(data).send().await.ok();
            },
            Err(_) => {}
        };
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ServiceForm {
    service_type         : String,
    access_token         : String,
    healthcheck_endpoint : String,
    port                 : u16
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct NewTokenForm {
    login    : String,
    password : String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Token {
    token: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct FindForm {
    service_type : Option<String>,
    limit        : Option<usize>,
    available    : Option<bool>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct AdminCredentials {
    login_hash    : u64,
    password_hash : u64,
}

#[derive(Clone, Debug)]
struct SubscribeParams{
    identifier      : String,
    on_registration : bool,
    on_acceptance   : bool,
    endpoint        : String
}

struct AppState {
    pub access_tokens_hashes : Vec<u64>,
    pub services             : HashMap<String, Service>,
    pub admin_credentials    : AdminCredentials,
    pub subscribers          : Vec<SubscribeParams>
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct AcceptForm {
    login      : String,
    password   : String,
    identifier : String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SubscribeForm {
    login           : String,
    password        : String,
    identifier      : String,
    on_registration : bool,
    on_acceptance   : bool,
    endpoint        : String,
}


#[actix_web::post("/newtoken")]
async fn new_token(
    data: actix_web::web::Data<std::sync::Mutex<AppState>>,
    form: actix_web::web::Form<NewTokenForm>,
) -> actix_web::HttpResponse {

    if let Ok(mut app_data) = data.lock(){

        // This is checking if the login and password are correct.
        // If they are not then it returns an unauthorized response.
        // If the login and password are correct then it creates a new token and returns it.
        if hashme!(form.login)    != app_data.admin_credentials.login_hash ||
           hashme!(form.password) != app_data.admin_credentials.password_hash {
            return actix_web::HttpResponse::Unauthorized().finish();
        }

        let new_token = uuid::Uuid::new_v4().to_string();
        app_data.access_tokens_hashes.push(hashme!(new_token));

        return actix_web::HttpResponse::Ok().insert_header((actix_web::http::header::CONTENT_TYPE,
                                                        "application/json"))
                                            .json(Token { token: new_token })
    }
    actix_web::HttpResponse::InternalServerError().finish()
}

#[actix_web::post("/accept_service")]
async fn accept_service(
    data: actix_web::web::Data<std::sync::Mutex<AppState>>,
    form: actix_web::web::Form<AcceptForm>,
) -> actix_web::HttpResponse {
    if let Ok(mut app_data) = data.lock(){

        // This is checking if the login and password are correct. If they are not then it returns an unauthorized response.
        // If the service is not found then it returns a not found response.
        // If the service is found then it sets the is_accepted to true.
        if hashme!(form.login)    != app_data.admin_credentials.login_hash ||
           hashme!(form.password) != app_data.admin_credentials.password_hash {
            return actix_web::HttpResponse::Unauthorized().finish();
        }

        if !app_data.services.contains_key(&form.identifier) {
            return actix_web::HttpResponse::NotFound().finish();
        }

        app_data.services.get_mut(&form.identifier)
                         .unwrap() //* If this fails then Mutex in rust does not work
                         .is_accepted = true;

        let cloned_app_data = data.clone();
        let cloned_service = app_data.services.get(&form.identifier).unwrap().clone(); // Still trusting mutex

        actix_web::rt::spawn(async move{
            if let Ok(app_state) = cloned_app_data.lock(){
                let subscribers = app_state.subscribers.clone();

                let services = app_state.services
                                        .values()
                                        .map(|service|
                                            match subscribers.iter().find(|params| {
                                                                            params.identifier == service.identifier
                                                                            && params.on_acceptance
                                            }) {
                                                // Clone for dropping mutex lock
                                                Some(found) => Some((service.clone(), found.endpoint.clone())),
                                                None => None,
                                            }
                                        ).flatten().collect::<Vec<_>>();
                drop(app_state);

                for service in services{
                    service.0.notify(&cloned_service, service.1.clone()).await
                }
            }
        });
        return actix_web::HttpResponse::Accepted().finish();
    }
    actix_web::HttpResponse::InternalServerError().finish()
}

#[actix_web::post("/disable_service")]
async fn disable_service(
    data: actix_web::web::Data<std::sync::Mutex<AppState>>,
    form: actix_web::web::Form<AcceptForm>,
) -> actix_web::HttpResponse {
    if let Ok(mut app_data) = data.lock(){

        // This is checking if the login and password are correct. If they are not then it returns an unauthorized response.
        // If the service is not found then it returns a not found response.
        // If the service is found then it sets the is_accepted to false.
        if hashme!(form.login)    != app_data.admin_credentials.login_hash ||
           hashme!(form.password) != app_data.admin_credentials.password_hash {
            return actix_web::HttpResponse::Unauthorized().finish();
        }

        if !app_data.services.contains_key(&form.identifier) {
            return actix_web::HttpResponse::NotFound().finish();
        }

        app_data.services
                .get_mut(&form.identifier)
                .unwrap() //* If this fails then Mutex in rust does not work
                .is_accepted = false;
        }

    actix_web::HttpResponse::Accepted().finish()
}

#[actix_web::get("/find")]
async fn find(
    data : actix_web::web::Data<std::sync::Mutex<AppState>>,
    query: actix_web::web::Query<FindForm>,
) -> actix_web::HttpResponse {
    if let Ok(app_data) = data.lock(){

        let mut all_services = app_data.services
                                       .values()
                                       .cloned()
                                       .filter(|service| service.is_accepted)
                                       .collect::<Vec<Service>>();

        if let Some(service_type) = &query.service_type {
            all_services = all_services.into_iter()
                                       .filter(|service| service.service_type.eq(service_type))
                                       .collect::<Vec<Service>>();
        }

        if let Some(is_available) = query.available {
            all_services = all_services.into_iter()
                                       .filter(|service| service.available == is_available)
                                       .collect::<Vec<Service>>();
        }

        if let Some(limit_size) = query.limit {
            all_services.truncate(limit_size);
        }

        return actix_web::HttpResponse::Ok()
                                        .insert_header((actix_web::http::header::CONTENT_TYPE,
                                                    "application/json"))
                                        .json(all_services);
    }
    actix_web::HttpResponse::InternalServerError().finish()
}

#[actix_web::post("/me")]
async fn register_service(
    request : actix_web::HttpRequest,
    data    : actix_web::web::Data<std::sync::Mutex<AppState>>,
    form    : actix_web::web::Form<ServiceForm>,
) -> actix_web::HttpResponse {

    let ip = {
        match request.peer_addr(){
           Some(addr) => url::Url::parse(&("http://".to_string() + &addr.to_string())).ok(),
           None =>{
                match request.connection_info().realip_remote_addr(){
                    Some(addr) => url::Url::parse(&("http://".to_string() + &addr.to_string())).ok(),
                    None => None
                }
           }
        }
    };


    if let Ok(mut app_data) = data.lock() &&
       let Some(mut retrieved_ip) = ip{
        retrieved_ip.set_port(Some(form.port)).ok(); // Trust
        // Checking if the access token is valid.
        let access_token = &form.access_token;
        let token_hash   = hashme!(access_token);

        // Creating a new service and inserting it into the hashmap.
        if app_data.access_tokens_hashes.contains(&token_hash) {
            let unique_identifier = uuid::Uuid::new_v4().to_string();

            let service = Service {
                            service_type        : form.service_type.clone(),
                            available           : false,
                            is_accepted         : false,
                            ip                  : retrieved_ip,
                            healthcheck_endpoint: form.healthcheck_endpoint.clone(),
                            identifier          : unique_identifier.clone(),
            };

            app_data.services.insert(unique_identifier.clone(), service.clone());

            // Removing the token from the list of tokens.
            if let Some(index) = app_data.access_tokens_hashes.iter().position(|hashed|
                                                                     *hashed == token_hash) {
                app_data.access_tokens_hashes.remove(index);
            }

            let cloned_app_data = data.clone();
            let cloned_service  = service.clone();
            actix_web::rt::spawn(async move{
                if let Ok(app_state) = cloned_app_data.lock(){
                    let subscribers = app_state.subscribers.clone();

                    let services = app_state.services
                                            .values()
                                            .map(|service|
                                                match subscribers.iter().find(|params| {
                                                                                params.identifier == service.identifier
                                                                                && params.on_registration
                                                }) {
                                                    // Clone for dropping mutex lock
                                                    Some(found) => Some((service.clone(), found.endpoint.clone())),
                                                    None => None,
                                                }
                                            ).flatten().collect::<Vec<_>>();
                    // drop mutex lock
                    drop(app_state);

                    for service in services{
                        service.0.notify(&cloned_service, service.1.clone()).await
                    }
                }
            });

            #[derive(serde::Serialize, serde::Deserialize)]
            struct Identifier{ identifier: String }

            return actix_web::HttpResponse::Ok().insert_header((actix_web::http::header::CONTENT_TYPE,
                                                            "application/json"))
                                                .json(Identifier{identifier: unique_identifier.clone()});
        }
        return actix_web::HttpResponse::Unauthorized().finish();
    }
    actix_web::HttpResponse::InternalServerError().finish()
}


#[actix_web::post("/subscribe")]
async fn subscribe(
    data: actix_web::web::Data<std::sync::Mutex<AppState>>,
    form: actix_web::web::Form<SubscribeForm>,
) -> actix_web::HttpResponse{
    if let Ok(mut app_data) = data.lock(){

        // This is checking if the login and password are correct.
        // If they are not then it returns an unauthorized response.
        if hashme!(form.login)    != app_data.admin_credentials.login_hash ||
           hashme!(form.password) != app_data.admin_credentials.password_hash {
            return actix_web::HttpResponse::Unauthorized().finish();
        }

        // Adding a new subscriber to the list of subscribers.
        match app_data.services.values().find(|service| service.identifier == form.identifier) {
            Some(_) => {
                app_data.subscribers.push(
                    SubscribeParams { identifier     : form.identifier.clone(),
                                      on_registration: form.on_registration,
                                      on_acceptance  : form.on_acceptance,
                                      endpoint       : form.endpoint.clone()}
                );
            }
            None => return actix_web::HttpResponse::NotFound().finish(),
        }
        return actix_web::HttpResponse::Ok().finish();
    }
    actix_web::HttpResponse::InternalServerError().finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();


    let config   = config::Config::init();
    let app_data = actix_web::web::Data::new(std::sync::Mutex::new(AppState {
        access_tokens_hashes : Vec::new(),
        services             : HashMap::new(),
        admin_credentials    : AdminCredentials {
                                        login_hash   : hashme!(&config.login),
                                        password_hash: hashme!(&config.password),
                             },
        subscribers          : vec![]
    }));

    let cloned_data   = app_data.clone();
    let cloned_config = config.clone();

    // A background task that is executed every `healthcheck_interval` seconds.
    // used to ping all services and update their availability.
    actix_web::rt::spawn(async move {
        let mut interval = actix_web::rt::time::interval(std::time::Duration::from_secs(
            cloned_config.healthcheck_interval,
        ));

        loop {
            interval.tick().await;

            // Cloning all the services that are accepted and dropping the lock on the data.
            if let Ok(data) = cloned_data.lock(){
                let mut services = data.services.iter()
                                                .filter(|service| service.1.is_accepted)
                                                .map(|s| (s.0.clone(), s.1.clone()))
                                                .collect::<Vec<_>>();

                drop(data);

                // So we are cloning all the services that are accepted and dropping the lock on the data.
                // Then we are pinging all the services and updating the availability.
                // Then we are locking the data again and updating the services.
                for service in services.iter_mut(){
                    service.1.ping().await
                }
                if let Ok(mut data) = cloned_data.lock(){
                    data.services = HashMap::from_iter(services);
                }
            }
        }
    });

    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .wrap(actix_web::middleware::Logger::default())
            .app_data(app_data.clone())
            .service(register_service)
            .service(find)
            .service(accept_service)
            .service(disable_service)
            .service(new_token)
            .service(subscribe)
    })
    .bind(format!("0.0.0.0:{}", &config.port))?
    .run()
    .await
}
