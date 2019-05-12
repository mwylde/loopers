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
}

class Command extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = new $pb.BuilderInfo('Command', package: const $pb.PackageName('protos'))
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
}

