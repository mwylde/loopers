///
//  Generated code. Do not modify.
//  source: loopers.proto
//
// @dart = 2.3
// ignore_for_file: camel_case_types,non_constant_identifier_names,library_prefixes,unused_import,unused_shown_name,return_of_invalid_type

import 'dart:async' as $async;

import 'dart:core' as $core;

import 'package:grpc/service_api.dart' as $grpc;
import 'loopers.pb.dart' as $0;
export 'loopers.pb.dart';

class LooperClient extends $grpc.Client {
  static final _$getState = $grpc.ClientMethod<$0.GetStateReq, $0.State>(
      '/protos.Looper/GetState',
      ($0.GetStateReq value) => value.writeToBuffer(),
      ($core.List<$core.int> value) => $0.State.fromBuffer(value));
  static final _$command = $grpc.ClientMethod<$0.CommandReq, $0.CommandResp>(
      '/protos.Looper/Command',
      ($0.CommandReq value) => value.writeToBuffer(),
      ($core.List<$core.int> value) => $0.CommandResp.fromBuffer(value));

  LooperClient($grpc.ClientChannel channel, {$grpc.CallOptions options})
      : super(channel, options: options);

  $grpc.ResponseStream<$0.State> getState($0.GetStateReq request,
      {$grpc.CallOptions options}) {
    final call = $createCall(_$getState, $async.Stream.fromIterable([request]),
        options: options);
    return $grpc.ResponseStream(call);
  }

  $grpc.ResponseFuture<$0.CommandResp> command($0.CommandReq request,
      {$grpc.CallOptions options}) {
    final call = $createCall(_$command, $async.Stream.fromIterable([request]),
        options: options);
    return $grpc.ResponseFuture(call);
  }
}

abstract class LooperServiceBase extends $grpc.Service {
  $core.String get $name => 'protos.Looper';

  LooperServiceBase() {
    $addMethod($grpc.ServiceMethod<$0.GetStateReq, $0.State>(
        'GetState',
        getState_Pre,
        false,
        true,
        ($core.List<$core.int> value) => $0.GetStateReq.fromBuffer(value),
        ($0.State value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.CommandReq, $0.CommandResp>(
        'Command',
        command_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.CommandReq.fromBuffer(value),
        ($0.CommandResp value) => value.writeToBuffer()));
  }

  $async.Stream<$0.State> getState_Pre(
      $grpc.ServiceCall call, $async.Future<$0.GetStateReq> request) async* {
    yield* getState(call, await request);
  }

  $async.Future<$0.CommandResp> command_Pre(
      $grpc.ServiceCall call, $async.Future<$0.CommandReq> request) async {
    return command(call, await request);
  }

  $async.Stream<$0.State> getState(
      $grpc.ServiceCall call, $0.GetStateReq request);
  $async.Future<$0.CommandResp> command(
      $grpc.ServiceCall call, $0.CommandReq request);
}
