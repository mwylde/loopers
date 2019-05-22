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

const CommandStatus$json = const {
  '1': 'CommandStatus',
  '2': const [
    const {'1': 'ACCEPTED', '2': 0},
    const {'1': 'FAILED', '2': 1},
  ],
};

const LooperCommandType$json = const {
  '1': 'LooperCommandType',
  '2': const [
    const {'1': 'ENABLE_RECORD', '2': 0},
    const {'1': 'ENABLE_READY', '2': 10},
    const {'1': 'DISABLE_RECORD', '2': 1},
    const {'1': 'ENABLE_OVERDUB', '2': 2},
    const {'1': 'DISABLE_OVERDUB', '2': 3},
    const {'1': 'ENABLE_MUTIPLY', '2': 4},
    const {'1': 'DISABLE_MULTIPLY', '2': 5},
    const {'1': 'ENABLE_PLAY', '2': 6},
    const {'1': 'DISABLE_PLAY', '2': 7},
    const {'1': 'SELECT', '2': 8},
    const {'1': 'DELETE', '2': 9},
  ],
};

const GlobalCommandType$json = const {
  '1': 'GlobalCommandType',
  '2': const [
    const {'1': 'RESET_TIME', '2': 0},
    const {'1': 'ADD_LOOPER', '2': 1},
  ],
};

const GetStateReq$json = const {
  '1': 'GetStateReq',
};

const LoopState$json = const {
  '1': 'LoopState',
  '2': const [
    const {'1': 'id', '3': 1, '4': 1, '5': 13, '10': 'id'},
    const {'1': 'record_mode', '3': 2, '4': 1, '5': 14, '6': '.protos.RecordMode', '10': 'recordMode'},
    const {'1': 'play_mode', '3': 3, '4': 1, '5': 14, '6': '.protos.PlayMode', '10': 'playMode'},
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

const CommandReq$json = const {
  '1': 'CommandReq',
  '2': const [
    const {'1': 'command', '3': 1, '4': 1, '5': 11, '6': '.protos.Command', '10': 'command'},
  ],
};

const CommandResp$json = const {
  '1': 'CommandResp',
  '2': const [
    const {'1': 'status', '3': 1, '4': 1, '5': 14, '6': '.protos.CommandStatus', '10': 'status'},
  ],
};

const LooperCommand$json = const {
  '1': 'LooperCommand',
  '2': const [
    const {'1': 'command_type', '3': 1, '4': 1, '5': 14, '6': '.protos.LooperCommandType', '10': 'commandType'},
    const {'1': 'loopers', '3': 2, '4': 3, '5': 13, '10': 'loopers'},
  ],
};

const GlobalCommand$json = const {
  '1': 'GlobalCommand',
  '2': const [
    const {'1': 'command', '3': 1, '4': 1, '5': 14, '6': '.protos.GlobalCommandType', '10': 'command'},
  ],
};

const Command$json = const {
  '1': 'Command',
  '2': const [
    const {'1': 'looper_command', '3': 1, '4': 1, '5': 11, '6': '.protos.LooperCommand', '9': 0, '10': 'looperCommand'},
    const {'1': 'global_command', '3': 2, '4': 1, '5': 11, '6': '.protos.GlobalCommand', '9': 0, '10': 'globalCommand'},
  ],
  '8': const [
    const {'1': 'command_oneof'},
  ],
};

