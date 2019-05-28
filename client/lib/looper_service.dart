import 'dart:io';
import 'dart:isolate';

import 'package:grpc/grpc.dart';
import 'src/generated/loopers.pb.dart';
import 'src/generated/loopers.pbgrpc.dart';

const ip = "127.0.0.1";
const port = 10000;

class LooperService {
  bool _isShutdown = false;
  ClientChannel _channel;
  LooperClient _stub;
  ReceivePort _statePort;
  Stream<dynamic> _broadcastStream;

  LooperService() {
    _statePort = ReceivePort();
    Isolate.spawn(_stateIsolate, _statePort.sendPort);
    _broadcastStream = _statePort.asBroadcastStream();
    reset();
  }

  reset() {
    _channel = new ClientChannel(ip,
        port: port,
        options: const ChannelOptions(
            credentials: const ChannelCredentials.insecure()));

    _stub = new LooperClient(_channel,
        options: new CallOptions(timeout: new Duration(seconds: 5)));
  }

  Stream<State> getState() {
    print("get state called");
    return _broadcastStream.map((m) {
      var state = State.fromBuffer(m);
      return state;
    });
  }

  static void _stateIsolate(SendPort portReceive) async {
    ClientChannel channel;
    do {
      channel = new ClientChannel(ip,
          port: port,
          options: const ChannelOptions(
              idleTimeout: Duration(seconds: 5),
              credentials: const ChannelCredentials.insecure()));

      var stub = new LooperClient(channel,
              options: new CallOptions(timeout: new Duration(days: 1)))
          .getState(GetStateReq());

      try {
        await for (var message in stub) {
          portReceive.send(message.writeToBuffer());
        }
      } catch (e) {
        print("Got error $e");
        portReceive.send(null);
        channel.shutdown();
        channel = null;
      }

      sleep(Duration(seconds: 1));
    } while (true);
  }

  Future<CommandResp> sendGlobalCommand(GlobalCommandType type) {
    var globalCommand = GlobalCommand();
    globalCommand.command = type;

    var command = Command();
    command.globalCommand = globalCommand;

    return sendCommand(command);
  }

  Future<CommandResp> sendLooperCommand(int id, LooperCommandType type) {
    var looperCommand = LooperCommand();
    looperCommand.commandType = type;
    looperCommand.loopers.add(id);

    var command = Command();
    command.looperCommand = looperCommand;

    return sendCommand(command);
  }

  Future<CommandResp> sendCommand(Command command, {int retries = 3}) async {
    var req = CommandReq();
    req.command = command;
    try {
      return await _stub.command(req);
    } catch (e) {
      if (retries > 0) {
        _channel.shutdown();
        reset();
        return sendCommand(command, retries: retries - 1);
      } else {
        return Future.error(e);
      }
    }
  }
}