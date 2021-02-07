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

//// TO IMPLEMENT A NEW COMMAND
// Add the command name to the Command enum with a description
// Implement the logic of the command in the match statement in answer()

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "Display this text")]
    Help,
    #[command(description = "Hold a beer")]
    Beer(String),
    #[command(description = "See what's on tap")]
    OnTap,
    #[command(description = "Drink a beer by index")]
    Quaff(String)
}

async fn answer(cx: UpdateWithCx<Message>, command: Command) -> ResponseResult<()> {
    match command {
	Command::Help => cx.answer(Command::descriptions()).send().await?,
	Command::Beer(b) => {
	    if b != "" {
		log::info!("Adding {} to list of beers", b);
		let cur = BEERS.fetch_add(1, Ordering::Relaxed) + 1;
		TAP.lock().await.push(b);
		cx.answer_str(format!("Currently holding {} beer{}", cur, if cur == 1 { "" } else { "s" })).await?
	    } else {
		// the given beer was an empty string, so don't actually store it
		cx.answer_str("Sorry, I can't hold that beer.").await?
	    }
	},
        Command::OnTap => {
	    log::info!("Printing list of beers");
	    let mut m: String = String::new();
	    for (i, b) in TAP.lock().await.iter().enumerate() {
		m += format!("[{}] {}\n", i, b).as_str();
	    }
	    cx.answer_str(m.as_str()).await?
	},
	Command::Quaff(beer) => {
	    log::info!("Quaffing beer #{}", beer);
	    // try to parse the user input as a usize
	    if let Ok(index) = beer.parse::<usize>() {
		let tap_lock = TAP.lock().await;
		if let Some(quaffed_beer) = tap_lock.get(index) {
		    let quaffed_beer = quaffed_beer.to_string();
		    drop(tap_lock);
		    let mut tap_lock = TAP.lock().await;
		    tap_lock.remove(index);
		    
		    // reduce the beer counter by 1
		    let _ = BEERS.fetch_sub(1, Ordering::Relaxed);
		    
		    // post a message informing the operation was successful
		    cx.answer_str(format!("You have quaffed \"{}\"", quaffed_beer)).await?		    
		} else {
		    cx.answer_str("Sorry, we don't have that beer on tap.").await?
		}
	    } else {
		cx.answer_str("Sorry, we don't have that beer on tap.").await?
	    }
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
