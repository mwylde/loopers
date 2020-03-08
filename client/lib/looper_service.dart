import 'dart:io';
import 'dart:isolate';

import 'package:grpc/grpc.dart';
import 'src/generated/loopers.pb.dart';
import 'src/generated/loopers.pbgrpc.dart';
import 'package:shared_preferences/shared_preferences.dart';

//const ip = "10.0.2.2"; // "127.0.0.1";
//const port = 10000;

class LooperService {
  bool _isShutdown = false;
  ClientChannel _channel;
  LooperClient _stub;
  ReceivePort _statePort;
  Stream<dynamic> _broadcastStream;
  String _host;
  int _port;

  bool _isLearning = false;

  LooperService() {
    _statePort = ReceivePort();
  }

  static Future<List> getHostPort() async {
    if (Platform.isAndroid || Platform.isIOS) {
      SharedPreferences prefs = await SharedPreferences.getInstance();
      if (prefs.containsKey("host")) {
        var cs = prefs.getString("host").split(":");
        return [cs[0], int.parse(cs[1])];
      } else {
        return ["192.168.0.107", 10000];
      }
    } else {
      return ["127.0.0.1", 10000];
    }
  }

  Future<void> start() async {
    var hostPort = await getHostPort();
    _host = hostPort[0];
    _port = hostPort[1];
    reset();
    Isolate.spawn(
        _stateIsolate, [_statePort.sendPort, hostPort[0], hostPort[1]]);
    _broadcastStream = _statePort.asBroadcastStream();
  }

  setLearning(bool enabled) {
    this._isLearning = enabled;
  }

  reset() {
    _channel = new ClientChannel(_host,
        port: _port,
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

  static void _stateIsolate(List args) async {
    SendPort portReceive = args[0];
    String host = args[1];
    int port = args[2];
    ClientChannel channel;
    do {
      channel = new ClientChannel(host,
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

  Future<CommandResp> sendLooperCommand(int index, LooperCommandType type) {
    var looperCommand = LooperCommand();
    looperCommand.commandType = type;
    looperCommand.targetNumber = TargetNumber();
    looperCommand.targetNumber.looperNumber = index;

    var command = Command();
    command.looperCommand = looperCommand;

    return sendCommand(command);
  }

  Future<CommandResp> sendSaveSessionCommand() {
    var saveCommand = SaveSessionCommand();
    saveCommand.path = "/home/mwylde/looper-sessions";
    var command = Command();
    command.saveSessionCommand = saveCommand;
    return sendCommand(command);
  }

  Future<CommandResp> sendLoadSessionCommand(String path) {
    var loadCommand = LoadSessionCommand();
    loadCommand.path = path;
    var command = Command();
    command.loadSessionCommand = loadCommand;
    return sendCommand(command);
  }

  Future<CommandResp> sendMetronomeVolume(double volume) {
    var metronomeCommand = MetronomeVolumeCommand();
    metronomeCommand.volume = volume;
    var command = Command();
    command.metronomeVolumeCommand = metronomeCommand;
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
