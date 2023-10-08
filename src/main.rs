// use crate::memory::*;
// use crate::util::{vec_to_list, string_to_proper_list};
// use crate::native::eval::eval_external;
// use crate::native::load_native_functions;

use tiny_http::{Server, Response};
use std::fs::File;
use serde::{Serialize, Deserialize};
use std::time::Duration;

const ADDRESS: &str = "0.0.0.0";
const PORT:    u32  = 8000;
const TIMEOUT: u64 = 60;


#[derive(Deserialize)]
struct FromClient {
    which_button: String,
}

#[derive(Serialize)]
struct ToClient {
    html: String,
}


fn main() -> Result<(), String> {
    let server = Server::http(format!("{}:{}", ADDRESS, PORT)).expect("could not start HTTP server");
    println!("Server started on {}:{}", ADDRESS, PORT);

    let mut keep_running = true;
    while keep_running {
        let mut request =
        if let Some(r) = server.recv_timeout(Duration::from_secs(TIMEOUT)).ok().expect("could not receive HTTP request") {
            r
        }
        else {
            break;
        };

        match request.url() {
            "/" => {
                let file = File::open("index.html").expect("could not open index.html file");
                let response = Response::from_file(file);
                request.respond(response).expect("could not send response");
            },
            "/update" => {
                let mut request_body = String::new();
                request.as_reader().read_to_string(&mut request_body).expect("could not read client POST request");
                let event: FromClient = serde_json::from_str(&request_body).expect("could not deserialize client POST request");

                let to_client = 
                match event.which_button.as_str() {
                    "start-button" => ToClient{ html: format!("<span style='color: red;'>Started</span>") },
                    "stop-button"  => ToClient{ html: format!("<span style='color: green;'>Stopped</span>") },
                    "quit-button"  => {
                        keep_running = false;
                        ToClient{ html: format!("<span style='color: cyan;'>Server stopped</span>") }
                    },
                    s       => ToClient{ html: format!("<span style='color: yellow;'>Unknown button: '{s}'</span>") },
                };

                let json_string = serde_json::to_string(&to_client).unwrap();
                let response    = Response::from_string(json_string);
                request.respond(response).expect("could not send response");

            },
            url => {
                let response = Response::from_string(&format!("unknown address: {url}"));
                request.respond(response).expect("could not send response");
            },
        }
    }

    if keep_running {
        println!("Stopping because client has been idle for more than {TIMEOUT} seconds.");
    }
    else {
        println!("Stopping because client has exited.");
    }

    // println!("PiciLisp");

    // let mut mem = Memory::new();

    // load_native_functions(&mut mem);

    // println!("Loaded native functions.");

    // // (load-all "prelude contents..." (quote prelude))
    // let prelude_str = include_str!("prelude.lisp");  
    // let prelude     = string_to_proper_list(&mut mem, prelude_str);
    // let source_name = vec![mem.symbol_for("quote"), mem.symbol_for("prelude")];
    // let vec         = vec![mem.symbol_for("load-all"), prelude, vec_to_list(&mut mem, &source_name)];
    // let expression  = vec_to_list(&mut mem, &vec);
    // eval_external(&mut mem, expression)?;

    // println!("Loaded prelude.");

    // // (repl ">>> " nil)
    // let vec        = vec![mem.symbol_for("repl"), string_to_proper_list(&mut mem, ">>> "), GcRef::nil()];
    // let expression = vec_to_list(&mut mem, &vec);
    // eval_external(&mut mem, expression)?;

    // println!("Bye!");

    Ok(())
}



mod metadata;
mod memory;
mod util;
mod native;
mod error_utils;
mod parser;
mod config;
