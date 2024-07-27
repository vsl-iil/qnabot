use std::error::Error;

use rusqlite::Connection;
use rusqlite::params;

pub struct Database {
    path: String
}

impl Database {
    pub fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        let path = path.to_owned();

        let conn = Connection::open(&path)?;

        conn.execute(
            "create table if not exists questions (
                id integer primary key,
                user_id integer not null,
                question text not null
            )", 
            [],
        )?;

        // соединение автоматически закрывается Drop'ом
        Ok(Self { path })
    }

    // Здесь я чуть было не начал расписывать полноценный 
    // универсальный класс для работы с БД, но вовремя 
    // остановился. Не попадайтесь в tar pit!
    pub fn insert_question(&mut self, question: &str, uid: u64) -> rusqlite::Result<()> {
        let conn = Connection::open(&self.path)?;

        conn.execute(
            "insert into questions (user_id, question) values (?1, ?2)",
            params![&uid, &question.to_owned()],
        )?;

        Ok(())
    }
}
