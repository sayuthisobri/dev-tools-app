#[allow(dead_code, unused_imports)]
// use crate::cookies;
use crate::errors::APIError;
use anyhow::{Context, Result};
// use log::trace;
use once_cell::sync::OnceCell;
// use base64::{engine::general_purpose, Engine as _};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Body, Client, Method};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use std::vec;
use tracing::debug;
use tracing_subscriber::Layer;
use url::Url;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HTTPRequestKVParam {
    pub key: String,
    pub value: String,
    pub enabled: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HTTPRequest {
    pub method: String,
    pub url: String,
    pub body: String,
    pub content_type: String,
    pub headers: Vec<HTTPRequestKVParam>,
    pub query: Vec<HTTPRequestKVParam>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequestTimeout {
    pub connect: u64,
    pub write: u64,
    pub read: u64,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct HTTPStats {
    pub remote_addr: String,
    pub is_https: bool,
    pub cipher: String,
    pub dns_lookup: u32,
    pub tcp: u32,
    pub tls: u32,
    pub send: u32,
    pub server_processing: u32,
    pub content_transfer: u32,
    pub total: u32,
}

impl HTTPStats {
    fn new() -> Self {
        HTTPStats {
            ..Default::default()
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HTTPResponse {
    pub url: String,
    // pub req: HTTPRequest,
    pub latency: u32,
    pub status: u16,
    pub headers: HashMap<String, Vec<String>>,
    pub body: String,
    pub stats: HTTPStats,
    pub length: u64,
}

struct JsonVisitor<'a>(&'a mut BTreeMap<String, String>);

impl<'a> tracing::field::Visit for JsonVisitor<'a> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.0
            .insert(field.name().to_string(), format!("{:?}", value));
    }
}

pub async fn request(
    http_request: HTTPRequest,
    timeout: Option<RequestTimeout>,
) -> Result<HTTPResponse, APIError> {
    let trace = get_http_trace();
    trace.reset();
    let mut client_builder = Client::builder();
    // let original_req = http_request.clone();
    // .http_stats(HTTPStats::default())

    let mut current_url: Url = Url::parse(http_request.url.as_str())?;
    for q in http_request.query {
        if !q.enabled {
            continue;
        }
        current_url.query_pairs_mut().append_pair(&q.key, &q.value);
    }

    if current_url.scheme() == "https" {
        trace.tls();
        client_builder = client_builder.use_rustls_tls().tls_info(true);
    }
    let client = client_builder.build().context("build client")?;
    // let mut req = Request::new(http_request.get_method(), http_request.uri.parse()?);
    let method = match http_request.method.to_uppercase().as_str() {
        "POST" => Method::POST,
        "PUT" => Method::PUT,
        "DELETE" => Method::DELETE,
        "HEAD" => Method::HEAD,
        "OPTIONS" => Method::OPTIONS,
        "CONNECT" => Method::CONNECT,
        "PATCH" => Method::PATCH,
        "TRACE" => Method::TRACE,
        _ => Method::GET,
    };

    let connect_timeout = Duration::from_secs(timeout.map_or(10, |t| t.connect));
    // let write_timeout = Duration::from_secs(timeout.write);
    // let read_timeout = Duration::from_secs(timeout.read);
    let mut req_headers = HeaderMap::new();
    // let mut set_content_type = false;
    for h in http_request.headers {
        if !h.enabled {
            continue;
        }
        // if h.key.to_lowercase() == "content-type" {
        //     set_content_type = true;
        // }
        req_headers.insert(
            h.key.parse::<HeaderName>()?,
            HeaderValue::from_str(h.value.as_str())?,
        );
    }
    req_headers.insert("Accept-Encoding", HeaderValue::from_str("gzip, br")?);

    let mut request_builder = client
        .request(method, current_url)
        .headers(req_headers)
        .timeout(connect_timeout);

    request_builder = if !http_request.body.is_empty() {
        let body = if http_request.content_type.starts_with("multipart/form-data") {
            // let buf = general_purpose::STANDARD.decode(http_request.body.as_bytes())?;
            // Body::from(buf)
            Body::from(http_request.body)
        } else {
            Body::from(http_request.body)
        };
        request_builder.body(body)
    } else {
        request_builder
    };
    let res = request_builder.send().await?;

    // let content_encoding_key = "content-encoding";
    let mut headers = HashMap::new();
    let mut set_cookies = Vec::new();

    for (name, value) in res.headers() {
        let mut key = name.to_string();
        key = key.to_lowercase();

        let value = value.to_str()?.to_string();
        if key == "set-cookie" {
            set_cookies.push(value.clone());
        }
        let values: Option<&Vec<String>> = headers.get(&key);
        match values {
            Some(values) => {
                let mut values = values.to_vec();
                values.push(value);
                headers.insert(key, values);
            }
            None => {
                headers.insert(key, vec![value]);
            }
        }
    }
    debug!("response: {:#?}", res);
    let url = res.url().to_string();
    let length = res.content_length().unwrap_or(0);
    let status = res.status().as_u16();
    let remote_addr: String = res.remote_addr().map_or("".into(), |a| a.to_string());
    let body = res.text().await?;
    trace.done();
    let mut stats: HTTPStats = trace.into();
    stats.remote_addr = remote_addr;

    let response = HTTPResponse {
        url,
        // req: original_req,
        length,
        latency: stats.total,
        status,
        headers,
        body,
        stats,
    };
    Ok(response)
}

impl From<&HTTPTrace> for HTTPStats {
    fn from(trace: &HTTPTrace) -> Self {
        let mut stats = HTTPStats::new();
        stats.is_https = trace.is_tls();
        stats.cipher = trace.get_cipher();
        stats.dns_lookup = trace.dns_consuming();
        stats.tcp = trace.tcp_consuming();
        stats.tls = trace.tls_consuming();
        stats.server_processing = trace.server_processing_consuming();
        stats.send = trace.send_consuming();
        stats.content_transfer = trace.content_transfer_consuming();
        stats.total = trace.consuming();
        stats
    }
}

#[derive(Default, Debug)]
struct HTTPTrace {
    is_tls_value: AtomicBool,
    cipher_value: Mutex<String>,
    start_value: AtomicU64,
    get_conn_value: AtomicU64,
    dns_start_value: AtomicU64,
    dns_done_value: AtomicU64,
    tcp_start_value: AtomicU64,
    tcp_done_value: AtomicU64,
    tls_start_value: AtomicU64,
    tls_done_value: AtomicU64,
    http_start_value: AtomicU64,
    written_value: AtomicU64,
    got_first_response_byte_value: AtomicU64,
    done_value: AtomicU64,
}

impl HTTPTrace {
    fn now(&self) -> u64 {
        chrono::Utc::now().timestamp_millis() as u64
    }
    fn new() -> Self {
        HTTPTrace {
            ..Default::default()
        }
    }
    fn reset(&self) {
        self.set_cipher("".to_string());
        self.is_tls_value.store(false, Ordering::Relaxed);
        self.start_value.store(0, Ordering::Relaxed);
        self.get_conn_value.store(0, Ordering::Relaxed);
        self.dns_start_value.store(0, Ordering::Relaxed);
        self.dns_done_value.store(0, Ordering::Relaxed);
        self.tcp_start_value.store(0, Ordering::Relaxed);
        self.tcp_done_value.store(0, Ordering::Relaxed);
        self.tls_start_value.store(0, Ordering::Relaxed);
        self.tls_done_value.store(0, Ordering::Relaxed);
        self.http_start_value.store(0, Ordering::Relaxed);
        self.written_value.store(0, Ordering::Relaxed);
        self.got_first_response_byte_value
            .store(0, Ordering::Relaxed);
        self.done_value.store(0, Ordering::Relaxed);
    }
    fn set_cipher(&self, value: String) {
        if let Ok(mut cipher) = self.cipher_value.lock() {
            *cipher = value;
        }
    }
    fn get_cipher(&self) -> String {
        if let Ok(cipher) = self.cipher_value.lock() {
            return cipher.to_string();
        }
        "".to_string()
    }
    fn is_tls(&self) -> bool {
        self.is_tls_value.load(Ordering::Relaxed)
    }
    fn tls(&self) {
        self.is_tls_value.store(true, Ordering::Relaxed);
    }
    fn get_conn_from_pool(&self) {
        self.start_value.store(self.now(), Ordering::Relaxed)
    }
    fn get_conn(&self) {
        self.get_conn_value.store(self.now(), Ordering::Relaxed)
    }
    fn dns_start(&self) {
        self.dns_start_value.store(self.now(), Ordering::Relaxed)
    }
    fn dns_done(&self) {
        self.dns_done_value.store(self.now(), Ordering::Relaxed);
    }
    fn tcp_start(&self) {
        self.tcp_start_value.store(self.now(), Ordering::Relaxed);
    }
    fn tcp_done(&self) {
        self.tcp_done_value.store(self.now(), Ordering::Relaxed);
    }
    fn tls_start(&self) {
        self.tls_start_value.store(self.now(), Ordering::Relaxed);
    }
    fn tls_done(&self) {
        self.tls_done_value.store(self.now(), Ordering::Relaxed);
    }
    fn http_start(&self) {
        self.http_start_value.store(self.now(), Ordering::Relaxed);
    }

    fn got_first_response_byte(&self) {
        if self.got_first_response_byte_value.load(Ordering::Relaxed) == 0 {
            self.got_first_response_byte_value
                .store(self.now(), Ordering::Relaxed);
        }
    }
    fn written(&self) {
        if self.written_value.load(Ordering::Relaxed) == 0 {
            self.written_value.store(self.now(), Ordering::Relaxed);
        }
    }
    fn done(&self) {
        self.done_value.store(self.now(), Ordering::Relaxed);
    }
    fn send_consuming(&self) -> u32 {
        let http_start_value = self.http_start_value.load(Ordering::Relaxed);
        let written_value = self.written_value.load(Ordering::Relaxed);
        if http_start_value == 0 || written_value == 0 {
            return 0;
        }
        (written_value - http_start_value) as u32
    }
    fn dns_consuming(&self) -> u32 {
        let dns_start_value = self.dns_start_value.load(Ordering::Relaxed);
        let dns_done_value = self.dns_done_value.load(Ordering::Relaxed);
        if dns_start_value == 0 || dns_done_value == 0 {
            return 0;
        }
        (dns_done_value - dns_start_value) as u32
    }
    fn tcp_consuming(&self) -> u32 {
        let tcp_start_value = self.tcp_start_value.load(Ordering::Relaxed);
        let tcp_done_value = self.tcp_done_value.load(Ordering::Relaxed);
        if tcp_start_value == 0 || tcp_done_value == 0 {
            return 0;
        }
        (tcp_done_value - tcp_start_value) as u32
    }
    fn tls_consuming(&self) -> u32 {
        let tls_start_value = self.tls_start_value.load(Ordering::Relaxed);
        let tls_done_value = self.tls_done_value.load(Ordering::Relaxed);
        if tls_start_value == 0 || tls_done_value == 0 {
            return 0;
        }
        (tls_done_value - tls_start_value) as u32
    }

    fn server_processing_consuming(&self) -> u32 {
        let written_value = self.written_value.load(Ordering::Relaxed);
        let got_first_response_byte_value =
            self.got_first_response_byte_value.load(Ordering::Relaxed);
        if written_value == 0 || got_first_response_byte_value == 0 {
            return 0;
        }

        (got_first_response_byte_value - written_value) as u32
    }
    fn content_transfer_consuming(&self) -> u32 {
        let got_first_response_byte_value =
            self.got_first_response_byte_value.load(Ordering::Relaxed);
        let done_value = self.done_value.load(Ordering::Relaxed);
        if got_first_response_byte_value == 0 || done_value == 0 {
            return 0;
        }
        (done_value - got_first_response_byte_value) as u32
    }
    fn consuming(&self) -> u32 {
        let start_value = self.start_value.load(Ordering::Relaxed);
        let done_value = self.done_value.load(Ordering::Relaxed);
        if start_value == 0 || done_value == 0 {
            return 0;
        }
        (done_value - start_value) as u32
    }
}
static HTTP_TRACE: OnceCell<HTTPTrace> = OnceCell::new();

fn get_http_trace() -> &'static HTTPTrace {
    HTTP_TRACE.get_or_init(HTTPTrace::new)
}
pub struct HTTPTraceLayer;
impl<S> Layer<S> for HTTPTraceLayer
where
    S: tracing::Subscriber,
    // Scary! But there's no need to even understand it. We just need it.
    S: for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
{
    fn on_event(&self, event: &tracing::Event<'_>, _: tracing_subscriber::layer::Context<'_, S>) {
        let trace = get_http_trace();
        // dbg!(event.metadata().target());
        let target = event.metadata().target();
        // if !target.starts_with("hyper") && !trace.is_tls() {
        //     return;
        // }

        let mut fields = BTreeMap::new();
        let mut visitor = JsonVisitor(&mut fields);

        event.record(&mut visitor);
        let message = fields.get("message");
        if message.is_none() {
            return;
        }
        let message = message.unwrap();
        if trace.is_tls() {
            if message.starts_with("Sending ClientHello Message") {
                trace.tls_start();
            } else if message.starts_with("Server cert is") {
                trace.tls_done();
            } else if message.starts_with("Using ciphersuite ") {
                let cipher = message.replace("Using ciphersuite ", "");
                trace.set_cipher(cipher);
                // let p = AtomicPtr::new(&mut cipher);
                // trace.cipher.store(&mut cipher, Ordering::Relaxed);
            }
        }
        if target.contains("pool") {
            if message.contains("checkout waiting for idle connection") {
                trace.get_conn_from_pool();
            }
        } else if target.contains("::connect::http") {
            if message.starts_with("Http::connect;") {
                trace.get_conn();
            } else if message.starts_with("connecting to") {
                trace.dns_done();
                trace.tcp_start();
            } else if message.starts_with("connected to") {
                trace.tcp_done();
                trace.http_start();
            }
        } else if target.starts_with("hickory_") {
            if message.starts_with("querying: ") {
                trace.dns_start();
            }
        } else if target.contains("::framed_write") {
            if message.contains("send") {
                trace.written();
            }
        } else if target.contains("::framed_read") {
            if message.contains("received") {
                trace.got_first_response_byte();
            }
        } else if target.contains("::client") {
            if message.contains("handshake complete") {
                trace.written();
            }
        }
    }
}
