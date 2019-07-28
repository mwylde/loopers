///
//  Generated code. Do not modify.
//  source: loopers.proto
///
// ignore_for_file: non_constant_identifier_names,library_prefixes,unused_import

// ignore: UNUSED_SHOWN_NAME
import 'dart:core' show int, bool, double, String, List, Map, override;

import 'package:fixnum/fixnum.dart';
import 'package:protobuf/protobuf.dart' as $pb;

import 'loopers.pbenum.dart';

export 'loopers.pbenum.dart';

class GetStateReq extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = new $pb.BuilderInfo('GetStateReq', package: const $pb.PackageName('protos'))
    ..hasRequiredFields = false
  ;

  GetStateReq() : super();
  GetStateReq.fromBuffer(List<int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) : super.fromBuffer(i, r);
  GetStateReq.fromJson(String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) : super.fromJson(i, r);
  GetStateReq clone() => new GetStateReq()..mergeFromMessage(this);
  GetStateReq copyWith(void Function(GetStateReq) updates) => super.copyWith((message) => updates(message as GetStateReq));
  $pb.BuilderInfo get info_ => _i;
  static GetStateReq create() => new GetStateReq();
  static $pb.PbList<GetStateReq> createRepeated() => new $pb.PbList<GetStateReq>();
  static GetStateReq getDefault() => _defaultInstance ??= create()..freeze();
  static GetStateReq _defaultInstance;
  static void $checkItem(GetStateReq v) {
    if (v is! GetStateReq) $pb.checkItemFailed(v, _i.qualifiedMessageName);
  }
}

class LoopState extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = new $pb.BuilderInfo('LoopState', package: const $pb.PackageName('protos'))
    ..a<int>(1, 'id', $pb.PbFieldType.OU3)
    ..e<RecordMode>(2, 'recordMode', $pb.PbFieldType.OE, RecordMode.NONE, RecordMode.valueOf, RecordMode.values)
    ..e<PlayMode>(3, 'playMode', $pb.PbFieldType.OE, PlayMode.PAUSED, PlayMode.valueOf, PlayMode.values)
    ..aInt64(4, 'time')
    ..aInt64(5, 'length')
    ..aOB(6, 'active')
    ..hasRequiredFields = false
  ;

  LoopState() : super();
  LoopState.fromBuffer(List<int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) : super.fromBuffer(i, r);
  LoopState.fromJson(String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) : super.fromJson(i, r);
  LoopState clone() => new LoopState()..mergeFromMessage(this);
  LoopState copyWith(void Function(LoopState) updates) => super.copyWith((message) => updates(message as LoopState));
  $pb.BuilderInfo get info_ => _i;
  static LoopState create() => new LoopState();
  static $pb.PbList<LoopState> createRepeated() => new $pb.PbList<LoopState>();
  static LoopState getDefault() => _defaultInstance ??= create()..freeze();
  static LoopState _defaultInstance;
  static void $checkItem(LoopState v) {
    if (v is! LoopState) $pb.checkItemFailed(v, _i.qualifiedMessageName);
  }

  int get id => $_get(0, 0);
  set id(int v) { $_setUnsignedInt32(0, v); }
  bool hasId() => $_has(0);
  void clearId() => clearField(1);

  RecordMode get recordMode => $_getN(1);
  set recordMode(RecordMode v) { setField(2, v); }
  bool hasRecordMode() => $_has(1);
  void clearRecordMode() => clearField(2);

  PlayMode get playMode => $_getN(2);
  set playMode(PlayMode v) { setField(3, v); }
  bool hasPlayMode() => $_has(2);
  void clearPlayMode() => clearField(3);

  Int64 get time => $_getI64(3);
  set time(Int64 v) { $_setInt64(3, v); }
  bool hasTime() => $_has(3);
  void clearTime() => clearField(4);

  Int64 get length => $_getI64(4);
  set length(Int64 v) { $_setInt64(4, v); }
  bool hasLength() => $_has(4);
  void clearLength() => clearField(5);

  bool get active => $_get(5, false);
  set active(bool v) { $_setBool(5, v); }
  bool hasActive() => $_has(5);
  void clearActive() => clearField(6);
}

class State extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = new $pb.BuilderInfo('State', package: const $pb.PackageName('protos'))
    ..pp<LoopState>(1, 'loops', $pb.PbFieldType.PM, LoopState.$checkItem, LoopState.create)
    ..aInt64(2, 'time')
    ..aInt64(3, 'length')
    ..a<Int64>(4, 'beat', $pb.PbFieldType.OU6, Int64.ZERO)
    ..hasRequiredFields = false
  ;

  State() : super();
  State.fromBuffer(List<int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) : super.fromBuffer(i, r);
  State.fromJson(String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) : super.fromJson(i, r);
  State clone() => new State()..mergeFromMessage(this);
  State copyWith(void Function(State) updates) => super.copyWith((message) => updates(message as State));
  $pb.BuilderInfo get info_ => _i;
  static State create() => new State();
  static $pb.PbList<State> createRepeated() => new $pb.PbList<State>();
  static State getDefault() => _defaultInstance ??= create()..freeze();
  static State _defaultInstance;
  static void $checkItem(State v) {
    if (v is! State) $pb.checkItemFailed(v, _i.qualifiedMessageName);
  }

  List<LoopState> get loops => $_getList(0);

  Int64 get time => $_getI64(1);
  set time(Int64 v) { $_setInt64(1, v); }
  bool hasTime() => $_has(1);
  void clearTime() => clearField(2);

  Int64 get length => $_getI64(2);
  set length(Int64 v) { $_setInt64(2, v); }
  bool hasLength() => $_has(2);
  void clearLength() => clearField(3);

  Int64 get beat => $_getI64(3);
  set beat(Int64 v) { $_setInt64(3, v); }
  bool hasBeat() => $_has(3);
  void clearBeat() => clearField(4);
}

class CommandReq extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = new $pb.BuilderInfo('CommandReq', package: const $pb.PackageName('protos'))
    ..a<Command>(1, 'command', $pb.PbFieldType.OM, Command.getDefault, Command.create)
    ..hasRequiredFields = false
  ;

  CommandReq() : super();
  CommandReq.fromBuffer(List<int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) : super.fromBuffer(i, r);
  CommandReq.fromJson(String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) : super.fromJson(i, r);
  CommandReq clone() => new CommandReq()..mergeFromMessage(this);
  CommandReq copyWith(void Function(CommandReq) updates) => super.copyWith((message) => updates(message as CommandReq));
  $pb.BuilderInfo get info_ => _i;
  static CommandReq create() => new CommandReq();
  static $pb.PbList<CommandReq> createRepeated() => new $pb.PbList<CommandReq>();
  static CommandReq getDefault() => _defaultInstance ??= create()..freeze();
  static CommandReq _defaultInstance;
  static void $checkItem(CommandReq v) {
    if (v is! CommandReq) $pb.checkItemFailed(v, _i.qualifiedMessageName);
  }

  Command get command => $_getN(0);
  set command(Command v) { setField(1, v); }
  bool hasCommand() => $_has(0);
  void clearCommand() => clearField(1);
}

class CommandResp extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = new $pb.BuilderInfo('CommandResp', package: const $pb.PackageName('protos'))
    ..e<CommandStatus>(1, 'status', $pb.PbFieldType.OE, CommandStatus.ACCEPTED, CommandStatus.valueOf, CommandStatus.values)
    ..hasRequiredFields = false
  ;

  CommandResp() : super();
  CommandResp.fromBuffer(List<int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) : super.fromBuffer(i, r);
  CommandResp.fromJson(String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) : super.fromJson(i, r);
  CommandResp clone() => new CommandResp()..mergeFromMessage(this);
  CommandResp copyWith(void Function(CommandResp) updates) => super.copyWith((message) => updates(message as CommandResp));
  $pb.BuilderInfo get info_ => _i;
  static CommandResp create() => new CommandResp();
  static $pb.PbList<CommandResp> createRepeated() => new $pb.PbList<CommandResp>();
  static CommandResp getDefault() => _defaultInstance ??= create()..freeze();
  static CommandResp _defaultInstance;
  static void $checkItem(CommandResp v) {
    if (v is! CommandResp) $pb.checkItemFailed(v, _i.qualifiedMessageName);
  }

  CommandStatus get status => $_getN(0);
  set status(CommandStatus v) { setField(1, v); }
  bool hasStatus() => $_has(0);
  void clearStatus() => clearField(1);
}

class LooperCommand extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = new $pb.BuilderInfo('LooperCommand', package: const $pb.PackageName('protos'))
    ..e<LooperCommandType>(1, 'commandType', $pb.PbFieldType.OE, LooperCommandType.ENABLE_RECORD, LooperCommandType.valueOf, LooperCommandType.values)
    ..p<int>(2, 'loopers', $pb.PbFieldType.PU3)
    ..hasRequiredFields = false
  ;

  LooperCommand() : super();
  LooperCommand.fromBuffer(List<int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) : super.fromBuffer(i, r);
  LooperCommand.fromJson(String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) : super.fromJson(i, r);
  LooperCommand clone() => new LooperCommand()..mergeFromMessage(this);
  LooperCommand copyWith(void Function(LooperCommand) updates) => super.copyWith((message) => updates(message as LooperCommand));
  $pb.BuilderInfo get info_ => _i;
  static LooperCommand create() => new LooperCommand();
  static $pb.PbList<LooperCommand> createRepeated() => new $pb.PbList<LooperCommand>();
  static LooperCommand getDefault() => _defaultInstance ??= create()..freeze();
  static LooperCommand _defaultInstance;
  static void $checkItem(LooperCommand v) {
    if (v is! LooperCommand) $pb.checkItemFailed(v, _i.qualifiedMessageName);
  }

  LooperCommandType get commandType => $_getN(0);
  set commandType(LooperCommandType v) { setField(1, v); }
  bool hasCommandType() => $_has(0);
  void clearCommandType() => clearField(1);

  List<int> get loopers => $_getList(1);
}

class GlobalCommand extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = new $pb.BuilderInfo('GlobalCommand', package: const $pb.PackageName('protos'))
    ..e<GlobalCommandType>(1, 'command', $pb.PbFieldType.OE, GlobalCommandType.RESET_TIME, GlobalCommandType.valueOf, GlobalCommandType.values)
    ..hasRequiredFields = false
  ;

  GlobalCommand() : super();
  GlobalCommand.fromBuffer(List<int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) : super.fromBuffer(i, r);
  GlobalCommand.fromJson(String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) : super.fromJson(i, r);
  GlobalCommand clone() => new GlobalCommand()..mergeFromMessage(this);
  GlobalCommand copyWith(void Function(GlobalCommand) updates) => super.copyWith((message) => updates(message as GlobalCommand));
  $pb.BuilderInfo get info_ => _i;
  static GlobalCommand create() => new GlobalCommand();
  static $pb.PbList<GlobalCommand> createRepeated() => new $pb.PbList<GlobalCommand>();
  static GlobalCommand getDefault() => _defaultInstance ??= create()..freeze();
  static GlobalCommand _defaultInstance;
  static void $checkItem(GlobalCommand v) {
    if (v is! GlobalCommand) $pb.checkItemFailed(v, _i.qualifiedMessageName);
  }

  GlobalCommandType get command => $_getN(0);
  set command(GlobalCommandType v) { setField(1, v); }
  bool hasCommand() => $_has(0);
  void clearCommand() => clearField(1);
}

class Command extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = new $pb.BuilderInfo('Command', package: const $pb.PackageName('protos'))
    ..a<LooperCommand>(1, 'looperCommand', $pb.PbFieldType.OM, LooperCommand.getDefault, LooperCommand.create)
    ..a<GlobalCommand>(2, 'globalCommand', $pb.PbFieldType.OM, GlobalCommand.getDefault, GlobalCommand.create)
    ..hasRequiredFields = false
  ;

  Command() : super();
  Command.fromBuffer(List<int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) : super.fromBuffer(i, r);
  Command.fromJson(String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) : super.fromJson(i, r);
  Command clone() => new Command()..mergeFromMessage(this);
  Command copyWith(void Function(Command) updates) => super.copyWith((message) => updates(message as Command));
  $pb.BuilderInfo get info_ => _i;
  static Command create() => new Command();
  static $pb.PbList<Command> createRepeated() => new $pb.PbList<Command>();
  static Command getDefault() => _defaultInstance ??= create()..freeze();
  static Command _defaultInstance;
  static void $checkItem(Command v) {
    if (v is! Command) $pb.checkItemFailed(v, _i.qualifiedMessageName);
  }

  LooperCommand get looperCommand => $_getN(0);
  set looperCommand(LooperCommand v) { setField(1, v); }
  bool hasLooperCommand() => $_has(0);
  void clearLooperCommand() => clearField(1);

  GlobalCommand get globalCommand => $_getN(1);
  set globalCommand(GlobalCommand v) { setField(2, v); }
  bool hasGlobalCommand() => $_has(1);
  void clearGlobalCommand() => clearField(2);
}

