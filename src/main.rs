use teloxide::prelude::*;

#[tokio::main]
async fn main() {
    teloxide::enable_logging!();
    log::info!("Starting BeerHolderBot...");

    let bot = Bot::from_env();

    teloxide::repl(bot, |message| async move {
        message.answer("Hello, world!").send().await?;
        ResponseResult::<()>::Ok(())
    })
    .await;
}
