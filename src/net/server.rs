use rouille::{Server, Request, Response};

use super::handlers;
use errors::ServerError;

// route incoming requests to matching handler
fn route(req: &Request) -> Result<Response, ServerError> {
    router!(req,
        (GET) (/) => { handlers::get_index(req) },
        (POST) (/transaction) => { handlers::post_transaction(req) },
        (POST) (/block) => { handlers::post_block(req) },
        (GET) (/local/wallet/new) => { handlers::local::get_new_wallet(req) },
        (GET) (/local/wallet/{address}) => { handlers::local::get_wallet(req, address) },
        (POST) (/local/transaction) => { handlers::local::post_transaction(req) },
        _ => Err(ServerError::NotFound)
    )
}

// handle incoming requests
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
    });

    match server {
        Ok(s) => s.run(),
        Err(e) => panic!("Can't start the HTTP server: {}", e), // TODO proper error handling
    }
}
