use crate::dto::{
    GetAccountMagicNos, GetAccounts, GetMagicNoStats, GetMagicNosRequestQuery, GetTradesStats,
    MagicApiJson, MagicApiQuery, Message, ResponseData,
};
use axum::extract::State;
use sea_orm::Condition;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait, QueryFilter, QuerySelect,
    SelectColumns,
};

use crate::entities::magic;
use crate::error::MagicApiResult;
use crate::routes::helpers::{
    format_timedelta, get_longest_lose_seq, get_longest_shortest_trades, get_longest_win_seq,
    get_win_loss_trades,
};

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

    if accounts.is_empty() {
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
    }
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

    if magic_numbers.is_empty() {
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
    }
}

#[axum::debug_handler]
pub async fn get_trades_stats(
    State(db): State<DatabaseConnection>,
    MagicApiQuery(params): MagicApiQuery<GetTradesStats>,
) -> MagicApiResult<MagicApiJson<ResponseData<GetMagicNoStats>>> {
    let magic_numbers = magic::Entity::find()
        .filter(
            Condition::all()
                .add(magic::Column::Account.eq(&params.account))
                .add(magic::Column::MagicNumber.eq(params.magic_no)),
        )
        .all(&db)
        .await?;

    if magic_numbers.is_empty() {
        warn!(
            "No entries found for Account: {}, Magic No: {}",
            params.account, params.magic_no
        );
        return Ok(MagicApiJson::new(ResponseData::new(
            200,
            Some(Message::EmptyData.into()),
            None,
        )));
    }

    let lot_size = magic_numbers[0].lots.parse::<f32>().unwrap();
    let (winners, losers) = get_win_loss_trades(&magic_numbers);

    let winners_count = winners.len();
    let losers_count = losers.len();

    let gross_profit = winners
        .iter()
        .map(|m| m.pnl.parse::<f32>().unwrap())
        .sum::<f32>();
    let gross_loss = losers
        .iter()
        .map(|m| m.pnl.parse::<f32>().unwrap())
        .sum::<f32>();
    let net_profit = gross_profit + gross_loss;
    let profit_factor = f32::abs(gross_profit / gross_loss);
    let win_percentage = (winners_count as f32 * 100.00) / magic_numbers.len() as f32;
    let lose_percentage = (losers_count as f32 * 100.00) / magic_numbers.len() as f32;
    let avg_win = gross_profit / (winners_count as f32);
    let avg_lose = gross_loss / (losers_count as f32);
    let avg_trade = net_profit / (magic_numbers.len() as f32);
    let best_trade = magic_numbers
        .iter()
        .map(|m| m.pnl.parse::<f32>().unwrap())
        .max_by(|x, y| x.partial_cmp(y).unwrap())
        .unwrap();
    let worst_trade = magic_numbers
        .iter()
        .map(|m| m.pnl.parse::<f32>().unwrap())
        .min_by(|x, y| x.partial_cmp(y).unwrap())
        .unwrap();
    let (longest_trade, longest_trade_time, shortest_trade, shortest_trade_time, total_time) =
        get_longest_shortest_trades(&magic_numbers);
    let avg_trade_time = total_time / (magic_numbers.len() as i32);
    let total_win_pips = gross_profit / (10.0 * lot_size);
    let total_lose_pips = gross_loss / (10.0 * lot_size);
    let net_pips = net_profit / (10.0 * lot_size);
    let avg_win_pips = total_win_pips / (winners_count as f32);
    let avg_lose_pips = total_lose_pips / (losers_count as f32);
    let avg_net_pips = net_pips / (magic_numbers.len() as f32);
    let mut consecutive_win_trades: Option<usize> = None;
    let mut consecutive_profits: Option<f32> = None;
    let mut consecutive_lose_trades: Option<usize> = None;
    let mut consecutive_losses: Option<f32> = None;

    if let Some((len, sum)) = get_longest_win_seq(
        &magic_numbers
            .iter()
            .map(|m| m.pnl.parse::<f32>().unwrap())
            .collect::<Vec<f32>>(),
    ) {
        consecutive_win_trades = Some(len);
        consecutive_profits = Some(sum);
    } else {
        trace!(
            "No win sequence for Account: {}, Magic No: {}",
            params.account,
            params.magic_no
        );
    }

    if let Some((len, sum)) = get_longest_lose_seq(
        &magic_numbers
            .iter()
            .map(|m| m.pnl.parse::<f32>().unwrap())
            .collect::<Vec<f32>>(),
    ) {
        consecutive_lose_trades = Some(len);
        consecutive_losses = Some(sum);
    } else {
        trace!(
            "No win sequence for Account: {}, Magic No: {}",
            params.account,
            params.magic_no
        );
    }

    let stats = GetMagicNoStats {
        account: params.account,
        magic_number: params.magic_no,
        trades_count: magic_numbers.len(),
        gross_profit,
        gross_loss,
        net_profit,
        profit_factor,
        winners_count,
        losers_count,
        win_percentage,
        lose_percentage,
        avg_win,
        avg_lose,
        avg_trade,
        best_trade,
        worst_trade,
        longest_trade: longest_trade.unwrap(),
        longest_trade_time: format_timedelta(&longest_trade_time),
        shortest_trade: shortest_trade.unwrap(),
        shortest_trade_time: format_timedelta(&shortest_trade_time),
        avg_trade_time: format_timedelta(&avg_trade_time),
        total_win_pips,
        total_lose_pips,
        net_pips,
        avg_win_pips,
        avg_lose_pips,
        avg_net_pips,
        consecutive_win_trades,
        consecutive_profits,
        consecutive_lose_trades,
        consecutive_losses,
    };

    Ok(MagicApiJson::new(ResponseData::new(200, None, Some(stats))))
}
