use std::env;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::str::FromStr;
use hyper::server::conn::AddrStream;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};


#[tokio::main]
async fn main() {
    let addr_string = env::var("LISTEN_ADDR").unwrap_or("".to_string());
    let addr = SocketAddr::from_str(&addr_string).unwrap_or(SocketAddr::from(([127, 0, 0, 1], 8080)));

    let make_svc = make_service_fn(move |_conn: &AddrStream| {
        // let addr = conn.remote_addr();
        async move {
            // let addr = addr.clone();
            Ok::<_, Infallible>(service_fn(move |_req : Request<Body>| {
                let file = include_str!("../index.html");
                let res = Response::builder()
                    .status(200)
                    .header("Content-Type", "text/html")
                    .body(Body::from(file))
                    .unwrap();
                async move {
                    Ok::<_, Infallible>(res)
                }
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
