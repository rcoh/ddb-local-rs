use aws_config::BehaviorVersion;
use aws_smithy_runtime_api::client::http::{
    HttpClient, HttpConnector, HttpConnectorSettings, SharedHttpClient, SharedHttpConnector,
};
use aws_smithy_runtime_api::client::orchestrator::{HttpRequest, HttpResponse};
use aws_smithy_runtime_api::client::runtime_components::RuntimeComponents;
use aws_smithy_types::body::SdkBody;
use dynamodb_local_server_sdk::server::body::BoxBody;
use dynamodb_local_server_sdk::{error, input, output};
use http::Uri;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower::Service;
use tower::util::BoxCloneService;

pub mod backend;

type DdbService = BoxCloneService<hyper::Request<SdkBody>, hyper::Response<BoxBody>, Infallible>;

#[derive(Clone)]
struct InMemoryHttpClient {
    // the service is not Sync for reasons I don't know.
    // But _this_ needs to be sync for it to actually work.
    service: Arc<Mutex<DdbService>>,
}

impl std::fmt::Debug for InMemoryHttpClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InMemoryHttpClient").finish()
    }
}

impl InMemoryHttpClient {
    fn new(service: DdbService) -> Self {
        Self {
            service: Arc::new(Mutex::new(service)),
        }
    }
}

impl HttpConnector for InMemoryHttpClient {
    fn call(
        &self,
        request: HttpRequest,
    ) -> aws_smithy_runtime_api::client::http::HttpConnectorFuture {
        let service = self.service.clone();
        let fut = async move {
            // Convert HttpRequest to hyper::Request
            let mut hyper_req = request.try_into_http02x().unwrap();
            // not sure why needed, but smithy rejects otherwise
            *hyper_req.uri_mut() = Uri::from_static("/");

            // Call the service
            let mut svc = service.lock().await;
            let response = svc.call(hyper_req).await.unwrap();

            // Convert hyper::Response to HttpResponse
            let (parts, body) = response.into_parts();
            let body_bytes = hyper::body::to_bytes(body).await.unwrap();

            let http_response = HttpResponse::new(
                parts.status.into(),
                aws_smithy_types::body::SdkBody::from(body_bytes.to_vec()),
            );

            Ok(http_response)
        };

        aws_smithy_runtime_api::client::http::HttpConnectorFuture::new(Box::pin(fut))
    }
}

impl HttpClient for InMemoryHttpClient {
    fn http_connector(
        &self,
        _settings: &HttpConnectorSettings,
        _components: &RuntimeComponents,
    ) -> SharedHttpConnector {
        SharedHttpConnector::new(Self {
            service: self.service.clone(),
        })
    }
}

/// Trait for DynamoDB backend implementations
#[async_trait::async_trait]
pub trait DynamoDb: Send + Sync {
    async fn get_item(
        &self,
        input: input::GetItemInput,
    ) -> Result<output::GetItemOutput, error::GetItemError>;

    async fn put_item(
        &self,
        input: input::PutItemInput,
    ) -> Result<output::PutItemOutput, error::PutItemError>;

    async fn create_table(
        &self,
        input: input::CreateTableInput,
    ) -> Result<output::CreateTableOutput, error::CreateTableError>;

    async fn update_item(
        &self,
        input: input::UpdateItemInput,
    ) -> Result<output::UpdateItemOutput, error::UpdateItemError>;
}

macro_rules! build_service {
    ($backend:expr) => {{
        use dynamodb_local_server_sdk::server::{
            instrumentation::InstrumentExt,
            plugin::{HttpPlugins, ModelPlugins},
        };
        use dynamodb_local_server_sdk::{DynamoDb20120810, DynamoDb20120810Config};

        let http_plugins = HttpPlugins::new().instrument();
        let model_plugins = ModelPlugins::new();

        let config = DynamoDb20120810Config::builder()
            .http_plugin(http_plugins)
            .model_plugin(model_plugins)
            .build();

        let get_backend = $backend.clone();
        let put_backend = $backend.clone();
        let create_table_backend = $backend.clone();
        let update_backend = $backend.clone();

        DynamoDb20120810::builder(config)
            .get_item(move |input| {
                let backend = get_backend.clone();
                async move { backend.get_item(input).await }
            })
            .put_item(move |input| {
                let backend = put_backend.clone();
                async move { backend.put_item(input).await }
            })
            .create_table(move |input| {
                let backend = create_table_backend.clone();
                async move { backend.create_table(input).await }
            })
            .update_item(move |input| {
                let backend = update_backend.clone();
                async move { backend.update_item(input).await }
            })
            .build()
            .expect("failed to build DynamoDB service")
    }};
}

/// Builder for DynamoDB local server
pub struct DynamoDbLocalBuilder {
    backend: Arc<dyn DynamoDb>,
}

impl DynamoDbLocalBuilder {
    /// Create a new builder with the default in-memory backend
    pub fn new() -> Self {
        Self {
            backend: Arc::new(backend::InMemoryDynamoDb::new()),
        }
    }

    /// Use a custom backend implementation
    pub fn with_backend(mut self, backend: impl DynamoDb + 'static) -> Self {
        self.backend = Arc::new(backend);
        self
    }

    /// Bind to an automatically assigned port
    pub async fn bind(self) -> std::io::Result<BoundDynamoDbLocal> {
        use tokio::net::TcpListener;

        let app = build_service!(self.backend);
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;

        tokio::spawn(async move {
            let make_service = app.into_make_service_with_connect_info::<std::net::SocketAddr>();
            let server = hyper::Server::from_tcp(listener.into_std().unwrap())
                .unwrap()
                .serve(make_service);
            server.await.unwrap();
        });

        Ok(BoundDynamoDbLocal {
            addr,
            backend: self.backend,
        })
    }

    /// Bind to a specific address and start the server
    pub async fn bind_to_address(
        self,
        addr: impl Into<std::net::SocketAddr>,
    ) -> std::io::Result<BoundDynamoDbLocal> {
        use tokio::net::TcpListener;

        let app = build_service!(self.backend);
        let listener = TcpListener::bind(addr.into()).await?;
        let addr = listener.local_addr()?;

        tokio::spawn(async move {
            let make_service = app.into_make_service_with_connect_info::<std::net::SocketAddr>();
            let server = hyper::Server::from_tcp(listener.into_std().unwrap())
                .unwrap()
                .serve(make_service);
            server.await.unwrap();
        });

        Ok(BoundDynamoDbLocal {
            addr,
            backend: self.backend,
        })
    }

    /// Create an in-memory transport (no network)
    pub fn as_http_client(self) -> InMemoryDynamoDbLocal {
        let app = build_service!(self.backend);
        let boxed = DdbService::new(app);
        let http_client = InMemoryHttpClient::new(boxed);

        InMemoryDynamoDbLocal {
            http_client,
            backend: self.backend,
        }
    }
}

impl Default for DynamoDbLocalBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// DynamoDB local bound to a network address
pub struct BoundDynamoDbLocal {
    addr: std::net::SocketAddr,
    backend: Arc<dyn DynamoDb>,
}

impl BoundDynamoDbLocal {
    /// Get the address the server is bound to
    pub fn addr(&self) -> std::net::SocketAddr {
        self.addr
    }

    /// Get the endpoint URL for this server
    pub fn endpoint_url(&self) -> String {
        format!("http://{}", self.addr)
    }

    /// Get a reference to the backend
    pub fn backend(&self) -> &dyn DynamoDb {
        &*self.backend
    }

    /// Create a pre-configured AWS SDK client pointing to this server
    pub async fn client(&self) -> aws_sdk_dynamodb::Client {
        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .endpoint_url(self.endpoint_url())
            .region(aws_config::Region::new("us-east-1"))
            .load()
            .await;
        aws_sdk_dynamodb::Client::new(&config)
    }
}

/// DynamoDB local using in-memory transport (no network)
pub struct InMemoryDynamoDbLocal {
    http_client: InMemoryHttpClient,
    backend: Arc<dyn DynamoDb>,
}

impl InMemoryDynamoDbLocal {
    /// Get the HttpClient for manual configuration
    pub fn http_client(&self) -> SharedHttpClient {
        SharedHttpClient::new(self.http_client.clone())
    }

    /// Get a reference to the backend
    pub fn backend(&self) -> &dyn DynamoDb {
        &*self.backend
    }

    /// Create a pre-configured AWS SDK client using the in-memory transport
    pub async fn client(&self) -> aws_sdk_dynamodb::Client {
        let config = aws_sdk_dynamodb::Config::builder()
            .http_client(SharedHttpClient::new(self.http_client.clone()))
            .with_test_defaults_v2()
            .behavior_version(BehaviorVersion::latest())
            .build();
        aws_sdk_dynamodb::Client::from_conf(config)
    }
}

/// Entry point for creating a DynamoDB local instance
pub struct DynamoDbLocal;

impl DynamoDbLocal {
    /// Create a new builder
    pub fn builder() -> DynamoDbLocalBuilder {
        DynamoDbLocalBuilder::new()
    }
}
