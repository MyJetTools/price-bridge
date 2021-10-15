use std::{net::SocketAddr, sync::Arc};

use hyper::{Body, Method, Request, Response, Server, StatusCode, service::{make_service_fn, service_fn}};

use crate::Metrics;

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, GenericError>;
static NOTFOUND: &[u8] = b"Not Found";


async fn response(
    req: Request<Body>,
    metrics: Arc<Metrics>
) -> Result<Response<Body>> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/metrics") => Ok(Response::new(Body::from(metrics.get_data()))),
        _ => {
            // Return 404 not found response.
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(NOTFOUND.into())
                .unwrap())
        }
    }
}

pub async fn start(addr: SocketAddr, metrics: Arc<Metrics>) {
    let new_service = make_service_fn(move |_| {
        let metrics = metrics.clone();
        async {

            Ok::<_, GenericError>(service_fn(move |req| {
                response(req, metrics.to_owned())
            }))
        }
    });

    let server = Server::bind(&addr).serve(new_service);

    println!("Listening on http://{}", addr);

    server.await.unwrap();
}
