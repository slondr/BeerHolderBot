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

#![allow(non_snake_case)]


use teloxide::{prelude::*, utils::command::BotCommand};
use tokio::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use lazy_static::lazy_static;
use std::error::Error;

struct Beer {
    id: i64,
    text: String
}

lazy_static! {
    static ref TAP: Mutex<Vec<String>> = Mutex::new(Vec::new());
    static ref BEERS: AtomicU64 = AtomicU64::new(0);
    static ref CONNECTION: Mutex<sqlite::Connection> = Mutex::new(sqlite::open("tap.db").unwrap());
}

async fn initialize_database() -> Result<(), Box<dyn std::error::Error>> {
    CONNECTION.lock().await.execute("CREATE TABLE IF NOT EXISTS tap (
      id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
      chat_id INTEGER NOT NULL,
      text TEXT)")?;
    Ok(())
}

async fn create_beer(chat_id: i64, content: String) -> Result<(), Box<dyn Error + Send + Sync>> {
    CONNECTION.lock().await
	.execute(format!("INSERT INTO tap (chat_id, text) VALUES ('{}', '{}')", chat_id, content))?;
    Ok(())
}

async fn get_all_beers(chat_id: i64) -> Result<Vec<Beer>, Box<dyn Error + Send + Sync>> {
    let mut beers: Vec<Beer> = Vec::new();
    let c =  CONNECTION.lock().await;
    let mut statement = c.prepare(format!("SELECT id, text FROM tap WHERE chat_id={}", chat_id))?;
    while let sqlite::State::Row = statement.next().unwrap() {
	beers.push(Beer {
	    id: statement.read::<i64>(0)?,
	    text: statement.read::<String>(1)?
	});
    }
    Ok(beers)
}

async fn quaff(id: i64) -> Result<String, Box<dyn Error + Send + Sync>> {
    let c = CONNECTION.lock().await;
    let mut statement = c.prepare(format!("SELECT text FROM tap WHERE id={}", id))?;
    if let sqlite::State::Row = statement.next()?  {
	let text = statement.read::<String>(0)?;
	// remove the beer from the database
	c.execute(format!("DELETE FROM tap WHERE id={}", id))?;
	Ok(text)
    } else {
	Err("could not retrieve beer text")?
    }
}

async fn harvest_corn() -> Result<String, Box<dyn Error + Send + Sync>> {
    if let Some(access) = std::env::var_os("UNSPLASH_ACCESS") {
	// call API to get a random picture of corn
	let auth_uri = format!("https://api.unsplash.com/photos/random/?client_id={}&query={}", access.into_string().unwrap(), "corn");
	let response = reqwest::get(&auth_uri)
	    .await.unwrap().text().await.unwrap();
	// response format is some pretty nested json
	let parsed_response = json::parse(&response);
	let img_url = &parsed_response.unwrap()["urls"]["raw"];
	Ok(img_url.to_string())
    } else {
	Err("You don't have a farm.")?
    }
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
    Quaff(String),
    #[command(description = "Harvest corn")]
    Corn
}

async fn answer(cx: UpdateWithCx<Message>, command: Command) -> ResponseResult<()> {
    match command {
	Command::Help => cx.answer(Command::descriptions()).send().await?,
	Command::Beer(b) => {
	    if b != "" {
		log::info!("Adding {} to list of beers", b);
		// add the beer to the database
		match create_beer(cx.chat_id(), b).await {
		    Err(e) => cx.reply_to(format!("Er, something went wrong.\n{}", e)).send().await?,
		    Ok(_) => {
			// increment the global beer counter
			let cur = BEERS.fetch_add(1, Ordering::Relaxed) + 1;
			// respond with how many beers are held (globally)
			cx.reply_to(format!("Currently holding {} beer{}", cur, if cur == 1 { "" } else { "s" }))
			    .send().await?
		    }

		}
	    } else {
		// the given beer was an empty string, so don't actually store it
		cx.reply_to("Sorry, I can't hold that beer.").send().await?
	    }
	},
        Command::OnTap => {
	    log::info!("Printing list of beers");
	    // if the tap is empty, print a special message so the Telegram API doesn't freak out
	    match get_all_beers(cx.chat_id()).await {
		Err(e) => cx.reply_to(format!("Uh, something went wrong.\n{}", e)).send().await?,
		Ok(beers) => {
		    let mut m: String = String::new();
		    if beers.len() == 0 {
			cx.reply_to("Sorry, I'm all empty.").send().await?
		    } else {
			for beer in beers {
			    m += format!("[{}] {}\n", beer.id, beer.text).as_str();
			}
			cx.reply_to(m.as_str()).send().await?
		    }
		}
	    }
	},
	Command::Quaff(beer) => {
	    log::info!("Quaffing beer #{}", beer);
	    // try to parse the user input as an integer
	    if let Ok(index) = beer.parse::<i64>() {
/*		let tap_lock = TAP.lock().await;
		if let Some(quaffed_beer) = tap_lock.get(index) {
		    let quaffed_beer = quaffed_beer.to_string();
		    drop(tap_lock);
		    let mut tap_lock = TAP.lock().await;
		    tap_lock.remove(index);
		 */
		let quaff_attempt = quaff(index);

		match quaff_attempt.await {
		    Err(e) => cx.reply_to(format!("Sorry, we can't do that.\n{}", e)).send().await?,
		    Ok(m) => {
			// reduce the global beer counter
			let _ = BEERS.fetch_sub(1, Ordering::Relaxed);
			// send a message informing which beer was quaffed
			cx.reply_to(format!("You have quaffed \"{}\"", m)).send().await?
		    }
		}
	    } else {
		cx.reply_to("Sorry, we don't have that beer on tap.").send().await?
	    }
	},
	Command::Corn => {
	    // harvest corn
	    log::info!("Harvesting corn");
	    if let Ok(corn) = harvest_corn().await {
		cx.reply_to(corn).send().await?
	    } else {
		log::error!("An error occurred within harvest_corn()");
		cx.reply_to("You don't have a farm.").send().await?
	    }
	}
    };
    
    Ok(())
}

async fn run() {
    teloxide::enable_logging!();

    log::info!("Initializing database");

    // connect to database
    let init_db = initialize_database();
    
    log::info!("Starting BeerHolderBot");

    // use the TELOXIDE_TOKEN environment variable for the Telegram API
    let bot = Bot::from_env();

    let bot_name = "BeerHolderBot";

    // make sure the database opened correctly before spawning the bot repl
    init_db.await.expect("Could not initialize database");

    // start the bot
    teloxide::commands_repl(bot, bot_name, answer).await;
}

#[tokio::main]
async fn main() {
    run().await
}
