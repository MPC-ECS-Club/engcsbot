use serenity::all::{ActivityData, Context, OnlineStatus};

#[cfg(not(debug_assertions))]
const STATUSES: &[&str] = &["engineering...", "programming...", "procrastinating..."];
#[cfg(not(debug_assertions))]
const STATUS_TIME: Duration = Duration::from_mins(2);

async fn run(ctx: Context) {
    #[cfg(debug_assertions)]
    {
        ctx.set_presence(
            Some(ActivityData::playing("debug mode")),
            OnlineStatus::Online,
        );
    }
    #[cfg(not(debug_assertions))]
    {
        let mut i = 0usize;
        loop {
            let desired = STATUSES[i];
            ctx.set_presence(Some(ActivityData::custom(desired)), OnlineStatus::Online);

            i = (i + 1) % STATUSES.len();
            tokio::time::sleep(STATUS_TIME).await;
        }
    }
}

pub fn start(ctx: Context) {
    tokio::spawn(run(ctx));
}