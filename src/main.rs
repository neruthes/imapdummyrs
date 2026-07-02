use dotenvy::dotenv;
use std::{
    env,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    thread,
};

fn send(stream: &mut TcpStream, text: &str) {
    stream.write_all(text.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn handle_client(mut stream: TcpStream) {
    send(&mut stream, "* OK Dummy IMAP Ready\r\n");

    let reader_stream = stream.try_clone().unwrap();
    let mut reader = BufReader::new(reader_stream);

    let mut authenticated = false;

    loop {
        let mut line = String::new();

        if reader.read_line(&mut line).unwrap() == 0 {
            break;
        }

        let line = line.trim_end_matches(['\r', '\n']);

        if line.is_empty() {
            continue;
        }

        println!("[IMAP] {}", line);

        let mut parts = line.split_whitespace();

        let tag = parts.next().unwrap_or("");

        let cmd = parts.next().unwrap_or("").to_uppercase();

        match cmd.as_str() {
            "CAPABILITY" => {
                send(
                    &mut stream,
                    "* CAPABILITY IMAP4rev1 AUTH=PLAIN LOGIN IDLE UIDPLUS\r\n",
                );
                send(
                    &mut stream,
                    &format!("{tag} OK CAPABILITY completed\r\n"),
                );
            }

            "LOGIN" => {
                authenticated = true;
                send(
                    &mut stream,
                    &format!("{tag} OK LOGIN completed\r\n"),
                );
            }

            "AUTHENTICATE" => {
                let mech = parts.next().unwrap_or("").to_uppercase();

                if mech == "PLAIN" {
                    send(&mut stream, "+ \r\n");
                } else {
                    send(
                        &mut stream,
                        &format!("{tag} NO Unsupported auth\r\n"),
                    );
                }
            }

            "" => {
                // Shouldn't normally happen.
            }

            _ => {
                // Handle AUTHENTICATE PLAIN base64 blob
                if !authenticated && cmd.is_empty() {
                    authenticated = true;
                    send(
                        &mut stream,
                        &format!("{tag} OK AUTHENTICATE completed\r\n"),
                    );
                    continue;
                }

                if !authenticated {
                    send(
                        &mut stream,
                        &format!("{tag} NO Authenticate first\r\n"),
                    );
                    continue;
                }

                match cmd.as_str() {
                    "NAMESPACE" => {
                        send(
                            &mut stream,
                            "* NAMESPACE ((\"\" \"/\")) NIL NIL\r\n",
                        );
                        send(&mut stream, &format!("{tag} OK\r\n"));
                    }

                    "ID" => {
                        send(
                            &mut stream,
                            "* ID (\"name\" \"dummy-imap\")\r\n",
                        );
                        send(&mut stream, &format!("{tag} OK\r\n"));
                    }

                    "LIST" | "LSUB" => {
                        send(
                            &mut stream,
                            "* LIST (\\HasNoChildren) \"/\" \"INBOX\"\r\n",
                        );
                        send(&mut stream, &format!("{tag} OK\r\n"));
                    }

                    "SELECT" | "EXAMINE" => {
                        send(
                            &mut stream,
                            "* FLAGS (\\Seen \\Answered \\Deleted \\Draft)\r\n",
                        );
                        send(&mut stream, "* 0 EXISTS\r\n");
                        send(&mut stream, "* 0 RECENT\r\n");
                        send(&mut stream, "* OK UIDVALIDITY 1\r\n");
                        send(&mut stream, "* OK UIDNEXT 1\r\n");
                        send(
                            &mut stream,
                            &format!("{tag} OK [READ-WRITE] SELECT completed\r\n"),
                        );
                    }

                    "STATUS" => {
                        send(
                            &mut stream,
                            "* STATUS \"INBOX\" (MESSAGES 0 UIDNEXT 1 UIDVALIDITY 1 UNSEEN 0)\r\n",
                        );
                        send(&mut stream, &format!("{tag} OK\r\n"));
                    }

                    "SEARCH" => {
                        send(&mut stream, "* SEARCH\r\n");
                        send(
                            &mut stream,
                            &format!("{tag} OK SEARCH completed\r\n"),
                        );
                    }

                    "UID" => {
                        let subcmd = parts.next().unwrap_or("").to_uppercase();

                        match subcmd.as_str() {
                            "SEARCH" => {
                                send(&mut stream, "* SEARCH\r\n");
                                send(
                                    &mut stream,
                                    &format!("{tag} OK SEARCH completed\r\n"),
                                );
                            }

                            "FETCH" => {
                                send(
                                    &mut stream,
                                    &format!("{tag} OK FETCH completed\r\n"),
                                );
                            }

                            _ => {
                                send(&mut stream, &format!("{tag} OK\r\n"));
                            }
                        }
                    }

                    "FETCH" => {
                        send(
                            &mut stream,
                            &format!("{tag} OK FETCH completed\r\n"),
                        );
                    }

                    "NOOP" => {
                        send(&mut stream, &format!("{tag} OK\r\n"));
                    }

                    "LOGOUT" => {
                        send(&mut stream, "* BYE\r\n");
                        send(
                            &mut stream,
                            &format!("{tag} OK LOGOUT completed\r\n"),
                        );
                        break;
                    }

                    _ => {
                        send(&mut stream, &format!("{tag} OK\r\n"));
                    }
                }
            }
        }
    }
}

fn main() {
    dotenv().ok();

    let port = env::var("IMAP_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(1143);

    let listener = TcpListener::bind(("0.0.0.0", port)).unwrap();

    println!("Dummy IMAP listening on {}", port);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream);
                });
            }
            Err(e) => eprintln!("{e}"),
        }
    }
}


