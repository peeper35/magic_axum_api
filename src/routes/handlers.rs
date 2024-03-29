use crate::dto::{
    GetAccountMagicNos, GetAccounts, GetMagicNosRequestQuery, MagicApiJson, MagicApiQuery, Message,
    ResponseData,
};
use axum::extract::State;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait, QueryFilter, QuerySelect,
    SelectColumns,
};

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

#[axum::debug_handler]
pub async fn get_magic_nos(
    State(db): State<DatabaseConnection>,
    MagicApiQuery(params): MagicApiQuery<GetMagicNosRequestQuery>,
) -> MagicApiResult<MagicApiJson<ResponseData<GetAccountMagicNos>>> {
    let magic_numbers = magic::Entity::find()
        .filter(magic::Column::Account.eq(&params.account))
        .select_column(magic::Column::MagicNumber)
        .distinct_on([magic::Column::MagicNumber])
        .all(&db)
        .await?
        .into_iter()
        .map(|row| row.get(magic::Column::MagicNumber).unwrap::<i32>())
        .collect::<Vec<i32>>();

    return if magic_numbers.is_empty() {
        Ok(MagicApiJson::new(ResponseData::new(
            200,
            Some(Message::EmptyData.into()),
            None,
        )))
    } else {
        Ok(MagicApiJson::new(ResponseData::new(
            200,
            None,
            Some(GetAccountMagicNos {
                magic_numbers,
                account: params.account,
            }),
        )))
    };
}
