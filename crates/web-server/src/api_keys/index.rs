use super::super::{CustomError, Jwt};
use axum::extract::Extension;
use axum::response::Html;
use db::authz;
use db::{queries, Pool};
use web_pages::{api_keys, routes::api_keys::Index};

pub async fn index(
    Index { team_id }: Index,
    current_user: Jwt,
    Extension(pool): Extension<Pool>,
) -> Result<Html<String>, CustomError> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let rbac = authz::get_permissions(&transaction, &current_user.into(), team_id).await?;

    if !rbac.can_use_api_keys() {
        return Err(CustomError::Authorization);
    }

    let api_keys = queries::api_keys::api_keys()
        .bind(&transaction, &team_id)
        .all()
        .await?;

    let assistants = queries::prompts::prompts()
        .bind(&transaction, &team_id, &db::PromptType::Assistant)
        .all()
        .await?;

    let models = queries::prompts::prompts()
        .bind(&transaction, &team_id, &db::PromptType::Model)
        .all()
        .await?;

    let html = api_keys::index::page(rbac, team_id, api_keys, assistants, models);

    Ok(Html(html))
}
