use actix_web::{get, http::header::ContentType, HttpResponse, Responder};


#[get("/info")]
pub async fn info() -> impl Responder {
    let banner = "
     __  __     ______     __     ______     ______     ______     ______    
    /\\ \\_\\ \\   /\\  __ \\   /\\ \\   /\\  ___\\   /\\  __ \\   /\\  == \\   /\\  ___\\   
    \\ \\  __ \\  \\ \\  __ \\  \\ \\ \\  \\ \\ \\____  \\ \\ \\/\\ \\  \\ \\  __<   \\ \\  __\\   
     \\ \\_\\ \\_\\  \\ \\_\\ \\_\\  \\ \\_\\  \\ \\_____\\  \\ \\_____\\  \\ \\_\\ \\_\\  \\ \\_____\\ 
      \\/_/\\/_/   \\/_/\\/_/   \\/_/   \\/_____/   \\/_____/   \\/_/ /_/   \\/_____/ 
                                                                             \n
    H[AI]CORE : FSMED ProtocolDB (Backend) v0.0.1
    Author: Tobias Rempe <tobias.rempe@rub.de>
    Current Maintainer: Tobias Rempe <tobias.rempe@rub.de>";
    

    HttpResponse::Ok().body(banner)
}

#[get("/")]
pub async fn home() -> impl Responder {
    HttpResponse::Ok().content_type(ContentType::html()).body("
        <html>
            <h1>Fachschaft Medizin</h1>
            <h2>Protokolldatenbank v0.0.1</h2>
            <p>Willkommen auf dem Backend der ProtokollDB. Wenn du etwas mit dieser API entwickeln willst, laden wir dich ein <a href = \"https://docs.fsi.rub.de/s/fsmed-protokolldb-docs\">hier</a> vorbeizuschauen.</p>
            <p>Wenn du auf der Suche nach der eigentlichen Website bist, dann klicke bitte <a href = \"https://leckere.aprikosenmarmela.de\">hier</a>.</p>
            <p><a href = \"/login\">Hier</a> kannst du dich alternativ auch direkt einloggen.</p>
        </html>
    ")
}



#[get("/invalidauth")]
pub async fn invalid_auth() -> impl Responder {
    HttpResponse::Ok().content_type(ContentType::html()).body("<html><h1>Authentication isn't configured correctly. Please contact your respective Server-Admin</h1></html>'")
}
