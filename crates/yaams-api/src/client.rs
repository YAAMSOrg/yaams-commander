use std::time::Duration;

use reqwest::{Method, RequestBuilder, header};
use serde::{Serialize, de::DeserializeOwned};

use crate::models::{
    Aircraft, AircraftQuery, AircraftStatus, Airline, Data, Flight, FlightQuery, InstanceInfo, NewPirep, OnlineNetwork,
    Page, User,
};
use crate::{ApiError, Result, normalize_base_url};

const TIMEOUT: Duration = Duration::from_secs(20);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// Upper bound when collecting all pages of a list, as a safety net.
const MAX_PAGES: u32 = 50;

/// Client for one YAAMS instance and API token.
///
/// Cheap to clone; clones share the connection pool.
#[derive(Debug, Clone)]
pub struct Client {
    http: reqwest::Client,
    base: url::Url,
    token: Option<String>,
}

impl Client {
    /// Creates a client. `base_url` is normalized with [`normalize_base_url`].
    /// TLS certificates are verified against the operating system trust store.
    pub fn new(base_url: &str, token: Option<String>) -> Result<Self> {
        let base = normalize_base_url(base_url)?;
        let mut headers = header::HeaderMap::new();
        headers.insert(header::ACCEPT, header::HeaderValue::from_static("application/json"));
        let http = reqwest::Client::builder()
            .user_agent(concat!("YAAMS-Commander/", env!("CARGO_PKG_VERSION")))
            .default_headers(headers)
            .timeout(TIMEOUT)
            .connect_timeout(CONNECT_TIMEOUT)
            .build()
            .map_err(ApiError::from_reqwest)?;
        Ok(Self {
            http,
            base,
            token: token.filter(|t| !t.trim().is_empty()).map(|t| t.trim().to_string()),
        })
    }

    /// The normalized instance base URL (with trailing slash).
    pub fn base_url(&self) -> &url::Url {
        &self.base
    }

    /// `GET /api/v1/info` - public, needs no token.
    pub async fn instance_info(&self) -> Result<InstanceInfo> {
        self.get_data("info", &()).await
    }

    /// `GET /api/v1/user` - the token owner with abilities and memberships.
    pub async fn current_user(&self) -> Result<User> {
        self.get_data("user", &()).await
    }

    /// `GET /api/v1/airlines/{airline}`.
    pub async fn airline(&self, airline_id: u64) -> Result<Airline> {
        self.get_data(&format!("airlines/{airline_id}"), &()).await
    }

    /// `GET /api/v1/airlines/{airline}/flights` - the token owner's own flights.
    pub async fn flights(&self, airline_id: u64, query: &FlightQuery) -> Result<Page<Flight>> {
        self.get(&format!("airlines/{airline_id}/flights"), &query.params())
            .await
    }

    /// `POST /api/v1/airlines/{airline}/flights` - files a PIREP.
    pub async fn file_pirep(&self, airline_id: u64, pirep: &NewPirep) -> Result<Flight> {
        let request = self
            .request(Method::POST, &format!("airlines/{airline_id}/flights"))?
            .json(pirep);
        let envelope: Data<Flight> = self.send(request).await?;
        Ok(envelope.data)
    }

    /// `GET /api/v1/airlines/{airline}/aircraft`.
    pub async fn aircraft(&self, airline_id: u64, query: &AircraftQuery) -> Result<Page<Aircraft>> {
        self.get(&format!("airlines/{airline_id}/aircraft"), query).await
    }

    /// All active aircraft of the airline, collected over every page.
    pub async fn active_aircraft(&self, airline_id: u64) -> Result<Vec<Aircraft>> {
        let mut query = AircraftQuery {
            status: Some(AircraftStatus::Active),
            page: Some(1),
            per_page: Some(100),
            ..AircraftQuery::default()
        };
        let mut all = Vec::new();
        loop {
            let page = self.aircraft(airline_id, &query).await?;
            let has_next = page.has_next();
            all.extend(page.data);
            let current = query.page.unwrap_or(1);
            if !has_next || current >= MAX_PAGES {
                break;
            }
            query.page = Some(current + 1);
        }
        Ok(all)
    }

    /// `GET /api/v1/online-networks` - unpaginated.
    pub async fn online_networks(&self) -> Result<Vec<OnlineNetwork>> {
        self.get_data("online-networks", &()).await
    }

    fn url(&self, path: &str) -> Result<url::Url> {
        self.base
            .join(&format!("api/v1/{path}"))
            .map_err(|e| ApiError::InvalidUrl(e.to_string()))
    }

    fn request(&self, method: Method, path: &str) -> Result<RequestBuilder> {
        let mut request = self.http.request(method, self.url(path)?);
        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }
        Ok(request)
    }

    async fn get<Q: Serialize + ?Sized, T: DeserializeOwned>(&self, path: &str, query: &Q) -> Result<T> {
        let request = self.request(Method::GET, path)?.query(query);
        self.send(request).await
    }

    async fn get_data<Q: Serialize + ?Sized, T: DeserializeOwned>(&self, path: &str, query: &Q) -> Result<T> {
        let envelope: Data<T> = self.get(path, query).await?;
        Ok(envelope.data)
    }

    async fn send<T: DeserializeOwned>(&self, request: RequestBuilder) -> Result<T> {
        let response = request.send().await.map_err(ApiError::from_reqwest)?;
        let status = response.status();
        let body = response.text().await.map_err(ApiError::from_reqwest)?;
        if !status.is_success() {
            return Err(ApiError::from_status(status.as_u16(), &body));
        }
        serde_json::from_str(&body).map_err(|e| {
            if body.trim_start().starts_with('<') {
                ApiError::Decode("received HTML instead of JSON - is this a YAAMS instance?".into())
            } else {
                ApiError::Decode(e.to_string())
            }
        })
    }
}
