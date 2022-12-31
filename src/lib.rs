mod chunked_iterator;
pub mod models;
pub mod schema;

use self::models::*;
use self::schema::tracks;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use dotenvy::dotenv;
use gtk::gio;
use gtk::glib::BoxedAnyObject;
use lofty::{ItemKey, Probe};
use std::collections::HashSet;
use std::env;
use std::rc::Rc;
use walkdir::WalkDir;
// use directories::ProjectDirs;

#[derive(Hash, Eq, Ord, PartialEq, PartialOrd, Debug)]
pub struct Facet {
  pub album_artist_or_artist: Option<String>,
  pub album_artist: Option<String>,
  pub album: Option<String>,
  pub all: bool,
}

pub fn connect_db() -> SqliteConnection {
  dotenv().ok();

  let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
  SqliteConnection::establish(&database_url)
    .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn process_file(path: &str) -> Option<NewTrack> {
  let tagged_file = Probe::open(path)
    .expect("ERROR: Bad path provided!")
    .read(true);
  match tagged_file {
    Ok(tagged_file) => {
      let tag = match tagged_file.primary_tag() {
        Some(primary_tag) => Some(primary_tag),
        None => tagged_file.first_tag(),
      };

      match tag {
        Some(tag) => Some(NewTrack {
          filename: &path,
          artist: tag.artist(),
          album: tag.album(),
          album_artist: tag.get_string(&ItemKey::AlbumArtist),
          title: tag.title(),
          track: tag.track(),
          genre: tag.genre(),
        }),
        None => None,
      }
    }
    Err(_) => None,
  }
}
fn hashset(data: &Vec<Rc<Track>>) -> HashSet<&std::string::String> {
  HashSet::from_iter(data.iter().map(|elt| &elt.filename))
}

const MAX_VAL: i32 = 10000000;

pub fn run_scan(folder: &str, rows: &Vec<Rc<Track>>) {
  let hash = hashset(rows);
  let mut conn = connect_db();
  let mut i = 0;
  let transaction_size = 20;

  for chunk in chunked_iterator::ChunkedIterator::new(
    WalkDir::new(folder).into_iter().filter_map(|e| e.ok()),
    transaction_size,
  ) {
    let v = Vec::new();
    for file in chunk {
      if file.file_type().is_file() && i < MAX_VAL {
        let path = file.path();
        let s = path.display().to_string();
        if !hash.contains(&s) {
          v.push(process_file(&s))
        }
        i = i + 1;
      }
    }

    diesel::insert_into(tracks::table)
      .values(&v)
      .execute(&mut conn);
  }
}

pub fn add_track_to_recently_played(path: &str) -> () {
  // let conn = connect_db();
  // conn.execute("INSERT INTO recently_played (filename) VALUES (?)", (path,))?;

  // Ok(())
}

pub fn load_tracks() -> Vec<Rc<Track>> {
  use self::schema::tracks::dsl::*;

  let conn = &mut connect_db();
  let results = tracks.load::<Track>(conn).expect("Error loading tracks");

  // println!("Displaying {} tracks", results.len());
  // for track in &results {
  //   println!("{}", track.filename);
  // }

  results.into_iter().map(|r| Rc::new(r)).collect()
}

pub fn load_playlist_store<'a, I>(vals: I, store: &gio::ListStore)
where
  I: Iterator<Item = &'a Rc<Track>>,
{
  for row in vals {
    store.append(&BoxedAnyObject::new(row.clone()));
  }
}

pub fn load_facet_store(rows: &[Rc<Track>], facet_store: &gio::ListStore) {
  let mut facets = HashSet::new();
  for row in rows {
    facets.insert(Facet {
      album: row.album.clone(),
      album_artist: row.album_artist.clone(),
      album_artist_or_artist: Some("".to_string()), //row.album_artist_or_artist.clone(),
      all: false,
    });
  }
  facet_store.append(&BoxedAnyObject::new(Facet {
    album: None,
    album_artist: None,
    album_artist_or_artist: None,
    all: true,
  }));
  let mut v = Vec::from_iter(facets);
  v.sort();
  for uniq in v {
    facet_store.append(&BoxedAnyObject::new(uniq))
  }
}