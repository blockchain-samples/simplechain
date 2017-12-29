use rouille::{Server, Request, Response};

use super::handlers;
use errors::ServerError;

// route incoming request to matching handler
fn route(req: &Request) -> Result<Response, ServerError> {
    router!(req,
        (GET) (/) => { handlers::get_index(req) },
        (POST) (/transaction) => { handlers::post_transaction(req) },
        (POST) (/block) => { handlers::post_block(req) },
        (GET) (/local/wallet) => { handlers::local::get_wallet(req) },
        (POST) (/local/transaction) => { handlers::local::post_transaction(req) },
        _ => Err(ServerError::NotFound) // Err(NotFound)
    )
}

// handle incoming request
fn handle(req: &Request) -> Response {
    println!("[+] {} {}", req.method(), req.raw_url());

    match route(req) {
        Ok(res) => res,
        Err(e) => {
            match e {
                ServerError::NotFound => {
                    Response::empty_404()
                },
                ServerError::InvalidTransaction => {
                    Response::empty_400()
                },
                ServerError::InvalidBlock => {
                    Response::empty_400()
                },
                _ => {
                    println!("error: {:?}", e);
                    Response::text("error")
                }
            }
        }
    }
}

// start the http server
pub fn start() {
    println!("STARTING NODE...");

    let server = Server::new("10.0.0.1:8000", |req| {
        handle(&req)
    }).unwrap();

    server.run();
}
