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

