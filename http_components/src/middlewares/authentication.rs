use super::otel::HTTPExtractor;
use crate::viewmodels::HTTPError;
use actix_service::{Service, Transform};
use actix_web::web::Data;
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error};
use actix_web::{http, HttpMessage};
use auth::manager::JwtManager;
use futures::future::{ok, Ready};
use futures::Future;
use opentelemetry::global;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

#[derive(Clone, Default)]
pub struct AuthenticationMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthenticationMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthenticationMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthenticationMiddlewareService {
            service: Rc::new(service),
        })
    }
}

pub struct AuthenticationMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthenticationMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let ctx = global::get_text_map_propagator(|propagator| {
            propagator.extract(&HTTPExtractor::new(req.headers()))
        });

        let Some(jwt_manager) = req.app_data::<Data<dyn JwtManager>>() else {
            return Box::pin(async move {
                return Err(actix_web::error::ErrorInternalServerError(
                    HTTPError::internal_server_error(
                        "bad middleware configuration",
                        "missing middleware manager",
                    ),
                ));
            });
        };

        let service = Rc::clone(&self.service);
        let manager = jwt_manager.clone();

        let token = match req.headers().get(http::header::AUTHORIZATION) {
            Some(header_value) => match header_value.to_str() {
                Ok(value) => match value.strip_prefix("Bearer ") {
                    Some(t) => t.to_owned(),
                    None => {
                        return unauthorized_pined("you are not logged in, please provide a token");
                    }
                },
                Err(_) => {
                    return unauthorized_pined("you are not logged in, please provide a token");
                }
            },
            None => {
                return unauthorized_pined("you are not logged in, please provide a token");
            }
        };

        Box::pin(async move {
            let Ok(claims) = manager.verify(&ctx, &token).await else {
                return Err(actix_web::error::ErrorUnauthorized(
                    HTTPError::unauthorized("unauthorized", "unauthorized"),
                ));
            };

            req.extensions_mut().insert(claims);

            let res = service.call(req).await?;
            Ok(res)
        })
    }
}

fn unauthorized_pined<S>(
    details: &'static str,
) -> Pin<Box<dyn Future<Output = Result<ServiceResponse<S>, actix_web::error::Error>>>> {
    Box::pin(async move {
        return Err(actix_web::error::ErrorUnauthorized(
            HTTPError::unauthorized("unauthorized", details),
        ));
    })
}
