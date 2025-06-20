use napi::bindgen_prelude::*;
use napi_derive::napi;
use steamworks::{LeaderboardDisplayType, LeaderboardSortMethod};

#[napi]
pub mod stats {
    use steamworks::{LeaderboardDisplayType, LeaderboardSortMethod};

    #[napi]
    pub fn get_int(name: String) -> Option<i32> {
        let client = crate::client::get_client();
        client.user_stats().get_stat_i32(&name).ok()
    }

    #[napi]
    pub fn set_int(name: String, value: i32) -> bool {
        let client = crate::client::get_client();
        client.user_stats().set_stat_i32(&name, value).is_ok()
    }

    #[napi]
    pub fn store() -> bool {
        let client = crate::client::get_client();
        client.user_stats().store_stats().is_ok()
    }

    #[napi]
    pub fn reset_all(achievements_too: bool) -> bool {
        let client = crate::client::get_client();
        client
            .user_stats()
            .reset_all_stats(achievements_too)
            .is_ok()
    }

    #[napi]
    pub async fn find_or_create_leaderboard(
        name: String,
        sort_method: i32,  // 0 = Ascending, 1 = Descending
        display_type: i32, // 0 = Numeric, 1 = TimeSeconds, 2 = TimeMilliSeconds
    ) -> Option<u64> {
        let client = crate::client::get_client();
        let sort = match sort_method {
            0 => LeaderboardSortMethod::Ascending,
            1 => LeaderboardSortMethod::Descending,
            _ => LeaderboardSortMethod::Descending,
        };
        let display = match display_type {
            0 => LeaderboardDisplayType::Numeric,
            1 => LeaderboardDisplayType::TimeSeconds,
            2 => LeaderboardDisplayType::TimeMilliSeconds,
            _ => LeaderboardDisplayType::Numeric,
        };
        let (tx, rx) = tokio::sync::oneshot::channel();
        client
            .user_stats()
            .find_or_create_leaderboard(&name, sort, display, move |res| {
                let _ = tx.send(res.ok().flatten().map(|lb| lb.raw()));
            });
        rx.await.ok().flatten()
    }
}
