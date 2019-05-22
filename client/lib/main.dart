import 'dart:io' show Platform;

import 'package:fixnum/fixnum.dart';
import 'package:flutter/foundation.dart'
    show debugDefaultTargetPlatformOverride;
import 'package:flutter/material.dart';
import 'package:loopers/looper_service.dart';
import 'src/generated/loopers.pb.dart' as protos;
import 'src/generated/loopers.pbgrpc.dart' as grpc;

void main() {
  // See https://github.com/flutter/flutter/wiki/Desktop-shells#target-platform-override
  debugDefaultTargetPlatformOverride = TargetPlatform.fuchsia;

  var testState = protos.State();
  var l1 = protos.LoopState();
  l1.id = 0;
  l1.length = Int64(120 * 1000);
  l1.time = Int64(50 * 1000);
  l1.recordMode = protos.RecordMode.READY;

  var l2 = protos.LoopState();
  l2.id = 1;
  l2.length = Int64(60 * 1000);
  l2.time = Int64(50 * 1000);
  l2.playMode = protos.PlayMode.PLAYING;
  l2.active = true;

  testState.loops.add(l1);
  testState.loops.add(l2);

  runApp(new MyApp(
    service: LooperService(),
  ));
}

class MyApp extends StatelessWidget {
  MyApp({this.service});

  // final protos.State state;
  final LooperService service;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Loopers',
      theme: ThemeData.dark(),
//      theme: ThemeData(
//        primarySwatch: Colors.blue,
//        // See https://github.com/flutter/flutter/wiki/Desktop-shells#fonts
//        fontFamily: 'Roboto',
//      ),

      home: MainPage(service: service),
    );
  }
}

class MainPage extends StatefulWidget {
  MainPage({this.service});

  final LooperService service;

  @override
  State<StatefulWidget> createState() {
    return MainPageState();
  }
}

class MainPageState extends State<MainPage> {
  @override
  Widget build(BuildContext context) {
    return new StreamBuilder<protos.State>(
        stream: widget.service.getState(),
        builder: (context, snapshot) {
          Widget loopers = Text("Could not connect to server");

          if (snapshot.data != null) {
            loopers = Column(
                children: snapshot.data.loops.map((f) {
              return LooperWidget(state: f, service: widget.service);
            }).toList());
          }

          return Scaffold(
            body: Column(children: [
              TimeWidget(state: snapshot.data),
              Container(child: loopers)
            ]),
            floatingActionButton: FloatingActionButton(
              tooltip: 'New Looper',
              child: Icon(Icons.add),
              onPressed: () {
                widget.service
                    .sendGlobalCommand(protos.GlobalCommandType.ADD_LOOPER);
              },
            ),
          );
        });
  }
}

class TimeWidget extends StatelessWidget {
  final protos.State state;

  const TimeWidget({Key key, this.state}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    if (state != null) {
      return Container(
        height: 100,
        child: Text("Time"),
      );
    } else {
      return Container(
        height: 100,
      );
    }
  }
}

class LooperButton extends StatelessWidget {
  final String text;
  final bool active;
  final bool primed;
  final Null Function() onPressed;

  const LooperButton(
      {Key key,
      this.text,
      this.active,
      this.primed = false,
      this.onPressed = null})
      : super(key: key);

  @override
  Widget build(BuildContext context) {
    var color =
        active ? Colors.red[400] : primed ? Colors.brown : Colors.black26;

    Widget button = FlatButton(
      color: color,
      onPressed: onPressed,
      child: Text(text),
    );

    return Container(
        padding: EdgeInsets.symmetric(horizontal: 4.0), child: button);
  }
}

class LooperWidget extends StatelessWidget {
  LooperWidget({this.state, this.service});

  final protos.LoopState state;
  final LooperService service;

  @override
  Widget build(BuildContext context) {
    var value = state.length.isZero
        ? 0.0
        : state.time.toDouble() / state.length.toDouble();

    var color = state.active ? Colors.black26 : Theme.of(context).cardColor;

    return InkWell(
        onTap: () {
          service.sendLooperCommand(state.id, protos.LooperCommandType.SELECT);
        },
        child: Container(
            //height: 120,
            padding: const EdgeInsets.all(8.0),
            decoration: BoxDecoration(
                border: Border(
                    bottom: BorderSide(color: Theme.of(context).dividerColor)),
                color: color),
            child: Column(
              mainAxisAlignment: MainAxisAlignment.spaceAround,
              children: <Widget>[
//            Container(
//              width: double.infinity,
//              padding: const EdgeInsets.all(8.0),
//              child: Text(
//                state.id.toString(),
//                textAlign: TextAlign.left,
//              ),
//            ),
                LinearProgressIndicator(
                  value: value,
                  semanticsLabel: "progress",
                  semanticsValue: "$value seconds",
                ),
                Row(
                  children: <Widget>[
                    LooperButton(
                        text: "RECORD",
                        active: state.recordMode == protos.RecordMode.RECORD,
                        primed: state.recordMode == protos.RecordMode.READY,
                        onPressed: () {
                          if (state.recordMode == protos.RecordMode.READY ||
                              state.recordMode == protos.RecordMode.RECORD) {
                            service.sendLooperCommand(state.id,
                                protos.LooperCommandType.DISABLE_RECORD);
                          } else {
                            service.sendLooperCommand(state.id,
                                protos.LooperCommandType.ENABLE_RECORD);
                          }
                        }),
                    LooperButton(
                      text: "OVERDUB",
                      active: state.recordMode == protos.RecordMode.OVERDUB,
                      onPressed: () {
                        if (state.recordMode == protos.RecordMode.OVERDUB) {
                          service.sendLooperCommand(state.id,
                              protos.LooperCommandType.DISABLE_OVERDUB);
                        } else {
                          service.sendLooperCommand(state.id,
                              protos.LooperCommandType.ENABLE_OVERDUB);
                        }
                      },
                    ),
                    LooperButton(
                      text: "MULTIPLY",
                      active: false,
                    ),
                    LooperButton(
                      text: "PLAY",
                      active: state.playMode == protos.PlayMode.PLAYING,
                      onPressed: () {
                        if (state.playMode == protos.PlayMode.PLAYING) {
                          service.sendLooperCommand(
                              state.id, protos.LooperCommandType.DISABLE_PLAY);
                        } else {
                          service.sendLooperCommand(
                              state.id, protos.LooperCommandType.ENABLE_PLAY);
                          if (state.recordMode == protos.RecordMode.RECORD) {
                            service.sendLooperCommand(state.id,
                                protos.LooperCommandType.ENABLE_OVERDUB);
                          }
                        }
                      },
                    ),
                    Spacer(),
                    IconButton(
                      icon: Icon(Icons.delete),
                      onPressed: () {
                        service.sendLooperCommand(
                            state.id, protos.LooperCommandType.DELETE);
                      },
                    )
                  ],
                )
              ],
            )));
  }
}
