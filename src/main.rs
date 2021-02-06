use teloxide::{prelude::*, utils::command::BotCommand};
use tokio::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use lazy_static::lazy_static;

lazy_static! {
    static ref TAP: Mutex<Vec<String>> = Mutex::new(Vec::new());
    static ref BEERS: AtomicU64 = AtomicU64::new(0);
}

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "Display this text")]
    Help,
    #[command(description = "Hold a beer")]
    Beer(String),
    #[command(description = "See what's on tap")]
    OnTap
}

async fn answer(cx: UpdateWithCx<Message>, command: Command) -> ResponseResult<()> {
    match command {
	Command::Help => cx.answer(Command::descriptions()).send().await?,
	Command::Beer(b) => {
	    let prev = BEERS.fetch_add(1, Ordering::Relaxed);
	    TAP.lock().await.push(b);
	    cx.answer_str(format!("Currently holding {} beers", prev + 1)).await?
	},
        Command::OnTap => {
	    cx.answer_str(TAP.lock().await.join("\n")).await?
	}
    };

    Ok(())
}

async fn run() {
    teloxide::enable_logging!();
    log::info!("Starting BeerHolderBot");
    
    let bot = Bot::from_env();

    let bot_name = "BeerHolderBot";
    
    teloxide::commands_repl(bot, bot_name, answer).await;
}

#[tokio::main]
async fn main() {
    run().await
}
