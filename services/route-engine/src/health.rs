use std::time::{Duration, Instant};

use serde::Deserialize;
use tonic::{Request, Response, Status};
use url::Url;

use crate::proto::{
    route_optimizer_server::RouteOptimizer, CheckRequest, CheckResponse, DependencyStatus,
    ServingStatus,
};

const SERVICE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone)]
pub struct HealthService {
    client: reqwest::Client,
    graphhopper_url: Url,
}

#[derive(Debug, Deserialize)]
struct GraphHopperInfo {
    version: String,
    #[serde(default)]
    import_date: String,
    data_date: String,
    #[serde(default)]
    profiles: Vec<serde_json::Value>,
    #[serde(default)]
    supported_vehicles: Vec<String>,
}

impl HealthService {
    pub fn new(graphhopper_url: Url, timeout: Duration) -> Result<Self, reqwest::Error> {
        let client = reqwest::Client::builder().timeout(timeout).build()?;
        Ok(Self {
            client,
            graphhopper_url,
        })
    }

    async fn check_graphhopper(&self) -> DependencyStatus {
        let started = Instant::now();
        let result = self.fetch_graphhopper_info().await;
        let latency_ms = started.elapsed().as_millis().min(i64::MAX as u128) as i64;
        match result {
            Ok(info) => DependencyStatus {
                name: "graphhopper".to_owned(),
                status: ServingStatus::Up.into(),
                version: info.version,
                dataset_version: dataset_version(&info.data_date, &info.import_date),
                error: String::new(),
                latency_ms,
            },
            Err(error) => DependencyStatus {
                name: "graphhopper".to_owned(),
                status: ServingStatus::Down.into(),
                version: String::new(),
                dataset_version: String::new(),
                error,
                latency_ms,
            },
        }
    }

    async fn fetch_graphhopper_info(&self) -> Result<GraphHopperInfo, String> {
        let endpoint = self
            .graphhopper_url
            .join("info")
            .map_err(|error| format!("invalid GraphHopper info URL: {error}"))?;
        let response = self
            .client
            .get(endpoint)
            .send()
            .await
            .map_err(|error| format!("GraphHopper info request failed: {error}"))?;
        if !response.status().is_success() {
            return Err(format!(
                "GraphHopper info endpoint returned {}",
                response.status()
            ));
        }
        let info: GraphHopperInfo = response
            .json()
            .await
            .map_err(|error| format!("invalid GraphHopper info response: {error}"))?;
        if info.version.is_empty() {
            return Err("GraphHopper version is empty".to_owned());
        }
        if info.data_date.is_empty() {
            return Err("GraphHopper data date is empty".to_owned());
        }
        if info.profiles.is_empty() && info.supported_vehicles.is_empty() {
            return Err("GraphHopper has no routing profiles".to_owned());
        }
        Ok(info)
    }
}

#[tonic::async_trait]
impl RouteOptimizer for HealthService {
    async fn check(
        &self,
        _request: Request<CheckRequest>,
    ) -> Result<Response<CheckResponse>, Status> {
        let graphhopper = self.check_graphhopper().await;
        let status = if graphhopper.status == i32::from(ServingStatus::Up) {
            ServingStatus::Up
        } else {
            ServingStatus::Down
        };
        Ok(Response::new(CheckResponse {
            status: status.into(),
            service_version: SERVICE_VERSION.to_owned(),
            dependencies: vec![graphhopper],
        }))
    }
}

fn dataset_version(data_date: &str, import_date: &str) -> String {
    if import_date.is_empty() {
        data_date.to_owned()
    } else {
        format!("{data_date}/{import_date}")
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };
    use tonic::Request;
    use url::Url;

    use super::{dataset_version, HealthService, RouteOptimizer, ServingStatus};
    use crate::proto::CheckRequest;

    #[test]
    fn dataset_version_includes_source_and_import_dates() {
        assert_eq!(
            dataset_version("2026-08-30", "2026-08-31"),
            "2026-08-30/2026-08-31"
        );
    }

    #[tokio::test]
    async fn check_reports_live_graphhopper_metadata() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = [0_u8; 1024];
            let read = stream.read(&mut request).await.unwrap();
            assert!(String::from_utf8_lossy(&request[..read]).starts_with("GET /info "));
            let body = r#"{"version":"10.2","import_date":"2026-08-31","data_date":"2026-08-30","profiles":[{"name":"foot"}]}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).await.unwrap();
        });

        let service = HealthService::new(
            Url::parse(&format!("http://{address}/")).unwrap(),
            Duration::from_secs(1),
        )
        .unwrap();
        let response = service
            .check(Request::new(CheckRequest {}))
            .await
            .unwrap()
            .into_inner();

        assert_eq!(response.status, ServingStatus::Up as i32);
        assert_eq!(response.dependencies.len(), 1);
        assert_eq!(
            response.dependencies[0].dataset_version,
            "2026-08-30/2026-08-31"
        );
    }
}
