use super::super::{CustomError, Jwt};
use axum::{
    extract::{Extension, Form, Path},
    response::IntoResponse,
};
use db::authz;
use db::queries;
use db::Pool;
use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate, Default, Debug)]
pub struct SetDetails {
    #[validate(length(min = 1, message = "The first name is mandatory"))]
    pub first_name: String,
    #[validate(length(min = 1, message = "The last name is mandatory"))]
    pub last_name: String,
}

pub async fn set_details(
    Path(team_id): Path<i32>,
    current_user: Jwt,
    Extension(pool): Extension<Pool>,
    Form(set_name): Form<SetDetails>,
) -> Result<impl IntoResponse, CustomError> {
    // Create a transaction and setup RLS
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;
    let rbac = authz::get_permissions(&transaction, &current_user.into(), team_id).await?;

    queries::users::set_name()
        .bind(
            &transaction,
            &set_name.first_name,
            &set_name.last_name,
            &rbac.user_id,
        )
        .await?;

    transaction.commit().await?;

    super::super::layout::redirect_and_snackbar(
        &super::super::profile::index_route(team_id),
        "Details Updated",
    )
}
