use std::time::Duration;
use crossbeam_queue::SegQueue;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use futures::{Stream, future};
use tower_grpc::{Response, Request};
use futures::sync::mpsc;
use futures::sink::Sink;
use crate::protos::*;
use tower_hyper::Server;
use tower_hyper::server::Http;
use tokio::net::TcpListener;
use futures::Future;

#[derive(Clone)]
pub struct Gui {
    state: GuiState,
}


#[derive(Clone)]
struct GuiState {
    active: u128,
    loopers: Vec<LoopState>,
    input: Arc<SegQueue<State>>,
    output: Arc<SegQueue<Command>>,
    channels: Arc<Mutex<Vec<mpsc::Sender<State>>>>,
}

impl Gui {
    pub fn new() -> (Gui, Arc<SegQueue<State>>, Arc<SegQueue<Command>>) {
        let input = Arc::new(SegQueue::new());
        let output = Arc::new(SegQueue::new());
        let gui = Gui {
            state: GuiState {
                active: 0,
                loopers: vec![],
                input: input.clone(),
                output: output.clone(),
                channels: Arc::new(Mutex::new(vec![])),
            },
        };
        (gui, input, output)
    }

    pub fn run(&self) {
        let new_service = server::LooperServer::new(self.state.clone());

        let mut server = Server::new(new_service);
        let http = Http::new().http2_only(true).clone();

        let addr = "127.0.0.1:10000".parse().unwrap();
        let bind = TcpListener::bind(&addr).expect("bind");

        println!("listening on {:?}", addr);

        let state = self.state.clone();
        std::thread::spawn(move||{
            loop {
                let mut message = None;
                loop {
                    match state.input.pop() {
                        Ok(m) => {
                            message = Some(m);
                        }
                        Err(_) => {
                            break // no more messages
                        }
                    }
                }

                if let Some(m) = message {
                    let mut channels = state.channels.lock().unwrap();

                    channels.retain(|tx| {
                        let mut tx = tx.clone().wait();
                        if let Err(err) = tx.send(m.clone()) {
                            eprintln!("error sending to channel {:?}", err);
                            false
                        } else {
                            true
                        }
                    });
                }

                std::thread::sleep(Duration::from_millis(100));
            }
        });

        let serve = bind
            .incoming()
            .for_each(move |sock| {
                if let Err(e) = sock.set_nodelay(true) {
                    return Err(e);
                }

                let serve = server.serve_with(sock, http.clone());
                tokio::spawn(serve.map_err(|e| eprintln!("h2 error: {:?}", e)));

                Ok(())
            })
            .map_err(|e| eprintln!("accept error: {}", e));

        tokio::run(serve);

    }
}

impl server::Looper for GuiState {
    type GetStateStream = Box<Stream<Item = State, Error = tower_grpc::Status> + Send>;
    type GetStateFuture = future::FutureResult<Response<Self::GetStateStream>, tower_grpc::Status>;

    fn get_state(&mut self, request: Request<GetStateReq>) -> Self::GetStateFuture {
        let input = self.input.clone();

        let (tx, rx) = mpsc::channel(4);

        self.channels.lock().unwrap().push(tx);

        let rx = rx.map_err(|_| unimplemented!());
        future::ok(Response::new(Box::new(rx)))
    }
}
