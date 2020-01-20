///
//  Generated code. Do not modify.
//  source: loopers.proto
//
// @dart = 2.3
// ignore_for_file: camel_case_types,non_constant_identifier_names,library_prefixes,unused_import,unused_shown_name,return_of_invalid_type

// ignore_for_file: UNDEFINED_SHOWN_NAME,UNUSED_SHOWN_NAME
import 'dart:core' as $core;
import 'package:protobuf/protobuf.dart' as $pb;

class LooperMode extends $pb.ProtobufEnum {
  static const LooperMode NONE = LooperMode._(0, 'NONE');
  static const LooperMode READY = LooperMode._(1, 'READY');
  static const LooperMode RECORD = LooperMode._(2, 'RECORD');
  static const LooperMode OVERDUB = LooperMode._(3, 'OVERDUB');
  static const LooperMode PLAYING = LooperMode._(4, 'PLAYING');
  static const LooperMode STOPPING = LooperMode._(5, 'STOPPING');

  static const $core.List<LooperMode> values = <LooperMode> [
    NONE,
    READY,
    RECORD,
    OVERDUB,
    PLAYING,
    STOPPING,
  ];

  static final $core.Map<$core.int, LooperMode> _byValue = $pb.ProtobufEnum.initByValue(values);
  static LooperMode valueOf($core.int value) => _byValue[value];

  const LooperMode._($core.int v, $core.String n) : super(v, n);
}

class CommandStatus extends $pb.ProtobufEnum {
  static const CommandStatus ACCEPTED = CommandStatus._(0, 'ACCEPTED');
  static const CommandStatus FAILED = CommandStatus._(1, 'FAILED');

  static const $core.List<CommandStatus> values = <CommandStatus> [
    ACCEPTED,
    FAILED,
  ];

  static final $core.Map<$core.int, CommandStatus> _byValue = $pb.ProtobufEnum.initByValue(values);
  static CommandStatus valueOf($core.int value) => _byValue[value];

  const CommandStatus._($core.int v, $core.String n) : super(v, n);
}

class LooperCommandType extends $pb.ProtobufEnum {
  static const LooperCommandType STOP = LooperCommandType._(0, 'STOP');
  static const LooperCommandType ENABLE_RECORD = LooperCommandType._(1, 'ENABLE_RECORD');
  static const LooperCommandType ENABLE_READY = LooperCommandType._(2, 'ENABLE_READY');
  static const LooperCommandType ENABLE_OVERDUB = LooperCommandType._(3, 'ENABLE_OVERDUB');
  static const LooperCommandType ENABLE_MUTIPLY = LooperCommandType._(4, 'ENABLE_MUTIPLY');
  static const LooperCommandType ENABLE_PLAY = LooperCommandType._(5, 'ENABLE_PLAY');
  static const LooperCommandType SELECT = LooperCommandType._(6, 'SELECT');
  static const LooperCommandType DELETE = LooperCommandType._(7, 'DELETE');

  static const $core.List<LooperCommandType> values = <LooperCommandType> [
    STOP,
    ENABLE_RECORD,
    ENABLE_READY,
    ENABLE_OVERDUB,
    ENABLE_MUTIPLY,
    ENABLE_PLAY,
    SELECT,
    DELETE,
  ];

  static final $core.Map<$core.int, LooperCommandType> _byValue = $pb.ProtobufEnum.initByValue(values);
  static LooperCommandType valueOf($core.int value) => _byValue[value];

  const LooperCommandType._($core.int v, $core.String n) : super(v, n);
}

class GlobalCommandType extends $pb.ProtobufEnum {
  static const GlobalCommandType RESET_TIME = GlobalCommandType._(0, 'RESET_TIME');
  static const GlobalCommandType ADD_LOOPER = GlobalCommandType._(1, 'ADD_LOOPER');
  static const GlobalCommandType ENABLE_LEARN_MODE = GlobalCommandType._(2, 'ENABLE_LEARN_MODE');
  static const GlobalCommandType DISABLE_LEARN_MODE = GlobalCommandType._(3, 'DISABLE_LEARN_MODE');

  static const $core.List<GlobalCommandType> values = <GlobalCommandType> [
    RESET_TIME,
    ADD_LOOPER,
    ENABLE_LEARN_MODE,
    DISABLE_LEARN_MODE,
  ];

  static final $core.Map<$core.int, GlobalCommandType> _byValue = $pb.ProtobufEnum.initByValue(values);
  static GlobalCommandType valueOf($core.int value) => _byValue[value];

  const GlobalCommandType._($core.int v, $core.String n) : super(v, n);
}

