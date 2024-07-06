use std::collections::HashMap;

use crate::dto::{
    GetAllTopTenEas, GetAllTradesStats, GetMagicNoTrades, GetMagicNosRequestQuery,
    GetSingleTradesStats, MagicApiJson, MagicApiQuery, Message, RGetAccountMagicNos, RGetAccounts,
    RGetAllMagicNoStats, RGetAllMagicNoTrades, RGetAllTopTenEas, RGetSingleMagicNoStats,
    ResponseData,
};
use axum::extract::State;
use itertools::Itertools;
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

async fn calculate_stats_for_magic_no(
    account: String,
    magic_no: i32,
    trades: Vec<magic::Model>,
) -> Option<RGetSingleMagicNoStats> {
    if trades.is_empty() {
        return None;
    }

    let lot_size = trades[0].lots.parse::<f32>().unwrap();
    let (winners, losers) = get_win_loss_trades(&trades);

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
    let win_percentage = (winners_count as f32 * 100.00) / trades.len() as f32;
    let lose_percentage = (losers_count as f32 * 100.00) / trades.len() as f32;
    let avg_win = gross_profit / (winners_count as f32);
    let avg_lose = gross_loss / (losers_count as f32);
    let avg_trade = net_profit / (trades.len() as f32);
    let best_trade = trades
        .iter()
        .map(|m| m.pnl.parse::<f32>().unwrap())
        .max_by(|x, y| x.partial_cmp(y).unwrap())
        .unwrap();
    let worst_trade = trades
        .iter()
        .map(|m| m.pnl.parse::<f32>().unwrap())
        .min_by(|x, y| x.partial_cmp(y).unwrap())
        .unwrap();
    let (longest_trade, longest_trade_time, shortest_trade, shortest_trade_time, total_time) =
        get_longest_shortest_trades(&trades);
    let avg_trade_time = total_time / (trades.len() as i32);
    let total_win_pips = gross_profit / (10.0 * lot_size);
    let total_lose_pips = gross_loss / (10.0 * lot_size);
    let net_pips = net_profit / (10.0 * lot_size);
    let avg_win_pips = total_win_pips / (winners_count as f32);
    let avg_lose_pips = total_lose_pips / (losers_count as f32);
    let avg_net_pips = net_pips / (trades.len() as f32);
    let mut consecutive_win_trades: Option<usize> = None;
    let mut consecutive_profits: Option<f32> = None;
    let mut consecutive_lose_trades: Option<usize> = None;
    let mut consecutive_losses: Option<f32> = None;

    if let Some((len, sum)) = get_longest_win_seq(
        &trades
            .iter()
            .map(|m| m.pnl.parse::<f32>().unwrap())
            .collect::<Vec<f32>>(),
    ) {
        consecutive_win_trades = Some(len);
        consecutive_profits = Some(sum);
    }

    if let Some((len, sum)) = get_longest_lose_seq(
        &trades
            .iter()
            .map(|m| m.pnl.parse::<f32>().unwrap())
            .collect::<Vec<f32>>(),
    ) {
        consecutive_lose_trades = Some(len);
        consecutive_losses = Some(sum);
    }

    Some(RGetSingleMagicNoStats {
        account,
        magic_number: magic_no,
        trades_count: trades.len(),
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
    })
}

async fn process_account(
    db: DatabaseConnection,
    account: String,
    params: MagicApiQuery<GetAllTopTenEas>,
) -> Option<(String, RGetAllMagicNoStats)> {
    let magic_numbers_entries = magic::Entity::find()
        .filter(Condition::all().add(magic::Column::Account.eq(&account)))
        .select_column(magic::Column::MagicNumber)
        .distinct_on([magic::Column::MagicNumber])
        .all(&db)
        .await
        .ok()?;

    if magic_numbers_entries.is_empty() {
        warn!("No entries found for Account: {}", &account);
        return None;
    }

    let magic_numbers = magic_numbers_entries
        .into_iter()
        .map(|row| row.get(magic::Column::MagicNumber).unwrap::<i32>())
        .collect::<Vec<i32>>();

    let trades = magic::Entity::find()
        .filter(
            Condition::all()
                .add(magic::Column::Account.eq(&account))
                .add(magic::Column::MagicNumber.is_in(magic_numbers.clone()))
                .add_option(params.0.start_date.map(|d| magic::Column::OpenTime.gte(d)))
                .add_option(params.0.end_date.map(|d| magic::Column::OpenTime.lte(d))),
        )
        .all(&db)
        .await
        .ok()?;

    let mut trades_map: HashMap<i32, Vec<magic::Model>> = HashMap::new();

    for trade in trades {
        trades_map
            .entry(trade.magic_number)
            .or_default()
            .push(trade);
    }

    let mut stats = RGetAllMagicNoStats::new();

    let stat_futures: Vec<_> = trades_map
        .into_iter()
        .map(|(magic_no, trades)| {
            let a = account.clone();
            tokio::spawn(async move { calculate_stats_for_magic_no(a, magic_no, trades).await })
        })
        .collect();

    for stat_future in stat_futures {
        if let Ok(Some(stat)) = stat_future.await {
            stats.push_stat(stat);
        }
    }

    let top_ten_stats = stats.filter_top_ten();

    Some((account, top_ten_stats))
}

#[axum::debug_handler]
pub async fn get_accounts(
    State(db): State<DatabaseConnection>,
) -> MagicApiResult<MagicApiJson<ResponseData<RGetAccounts>>> {
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
            Some(RGetAccounts { accounts }),
        )))
    }
}

#[axum::debug_handler]
pub async fn get_magic_nos(
    State(db): State<DatabaseConnection>,
    MagicApiQuery(params): MagicApiQuery<GetMagicNosRequestQuery>,
) -> MagicApiResult<MagicApiJson<ResponseData<RGetAccountMagicNos>>> {
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
            Some(RGetAccountMagicNos {
                magic_numbers,
                account: params.account,
            }),
        )))
    }
}

#[axum::debug_handler]
pub async fn get_single_trades_stats(
    State(db): State<DatabaseConnection>,
    MagicApiQuery(params): MagicApiQuery<GetSingleTradesStats>,
) -> MagicApiResult<MagicApiJson<ResponseData<RGetSingleMagicNoStats>>> {
    let magic_numbers = if params.start_date.is_some() && params.end_date.is_some() {
        magic::Entity::find()
            .filter(
                Condition::all()
                    .add(magic::Column::Account.eq(&params.account))
                    .add(magic::Column::MagicNumber.eq(params.magic_no))
                    .add(magic::Column::OpenTime.gte(params.start_date.unwrap()))
                    .add(magic::Column::OpenTime.lte(params.end_date.unwrap())),
            )
            .all(&db)
            .await?
    } else {
        magic::Entity::find()
            .filter(
                Condition::all()
                    .add(magic::Column::Account.eq(&params.account))
                    .add(magic::Column::MagicNumber.eq(params.magic_no)),
            )
            .all(&db)
            .await?
    };

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

    let stats = RGetSingleMagicNoStats {
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

#[axum::debug_handler]
pub async fn get_all_trades_stats(
    State(db): State<DatabaseConnection>,
    MagicApiQuery(params): MagicApiQuery<GetAllTradesStats>,
) -> MagicApiResult<MagicApiJson<ResponseData<RGetAllMagicNoStats>>> {
    // Retrieve distinct magic numbers and their corresponding trades in one go
    let magic_numbers_trades = if params.start_date.is_some() && params.end_date.is_some() {
        magic::Entity::find()
            .filter(
                Condition::all()
                    .add(magic::Column::Account.eq(&params.account))
                    .add(magic::Column::OpenTime.gte(params.start_date.unwrap()))
                    .add(magic::Column::OpenTime.lte(params.end_date.unwrap())),
            )
            .all(&db)
            .await?
    } else {
        magic::Entity::find()
            .filter(magic::Column::Account.eq(&params.account))
            .all(&db)
            .await?
    };

    if magic_numbers_trades.is_empty() {
        warn!("No entries found for Account: {}", params.account);
        return Ok(MagicApiJson::new(ResponseData::new(
            200,
            Some(Message::EmptyData.into()),
            None,
        )));
    }

    let magic_number_groups: HashMap<i32, Vec<magic::Model>> = magic_numbers_trades
        .into_iter()
        .chunk_by(|trade| trade.magic_number)
        .into_iter()
        .map(|(key, group)| (key, group.collect()))
        .collect();

    let mut stats = RGetAllMagicNoStats::new();

    let stat_futures: Vec<_> = magic_number_groups
        .into_iter()
        .map(|(magic_no, trades)| {
            let account = params.account.clone();
            tokio::spawn(
                async move { calculate_stats_for_magic_no(account, magic_no, trades).await },
            )
        })
        .collect();

    for stat_future in stat_futures {
        if let Ok(Some(stat)) = stat_future.await {
            stats.push_stat(stat);
        }
    }

    if stats.data().is_empty() {
        Ok(MagicApiJson::new(ResponseData::new(
            200,
            Some(Message::EmptyData.into()),
            None,
        )))
    } else {
        Ok(MagicApiJson::new(ResponseData::new(200, None, Some(stats))))
    }
}

#[axum::debug_handler]
pub async fn get_magic_no_trades(
    State(db): State<DatabaseConnection>,
    MagicApiQuery(params): MagicApiQuery<GetMagicNoTrades>,
) -> MagicApiResult<MagicApiJson<ResponseData<RGetAllMagicNoTrades>>> {
    let magic_number_trades = if params.page.is_some() && params.page_size.is_some() {
        let page = params.page.unwrap_or(1);
        let page_size = params.page_size.unwrap_or(1000);
        let offset = (page - 1) * page_size;

        if params.start_date.is_some() && params.end_date.is_some() {
            magic::Entity::find()
                .filter(
                    Condition::all()
                        .add(magic::Column::Account.eq(&params.account))
                        .add(magic::Column::MagicNumber.eq(params.magic_no))
                        .add(magic::Column::OpenTime.gte(params.start_date.unwrap()))
                        .add(magic::Column::OpenTime.lte(params.end_date.unwrap())),
                )
                .limit(page_size as u64)
                .offset(offset as u64)
                .all(&db)
                .await?
        } else {
            magic::Entity::find()
                .filter(
                    Condition::all()
                        .add(magic::Column::Account.eq(&params.account))
                        .add(magic::Column::MagicNumber.eq(params.magic_no)),
                )
                .limit(page_size as u64)
                .offset(offset as u64)
                .all(&db)
                .await?
        }
    } else if params.start_date.is_some() && params.end_date.is_some() {
        magic::Entity::find()
            .filter(
                Condition::all()
                    .add(magic::Column::Account.eq(&params.account))
                    .add(magic::Column::MagicNumber.eq(params.magic_no))
                    .add(magic::Column::OpenTime.gte(params.start_date.unwrap()))
                    .add(magic::Column::OpenTime.lte(params.end_date.unwrap())),
            )
            .all(&db)
            .await?
    } else {
        magic::Entity::find()
            .filter(
                Condition::all()
                    .add(magic::Column::Account.eq(&params.account))
                    .add(magic::Column::MagicNumber.eq(params.magic_no)),
            )
            .all(&db)
            .await?
    };

    if magic_number_trades.is_empty() {
        Ok(MagicApiJson::new(ResponseData::new(
            200,
            Some(Message::EmptyData.into()),
            None,
        )))
    } else {
        Ok(MagicApiJson::new(ResponseData::new(
            200,
            None,
            Some(RGetAllMagicNoTrades {
                entries_count: magic_number_trades.len(),
                data: magic_number_trades,
            }),
        )))
    }
}

#[axum::debug_handler]
pub async fn get_all_trades_stats_top_ten(
    State(db): State<DatabaseConnection>,
    MagicApiQuery(params): MagicApiQuery<GetAllTopTenEas>,
) -> MagicApiResult<MagicApiJson<ResponseData<RGetAllTopTenEas>>> {
    let accounts = magic::Entity::find()
        .select_column(magic::Column::Account)
        .distinct_on([magic::Column::Account])
        .all(&db)
        .await?
        .into_iter()
        .map(|row| row.get(magic::Column::Account).unwrap::<String>())
        .collect::<Vec<String>>();

    if accounts.is_empty() {
        warn!("No Accounts found in db");
        return Ok(MagicApiJson::new(ResponseData::new(
            200,
            Some(Message::EmptyData.into()),
            None,
        )));
    }

    // Parallel processing of each account
    let response_futures: Vec<_> = accounts
        .into_iter()
        .map(|account| {
            let db_clone = db.clone();
            let params_clone = MagicApiQuery(params.clone());
            tokio::spawn(async move { process_account(db_clone, account, params_clone).await })
        })
        .collect();

    let mut response = RGetAllTopTenEas::new();

    for future in response_futures {
        if let Ok(Some((_account, top_ten_stats))) = future.await {
            response.insert_data(top_ten_stats);
        }
    }

    if response.data.is_empty() {
        Ok(MagicApiJson::new(ResponseData::new(
            200,
            Some(Message::EmptyData.into()),
            None,
        )))
    } else {
        Ok(MagicApiJson::new(ResponseData::new(
            200,
            None,
            Some(response),
        )))
    }
}
