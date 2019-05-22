import 'package:grpc/grpc.dart';
import 'src/generated/loopers.pb.dart';
import 'src/generated/loopers.pbgrpc.dart';

const ip = "127.0.0.1";
const port = 10000;

class LooperService {
  bool _isShutdown = false;
  ClientChannel _channel;
  LooperClient _stub;

  LooperService() {
    _channel = new ClientChannel(ip,
        port: port,
        options: const ChannelOptions(
            credentials: const ChannelCredentials.insecure()));

    _stub = new LooperClient(_channel,
        options: new CallOptions(timeout: new Duration(minutes: 10)));
  }

  Stream<State> getState() {
    return _stub.getState(GetStateReq()).map((f) {
      return f;
    });
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

  Future<CommandResp> sendCommand(Command command) {
    var req = CommandReq();
    req.command = command;
    return _stub.command(req);
  }
}
