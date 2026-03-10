use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use uuid::Uuid;

use crate::error::Error;

/// Extracted and validated organization ID from X-Org-Id header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrgId(pub Uuid);

const HEADER_NAME: &str = "x-org-id";

#[async_trait]
impl<S> FromRequestParts<S> for OrgId
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let value = parts
            .headers
            .get(HEADER_NAME)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| Error::bad_request("missing or invalid X-Org-Id header"))?;

        let uuid = Uuid::parse_str(value)
            .map_err(|e| Error::bad_request(format!("invalid org_id (expected UUID): {}", e)))?;

        Ok(OrgId(uuid))
    }
}

impl OrgId {
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    pub fn into_uuid(self) -> Uuid {
        self.0
    }
}
