use crossbeam_queue::SegQueue;
use futures::sink::Sink;
use futures::sync::mpsc;
use futures::Future;
use futures::{future, Stream};
use loopers_common::protos::*;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::net::TcpListener;
use tower_grpc::{Request, Response};
use tower_hyper::server::Http;
use tower_hyper::Server;

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

        // let addr = "0.0.0.0:10000".parse().unwrap();
        let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 10000));
        let bind = TcpListener::bind(&addr).expect("bind");

        println!("listening on {:?}", addr);

        let state = self.state.clone();
        std::thread::spawn(move || {
            loop {
                let mut message = None;
                loop {
                    match state.input.pop() {
                        Ok(m) => {
                            message = Some(m);
                        }
                        Err(_) => {
                            break; // no more messages
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

                std::thread::sleep(Duration::from_millis(1000 / 10));
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
    type GetStateStream = Box<dyn Stream<Item = State, Error = tower_grpc::Status> + Send>;
    type GetStateFuture = future::FutureResult<Response<Self::GetStateStream>, tower_grpc::Status>;
    type CommandFuture = future::FutureResult<Response<CommandResp>, tower_grpc::Status>;

    fn get_state(&mut self, _request: Request<GetStateReq>) -> Self::GetStateFuture {
        //let input = self.input.clone();

        let (tx, rx) = mpsc::channel(4);

        self.channels.lock().unwrap().push(tx);

        let rx = rx.map_err(|_| unimplemented!());
        future::ok(Response::new(Box::new(rx)))
    }

    fn command(&mut self, request: Request<CommandReq>) -> Self::CommandFuture {
        if let Some(command) = request.into_inner().command {
            self.output.push(command);
        }

        future::ok(Response::new(CommandResp {
            status: CommandStatus::Accepted as i32,
        }))
    }
}