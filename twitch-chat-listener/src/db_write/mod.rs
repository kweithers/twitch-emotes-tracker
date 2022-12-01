use chrono::prelude::*;
use redis::Commands;
use tokio_postgres::{Error, NoTls};
use tokio_postgres::binary_copy::BinaryCopyInWriter;
use tokio_postgres::types::{ToSql, Type};
use futures::pin_mut;

pub struct TableRow {
  pub streamer: String,
  pub date: NaiveDate,
  pub emote: String,
  pub n: i32,
}

pub async fn db_write_and_redis_clear() -> Result<(), Error> {
    let streamers = include_str!("../streamers.txt");
    let emotes = include_str!("../emotes.txt");

    let redis_client = redis::Client::open("redis://redis").unwrap();
    let mut redis_con = redis_client.get_connection().unwrap();

    let (mut postgres_client, postgres_connection) =
        tokio_postgres::connect("host=db user=kmw password=bald", NoTls).await?;
    println!("connected to postgres");

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = postgres_connection.await {
            println!("connection error: {}", e);
        }
    });

    let current_date = Local::now().date_naive();
    let types: Vec<Type> = vec![Type::VARCHAR, Type::DATE, Type::VARCHAR, Type::INT4];
    let mut data = Vec::new();
    
    for stream in streamers.lines() {
    // let stream = "asmongold".to_owned();
    for emote in emotes.lines() {
        let key = stream.to_owned() + ":20221130:" + emote;
        let mut val: i32 = 0;
        if let Ok(n) = redis_con.get::<String, redis::Value>(key) {
            if let Ok(v) = redis::from_redis_value(&n) {
                val = v;
            }
        }
        data.push(TableRow{streamer:stream.to_owned(), date: current_date,emote: emote.to_owned(), n: val});
    }
    }
    
    let tx = postgres_client.transaction().await?;
    let sink = tx
        .copy_in("COPY EMOTES (streamer,date,emote,n) FROM STDIN BINARY")
        .await?;
    let writer = BinaryCopyInWriter::new(sink, &types);
    let num_written = write(writer, &data).await?;
    tx.commit().await?;
    
    println!("wrote to postgres {} rows", num_written);
    Ok(())
}

async fn write(writer: BinaryCopyInWriter, data: &Vec<TableRow>) -> Result<usize,tokio_postgres::Error> {
  pin_mut!(writer);

  let mut row: Vec<&'_ (dyn ToSql + Sync)> = Vec::new();
  for m in data {
      row.clear();
      row.push(&m.streamer);
      row.push(&m.date);
      row.push(&m.emote);
      row.push(&m.n);
      writer.as_mut().write(&row).await?;
  }

  writer.finish().await?;
  Ok(data.len())
}