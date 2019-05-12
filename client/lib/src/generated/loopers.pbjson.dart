///
//  Generated code. Do not modify.
//  source: loopers.proto
///
// ignore_for_file: non_constant_identifier_names,library_prefixes,unused_import

const RecordMode$json = const {
  '1': 'RecordMode',
  '2': const [
    const {'1': 'NONE', '2': 0},
    const {'1': 'READY', '2': 1},
    const {'1': 'RECORD', '2': 2},
    const {'1': 'OVERDUB', '2': 3},
  ],
};

const PlayMode$json = const {
  '1': 'PlayMode',
  '2': const [
    const {'1': 'PAUSED', '2': 0},
    const {'1': 'PLAYING', '2': 1},
  ],
};

const GetStateReq$json = const {
  '1': 'GetStateReq',
};

const LoopState$json = const {
  '1': 'LoopState',
  '2': const [
    const {'1': 'id', '3': 1, '4': 1, '5': 13, '10': 'id'},
    const {'1': 'recordMode', '3': 2, '4': 1, '5': 14, '6': '.protos.RecordMode', '10': 'recordMode'},
    const {'1': 'playMode', '3': 3, '4': 1, '5': 14, '6': '.protos.PlayMode', '10': 'playMode'},
    const {'1': 'time', '3': 4, '4': 1, '5': 3, '10': 'time'},
    const {'1': 'length', '3': 5, '4': 1, '5': 3, '10': 'length'},
    const {'1': 'active', '3': 6, '4': 1, '5': 8, '10': 'active'},
  ],
};

const State$json = const {
  '1': 'State',
  '2': const [
    const {'1': 'loops', '3': 1, '4': 3, '5': 11, '6': '.protos.LoopState', '10': 'loops'},
  ],
};

const Command$json = const {
  '1': 'Command',
};

