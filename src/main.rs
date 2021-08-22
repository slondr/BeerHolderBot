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

use teloxide::{prelude::*, utils::command::BotCommand, requests::ResponseResult};
use tokio::sync::Mutex;
use lazy_static::lazy_static;
use std::{error::Error, fs::File, io::prelude::*};
use rand::prelude::*;

type AsyncResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

struct Beer {
    id: i64,
    text: String
}

lazy_static! {
    static ref TAP: Mutex<Vec<String>> = Mutex::new(Vec::new());
    static ref CONNECTION: Mutex<sqlite::Connection> = Mutex::new(sqlite::open("tap.db").unwrap());
}

async fn die() -> AsyncResult<String> {
    // first, open the deaths file
    let path = std::path::Path::new("deaths.txt");
    let lines = 1107; // number of choices
    
    // open the file (read-only)
    let mut file = match File::open(&path) {
		  Ok(f) => f,
		  Err(e) => return Err(Box::new(e))
    };

    // read the file's contents into a string
    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
		  Ok(_) => (),
		  Err(reason) => return Err(Box::new(reason))
    }

    // vectorize contents
    let contents: Vec<&str> = contents.split('\n').collect::<Vec<&str>>();

    // generate a random death index
    let mut rng = rand::thread_rng();
    let index: usize = rng.gen_range(0..lines - 1);

    // return the chosen death
    Ok(String::from(contents[index]))
}

async fn initialize_database() -> AsyncResult<()> {
    CONNECTION.lock().await.execute("CREATE TABLE IF NOT EXISTS tap (
      id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
      chat_id INTEGER NOT NULL,
      text TEXT)")?;
    Ok(())
}

async fn create_beer(chat_id: i64, content: String) -> AsyncResult<()> {
    CONNECTION.lock().await
		  .execute(format!("INSERT INTO tap (chat_id, text) VALUES ('{}', '{}')", chat_id, content))?;
    Ok(())
}

async fn get_all_beers(chat_id: i64) -> AsyncResult<Vec<Beer>> {
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

async fn get_beer_count(chat_id: i64) -> AsyncResult<i64> {
    let c = CONNECTION.lock().await;
    let mut statement = c.prepare(format!("SELECT COUNT(id) FROM tap WHERE chat_id={}", chat_id))?;
    if let sqlite::State::Row = statement.next().unwrap() {
		  Ok(statement.read::<i64>(0)?)
    } else {
		  Err("could not retrieve beer count".into())
    }
}

async fn quaff(id: i64) -> AsyncResult<String> {
    let c = CONNECTION.lock().await;
    let mut statement = c.prepare(format!("SELECT text FROM tap WHERE id={}", id))?;
    if let sqlite::State::Row = statement.next()?  {
		  let text = statement.read::<String>(0)?;
		  // remove the beer from the database
		  c.execute(format!("DELETE FROM tap WHERE id={}", id))?;
		  Ok(text)
    } else {
		  Err("could not retrieve beer text".into())
    }
}

async fn grab_photo(query: &'static str) -> AsyncResult<String> {
	     if let Some(access) = std::env::var_os("UNSPLASH_ACCESS") {
		  // call API to get a random picture of corn
		  let auth_uri = format!("https://api.unsplash.com/photos/random/?client_id={}&query={}", access.into_string().unwrap(), query);
		  let response = reqwest::get(&auth_uri)
				.await.unwrap().text().await.unwrap();
		  // response format is some pretty nested json
		  let parsed_response = json::parse(&response);
		  if let Ok(url) = parsed_response {
				let img_url = &url["urls"]["regular"];
				Ok(img_url.to_string())
		  } else {
				Err("No corn to harvest.".into())
		  }
    } else {
		  Err("You don't have a farm.".into())
    }
}

async fn harvest_corn() -> AsyncResult<String> {
	 grab_photo("corn").await
}

async fn bovine() -> AsyncResult<String> {
	 grab_photo("cow").await
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
    Corn,
    #[command(description = "Post a new message")]
    Post,
    #[command(description = "Get the number of beers on tap")]
    Count,
    #[command(description = "Die stupidly")]
    Yasd,
	 #[command(description = "Take a tripe to the pasture")]
	 Bovine
}

async fn answer(cx: UpdateWithCx<AutoSend<Bot>, Message>, command: Command) -> ResponseResult<()> {
    match command {
		  Command::Help => cx.answer(Command::descriptions()).await?,
		  Command::Beer(b) => {
				if !b.is_empty() {
					 log::info!("Adding {} to list of beers", b);
					 // add the beer to the database
					 match create_beer(cx.chat_id(), b).await {
						  Err(e) => cx.reply_to(format!("Er, something went wrong.\n{}", e)).await?,
						  Ok(_) => {
								// increment the global beer counter
								let cur = get_beer_count(cx.chat_id()).await.unwrap();
								// respond with how many beers are held (globally)
								cx.reply_to(format!("Currently holding {} beer{}", cur, if cur == 1 { "" } else { "s" }))
									 .await?
						  }

					 }
				} else {
					 // the given beer was an empty string, so don't actually store it
					 cx.reply_to("Sorry, I can't hold that beer.").await?
				}
		  },
        Command::OnTap => {
				log::info!("Printing list of beers");
				// if the tap is empty, print a special message so the Telegram API doesn't freak out
				match get_all_beers(cx.chat_id()).await {
					 Err(e) => cx.reply_to(format!("Uh, something went wrong.\n{}", e)).await?,
					 Ok(beers) => {
						  let mut m: String = String::new();
						  if beers.is_empty() {
								cx.reply_to("Sorry, I'm all empty.").await?
						  } else {
								for beer in beers {
									 m += format!("[{}] {}\n", beer.id, beer.text).as_str();
								}
								cx.reply_to(m.as_str()).await?
						  }
					 }
				}
		  },
		  Command::Quaff(beer) => {
				log::info!("Quaffing beer #{}", beer);
				// try to parse the user input as an integer
				if let Ok(index) = beer.parse::<i64>() {
					 let quaff_attempt = quaff(index);

					 match quaff_attempt.await {
						  Err(e) => cx.reply_to(format!("Sorry, we can't do that.\n{}", e)).await?,
						  Ok(m) => {
								// send a message informing which beer was quaffed
								cx.reply_to(format!("You have quaffed \"{}\"", m)).await?
						  }
					 }
				} else {
					 cx.reply_to("Sorry, we don't have that beer on tap.").await?
				}
		  },
		  Command::Corn => {
				// harvest corn
				log::info!("Harvesting corn");
				if let Ok(corn) = harvest_corn().await {
					 cx.answer_photo(teloxide::types::InputFile::url(corn)).await?
					 // cx.reply_to(format!("<img src=\"{}\"/>", corn)).parse_mode(teloxide::types::ParseMode::Html).await?
				} else {
					 log::error!("An error occurred within harvest_corn()");
					 cx.reply_to("You don't have a farm.").await?
				}
		  },
		  Command::Bovine => {
				log::info!("cow time");
				if let Ok(bovine) = bovine().await {
					 cx.answer_photo(teloxide::types::InputFile::url(bovine)).await?
				} else {
					 log::error!("An error occurred within bovine()");
					 cx.reply_to("No bovine").await?
				}
		  },
		  Command::Post => {
				log::info!("Generating new message");
				let new_msg = telegram_markov_chain::chain();
				log::info!("Posting new message");
				cx.reply_to(new_msg).await?
		  },
		  Command::Count => {
				log::info!("Counting bottles of beer on the wall");
				if let Ok(count) = get_beer_count(cx.chat_id()).await {
					 cx.reply_to(format!("{} bottles of beer on the wall.", count)).await?
				} else {
					 cx.reply_to("I can't seem to find any beers.").await?
				}
		  },
		  Command::Yasd => {
				log::info!("Dying stupidly...");
				if let Ok(death) = die().await {
					 let caller = cx.update.from().unwrap().username.as_ref().unwrap();
					 cx.reply_to(format!("<code>
               ----------
              /          \\
              /    REST    \\
              /      IN      \\
				  /     PEACE      \\
				  /                  \\
				  |     @{}     
          | {} 
          |                  |
          |                  |
          |                  |
          |       2021       |
         *|     *  *  *      | *
_________)/\\\\_//(\\/(/\\)/\\//\\/|_)_______</code>", caller, death)).parse_mode(teloxide::types::ParseMode::Html).await?
				} else {
					 cx.reply_to("I just can't seem to die.").await?
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
    let bot = Bot::from_env().auto_send();

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
