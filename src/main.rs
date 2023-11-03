use std::env;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::str::FromStr;
use hyper::server::conn::AddrStream;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use askama::Template;


#[derive(Template, Clone, Debug)]
#[template(path = "index.html")]
struct IndexTemplate {
    router_id: String,
    address_v4: String,
    address_v6: String,
}

#[tokio::main]
async fn main() {
    let addr_string = env::var("LISTEN_ADDR").unwrap_or("".to_string());
    let addr = SocketAddr::from_str(&addr_string).unwrap_or(SocketAddr::from(([127, 0, 0, 1], 8080)));

    let make_svc = make_service_fn(move |_conn: &AddrStream| {
        // let addr = conn.remote_addr();
        async move {
            // let addr = addr.clone();
            Ok::<_, Infallible>(service_fn(move |_req : Request<Body>| {
                let router_id = env::var("ROUTER_ID").unwrap_or("rv128".to_string());
                let address_v4 = env::var("ADDRESS_V4").unwrap_or("43.228.174.128".to_string());
                let address_v6 = env::var("ADDRESS_V6").unwrap_or("2001:df3:14c0:1128::1".to_string());
                let index = IndexTemplate {
                    router_id,
                    address_v4,
                    address_v6,
                };
                let res = Response::builder()
                    .status(200)
                    .header("Content-Type", "text/html")
                    .body(Body::from(index.render().unwrap()))
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
