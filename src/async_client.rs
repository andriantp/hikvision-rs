use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::{Client as HttpClient, Method, Request, Response, StatusCode};

use crate::auth::{self, AuthCache, AuthTarget};
use crate::client::ClientConfig;
use crate::device::AsyncDevice;
use crate::error::{HikvisionError, Result};
use crate::isapi::{self, IsapiRequest, ptz, snapshot, system};
use crate::models::{Camera, DeviceInfo, Preset, Snapshot, SnapshotOptions};

/// Async ISAPI client, the counterpart of [`Client`](crate::Client).
///
/// Needs a Tokio runtime. Cheap to clone: clones share the connection pool
/// and the digest cache.
#[derive(Debug, Clone)]
pub struct AsyncClient {
    http: HttpClient,
    auth: AuthCache,
}

impl AsyncClient {
    pub fn new(config: ClientConfig) -> Result<Self> {
        let config = config.normalized();

        let http = HttpClient::builder()
            .timeout(config.timeout)
            .danger_accept_invalid_certs(config.accept_invalid_certs)
            .build()?;

        Ok(Self {
            http,
            auth: AuthCache::default(),
        })
    }

    /// Binds this client to one camera, so calls don't need `&Camera`.
    pub fn device(&self, camera: Camera) -> AsyncDevice {
        AsyncDevice::new(self.clone(), camera)
    }

    pub async fn get_device_info(&self, camera: &Camera) -> Result<DeviceInfo> {
        let xml = self
            .execute(camera, system::device_info_request())
            .await?
            .text()
            .await?;

        system::parse_device_info(&xml)
    }

    pub async fn get_presets(&self, camera: &Camera, channel: u32) -> Result<Vec<Preset>> {
        let xml = self
            .execute(camera, ptz::presets_request(channel))
            .await?
            .text()
            .await?;

        ptz::parse_presets(&xml)
    }

    pub async fn goto_preset(&self, camera: &Camera, channel: u32, id: u32) -> Result<()> {
        let response = self
            .execute(camera, ptz::goto_preset_request(channel, id))
            .await?;
        let status = response.status().as_u16();

        isapi::check_response_status(status, response.text().await?)
    }

    pub async fn get_snapshot(
        &self,
        camera: &Camera,
        channel: u32,
        options: SnapshotOptions,
    ) -> Result<Snapshot> {
        let response = self
            .execute(camera, snapshot::snapshot_request(channel, options))
            .await?;

        match snapshot::image_content_type(response.headers()) {
            Ok(content_type) => Ok(Snapshot {
                content_type,
                data: response.bytes().await?,
            }),
            Err(content_type) => {
                let body = response
                    .text()
                    .await
                    .unwrap_or_else(|err| format!("Failed to read response body: {}", err));

                Err(snapshot::unexpected_content_type(content_type, body))
            }
        }
    }

    async fn execute(&self, camera: &Camera, request: IsapiRequest) -> Result<Response> {
        self.request(
            request.method,
            camera,
            &request.path,
            &request.query,
            request.body.map(String::into_bytes),
        )
        .await
    }

    /// Sends a raw ISAPI request with digest authentication.
    ///
    /// Non-2xx responses are returned as [`HikvisionError::Api`].
    pub async fn request(
        &self,
        method: Method,
        camera: &Camera,
        path: &str,
        query: &[(&str, String)],
        body: Option<Vec<u8>>,
    ) -> Result<Response> {
        let mut builder = self
            .http
            .request(method, isapi::build_url(&camera.host, path));

        if !query.is_empty() {
            builder = builder.query(query);
        }

        if let Some(body) = body {
            builder = builder.header(CONTENT_TYPE, "application/xml").body(body);
        }

        let request = builder.build()?;

        // Signing is synchronous; compute both possible answers before any
        // `.await` so no non-Send digest state is held across it.
        let target = AuthTarget::new(request.url(), &camera.username);
        let cached = {
            let context = target.context(
                camera,
                request.method(),
                request.body().and_then(|body| body.as_bytes()),
            );
            self.auth.cached_answer(&target.key, &context)
        };

        let response = self.send(&request, cached).await?;

        if response.status() != StatusCode::UNAUTHORIZED {
            return Self::validate_response(response).await;
        }

        let answer = {
            let context = target.context(
                camera,
                request.method(),
                request.body().and_then(|body| body.as_bytes()),
            );
            self.auth
                .answer_challenge(&target.key, response.headers(), &context)?
        };

        let Some(answer) = answer else {
            return Self::validate_response(response).await;
        };

        let response = self.send(&request, Some(answer)).await?;

        if response.status() == StatusCode::UNAUTHORIZED {
            self.auth.forget(&target.key);
        }

        Self::validate_response(response).await
    }

    async fn send(&self, request: &Request, authorization: Option<String>) -> Result<Response> {
        let mut request = request
            .try_clone()
            .ok_or_else(|| HikvisionError::Digest("request body is not cloneable".to_string()))?;

        if let Some(authorization) = authorization {
            request
                .headers_mut()
                .insert(AUTHORIZATION, auth::header_value(&authorization)?);
        }

        Ok(self.http.execute(request).await?)
    }

    async fn validate_response(response: Response) -> Result<Response> {
        if response.status().is_success() {
            return Ok(response);
        }

        let status = response.status().as_u16();
        let message = response
            .text()
            .await
            .unwrap_or_else(|err| format!("Failed to read error response: {}", err));

        Err(HikvisionError::api(status, message))
    }
}
