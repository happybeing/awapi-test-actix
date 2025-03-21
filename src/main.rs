/*
*   An HTTP API for the Autonomi peer-to-peer network

*   Copyright (c) 2024 Mark Hughes

*   This program is free software: you can redistribute it and/or modify
*   it under the terms of the GNU Affero General Public License as published
*   by the Free Software Foundation, either version 3 of the License, or
*   (at your option) any later version.

*   This program is distributed in the hope that it will be useful,
*   but WITHOUT ANY WARRANTY; without even the implied warranty of
*   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
*   GNU Affero General Public License for more details.

*   You should have received a copy of the GNU Affero General Public License
*   along with this program.  If not, see <https://www.gnu.org/licenses/>.

*/

#[macro_use]
extern crate tracing;

mod actions;
mod opt;

use std::io;
use std::time::Duration;

use clap::Parser;
use color_eyre::{eyre::eyre, Result};
use xor_name::XorName;

use dweb::autonomi::access;

pub use access::keys;
pub use access::user_data;

use opt::Opt;
// #[cfg(feature = "metrics")]
// use ant_logging::metrics::init_metrics;
use ant_logging::{LogBuilder, LogFormat, ReloadHandle, WorkerGuard};
use tracing::Level;

use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};

const CONNECTION_TIMEOUT: u64 = 75;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

async fn manual_test_default_route(request: HttpRequest) -> impl Responder {
    return HttpResponse::Ok().body(format!(
        "<!DOCTYPE html><head></head><body>test-default-route '/':<br/>uri: {}<br/>method: {}<body>",
        request.uri(),
        request.method()
    ));
}

async fn manual_test_show_request(request: HttpRequest) -> impl Responder {
    return HttpResponse::Ok().body(format!(
        "<!DOCTYPE html><head></head><body>test-show-request:<br/>uri: {}<br/>method: {}<body>",
        request.uri(),
        request.method()
    ));
}

#[get("/awf/{datamap_address:.*}")]
async fn test_fetch_file(datamap_address: web::Path<String>) -> impl Responder {
    // return HttpResponse::Ok().body(format!(
    //     "<!DOCTYPE html><head></head><body>test /awf/&lt;DATAMAP-ADDRESS&gt;:<br/>xor: {}<body>",
    //     datamap_address.to_string()
    // ));

    // HttpResponse::Ok().body(fetch_content(&datamap_address).await)
    HttpResponse::Ok().body("TODO: implement test_fetch_file()")
}

async fn manual_test_connect() -> impl Responder {
    let opt = Opt::parse();
    if let Ok(_client) = crate::actions::connect_to_network(opt.peers).await {
        return HttpResponse::Ok().body(
            "Testing connect to Autonomi..\
        SUCCESS!",
        );
    } else {
        return HttpResponse::Ok().body(
            "Testing connect to Autonomi..\
        ERROR: failed to connect",
        );
    };
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    color_eyre::install().expect("Failed to initialise error handler");
    let opt = Opt::parse();
    let _result_log_guards = init_logging_and_metrics(&opt);

    // Log the full command that was run and the git version
    info!("\"{}\"", std::env::args().collect::<Vec<_>>().join(" "));
    let version = ant_build_info::git_info();
    info!("autonomi client built with git version: {version}");
    println!("autonomi client built with git version: {version}");

    // commands::handle_subcommand(opt).await?;

    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .service(test_fetch_file)
            .route("/hey", web::get().to(manual_hello))
            .route(
                "/test-show-request",
                web::get().to(manual_test_show_request),
            )
            .route("/test-connect", web::get().to(manual_test_connect))
            // .service(web::scope("/awf").default_service(web::get().to(manual_test_default_route)))
            .default_service(web::get().to(manual_test_default_route))
    })
    .keep_alive(Duration::from_secs(CONNECTION_TIMEOUT))
    .bind(("127.0.0.1", 8081))?
    .run()
    .await
}

fn init_logging_and_metrics(opt: &Opt) -> Result<(ReloadHandle, Option<WorkerGuard>)> {
    let logging_targets = vec![
        ("autonomi-cli".to_string(), Level::TRACE),
        ("autonomi".to_string(), Level::TRACE),
        ("evmlib".to_string(), Level::TRACE),
        ("ant_evm".to_string(), Level::TRACE),
        ("ant_networking".to_string(), Level::INFO),
        ("ant_build_info".to_string(), Level::TRACE),
        ("ant_logging".to_string(), Level::TRACE),
        // ("ant-peers-acuisition".to_string(), Level::INFO),
        ("ant_protocol".to_string(), Level::TRACE),
    ];
    let mut log_builder = LogBuilder::new(logging_targets);
    log_builder.output_dest(opt.log_output_dest.clone());
    log_builder.format(opt.log_format.unwrap_or(LogFormat::Default));
    let guards = log_builder.initialize()?;
    Ok(guards)
}

// TODO may be better to return bytes if the response accepts those, and convert the error messages into Bytes (str?)
// Connect to network and attempt to get file content at the datamap_address
// async fn fetch_content(datamap_address: &String) -> String {
//     let content = if let Ok(address) = str_to_xor_name(datamap_address) {
//         let opt = Opt::parse();
//         if let Ok(peers) = crate::access::network::get_peers(opt.peers).await {
//             if let Ok(client) = crate::actions::connect_to_network(peers).await {
//                 match client.data_get(address).await {
//                     Ok(bytes) => string_from_utf8_lossy(&bytes),
//                     Err(e) => {
//                         format!("Failed to get file from network<br/>ERROR: {e}")
//                     }
//                 }
//             } else {
//                 String::from("Testing /awf..<br/>ERROR: failed to connect")
//             }
//         } else {
//             String::from("Testing connect to Autonomi..<br/>ERROR: failed to get peers")
//         }
//     } else {
//         format!("ERROR: invalid xor name {datamap_address}<br/>ERROR: failed to fetch content from network")
//     };

//     return format!(
//         "<!DOCTYPE html><head></head><body>test /awf/&lt;DATAMAP-ADDRESS&gt;:<br/>xor: {}<br/><br/>{content}<body>",
//         datamap_address.to_string()
//     );
// }

fn string_from_utf8_lossy(input: &[u8]) -> String {
    let mut string = String::new();
    utf8::LossyDecoder::new(|s| string.push_str(s)).feed(input);
    string
}

/// Parse a hex xor address with optional URL scheme
pub fn str_to_xor_name(str: &str) -> Result<XorName> {
    let mut str = if str.ends_with('/') {
        &str[0..str.len() - 1]
    } else {
        str
    };

    match hex::decode(str) {
        Ok(bytes) => match bytes.try_into() {
            Ok(xor_name_bytes) => Ok(XorName(xor_name_bytes)),
            Err(e) => Err(eyre!("XorName not valid due to {e:?}")),
        },
        Err(e) => Err(eyre!("XorName not valid due to {e:?}")),
    }
}
