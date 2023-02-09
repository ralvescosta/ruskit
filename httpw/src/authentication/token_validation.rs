// use std::sync::Arc;

// use actix_web::{dev::ServiceRequest, Error};
// use actix_web_httpauth::extractors::bearer::{BearerAuth, Config};
// use actix_web_httpauth::extractors::AuthenticationError;
// use auth::AuthMiddleware;
// use opentelemetry::Context;

// pub async fn validator(
//     req: ServiceRequest,
//     credentials: BearerAuth,
// ) -> Result<ServiceRequest, (Error, ServiceRequest)> {
//     let auth_mid = req
//         .app_data::<Arc<dyn AuthMiddleware + Send + Sync>>()
//         .map(|data| data.clone())
//         .unwrap();

//     let config = req
//         .app_data::<Config>()
//         .map(|data| data.clone())
//         .unwrap_or_else(Default::default);

//     match auth_mid
//         .authenticate(&Context::new(), credentials.token())
//         .await
//     {
//         Ok(res) => {
//             if res == true {
//                 Ok(req)
//             } else {
//                 Err((AuthenticationError::from(config).into(), req))
//             }
//         }
//         Err(_) => Err((AuthenticationError::from(config).into(), req)),
//     }
// }
