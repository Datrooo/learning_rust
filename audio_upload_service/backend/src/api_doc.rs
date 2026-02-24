use utoipa::OpenApi;

use crate::progress::{Stage, UploadProgress};
use crate::upload::{UploadRequest, UploadResponse};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::upload::upload_audio,
        crate::progress::progress_sse
    ),
    components(
        schemas(UploadRequest, UploadResponse, UploadProgress, Stage)
    ),
    tags(
        (name = "media", description = "Media upload and progress")
    )
)]
pub struct ApiDoc;
