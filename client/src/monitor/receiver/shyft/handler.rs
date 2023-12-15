
// home handler to receive requests from shyft
#[axum_macros::debug_handler]
async fn home(
    Extension(page_config): Extension<Arc<ShardedDb>>,
    b: String,
) -> ServerResult<impl IntoResponse> {
    // if b iz string
    let body = serde_json::from_str::<Vec<HomeHtml2>>(&b).unwrap();
    for i in 0..body.len() {
        page_config.lock().unwrap().insert(
            body[i].account.parsed.pubkey.clone(),
            Account {
                lamports: body[i].account.parsed.lamports,
                data: base64::decode(&body[i].account.parsed.data[0]).unwrap(),
                owner: Pubkey::from_str(&body[i].account.parsed.owner).unwrap(),
                executable: body[i].account.parsed.executable,
                rent_epoch: body[i].account.parsed.rent_epoch,
            },
        );
    }
    Ok(Json(SetJson { status_code: 200 }).into_response())
}