
# Table of Contents

1.  [Commands](#org14e2763)
2.  [Building & Running](#org2d4c59b)

A Telegram bot that holds your beer

<a href="https://crates.io/crates/BeerHolderBot"><img alt="Crates.io" src="https://img.shields.io/crates/d/BeerHolderBot?style=for-the-badge"></img></a>


<a id="org14e2763"></a>

# Commands

-   `/beer <text>`
    
    Ask the bot to hold your beer. The text you give it will be associated with that beer in the future.
-   `/ontap`
    
    See what's on tap.
-   `/quaff <number>`
    
    Drink the beer with the given number. Numbers will be displayed by the `/ontap` command.

-   `/corn`
    
    Harvest some corn.


<a id="org2d4c59b"></a>

# Building & Running

This project is managed with Cargo, so just build it with `cargo build` and whatever flags you want.

To run it, grab an API token from `@BotFather` on Telegram, and store it in an environment variable called `TELOXIDE_TOKEN`. Then just `cargo run` or execute the binary directly.

