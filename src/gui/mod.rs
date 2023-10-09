use crate::config::*;
use tiny_http::{Server, Response};
use serde::{Serialize, Deserialize};
use std::time::Duration;


#[derive(Deserialize)]
struct FromClient {
    which_button: String,
}

#[derive(Serialize)]
struct ToClient {
    html: String,
}


pub fn run() {
    let server = Server::http(format!("{}:{}", SERVER_ADDRESS, SERVER_PORT)).expect("could not start HTTP server");
    println!("Server started on {}:{}", SERVER_ADDRESS, SERVER_PORT);

    let mut keep_running = true;
    while keep_running {
        let mut request =
        if let Some(r) = server.recv_timeout(Duration::from_secs(SERVER_TIMEOUT)).ok().expect("could not receive HTTP request") {
            r
        }
        else {
            break;
        };

        match request.url() {
            "/" | "/index.html" => {
                let index_html = include_str!("../index.html");
                let response = Response::from_data(index_html);
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
        println!("Stopping because client has been idle for more than {SERVER_TIMEOUT} seconds.");
    }
    else {
        println!("Stopping because client has exited.");
    }
}


fn highlight_parens(string: &str, cursor: usize, ok_style: &str, unbalanced_style: &str) -> String {
    let mut open_paren  = 0;
    let mut close_paren = 0;
    let mut balanced    = true;
    let mut open_parens = vec![];
    
    for (j, c) in string.chars().enumerate() {
        let i = j + 1; // editor strings are indexed form 1
        match c {
            '(' => {
                if i == cursor {
                    open_paren = i;
                }
                open_parens.push(i);
            },
            ')' => {
                if i == cursor {
                    close_paren = i;
                }
                if let Some(last) = open_parens.pop() {
                    if open_paren == 0 && i == cursor {
                        open_paren = last;
                    }
                    if open_paren == last {
                        close_paren = i;
                    }
                }
                else {
                    balanced = false;
                }
            },
            _   => {/* just keep going */},
        }
    }

    balanced = balanced && open_parens.len() == 0;
    let style = if balanced {ok_style} else {unbalanced_style};

    let mut result = String::new();
    if open_paren != 0 {
        let op = open_paren - 1; // editor strings are indexed form 1
        result.push_str(&string[..op]);
        result.push_str(&format!("<span style='{}'>(</span>", style));
        if close_paren != 0 {
            let cp = close_paren - 1; // editor strings are indexed form 1
            result.push_str(&string[(op + 1)..cp]);
            result.push_str(&format!("<span style='{}'>)</span>", style));
            result.push_str(&string[(cp + 1)..]);
        }
        else {
            result.push_str(&string[(op + 1)..]);
        }
    }
    else {
        if close_paren != 0 {
            let cp = close_paren - 1; // editor strings are indexed form 1
            result.push_str(&string[..cp]);
            result.push_str(&format!("<span style='{}'>)</span>", style));
            result.push_str(&string[(cp + 1)..]);
        }
        else {
            result.push_str(string);
        }
    }
    
    result
}


#[cfg(test)]
mod tests;
