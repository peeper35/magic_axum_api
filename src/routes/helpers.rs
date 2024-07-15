use crate::entities::magic::Model;
use chrono::{Duration, TimeDelta};

pub fn get_win_loss_trades(trades: &[Model]) -> (Vec<&Model>, Vec<&Model>) {
    let (winners, losers): (Vec<_>, Vec<_>) = trades
        .iter()
        .partition(|m| m.pnl.parse::<f32>().unwrap() >= 0.0);

    (winners, losers)
}

pub fn get_longest_shortest_trades(
    trades: &[Model],
) -> (
    Option<String>,
    TimeDelta,
    Option<String>,
    TimeDelta,
    TimeDelta,
) {
    let (longest_trade, longest_time) = trades.iter().fold(
        (None, TimeDelta::new(0, 0).unwrap()),
        |(mut acc_longest, mut acc_time), trade| {
            let difference = trade.close_time - trade.open_time;
            if difference > acc_time {
                acc_longest = Some(trade.order_id.to_string());
                acc_time = difference;
            }
            (acc_longest, acc_time)
        },
    );

    let (shortest_trade, shortest_time, total_duration) = trades.iter().fold(
        (None, TimeDelta::new(31560000, 0).unwrap(), Duration::zero()),
        |(mut acc_shortest, mut acc_time, mut acc_dur), trade| {
            let difference = trade.close_time - trade.open_time;
            if difference < acc_time {
                acc_shortest = Some(trade.order_id.to_string());
                acc_time = difference;
            }
            acc_dur += trade.close_time - trade.open_time;
            (acc_shortest, acc_time, acc_dur)
        },
    );

    (
        longest_trade,
        longest_time,
        shortest_trade,
        shortest_time,
        total_duration,
    )
}

// pub fn get_longest_win_seq(pnls: &[f32]) -> Option<(usize, f32)> {
//     let mut max_idx = 0;
//     let mut max_len = 0;
//     let mut curr_len = 0;
//     let mut curr_idx = 0;

//     for (k, &num) in pnls.iter().enumerate() {
//         if num > 0.0 {
//             curr_len += 1;

//             // New sequence, store beginning index.
//             if curr_len == 1 {
//                 curr_idx = k;
//             }
//         } else {
//             if curr_len > max_len {
//                 max_len = curr_len;
//                 max_idx = curr_idx;
//             }
//             curr_len = 0;
//         }
//     }

//     if curr_len > max_len {
//         max_len = curr_len;
//         max_idx = curr_idx;
//     }

//     if max_len > 0 {
//         return Some((
//             max_len,
//             pnls[max_idx..max_idx + max_len].iter().sum::<f32>(),
//         ));
//     }

//     None
// }

pub fn get_longest_win_seq(pnls: &[f32]) -> Option<(usize, f32)> {
    let mut max_start_idx = 0;
    let mut max_length = 0;
    let mut current_length = 0;
    let mut current_start_idx = 0;

    for (i, &pnl) in pnls.iter().enumerate() {
        if pnl > 0.0 {
            if current_length == 0 {
                current_start_idx = i;
            }
            current_length += 1;
        } else {
            if current_length > max_length {
                max_length = current_length;
                max_start_idx = current_start_idx;
            }
            current_length = 0;
        }
    }

    if current_length > max_length {
        max_length = current_length;
        max_start_idx = current_start_idx;
    }

    if max_length > 0 {
        Some((
            max_length,
            f32_round(
                pnls[max_start_idx..max_start_idx + max_length].iter().sum(),
                2,
            ),
        ))
    } else {
        None
    }
}

// pub fn get_longest_lose_seq(pnls: &[f32]) -> Option<(usize, f32)> {
//     let mut max_idx = 0;
//     let mut max_len = 0;
//     let mut curr_len = 0;
//     let mut curr_idx = 0;

//     for (k, &num) in pnls.iter().enumerate() {
//         if num < 0.0 {
//             curr_len += 1;

//             // New sequence, store beginning index.
//             if curr_len == 1 {
//                 curr_idx = k;
//             }
//         } else {
//             if curr_len > max_len {
//                 max_len = curr_len;
//                 max_idx = curr_idx;
//             }
//             curr_len = 0;
//         }
//     }

//     if curr_len > max_len {
//         max_len = curr_len;
//         max_idx = curr_idx;
//     }

//     if max_len > 0 {
//         return Some((
//             max_len,
//             pnls[max_idx..max_idx + max_len].iter().sum::<f32>(),
//         ));
//     }

//     None
// }

pub fn get_longest_lose_seq(pnls: &[f32]) -> Option<(usize, f32)> {
    let mut max_start_idx = 0;
    let mut max_length = 0;
    let mut current_length = 0;
    let mut current_start_idx = 0;

    for (i, &pnl) in pnls.iter().enumerate() {
        if pnl < 0.0 {
            if current_length == 0 {
                current_start_idx = i;
            }
            current_length += 1;
        } else {
            if current_length > max_length {
                max_length = current_length;
                max_start_idx = current_start_idx;
            }
            current_length = 0;
        }
    }

    if current_length > max_length {
        max_length = current_length;
        max_start_idx = current_start_idx;
    }

    if max_length > 0 {
        Some((
            max_length,
            f32_round(
                pnls[max_start_idx..max_start_idx + max_length].iter().sum(),
                2,
            ),
        ))
    } else {
        None
    }
}

pub fn format_timedelta(time: &TimeDelta) -> String {
    let hours = time.num_hours();
    let minutes = time.num_minutes() % 60;

    format!("{} hours {} mins", hours, minutes)
}

// pub fn first_digit(n: i32) -> i32 {
//     let mut n = n;
//     while n >= 10 {
//         n /= 10;
//     }
//     n
// }

pub fn f32_round(x: f32, decimals: u32) -> f32 {
    let y = 10i32.pow(decimals) as f32;
    (x * y).round() / y
}
