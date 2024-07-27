#![allow(dead_code)]

use std::error::Error;
mod telegram;
mod serde;
mod util;
mod arenatree;
mod error;
mod db;

static TOKEN:   &str = "";
static DBPATH:  &str = "/mnt/c/Users/user/projects/mr-deeds/questions.db";
static QNAPATH: &str = "/mnt/c/Users/user/projects/mr-deeds/db.json";

fn main() -> Result<(), Box<dyn Error>> {
    let suggestions_db = db::Database::new(DBPATH)?;
    let mut bot = telegram::TelegramSender::new(TOKEN, QNAPATH, suggestions_db);
    bot.start_reply_loop();

    Ok(())
}
