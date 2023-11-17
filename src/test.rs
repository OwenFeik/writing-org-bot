use actix_web::{test, App};

use super::*;

#[actix_web::test]
async fn test_index_get() {
    let app = test::init_service(App::new().service(interactions)).await;
    let req = test::TestRequest::post()
        .uri("/api/interactions")
        .insert_header(("x-signature-timestamp", "1700181650"))
        .insert_header(("x-signature-ed25519", "cccce6d5673c21031bf971d73b50d92033ff78942c7e982d8953d70682b2b7944850d2d68038c800b454157b6d0740ebcf04169ac52537a9cead18bbac141f05"))
        .set_payload("{\"application_id\":\"1172336119589912637\",\"entitlements\":[],\"id\":\"1174871734504149013\",\"token\":\"aW50ZXJhY3Rpb246MTE3NDg3MTczNDUwNDE0OTAxMzpSR0lpQVNuOVZSWVFuU2JwY2dsUFJzR2tQWFhxSWw5S3ZhNFFDSDBEUkFnanVHbWJESmNaUzRLZFRhQ3VUb3ZiUTN2ZGZZb2phcllVcFlseFpoU3oxVjdiZFpQbXp3SXFZUkszUXlvRWFpQVhoMWFEU0JJZzlHazdyVTZYdk11NQ\",\"type\":1,\"user\":{\"avatar\":\"c6dc1d999777a1332ec8770a76c4b849\",\"avatar_decoration_data\":null,\"discriminator\":\"0\",\"global_name\":\"Skoraeus\",\"id\":\"288943895428071425\",\"public_flags\":0,\"username\":\"skoraeusstonebones\"},\"version\":1}")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}
