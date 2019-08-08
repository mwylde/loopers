///
//  Generated code. Do not modify.
//  source: loopers.proto
///
// ignore_for_file: non_constant_identifier_names,library_prefixes,unused_import

// ignore_for_file: UNDEFINED_SHOWN_NAME,UNUSED_SHOWN_NAME
import 'dart:core' show int, dynamic, String, List, Map;
import 'package:protobuf/protobuf.dart' as $pb;

class LooperMode extends $pb.ProtobufEnum {
  static const LooperMode NONE = const LooperMode._(0, 'NONE');
  static const LooperMode READY = const LooperMode._(1, 'READY');
  static const LooperMode RECORD = const LooperMode._(2, 'RECORD');
  static const LooperMode OVERDUB = const LooperMode._(3, 'OVERDUB');
  static const LooperMode PLAYING = const LooperMode._(4, 'PLAYING');

  static const List<LooperMode> values = const <LooperMode> [
    NONE,
    READY,
    RECORD,
    OVERDUB,
    PLAYING,
  ];

  static final Map<int, LooperMode> _byValue = $pb.ProtobufEnum.initByValue(values);
  static LooperMode valueOf(int value) => _byValue[value];
  static void $checkItem(LooperMode v) {
    if (v is! LooperMode) $pb.checkItemFailed(v, 'LooperMode');
  }

  const LooperMode._(int v, String n) : super(v, n);
}

class CommandStatus extends $pb.ProtobufEnum {
  static const CommandStatus ACCEPTED = const CommandStatus._(0, 'ACCEPTED');
  static const CommandStatus FAILED = const CommandStatus._(1, 'FAILED');

  static const List<CommandStatus> values = const <CommandStatus> [
    ACCEPTED,
    FAILED,
  ];

  static final Map<int, CommandStatus> _byValue = $pb.ProtobufEnum.initByValue(values);
  static CommandStatus valueOf(int value) => _byValue[value];
  static void $checkItem(CommandStatus v) {
    if (v is! CommandStatus) $pb.checkItemFailed(v, 'CommandStatus');
  }

  const CommandStatus._(int v, String n) : super(v, n);
}

class LooperCommandType extends $pb.ProtobufEnum {
  static const LooperCommandType STOP = const LooperCommandType._(0, 'STOP');
  static const LooperCommandType ENABLE_RECORD = const LooperCommandType._(1, 'ENABLE_RECORD');
  static const LooperCommandType ENABLE_READY = const LooperCommandType._(2, 'ENABLE_READY');
  static const LooperCommandType ENABLE_OVERDUB = const LooperCommandType._(3, 'ENABLE_OVERDUB');
  static const LooperCommandType ENABLE_MUTIPLY = const LooperCommandType._(4, 'ENABLE_MUTIPLY');
  static const LooperCommandType ENABLE_PLAY = const LooperCommandType._(5, 'ENABLE_PLAY');
  static const LooperCommandType SELECT = const LooperCommandType._(6, 'SELECT');
  static const LooperCommandType DELETE = const LooperCommandType._(7, 'DELETE');

  static const List<LooperCommandType> values = const <LooperCommandType> [
    STOP,
    ENABLE_RECORD,
    ENABLE_READY,
    ENABLE_OVERDUB,
    ENABLE_MUTIPLY,
    ENABLE_PLAY,
    SELECT,
    DELETE,
  ];

  static final Map<int, LooperCommandType> _byValue = $pb.ProtobufEnum.initByValue(values);
  static LooperCommandType valueOf(int value) => _byValue[value];
  static void $checkItem(LooperCommandType v) {
    if (v is! LooperCommandType) $pb.checkItemFailed(v, 'LooperCommandType');
  }

  const LooperCommandType._(int v, String n) : super(v, n);
}

class GlobalCommandType extends $pb.ProtobufEnum {
  static const GlobalCommandType RESET_TIME = const GlobalCommandType._(0, 'RESET_TIME');
  static const GlobalCommandType ADD_LOOPER = const GlobalCommandType._(1, 'ADD_LOOPER');

  static const List<GlobalCommandType> values = const <GlobalCommandType> [
    RESET_TIME,
    ADD_LOOPER,
  ];

  static final Map<int, GlobalCommandType> _byValue = $pb.ProtobufEnum.initByValue(values);
  static GlobalCommandType valueOf(int value) => _byValue[value];
  static void $checkItem(GlobalCommandType v) {
    if (v is! GlobalCommandType) $pb.checkItemFailed(v, 'GlobalCommandType');
  }

  const GlobalCommandType._(int v, String n) : super(v, n);
}

