//! This example adds upstream CORS layers to the RPC service,
//! with access control allowing requests from all hosts.

use hyper::Method;
use jsonrpsee::server::{AllowHosts, RpcModule, ServerBuilder};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
        .expect("setting default subscriber failed");

    // Start up a JSON-RPC server that allows cross origin requests.
    let server_addr = run_server().await?;

    // Print instructions for testing CORS from a browser.
    println!("Run the following snippet in the developer console in any Website.");
    println!(
        r#"
        fetch("http://{}", {{
            method: 'POST',
            mode: 'cors',
            headers: {{ 'Content-Type': 'application/json' }},
            body: JSON.stringify({{
                jsonrpc: '2.0',
                method: 'say_hello',
                id: 1
            }})
        }}).then(res => {{
            console.log("Response:", res);
            return res.text()
        }}).then(body => {{
            console.log("Response Body:", body)
        }});
    "#,
        server_addr
    );

    futures::future::pending().await
}

async fn run_server() -> anyhow::Result<SocketAddr> {
    // Add a CORS middleware for handling HTTP requests.
    // This middleware does affect the response, including appropriate
    // headers to satisfy CORS. Because any origins are allowed, the
    // "Access-Control-Allow-Origin: *" header is appended to the response.
    let cors = CorsLayer::new()
        // Allow `POST` when accessing the resource
        .allow_methods([Method::POST])
        // Allow requests from any origin
        .allow_origin(Any)
        .allow_headers([hyper::header::CONTENT_TYPE]);
    let middleware = tower::ServiceBuilder::new().layer(cors);

    // The RPC exposes the access control for filtering and the middleware for
    // modifying requests / responses. These features are independent of one another
    // and can also be used separately.
    // In this example, we use both features.
    let server = ServerBuilder::new()
        .set_host_filtering(AllowHosts::Any)
        .set_middleware(middleware)
        .build("0.0.0.0:33035".parse::<SocketAddr>()?)
        .await?;

    let mut module = RpcModule::new(());
    module.register_method("say_hello", |_, _| {
        println!("say_hello method called!");
        "Hello there!!"
    })?;

    let addr = server.local_addr()?;
    let handle = server.start(module);

    // In this example we don't care about doing shutdown so let's it run forever.
    // You may use the `ServerHandle` to shut it down or manage it yourself.
    tokio::spawn(handle?.stopped());

    Ok(addr)
}
