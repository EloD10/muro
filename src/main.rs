#![allow(warnings)]

#[macro_use] extern crate diesel;
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate futures;
extern crate num_cpus;
extern crate actix;
extern crate actix_web;
extern crate env_logger;
extern crate dotenv;
extern crate chrono;
extern crate bcrypt;
extern crate http;
extern crate ring;
extern crate data_encoding;
extern crate postgres;
extern crate timeago;
extern crate pulldown_cmark;

use actix::*;
use actix_web::{server, App, fs, middleware};
use actix_web::middleware::cors::Cors;
use actix_web::http::{header, Method};
use diesel::prelude::PgConnection;
use diesel::r2d2::{ Pool, ConnectionManager };

mod api;
mod handler;
mod model;
mod utils;

use model::db::ConnDsl;
use api::index::{AppState, home, path};
use api::auth::{signup, signin};
use api::theme::{theme_and_comments,theme_list, theme_new, theme_add_comment};
use api::user::{user_info, user_delete, user_update};

fn main() {
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    ::std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();
    let sys = actix::System::new("webapp");

    let db_url = dotenv::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(db_url);
    let conn = Pool::builder().build(manager).expect("Failed to create pool.");
    let addr = SyncArbiter::start( num_cpus::get() * 4, move || { ConnDsl(conn.clone()) });
    server::new( move || App::with_state(AppState{ db: addr.clone()})
            .middleware(middleware::Logger::default())
            .resource("/", |r| r.h(home))
            .resource("/a/{tail:.*}", |r| r.h(path))
            .configure(|app| Cors::for_app(app)
            // .allowed_origin("http://localhost:1234")    // let CORS default to all
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
            .allowed_header(header::CONTENT_TYPE)
            .max_age(3600)
            .resource("/user/signup", |r| { r.method(Method::POST).with2(signup); })
            .resource("/user/signin", |r| { r.method(Method::POST).with2(signin); })
            .resource("/api/user_info", |r| { r.method(Method::GET).h(user_info); })
            .resource("/api/user_delete", |r| { r.method(Method::GET).h(user_delete); })
            .resource("/api/user_update", |r| { r.method(Method::POST).with2(user_update); })
            .resource("/api/theme_list", |r| { r.method(Method::GET).h(theme_list); })
            .resource("/api/theme_new", |r| { r.method(Method::POST).with2(theme_new); })
            .resource("/api/{theme_id}", |r| {
                r.method(Method::GET).h(theme_and_comments);
                r.method(Method::POST).with2(theme_add_comment);
            })
            .register())
            .handler("/", fs::StaticFiles::new("public")))
        .bind("0.0.0.0:8000").unwrap()
        .shutdown_timeout(2)
        .start();

    sys.run();
}
