use crate::dto::{GetAccounts, MagicApiJson, Message, ResponseData};
use axum::extract::State;
use sea_orm::{DatabaseConnection, EntityTrait, ModelTrait, QuerySelect, SelectColumns};

use crate::entities::magic;
use crate::error::MagicApiResult;

#[axum::debug_handler]
pub async fn get_accounts(
    State(db): State<DatabaseConnection>,
) -> MagicApiResult<MagicApiJson<ResponseData<GetAccounts>>> {
    let accounts = magic::Entity::find()
        .select_column(magic::Column::Account)
        .distinct_on([magic::Column::Account])
        .all(&db)
        .await?
        .into_iter()
        .map(|row| row.get(magic::Column::Account).unwrap::<String>())
        .collect::<Vec<String>>();

    return if accounts.is_empty() {
        Ok(MagicApiJson::new(ResponseData::new(
            200,
            Some(Message::EmptyData.into()),
            None,
        )))
    } else {
        Ok(MagicApiJson::new(ResponseData::new(
            200,
            None,
            Some(GetAccounts { accounts }),
        )))
    };
}
