use napi_derive::napi;

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

    #[napi]
    pub async fn upload_leaderboard_score(
        leaderboard_id: String, // Accept as String for NAPI compatibility
        method: i32,            // 0 = KeepBest, 1 = ForceUpdate
        score: i32,
        details: Vec<i32>,
    ) -> Option<UploadedScoreInfo> {
        use steamworks::UploadScoreMethod;
        let client = crate::client::get_client();
        let leaderboard_id = leaderboard_id.parse::<u64>().ok()?;
        let leaderboard =
            unsafe { std::mem::transmute::<u64, steamworks::Leaderboard>(leaderboard_id) };
        let upload_method = match method {
            0 => UploadScoreMethod::KeepBest,
            1 => UploadScoreMethod::ForceUpdate,
            _ => UploadScoreMethod::KeepBest,
        };
        let (tx, rx) = tokio::sync::oneshot::channel();
        client.user_stats().upload_leaderboard_score(
            &leaderboard,
            upload_method,
            score,
            &details,
            move |res| {
                let _ = tx.send(res.ok().flatten().map(|v| UploadedScoreInfo {
                    score: v.score,
                    was_changed: v.was_changed,
                    global_rank_new: v.global_rank_new,
                    global_rank_previous: v.global_rank_previous,
                }));
            },
        );
        rx.await.ok().flatten()
    }

    #[napi(object)]
    pub struct UploadedScoreInfo {
        pub score: i32,
        pub was_changed: bool,
        pub global_rank_new: i32,
        pub global_rank_previous: i32,
    }

    #[napi]
    pub async fn download_leaderboard_entries(
        leaderboard_id: String, // Accept as String for NAPI compatibility
        request: i32,           // 0 = Global, 1 = GlobalAroundUser, 2 = Friends
        start: i32,
        end: i32,
        max_details_len: i32,
    ) -> Option<Vec<LeaderboardEntryInfo>> {
        use steamworks::LeaderboardDataRequest;
        let client = crate::client::get_client();
        let leaderboard_id = leaderboard_id.parse::<u64>().ok()?;
        let leaderboard =
            unsafe { std::mem::transmute::<u64, steamworks::Leaderboard>(leaderboard_id) };
        let request_type = match request {
            0 => LeaderboardDataRequest::Global,
            1 => LeaderboardDataRequest::GlobalAroundUser,
            2 => LeaderboardDataRequest::Friends,
            _ => LeaderboardDataRequest::Global,
        };
        let (tx, rx) = tokio::sync::oneshot::channel();
        client.user_stats().download_leaderboard_entries(
            &leaderboard,
            request_type,
            start as usize,
            end as usize,
            max_details_len as usize,
            move |res| {
                let _ = tx.send(res.ok().map(|entries| {
                    entries
                        .into_iter()
                        .map(|e| LeaderboardEntryInfo {
                            user: e.user.raw(),
                            global_rank: e.global_rank,
                            score: e.score,
                            details: e.details,
                        })
                        .collect()
                }));
            },
        );
        rx.await.ok().flatten()
    }

    #[napi(object)]
    pub struct LeaderboardEntryInfo {
        pub user: u64,
        pub global_rank: i32,
        pub score: i32,
        pub details: Vec<i32>,
    }
}
