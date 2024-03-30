use crate::entities::magic::Model;

pub fn get_win_loss_trades(trades: &Vec<Model>) -> (usize, usize) {
    let winners = trades.iter().map(|m| m.pnl.parse::<f32>().unwrap()).filter(|pnl| *pnl >= 0.0).count();
    let losers = trades.iter().map(|m| m.pnl.parse::<f32>().unwrap()).filter(|pnl| *pnl < 0.0).count();
    (winners, losers)
}