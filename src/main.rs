    // BeerHolderBot
    // Copyright (C) 2021  Eric S. Londres

    // This program is free software: you can redistribute it and/or modify
    // it under the terms of the GNU Affero General Public License as published
    // by the Free Software Foundation, either version 3 of the License, or
    // (at your option) any later version.

    // This program is distributed in the hope that it will be useful,
    // but WITHOUT ANY WARRANTY; without even the implied warranty of
    // MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    // GNU Affero General Public License for more details.

    // You should have received a copy of the GNU Affero General Public License
    // along with this program.  If not, see <https://www.gnu.org/licenses/>.



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
