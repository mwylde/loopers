///
//  Generated code. Do not modify.
//  source: loopers.proto
//
// @dart = 2.3
// ignore_for_file: camel_case_types,non_constant_identifier_names,library_prefixes,unused_import,unused_shown_name,return_of_invalid_type

const LooperMode$json = const {
  '1': 'LooperMode',
  '2': const [
    const {'1': 'NONE', '2': 0},
    const {'1': 'READY', '2': 1},
    const {'1': 'RECORD', '2': 2},
    const {'1': 'OVERDUB', '2': 3},
    const {'1': 'PLAYING', '2': 4},
    const {'1': 'STOPPING', '2': 5},
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
    const {'1': 'STOP', '2': 0},
    const {'1': 'ENABLE_RECORD', '2': 1},
    const {'1': 'ENABLE_READY', '2': 2},
    const {'1': 'ENABLE_OVERDUB', '2': 3},
    const {'1': 'ENABLE_MUTIPLY', '2': 4},
    const {'1': 'ENABLE_PLAY', '2': 5},
    const {'1': 'SELECT', '2': 6},
    const {'1': 'DELETE', '2': 7},
    const {'1': 'READY_OVERDUB_PLAY', '2': 8},
  ],
};

const GlobalCommandType$json = const {
  '1': 'GlobalCommandType',
  '2': const [
    const {'1': 'RESET_TIME', '2': 0},
    const {'1': 'ADD_LOOPER', '2': 1},
    const {'1': 'ENABLE_LEARN_MODE', '2': 2},
    const {'1': 'DISABLE_LEARN_MODE', '2': 3},
  ],
};

const GetStateReq$json = const {
  '1': 'GetStateReq',
};

const LoopState$json = const {
  '1': 'LoopState',
  '2': const [
    const {'1': 'id', '3': 1, '4': 1, '5': 13, '10': 'id'},
    const {'1': 'mode', '3': 2, '4': 1, '5': 14, '6': '.protos.LooperMode', '10': 'mode'},
    const {'1': 'time', '3': 4, '4': 1, '5': 3, '10': 'time'},
    const {'1': 'length', '3': 5, '4': 1, '5': 3, '10': 'length'},
    const {'1': 'active', '3': 6, '4': 1, '5': 8, '10': 'active'},
  ],
};

const State$json = const {
  '1': 'State',
  '2': const [
    const {'1': 'loops', '3': 1, '4': 3, '5': 11, '6': '.protos.LoopState', '10': 'loops'},
    const {'1': 'time', '3': 2, '4': 1, '5': 3, '10': 'time'},
    const {'1': 'length', '3': 3, '4': 1, '5': 3, '10': 'length'},
    const {'1': 'beat', '3': 4, '4': 1, '5': 3, '10': 'beat'},
    const {'1': 'bpm', '3': 5, '4': 1, '5': 2, '10': 'bpm'},
    const {'1': 'time_signature_upper', '3': 6, '4': 1, '5': 4, '10': 'timeSignatureUpper'},
    const {'1': 'time_signature_lower', '3': 7, '4': 1, '5': 4, '10': 'timeSignatureLower'},
    const {'1': 'learn_mode', '3': 8, '4': 1, '5': 8, '10': 'learnMode'},
    const {'1': 'last_midi', '3': 9, '4': 1, '5': 12, '10': 'lastMidi'},
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

const TargetAll$json = const {
  '1': 'TargetAll',
};

const TargetSelected$json = const {
  '1': 'TargetSelected',
};

const TargetNumber$json = const {
  '1': 'TargetNumber',
  '2': const [
    const {'1': 'looper_number', '3': 1, '4': 1, '5': 13, '10': 'looperNumber'},
  ],
};

const LooperCommand$json = const {
  '1': 'LooperCommand',
  '2': const [
    const {'1': 'command_type', '3': 1, '4': 1, '5': 14, '6': '.protos.LooperCommandType', '10': 'commandType'},
    const {'1': 'target_all', '3': 2, '4': 1, '5': 11, '6': '.protos.TargetAll', '9': 0, '10': 'targetAll'},
    const {'1': 'target_selected', '3': 3, '4': 1, '5': 11, '6': '.protos.TargetSelected', '9': 0, '10': 'targetSelected'},
    const {'1': 'target_number', '3': 4, '4': 1, '5': 11, '6': '.protos.TargetNumber', '9': 0, '10': 'targetNumber'},
  ],
  '8': const [
    const {'1': 'target_oneof'},
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

const MidiMapping$json = const {
  '1': 'MidiMapping',
  '2': const [
    const {'1': 'controller_number', '3': 1, '4': 1, '5': 13, '10': 'controllerNumber'},
    const {'1': 'data', '3': 2, '4': 1, '5': 13, '10': 'data'},
    const {'1': 'command', '3': 3, '4': 1, '5': 11, '6': '.protos.Command', '10': 'command'},
  ],
};

const Config$json = const {
  '1': 'Config',
  '2': const [
    const {'1': 'midi_mappings', '3': 1, '4': 3, '5': 11, '6': '.protos.MidiMapping', '10': 'midiMappings'},
  ],
};

