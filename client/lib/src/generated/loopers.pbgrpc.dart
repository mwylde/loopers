///
//  Generated code. Do not modify.
//  source: loopers.proto
///
// ignore_for_file: non_constant_identifier_names,library_prefixes,unused_import

import 'dart:async' as $async;

import 'package:grpc/grpc.dart';

import 'loopers.pb.dart';
export 'loopers.pb.dart';

class LooperClient extends Client {
  static final _$getState = new ClientMethod<GetStateReq, State>(
      '/protos.Looper/GetState',
      (GetStateReq value) => value.writeToBuffer(),
      (List<int> value) => new State.fromBuffer(value));

  LooperClient(ClientChannel channel, {CallOptions options})
      : super(channel, options: options);

  ResponseStream<State> getState(GetStateReq request, {CallOptions options}) {
    final call = $createCall(
        _$getState, new $async.Stream.fromIterable([request]),
        options: options);
    return new ResponseStream(call);
  }
}

abstract class LooperServiceBase extends Service {
  String get $name => 'protos.Looper';

  LooperServiceBase() {
    $addMethod(new ServiceMethod<GetStateReq, State>(
        'GetState',
        getState_Pre,
        false,
        true,
        (List<int> value) => new GetStateReq.fromBuffer(value),
        (State value) => value.writeToBuffer()));
  }

  $async.Stream<State> getState_Pre(
      ServiceCall call, $async.Future request) async* {
    yield* getState(call, (await request) as GetStateReq);
  }

  $async.Stream<State> getState(ServiceCall call, GetStateReq request);
}
