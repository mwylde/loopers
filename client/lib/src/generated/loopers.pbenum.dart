///
//  Generated code. Do not modify.
//  source: loopers.proto
///
// ignore_for_file: non_constant_identifier_names,library_prefixes,unused_import

// ignore_for_file: UNDEFINED_SHOWN_NAME,UNUSED_SHOWN_NAME
import 'dart:core' show int, dynamic, String, List, Map;
import 'package:protobuf/protobuf.dart' as $pb;

class RecordMode extends $pb.ProtobufEnum {
  static const RecordMode NONE = const RecordMode._(0, 'NONE');
  static const RecordMode READY = const RecordMode._(1, 'READY');
  static const RecordMode RECORD = const RecordMode._(2, 'RECORD');
  static const RecordMode OVERDUB = const RecordMode._(3, 'OVERDUB');

  static const List<RecordMode> values = const <RecordMode> [
    NONE,
    READY,
    RECORD,
    OVERDUB,
  ];

  static final Map<int, RecordMode> _byValue = $pb.ProtobufEnum.initByValue(values);
  static RecordMode valueOf(int value) => _byValue[value];
  static void $checkItem(RecordMode v) {
    if (v is! RecordMode) $pb.checkItemFailed(v, 'RecordMode');
  }

  const RecordMode._(int v, String n) : super(v, n);
}

class PlayMode extends $pb.ProtobufEnum {
  static const PlayMode PAUSED = const PlayMode._(0, 'PAUSED');
  static const PlayMode PLAYING = const PlayMode._(1, 'PLAYING');

  static const List<PlayMode> values = const <PlayMode> [
    PAUSED,
    PLAYING,
  ];

  static final Map<int, PlayMode> _byValue = $pb.ProtobufEnum.initByValue(values);
  static PlayMode valueOf(int value) => _byValue[value];
  static void $checkItem(PlayMode v) {
    if (v is! PlayMode) $pb.checkItemFailed(v, 'PlayMode');
  }

  const PlayMode._(int v, String n) : super(v, n);
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
  static const LooperCommandType ENABLE_RECORD = const LooperCommandType._(0, 'ENABLE_RECORD');
  static const LooperCommandType ENABLE_READY = const LooperCommandType._(10, 'ENABLE_READY');
  static const LooperCommandType DISABLE_RECORD = const LooperCommandType._(1, 'DISABLE_RECORD');
  static const LooperCommandType ENABLE_OVERDUB = const LooperCommandType._(2, 'ENABLE_OVERDUB');
  static const LooperCommandType DISABLE_OVERDUB = const LooperCommandType._(3, 'DISABLE_OVERDUB');
  static const LooperCommandType ENABLE_MUTIPLY = const LooperCommandType._(4, 'ENABLE_MUTIPLY');
  static const LooperCommandType DISABLE_MULTIPLY = const LooperCommandType._(5, 'DISABLE_MULTIPLY');
  static const LooperCommandType ENABLE_PLAY = const LooperCommandType._(6, 'ENABLE_PLAY');
  static const LooperCommandType DISABLE_PLAY = const LooperCommandType._(7, 'DISABLE_PLAY');
  static const LooperCommandType SELECT = const LooperCommandType._(8, 'SELECT');
  static const LooperCommandType DELETE = const LooperCommandType._(9, 'DELETE');

  static const List<LooperCommandType> values = const <LooperCommandType> [
    ENABLE_RECORD,
    ENABLE_READY,
    DISABLE_RECORD,
    ENABLE_OVERDUB,
    DISABLE_OVERDUB,
    ENABLE_MUTIPLY,
    DISABLE_MULTIPLY,
    ENABLE_PLAY,
    DISABLE_PLAY,
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

