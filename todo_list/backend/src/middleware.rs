use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::Error;
use actix_web_lab::middleware::Next;

pub async fn log_request<B>(
    req: ServiceRequest,
    next: Next<B>,
) -> Result<ServiceResponse<B>, Error> {
    log::info!("{} {}", req.method(), req.path());
    next.call(req).await
}
