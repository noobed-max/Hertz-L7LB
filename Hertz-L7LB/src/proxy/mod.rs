use http_body_util::{BodyExt, Empty};
use http_body_util::combinators::BoxBody;
use hyper::body::Incoming;
use hyper::{Request, Response, StatusCode};
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use bytes::Bytes;
use tracing::{error, info, instrument};

// Type alias for the Client
type HttpClient = Client<hyper_util::client::legacy::connect::HttpConnector, Incoming>;

#[derive(Clone)]
pub struct ProxyService {
    client: HttpClient,
    backend_url: String,
}

impl ProxyService {
    pub fn new() -> Self {
        let client = Client::builder(TokioExecutor::new()).build_http();
        
        Self {
            client,
            backend_url: "http://127.0.0.1:8080".to_string(),
        }
    }

    #[instrument(skip(self, req), fields(method = %req.method(), uri = %req.uri()))]
    pub async fn handle_request(&self, mut req: Request<Incoming>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
        let start = std::time::Instant::now();

        // 1. Rewrite the URI
        let uri_string = format!(
            "{}{}",
            self.backend_url,
            req.uri().path_and_query().map(|x| x.as_str()).unwrap_or("/")
        );
        
        let uri = uri_string.parse::<hyper::Uri>().expect("Invalid backend URI");
        *req.uri_mut() = uri;

        // 2. Remove headers
        req.headers_mut().remove(hyper::header::HOST);

        // 3. Forward request
        match self.client.request(req).await {
            Ok(res) => {
                let duration = start.elapsed();
                info!(status = %res.status(), latency_us = duration.as_micros(), "Request proxied");
                
                // FIX: Box the incoming body so it matches the return type
                Ok(res.map(|b| b.boxed()))
            }
            Err(e) => {
                error!(error = %e, "Upstream connection failed");
                
                // FIX: Return a boxed Empty body that matches <Bytes, hyper::Error>
                Ok(Response::builder()
                    .status(StatusCode::BAD_GATEWAY)
                    .body(
                        Empty::<Bytes>::new()
                            .map_err(|e| -> hyper::Error { match e {} })
                            .boxed()
                    )
                    .unwrap())
            }
        }
    }
}