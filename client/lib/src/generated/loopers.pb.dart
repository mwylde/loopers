///
//  Generated code. Do not modify.
//  source: loopers.proto
//
// @dart = 2.3
// ignore_for_file: camel_case_types,non_constant_identifier_names,library_prefixes,unused_import,unused_shown_name,return_of_invalid_type

import 'dart:core' as $core;

import 'package:fixnum/fixnum.dart' as $fixnum;
import 'package:protobuf/protobuf.dart' as $pb;

import 'loopers.pbenum.dart';

export 'loopers.pbenum.dart';

class GetStateReq extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo('GetStateReq', package: const $pb.PackageName('protos'), createEmptyInstance: create)
    ..hasRequiredFields = false
  ;

  GetStateReq._() : super();
  factory GetStateReq() => create();
  factory GetStateReq.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory GetStateReq.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  GetStateReq clone() => GetStateReq()..mergeFromMessage(this);
  GetStateReq copyWith(void Function(GetStateReq) updates) => super.copyWith((message) => updates(message as GetStateReq));
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static GetStateReq create() => GetStateReq._();
  GetStateReq createEmptyInstance() => create();
  static $pb.PbList<GetStateReq> createRepeated() => $pb.PbList<GetStateReq>();
  @$core.pragma('dart2js:noInline')
  static GetStateReq getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<GetStateReq>(create);
  static GetStateReq _defaultInstance;
}

class LoopState extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo('LoopState', package: const $pb.PackageName('protos'), createEmptyInstance: create)
    ..a<$core.int>(1, 'id', $pb.PbFieldType.OU3)
    ..e<LooperMode>(2, 'mode', $pb.PbFieldType.OE, defaultOrMaker: LooperMode.NONE, valueOf: LooperMode.valueOf, enumValues: LooperMode.values)
    ..aInt64(4, 'time')
    ..aInt64(5, 'length')
    ..aOB(6, 'active')
    ..hasRequiredFields = false
  ;

  LoopState._() : super();
  factory LoopState() => create();
  factory LoopState.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory LoopState.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  LoopState clone() => LoopState()..mergeFromMessage(this);
  LoopState copyWith(void Function(LoopState) updates) => super.copyWith((message) => updates(message as LoopState));
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static LoopState create() => LoopState._();
  LoopState createEmptyInstance() => create();
  static $pb.PbList<LoopState> createRepeated() => $pb.PbList<LoopState>();
  @$core.pragma('dart2js:noInline')
  static LoopState getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<LoopState>(create);
  static LoopState _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get id => $_getIZ(0);
  @$pb.TagNumber(1)
  set id($core.int v) { $_setUnsignedInt32(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => clearField(1);

  @$pb.TagNumber(2)
  LooperMode get mode => $_getN(1);
  @$pb.TagNumber(2)
  set mode(LooperMode v) { setField(2, v); }
  @$pb.TagNumber(2)
  $core.bool hasMode() => $_has(1);
  @$pb.TagNumber(2)
  void clearMode() => clearField(2);

  @$pb.TagNumber(4)
  $fixnum.Int64 get time => $_getI64(2);
  @$pb.TagNumber(4)
  set time($fixnum.Int64 v) { $_setInt64(2, v); }
  @$pb.TagNumber(4)
  $core.bool hasTime() => $_has(2);
  @$pb.TagNumber(4)
  void clearTime() => clearField(4);

  @$pb.TagNumber(5)
  $fixnum.Int64 get length => $_getI64(3);
  @$pb.TagNumber(5)
  set length($fixnum.Int64 v) { $_setInt64(3, v); }
  @$pb.TagNumber(5)
  $core.bool hasLength() => $_has(3);
  @$pb.TagNumber(5)
  void clearLength() => clearField(5);

  @$pb.TagNumber(6)
  $core.bool get active => $_getBF(4);
  @$pb.TagNumber(6)
  set active($core.bool v) { $_setBool(4, v); }
  @$pb.TagNumber(6)
  $core.bool hasActive() => $_has(4);
  @$pb.TagNumber(6)
  void clearActive() => clearField(6);
}

class State extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo('State', package: const $pb.PackageName('protos'), createEmptyInstance: create)
    ..pc<LoopState>(1, 'loops', $pb.PbFieldType.PM, subBuilder: LoopState.create)
    ..aInt64(2, 'time')
    ..aInt64(3, 'length')
    ..aInt64(4, 'beat')
    ..a<$core.double>(5, 'bpm', $pb.PbFieldType.OF)
    ..a<$fixnum.Int64>(6, 'timeSignatureUpper', $pb.PbFieldType.OU6, defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(7, 'timeSignatureLower', $pb.PbFieldType.OU6, defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOB(8, 'learnMode')
    ..a<$core.List<$core.int>>(9, 'lastMidi', $pb.PbFieldType.OY)
    ..hasRequiredFields = false
  ;

  State._() : super();
  factory State() => create();
  factory State.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory State.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  State clone() => State()..mergeFromMessage(this);
  State copyWith(void Function(State) updates) => super.copyWith((message) => updates(message as State));
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static State create() => State._();
  State createEmptyInstance() => create();
  static $pb.PbList<State> createRepeated() => $pb.PbList<State>();
  @$core.pragma('dart2js:noInline')
  static State getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<State>(create);
  static State _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<LoopState> get loops => $_getList(0);

  @$pb.TagNumber(2)
  $fixnum.Int64 get time => $_getI64(1);
  @$pb.TagNumber(2)
  set time($fixnum.Int64 v) { $_setInt64(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasTime() => $_has(1);
  @$pb.TagNumber(2)
  void clearTime() => clearField(2);

  @$pb.TagNumber(3)
  $fixnum.Int64 get length => $_getI64(2);
  @$pb.TagNumber(3)
  set length($fixnum.Int64 v) { $_setInt64(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasLength() => $_has(2);
  @$pb.TagNumber(3)
  void clearLength() => clearField(3);

  @$pb.TagNumber(4)
  $fixnum.Int64 get beat => $_getI64(3);
  @$pb.TagNumber(4)
  set beat($fixnum.Int64 v) { $_setInt64(3, v); }
  @$pb.TagNumber(4)
  $core.bool hasBeat() => $_has(3);
  @$pb.TagNumber(4)
  void clearBeat() => clearField(4);

  @$pb.TagNumber(5)
  $core.double get bpm => $_getN(4);
  @$pb.TagNumber(5)
  set bpm($core.double v) { $_setFloat(4, v); }
  @$pb.TagNumber(5)
  $core.bool hasBpm() => $_has(4);
  @$pb.TagNumber(5)
  void clearBpm() => clearField(5);

  @$pb.TagNumber(6)
  $fixnum.Int64 get timeSignatureUpper => $_getI64(5);
  @$pb.TagNumber(6)
  set timeSignatureUpper($fixnum.Int64 v) { $_setInt64(5, v); }
  @$pb.TagNumber(6)
  $core.bool hasTimeSignatureUpper() => $_has(5);
  @$pb.TagNumber(6)
  void clearTimeSignatureUpper() => clearField(6);

  @$pb.TagNumber(7)
  $fixnum.Int64 get timeSignatureLower => $_getI64(6);
  @$pb.TagNumber(7)
  set timeSignatureLower($fixnum.Int64 v) { $_setInt64(6, v); }
  @$pb.TagNumber(7)
  $core.bool hasTimeSignatureLower() => $_has(6);
  @$pb.TagNumber(7)
  void clearTimeSignatureLower() => clearField(7);

  @$pb.TagNumber(8)
  $core.bool get learnMode => $_getBF(7);
  @$pb.TagNumber(8)
  set learnMode($core.bool v) { $_setBool(7, v); }
  @$pb.TagNumber(8)
  $core.bool hasLearnMode() => $_has(7);
  @$pb.TagNumber(8)
  void clearLearnMode() => clearField(8);

  @$pb.TagNumber(9)
  $core.List<$core.int> get lastMidi => $_getN(8);
  @$pb.TagNumber(9)
  set lastMidi($core.List<$core.int> v) { $_setBytes(8, v); }
  @$pb.TagNumber(9)
  $core.bool hasLastMidi() => $_has(8);
  @$pb.TagNumber(9)
  void clearLastMidi() => clearField(9);
}

class CommandReq extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo('CommandReq', package: const $pb.PackageName('protos'), createEmptyInstance: create)
    ..aOM<Command>(1, 'command', subBuilder: Command.create)
    ..hasRequiredFields = false
  ;

  CommandReq._() : super();
  factory CommandReq() => create();
  factory CommandReq.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory CommandReq.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  CommandReq clone() => CommandReq()..mergeFromMessage(this);
  CommandReq copyWith(void Function(CommandReq) updates) => super.copyWith((message) => updates(message as CommandReq));
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static CommandReq create() => CommandReq._();
  CommandReq createEmptyInstance() => create();
  static $pb.PbList<CommandReq> createRepeated() => $pb.PbList<CommandReq>();
  @$core.pragma('dart2js:noInline')
  static CommandReq getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<CommandReq>(create);
  static CommandReq _defaultInstance;

  @$pb.TagNumber(1)
  Command get command => $_getN(0);
  @$pb.TagNumber(1)
  set command(Command v) { setField(1, v); }
  @$pb.TagNumber(1)
  $core.bool hasCommand() => $_has(0);
  @$pb.TagNumber(1)
  void clearCommand() => clearField(1);
  @$pb.TagNumber(1)
  Command ensureCommand() => $_ensure(0);
}

class CommandResp extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo('CommandResp', package: const $pb.PackageName('protos'), createEmptyInstance: create)
    ..e<CommandStatus>(1, 'status', $pb.PbFieldType.OE, defaultOrMaker: CommandStatus.ACCEPTED, valueOf: CommandStatus.valueOf, enumValues: CommandStatus.values)
    ..hasRequiredFields = false
  ;

  CommandResp._() : super();
  factory CommandResp() => create();
  factory CommandResp.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory CommandResp.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  CommandResp clone() => CommandResp()..mergeFromMessage(this);
  CommandResp copyWith(void Function(CommandResp) updates) => super.copyWith((message) => updates(message as CommandResp));
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static CommandResp create() => CommandResp._();
  CommandResp createEmptyInstance() => create();
  static $pb.PbList<CommandResp> createRepeated() => $pb.PbList<CommandResp>();
  @$core.pragma('dart2js:noInline')
  static CommandResp getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<CommandResp>(create);
  static CommandResp _defaultInstance;

  @$pb.TagNumber(1)
  CommandStatus get status => $_getN(0);
  @$pb.TagNumber(1)
  set status(CommandStatus v) { setField(1, v); }
  @$pb.TagNumber(1)
  $core.bool hasStatus() => $_has(0);
  @$pb.TagNumber(1)
  void clearStatus() => clearField(1);
}

class TargetAll extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo('TargetAll', package: const $pb.PackageName('protos'), createEmptyInstance: create)
    ..hasRequiredFields = false
  ;

  TargetAll._() : super();
  factory TargetAll() => create();
  factory TargetAll.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory TargetAll.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  TargetAll clone() => TargetAll()..mergeFromMessage(this);
  TargetAll copyWith(void Function(TargetAll) updates) => super.copyWith((message) => updates(message as TargetAll));
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static TargetAll create() => TargetAll._();
  TargetAll createEmptyInstance() => create();
  static $pb.PbList<TargetAll> createRepeated() => $pb.PbList<TargetAll>();
  @$core.pragma('dart2js:noInline')
  static TargetAll getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<TargetAll>(create);
  static TargetAll _defaultInstance;
}

class TargetSelected extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo('TargetSelected', package: const $pb.PackageName('protos'), createEmptyInstance: create)
    ..hasRequiredFields = false
  ;

  TargetSelected._() : super();
  factory TargetSelected() => create();
  factory TargetSelected.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory TargetSelected.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  TargetSelected clone() => TargetSelected()..mergeFromMessage(this);
  TargetSelected copyWith(void Function(TargetSelected) updates) => super.copyWith((message) => updates(message as TargetSelected));
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static TargetSelected create() => TargetSelected._();
  TargetSelected createEmptyInstance() => create();
  static $pb.PbList<TargetSelected> createRepeated() => $pb.PbList<TargetSelected>();
  @$core.pragma('dart2js:noInline')
  static TargetSelected getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<TargetSelected>(create);
  static TargetSelected _defaultInstance;
}

class TargetNumber extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo('TargetNumber', package: const $pb.PackageName('protos'), createEmptyInstance: create)
    ..a<$core.int>(1, 'looperNumber', $pb.PbFieldType.OU3)
    ..hasRequiredFields = false
  ;

  TargetNumber._() : super();
  factory TargetNumber() => create();
  factory TargetNumber.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory TargetNumber.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  TargetNumber clone() => TargetNumber()..mergeFromMessage(this);
  TargetNumber copyWith(void Function(TargetNumber) updates) => super.copyWith((message) => updates(message as TargetNumber));
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static TargetNumber create() => TargetNumber._();
  TargetNumber createEmptyInstance() => create();
  static $pb.PbList<TargetNumber> createRepeated() => $pb.PbList<TargetNumber>();
  @$core.pragma('dart2js:noInline')
  static TargetNumber getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<TargetNumber>(create);
  static TargetNumber _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get looperNumber => $_getIZ(0);
  @$pb.TagNumber(1)
  set looperNumber($core.int v) { $_setUnsignedInt32(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasLooperNumber() => $_has(0);
  @$pb.TagNumber(1)
  void clearLooperNumber() => clearField(1);
}

enum LooperCommand_TargetOneof {
  targetAll, 
  targetSelected, 
  targetNumber, 
  notSet
}

class LooperCommand extends $pb.GeneratedMessage {
  static const $core.Map<$core.int, LooperCommand_TargetOneof> _LooperCommand_TargetOneofByTag = {
    2 : LooperCommand_TargetOneof.targetAll,
    3 : LooperCommand_TargetOneof.targetSelected,
    4 : LooperCommand_TargetOneof.targetNumber,
    0 : LooperCommand_TargetOneof.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo('LooperCommand', package: const $pb.PackageName('protos'), createEmptyInstance: create)
    ..oo(0, [2, 3, 4])
    ..e<LooperCommandType>(1, 'commandType', $pb.PbFieldType.OE, defaultOrMaker: LooperCommandType.STOP, valueOf: LooperCommandType.valueOf, enumValues: LooperCommandType.values)
    ..aOM<TargetAll>(2, 'targetAll', subBuilder: TargetAll.create)
    ..aOM<TargetSelected>(3, 'targetSelected', subBuilder: TargetSelected.create)
    ..aOM<TargetNumber>(4, 'targetNumber', subBuilder: TargetNumber.create)
    ..hasRequiredFields = false
  ;

  LooperCommand._() : super();
  factory LooperCommand() => create();
  factory LooperCommand.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory LooperCommand.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  LooperCommand clone() => LooperCommand()..mergeFromMessage(this);
  LooperCommand copyWith(void Function(LooperCommand) updates) => super.copyWith((message) => updates(message as LooperCommand));
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static LooperCommand create() => LooperCommand._();
  LooperCommand createEmptyInstance() => create();
  static $pb.PbList<LooperCommand> createRepeated() => $pb.PbList<LooperCommand>();
  @$core.pragma('dart2js:noInline')
  static LooperCommand getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<LooperCommand>(create);
  static LooperCommand _defaultInstance;

  LooperCommand_TargetOneof whichTargetOneof() => _LooperCommand_TargetOneofByTag[$_whichOneof(0)];
  void clearTargetOneof() => clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  LooperCommandType get commandType => $_getN(0);
  @$pb.TagNumber(1)
  set commandType(LooperCommandType v) { setField(1, v); }
  @$pb.TagNumber(1)
  $core.bool hasCommandType() => $_has(0);
  @$pb.TagNumber(1)
  void clearCommandType() => clearField(1);

  @$pb.TagNumber(2)
  TargetAll get targetAll => $_getN(1);
  @$pb.TagNumber(2)
  set targetAll(TargetAll v) { setField(2, v); }
  @$pb.TagNumber(2)
  $core.bool hasTargetAll() => $_has(1);
  @$pb.TagNumber(2)
  void clearTargetAll() => clearField(2);
  @$pb.TagNumber(2)
  TargetAll ensureTargetAll() => $_ensure(1);

  @$pb.TagNumber(3)
  TargetSelected get targetSelected => $_getN(2);
  @$pb.TagNumber(3)
  set targetSelected(TargetSelected v) { setField(3, v); }
  @$pb.TagNumber(3)
  $core.bool hasTargetSelected() => $_has(2);
  @$pb.TagNumber(3)
  void clearTargetSelected() => clearField(3);
  @$pb.TagNumber(3)
  TargetSelected ensureTargetSelected() => $_ensure(2);

  @$pb.TagNumber(4)
  TargetNumber get targetNumber => $_getN(3);
  @$pb.TagNumber(4)
  set targetNumber(TargetNumber v) { setField(4, v); }
  @$pb.TagNumber(4)
  $core.bool hasTargetNumber() => $_has(3);
  @$pb.TagNumber(4)
  void clearTargetNumber() => clearField(4);
  @$pb.TagNumber(4)
  TargetNumber ensureTargetNumber() => $_ensure(3);
}

class GlobalCommand extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo('GlobalCommand', package: const $pb.PackageName('protos'), createEmptyInstance: create)
    ..e<GlobalCommandType>(1, 'command', $pb.PbFieldType.OE, defaultOrMaker: GlobalCommandType.RESET_TIME, valueOf: GlobalCommandType.valueOf, enumValues: GlobalCommandType.values)
    ..hasRequiredFields = false
  ;

  GlobalCommand._() : super();
  factory GlobalCommand() => create();
  factory GlobalCommand.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory GlobalCommand.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  GlobalCommand clone() => GlobalCommand()..mergeFromMessage(this);
  GlobalCommand copyWith(void Function(GlobalCommand) updates) => super.copyWith((message) => updates(message as GlobalCommand));
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static GlobalCommand create() => GlobalCommand._();
  GlobalCommand createEmptyInstance() => create();
  static $pb.PbList<GlobalCommand> createRepeated() => $pb.PbList<GlobalCommand>();
  @$core.pragma('dart2js:noInline')
  static GlobalCommand getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<GlobalCommand>(create);
  static GlobalCommand _defaultInstance;

  @$pb.TagNumber(1)
  GlobalCommandType get command => $_getN(0);
  @$pb.TagNumber(1)
  set command(GlobalCommandType v) { setField(1, v); }
  @$pb.TagNumber(1)
  $core.bool hasCommand() => $_has(0);
  @$pb.TagNumber(1)
  void clearCommand() => clearField(1);
}

class SaveSessionCommand extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo('SaveSessionCommand', package: const $pb.PackageName('protos'), createEmptyInstance: create)
    ..aOS(1, 'path')
    ..hasRequiredFields = false
  ;

  SaveSessionCommand._() : super();
  factory SaveSessionCommand() => create();
  factory SaveSessionCommand.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory SaveSessionCommand.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  SaveSessionCommand clone() => SaveSessionCommand()..mergeFromMessage(this);
  SaveSessionCommand copyWith(void Function(SaveSessionCommand) updates) => super.copyWith((message) => updates(message as SaveSessionCommand));
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static SaveSessionCommand create() => SaveSessionCommand._();
  SaveSessionCommand createEmptyInstance() => create();
  static $pb.PbList<SaveSessionCommand> createRepeated() => $pb.PbList<SaveSessionCommand>();
  @$core.pragma('dart2js:noInline')
  static SaveSessionCommand getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<SaveSessionCommand>(create);
  static SaveSessionCommand _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get path => $_getSZ(0);
  @$pb.TagNumber(1)
  set path($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasPath() => $_has(0);
  @$pb.TagNumber(1)
  void clearPath() => clearField(1);
}

class LoadSessionCommand extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo('LoadSessionCommand', package: const $pb.PackageName('protos'), createEmptyInstance: create)
    ..aOS(1, 'path')
    ..hasRequiredFields = false
  ;

  LoadSessionCommand._() : super();
  factory LoadSessionCommand() => create();
  factory LoadSessionCommand.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory LoadSessionCommand.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  LoadSessionCommand clone() => LoadSessionCommand()..mergeFromMessage(this);
  LoadSessionCommand copyWith(void Function(LoadSessionCommand) updates) => super.copyWith((message) => updates(message as LoadSessionCommand));
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static LoadSessionCommand create() => LoadSessionCommand._();
  LoadSessionCommand createEmptyInstance() => create();
  static $pb.PbList<LoadSessionCommand> createRepeated() => $pb.PbList<LoadSessionCommand>();
  @$core.pragma('dart2js:noInline')
  static LoadSessionCommand getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<LoadSessionCommand>(create);
  static LoadSessionCommand _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get path => $_getSZ(0);
  @$pb.TagNumber(1)
  set path($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasPath() => $_has(0);
  @$pb.TagNumber(1)
  void clearPath() => clearField(1);
}

enum Command_CommandOneof {
  looperCommand, 
  globalCommand, 
  saveSessionCommand, 
  loadSessionCommand, 
  notSet
}

class Command extends $pb.GeneratedMessage {
  static const $core.Map<$core.int, Command_CommandOneof> _Command_CommandOneofByTag = {
    1 : Command_CommandOneof.looperCommand,
    2 : Command_CommandOneof.globalCommand,
    3 : Command_CommandOneof.saveSessionCommand,
    4 : Command_CommandOneof.loadSessionCommand,
    0 : Command_CommandOneof.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo('Command', package: const $pb.PackageName('protos'), createEmptyInstance: create)
    ..oo(0, [1, 2, 3, 4])
    ..aOM<LooperCommand>(1, 'looperCommand', subBuilder: LooperCommand.create)
    ..aOM<GlobalCommand>(2, 'globalCommand', subBuilder: GlobalCommand.create)
    ..aOM<SaveSessionCommand>(3, 'saveSessionCommand', subBuilder: SaveSessionCommand.create)
    ..aOM<LoadSessionCommand>(4, 'loadSessionCommand', subBuilder: LoadSessionCommand.create)
    ..hasRequiredFields = false
  ;

  Command._() : super();
  factory Command() => create();
  factory Command.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory Command.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  Command clone() => Command()..mergeFromMessage(this);
  Command copyWith(void Function(Command) updates) => super.copyWith((message) => updates(message as Command));
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static Command create() => Command._();
  Command createEmptyInstance() => create();
  static $pb.PbList<Command> createRepeated() => $pb.PbList<Command>();
  @$core.pragma('dart2js:noInline')
  static Command getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Command>(create);
  static Command _defaultInstance;

  Command_CommandOneof whichCommandOneof() => _Command_CommandOneofByTag[$_whichOneof(0)];
  void clearCommandOneof() => clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  LooperCommand get looperCommand => $_getN(0);
  @$pb.TagNumber(1)
  set looperCommand(LooperCommand v) { setField(1, v); }
  @$pb.TagNumber(1)
  $core.bool hasLooperCommand() => $_has(0);
  @$pb.TagNumber(1)
  void clearLooperCommand() => clearField(1);
  @$pb.TagNumber(1)
  LooperCommand ensureLooperCommand() => $_ensure(0);

  @$pb.TagNumber(2)
  GlobalCommand get globalCommand => $_getN(1);
  @$pb.TagNumber(2)
  set globalCommand(GlobalCommand v) { setField(2, v); }
  @$pb.TagNumber(2)
  $core.bool hasGlobalCommand() => $_has(1);
  @$pb.TagNumber(2)
  void clearGlobalCommand() => clearField(2);
  @$pb.TagNumber(2)
  GlobalCommand ensureGlobalCommand() => $_ensure(1);

  @$pb.TagNumber(3)
  SaveSessionCommand get saveSessionCommand => $_getN(2);
  @$pb.TagNumber(3)
  set saveSessionCommand(SaveSessionCommand v) { setField(3, v); }
  @$pb.TagNumber(3)
  $core.bool hasSaveSessionCommand() => $_has(2);
  @$pb.TagNumber(3)
  void clearSaveSessionCommand() => clearField(3);
  @$pb.TagNumber(3)
  SaveSessionCommand ensureSaveSessionCommand() => $_ensure(2);

  @$pb.TagNumber(4)
  LoadSessionCommand get loadSessionCommand => $_getN(3);
  @$pb.TagNumber(4)
  set loadSessionCommand(LoadSessionCommand v) { setField(4, v); }
  @$pb.TagNumber(4)
  $core.bool hasLoadSessionCommand() => $_has(3);
  @$pb.TagNumber(4)
  void clearLoadSessionCommand() => clearField(4);
  @$pb.TagNumber(4)
  LoadSessionCommand ensureLoadSessionCommand() => $_ensure(3);
}

class MidiMapping extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo('MidiMapping', package: const $pb.PackageName('protos'), createEmptyInstance: create)
    ..a<$core.int>(1, 'controllerNumber', $pb.PbFieldType.OU3)
    ..a<$core.int>(2, 'data', $pb.PbFieldType.OU3)
    ..aOM<Command>(3, 'command', subBuilder: Command.create)
    ..hasRequiredFields = false
  ;

  MidiMapping._() : super();
  factory MidiMapping() => create();
  factory MidiMapping.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory MidiMapping.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  MidiMapping clone() => MidiMapping()..mergeFromMessage(this);
  MidiMapping copyWith(void Function(MidiMapping) updates) => super.copyWith((message) => updates(message as MidiMapping));
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static MidiMapping create() => MidiMapping._();
  MidiMapping createEmptyInstance() => create();
  static $pb.PbList<MidiMapping> createRepeated() => $pb.PbList<MidiMapping>();
  @$core.pragma('dart2js:noInline')
  static MidiMapping getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<MidiMapping>(create);
  static MidiMapping _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get controllerNumber => $_getIZ(0);
  @$pb.TagNumber(1)
  set controllerNumber($core.int v) { $_setUnsignedInt32(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasControllerNumber() => $_has(0);
  @$pb.TagNumber(1)
  void clearControllerNumber() => clearField(1);

  @$pb.TagNumber(2)
  $core.int get data => $_getIZ(1);
  @$pb.TagNumber(2)
  set data($core.int v) { $_setUnsignedInt32(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasData() => $_has(1);
  @$pb.TagNumber(2)
  void clearData() => clearField(2);

  @$pb.TagNumber(3)
  Command get command => $_getN(2);
  @$pb.TagNumber(3)
  set command(Command v) { setField(3, v); }
  @$pb.TagNumber(3)
  $core.bool hasCommand() => $_has(2);
  @$pb.TagNumber(3)
  void clearCommand() => clearField(3);
  @$pb.TagNumber(3)
  Command ensureCommand() => $_ensure(2);
}

class Config extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo('Config', package: const $pb.PackageName('protos'), createEmptyInstance: create)
    ..pc<MidiMapping>(1, 'midiMappings', $pb.PbFieldType.PM, subBuilder: MidiMapping.create)
    ..hasRequiredFields = false
  ;

  Config._() : super();
  factory Config() => create();
  factory Config.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory Config.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  Config clone() => Config()..mergeFromMessage(this);
  Config copyWith(void Function(Config) updates) => super.copyWith((message) => updates(message as Config));
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static Config create() => Config._();
  Config createEmptyInstance() => create();
  static $pb.PbList<Config> createRepeated() => $pb.PbList<Config>();
  @$core.pragma('dart2js:noInline')
  static Config getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Config>(create);
  static Config _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<MidiMapping> get midiMappings => $_getList(0);
}

class SavedLooper extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo('SavedLooper', package: const $pb.PackageName('protos'), createEmptyInstance: create)
    ..a<$core.int>(1, 'id', $pb.PbFieldType.OU3)
    ..pPS(2, 'samples')
    ..hasRequiredFields = false
  ;

  SavedLooper._() : super();
  factory SavedLooper() => create();
  factory SavedLooper.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory SavedLooper.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  SavedLooper clone() => SavedLooper()..mergeFromMessage(this);
  SavedLooper copyWith(void Function(SavedLooper) updates) => super.copyWith((message) => updates(message as SavedLooper));
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static SavedLooper create() => SavedLooper._();
  SavedLooper createEmptyInstance() => create();
  static $pb.PbList<SavedLooper> createRepeated() => $pb.PbList<SavedLooper>();
  @$core.pragma('dart2js:noInline')
  static SavedLooper getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<SavedLooper>(create);
  static SavedLooper _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get id => $_getIZ(0);
  @$pb.TagNumber(1)
  set id($core.int v) { $_setUnsignedInt32(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => clearField(1);

  @$pb.TagNumber(2)
  $core.List<$core.String> get samples => $_getList(1);
}

class SavedSession extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo('SavedSession', package: const $pb.PackageName('protos'), createEmptyInstance: create)
    ..aInt64(1, 'saveTime')
    ..a<$fixnum.Int64>(2, 'timeSignatureUpper', $pb.PbFieldType.OU6, defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(3, 'timeSignatureLower', $pb.PbFieldType.OU6, defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(4, 'tempoMbpm', $pb.PbFieldType.OU6, defaultOrMaker: $fixnum.Int64.ZERO)
    ..pc<SavedLooper>(5, 'loopers', $pb.PbFieldType.PM, subBuilder: SavedLooper.create)
    ..hasRequiredFields = false
  ;

  SavedSession._() : super();
  factory SavedSession() => create();
  factory SavedSession.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory SavedSession.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  SavedSession clone() => SavedSession()..mergeFromMessage(this);
  SavedSession copyWith(void Function(SavedSession) updates) => super.copyWith((message) => updates(message as SavedSession));
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static SavedSession create() => SavedSession._();
  SavedSession createEmptyInstance() => create();
  static $pb.PbList<SavedSession> createRepeated() => $pb.PbList<SavedSession>();
  @$core.pragma('dart2js:noInline')
  static SavedSession getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<SavedSession>(create);
  static SavedSession _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get saveTime => $_getI64(0);
  @$pb.TagNumber(1)
  set saveTime($fixnum.Int64 v) { $_setInt64(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasSaveTime() => $_has(0);
  @$pb.TagNumber(1)
  void clearSaveTime() => clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get timeSignatureUpper => $_getI64(1);
  @$pb.TagNumber(2)
  set timeSignatureUpper($fixnum.Int64 v) { $_setInt64(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasTimeSignatureUpper() => $_has(1);
  @$pb.TagNumber(2)
  void clearTimeSignatureUpper() => clearField(2);

  @$pb.TagNumber(3)
  $fixnum.Int64 get timeSignatureLower => $_getI64(2);
  @$pb.TagNumber(3)
  set timeSignatureLower($fixnum.Int64 v) { $_setInt64(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasTimeSignatureLower() => $_has(2);
  @$pb.TagNumber(3)
  void clearTimeSignatureLower() => clearField(3);

  @$pb.TagNumber(4)
  $fixnum.Int64 get tempoMbpm => $_getI64(3);
  @$pb.TagNumber(4)
  set tempoMbpm($fixnum.Int64 v) { $_setInt64(3, v); }
  @$pb.TagNumber(4)
  $core.bool hasTempoMbpm() => $_has(3);
  @$pb.TagNumber(4)
  void clearTempoMbpm() => clearField(4);

  @$pb.TagNumber(5)
  $core.List<SavedLooper> get loopers => $_getList(4);
}

