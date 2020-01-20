#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct GetStateReq {
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct LoopState {
    #[prost(uint32, tag="1")]
    pub id: u32,
    #[prost(enumeration="LooperMode", tag="2")]
    pub mode: i32,
    #[prost(int64, tag="4")]
    pub time: i64,
    #[prost(int64, tag="5")]
    pub length: i64,
    #[prost(bool, tag="6")]
    pub active: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct State {
    #[prost(message, repeated, tag="1")]
    pub loops: ::std::vec::Vec<LoopState>,
    #[prost(int64, tag="2")]
    pub time: i64,
    #[prost(int64, tag="3")]
    pub length: i64,
    #[prost(uint64, tag="4")]
    pub beat: u64,
    #[prost(float, tag="5")]
    pub bpm: f32,
    #[prost(uint64, tag="6")]
    pub time_signature_upper: u64,
    #[prost(uint64, tag="7")]
    pub time_signature_lower: u64,
    #[prost(bool, tag="8")]
    pub learn_mode: bool,
    #[prost(bytes, tag="9")]
    pub last_midi: std::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct CommandReq {
    #[prost(message, optional, tag="1")]
    pub command: ::std::option::Option<Command>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct CommandResp {
    #[prost(enumeration="CommandStatus", tag="1")]
    pub status: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct TargetAll {
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct TargetSelected {
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct TargetNumber {
    #[prost(uint32, tag="1")]
    pub looper_number: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct LooperCommand {
    #[prost(enumeration="LooperCommandType", tag="1")]
    pub command_type: i32,
    #[prost(oneof="looper_command::TargetOneof", tags="2, 3, 4")]
    pub target_oneof: ::std::option::Option<looper_command::TargetOneof>,
}
pub mod looper_command {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub enum TargetOneof {
        #[prost(message, tag="2")]
        TargetAll(super::TargetAll),
        #[prost(message, tag="3")]
        TargetSelected(super::TargetSelected),
        #[prost(message, tag="4")]
        TargetNumber(super::TargetNumber),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct GlobalCommand {
    #[prost(enumeration="GlobalCommandType", tag="1")]
    pub command: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Command {
    #[prost(oneof="command::CommandOneof", tags="1, 2")]
    pub command_oneof: ::std::option::Option<command::CommandOneof>,
}
pub mod command {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub enum CommandOneof {
        #[prost(message, tag="1")]
        LooperCommand(super::LooperCommand),
        #[prost(message, tag="2")]
        GlobalCommand(super::GlobalCommand),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct MidiMapping {
    #[prost(uint32, tag="1")]
    pub controller_number: u32,
    #[prost(uint32, tag="2")]
    pub data: u32,
    #[prost(message, optional, tag="3")]
    pub command: ::std::option::Option<Command>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Config {
    #[prost(message, repeated, tag="1")]
    pub midi_mappings: ::std::vec::Vec<MidiMapping>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum LooperMode {
    None = 0,
    Ready = 1,
    Record = 2,
    Overdub = 3,
    Playing = 4,
    Stopping = 5,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum CommandStatus {
    Accepted = 0,
    Failed = 1,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum LooperCommandType {
    Stop = 0,
    EnableRecord = 1,
    EnableReady = 2,
    EnableOverdub = 3,
    EnableMutiply = 4,
    EnablePlay = 5,
    Select = 6,
    Delete = 7,
    ReadyOverdubPlay = 8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum GlobalCommandType {
    ResetTime = 0,
    AddLooper = 1,
    EnableLearnMode = 2,
    DisableLearnMode = 3,
}
pub mod client {
    use ::tower_grpc::codegen::client::*;
    use super::{GetStateReq, State, CommandReq, CommandResp};

    #[derive(Debug, Clone)]
    pub struct Looper<T> {
        inner: grpc::Grpc<T>,
    }

    impl<T> Looper<T> {
        pub fn new(inner: T) -> Self {
            let inner = grpc::Grpc::new(inner);
            Self { inner }
        }

        /// Poll whether this client is ready to send another request.
        pub fn poll_ready<R>(&mut self) -> futures::Poll<(), grpc::Status>
        where T: grpc::GrpcService<R>,
        {
            self.inner.poll_ready()
        }

        /// Get a `Future` of when this client is ready to send another request.
        pub fn ready<R>(self) -> impl futures::Future<Item = Self, Error = grpc::Status>
        where T: grpc::GrpcService<R>,
        {
            futures::Future::map(self.inner.ready(), |inner| Self { inner })
        }

        pub fn get_state<R>(&mut self, request: grpc::Request<GetStateReq>) -> grpc::server_streaming::ResponseFuture<State, T::Future>
        where T: grpc::GrpcService<R>,
              grpc::unary::Once<GetStateReq>: grpc::Encodable<R>,
        {
            let path = http::PathAndQuery::from_static("/protos.Looper/GetState");
            self.inner.server_streaming(request, path)
        }

        pub fn command<R>(&mut self, request: grpc::Request<CommandReq>) -> grpc::unary::ResponseFuture<CommandResp, T::Future, T::ResponseBody>
        where T: grpc::GrpcService<R>,
              grpc::unary::Once<CommandReq>: grpc::Encodable<R>,
        {
            let path = http::PathAndQuery::from_static("/protos.Looper/Command");
            self.inner.unary(request, path)
        }
    }
}

pub mod server {
    use ::tower_grpc::codegen::server::*;
    use super::{GetStateReq, State, CommandReq, CommandResp};

    // Redefine the try_ready macro so that it doesn't need to be explicitly
    // imported by the user of this generated code.
    macro_rules! try_ready {
        ($e:expr) => (match $e {
            Ok(futures::Async::Ready(t)) => t,
            Ok(futures::Async::NotReady) => return Ok(futures::Async::NotReady),
            Err(e) => return Err(From::from(e)),
        })
    }

    pub trait Looper: Clone {
        type GetStateStream: futures::Stream<Item = State, Error = grpc::Status>;
        type GetStateFuture: futures::Future<Item = grpc::Response<Self::GetStateStream>, Error = grpc::Status>;
        type CommandFuture: futures::Future<Item = grpc::Response<CommandResp>, Error = grpc::Status>;

        fn get_state(&mut self, request: grpc::Request<GetStateReq>) -> Self::GetStateFuture;

        fn command(&mut self, request: grpc::Request<CommandReq>) -> Self::CommandFuture;
    }

    #[derive(Debug, Clone)]
    pub struct LooperServer<T> {
        looper: T,
    }

    impl<T> LooperServer<T>
    where T: Looper,
    {
        pub fn new(looper: T) -> Self {
            Self { looper }
        }
    }

    impl<T> tower::Service<http::Request<grpc::BoxBody>> for LooperServer<T>
    where T: Looper,
    {
        type Response = http::Response<looper::ResponseBody<T>>;
        type Error = grpc::Never;
        type Future = looper::ResponseFuture<T>;

        fn poll_ready(&mut self) -> futures::Poll<(), Self::Error> {
            Ok(().into())
        }

        fn call(&mut self, request: http::Request<grpc::BoxBody>) -> Self::Future {
            use self::looper::Kind::*;

            match request.uri().path() {
                "/protos.Looper/GetState" => {
                    let service = looper::methods::GetState(self.looper.clone());
                    let response = grpc::server_streaming(service, request);
                    looper::ResponseFuture { kind: GetState(response) }
                }
                "/protos.Looper/Command" => {
                    let service = looper::methods::Command(self.looper.clone());
                    let response = grpc::unary(service, request);
                    looper::ResponseFuture { kind: Command(response) }
                }
                _ => {
                    looper::ResponseFuture { kind: __Generated__Unimplemented(grpc::unimplemented(format!("unknown service: {:?}", request.uri().path()))) }
                }
            }
        }
    }

    impl<T> tower::Service<()> for LooperServer<T>
    where T: Looper,
    {
        type Response = Self;
        type Error = grpc::Never;
        type Future = futures::FutureResult<Self::Response, Self::Error>;

        fn poll_ready(&mut self) -> futures::Poll<(), Self::Error> {
            Ok(futures::Async::Ready(()))
        }

        fn call(&mut self, _target: ()) -> Self::Future {
            futures::ok(self.clone())
        }
    }

    impl<T> tower::Service<http::Request<tower_hyper::Body>> for LooperServer<T>
    where T: Looper,
    {
        type Response = <Self as tower::Service<http::Request<grpc::BoxBody>>>::Response;
        type Error = <Self as tower::Service<http::Request<grpc::BoxBody>>>::Error;
        type Future = <Self as tower::Service<http::Request<grpc::BoxBody>>>::Future;

        fn poll_ready(&mut self) -> futures::Poll<(), Self::Error> {
            tower::Service::<http::Request<grpc::BoxBody>>::poll_ready(self)
        }

        fn call(&mut self, request: http::Request<tower_hyper::Body>) -> Self::Future {
            let request = request.map(|b| grpc::BoxBody::map_from(b));
            tower::Service::<http::Request<grpc::BoxBody>>::call(self, request)
        }
    }

    pub mod looper {
        use ::tower_grpc::codegen::server::*;
        use super::Looper;
        use super::super::{GetStateReq, CommandReq};

        pub struct ResponseFuture<T>
        where T: Looper,
        {
            pub(super) kind: Kind<
                // GetState
                grpc::server_streaming::ResponseFuture<methods::GetState<T>, grpc::BoxBody, GetStateReq>,
                // Command
                grpc::unary::ResponseFuture<methods::Command<T>, grpc::BoxBody, CommandReq>,
                // A generated catch-all for unimplemented service calls
                grpc::unimplemented::ResponseFuture,
            >,
        }

        impl<T> futures::Future for ResponseFuture<T>
        where T: Looper,
        {
            type Item = http::Response<ResponseBody<T>>;
            type Error = grpc::Never;

            fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
                use self::Kind::*;

                match self.kind {
                    GetState(ref mut fut) => {
                        let response = try_ready!(fut.poll());
                        let response = response.map(|body| {
                            ResponseBody { kind: GetState(body) }
                        });
                        Ok(response.into())
                    }
                    Command(ref mut fut) => {
                        let response = try_ready!(fut.poll());
                        let response = response.map(|body| {
                            ResponseBody { kind: Command(body) }
                        });
                        Ok(response.into())
                    }
                    __Generated__Unimplemented(ref mut fut) => {
                        let response = try_ready!(fut.poll());
                        let response = response.map(|body| {
                            ResponseBody { kind: __Generated__Unimplemented(body) }
                        });
                        Ok(response.into())
                    }
                }
            }
        }

        pub struct ResponseBody<T>
        where T: Looper,
        {
            pub(super) kind: Kind<
                // GetState
                grpc::Encode<<methods::GetState<T> as grpc::ServerStreamingService<GetStateReq>>::ResponseStream>,
                // Command
                grpc::Encode<grpc::unary::Once<<methods::Command<T> as grpc::UnaryService<CommandReq>>::Response>>,
                // A generated catch-all for unimplemented service calls
                (),
            >,
        }

        impl<T> tower::HttpBody for ResponseBody<T>
        where T: Looper,
        {
            type Data = <grpc::BoxBody as grpc::Body>::Data;
            type Error = grpc::Status;

            fn is_end_stream(&self) -> bool {
                use self::Kind::*;

                match self.kind {
                    GetState(ref v) => v.is_end_stream(),
                    Command(ref v) => v.is_end_stream(),
                    __Generated__Unimplemented(_) => true,
                }
            }

            fn poll_data(&mut self) -> futures::Poll<Option<Self::Data>, Self::Error> {
                use self::Kind::*;

                match self.kind {
                    GetState(ref mut v) => v.poll_data(),
                    Command(ref mut v) => v.poll_data(),
                    __Generated__Unimplemented(_) => Ok(None.into()),
                }
            }

            fn poll_trailers(&mut self) -> futures::Poll<Option<http::HeaderMap>, Self::Error> {
                use self::Kind::*;

                match self.kind {
                    GetState(ref mut v) => v.poll_trailers(),
                    Command(ref mut v) => v.poll_trailers(),
                    __Generated__Unimplemented(_) => Ok(None.into()),
                }
            }
        }

        #[allow(non_camel_case_types)]
        #[derive(Debug, Clone)]
        pub(super) enum Kind<GetState, Command, __Generated__Unimplemented> {
            GetState(GetState),
            Command(Command),
            __Generated__Unimplemented(__Generated__Unimplemented),
        }

        pub mod methods {
            use ::tower_grpc::codegen::server::*;
            use super::super::{Looper, GetStateReq, CommandReq, CommandResp};

            pub struct GetState<T>(pub T);

            impl<T> tower::Service<grpc::Request<GetStateReq>> for GetState<T>
            where T: Looper,
            {
                type Response = grpc::Response<T::GetStateStream>;
                type Error = grpc::Status;
                type Future = T::GetStateFuture;

                fn poll_ready(&mut self) -> futures::Poll<(), Self::Error> {
                    Ok(futures::Async::Ready(()))
                }

                fn call(&mut self, request: grpc::Request<GetStateReq>) -> Self::Future {
                    self.0.get_state(request)
                }
            }

            pub struct Command<T>(pub T);

            impl<T> tower::Service<grpc::Request<CommandReq>> for Command<T>
            where T: Looper,
            {
                type Response = grpc::Response<CommandResp>;
                type Error = grpc::Status;
                type Future = T::CommandFuture;

                fn poll_ready(&mut self) -> futures::Poll<(), Self::Error> {
                    Ok(futures::Async::Ready(()))
                }

                fn call(&mut self, request: grpc::Request<CommandReq>) -> Self::Future {
                    self.0.command(request)
                }
            }
        }
    }
}
