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
      theme: ThemeData(
        primarySwatch: Colors.blue,
        // See https://github.com/flutter/flutter/wiki/Desktop-shells#fonts
        fontFamily: 'Roboto',
      ),
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
          return Scaffold(
            appBar: AppBar(
              title: Text("Loopers"),
            ),
            body: Container(
              padding: EdgeInsets.symmetric(vertical: 8.0),
              child: Column(
                  children: snapshot.data.loops.map((f) {
                return LooperWidget(state: f);
              }).toList()),
            ),
            floatingActionButton: FloatingActionButton(
              tooltip: 'New Looper',
              child: Icon(Icons.add),
            ),
          );
        });
  }
}

class LooperButton extends StatelessWidget {
  final String text;
  final bool active;
  final bool primed;

  const LooperButton({Key key, this.text, this.active, this.primed = false})
      : super(key: key);

  @override
  Widget build(BuildContext context) {
    var color = active || primed ? Colors.red[300] : Colors.black26;

    Widget button = FlatButton(
      color: color,
      onPressed: () {},
      child: Text(text),
    );

    return Container(
        padding: EdgeInsets.symmetric(horizontal: 4.0), child: button);
  }
}

class LooperWidget extends StatelessWidget {
  LooperWidget({this.state});

  final protos.LoopState state;

  @override
  Widget build(BuildContext context) {
    var value = state.length.isZero
        ? 0.0
        : state.time.toDouble() / state.length.toDouble();

    print("${state.time}, ${state.length}");

    var color = state.active ? Colors.orange[100] : Colors.black12;

    return Container(
        height: 200,
        margin: const EdgeInsets.symmetric(vertical: 8.0),
        padding: const EdgeInsets.all(8.0),
        decoration: BoxDecoration(color: color),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.spaceAround,
          children: <Widget>[
            Text("looper ${state.id}"),
            LinearProgressIndicator(value: value),
            Row(
              children: <Widget>[
                LooperButton(
                  text: "RECORD",
                  active: state.recordMode == protos.RecordMode.RECORD,
                  primed: state.recordMode == protos.RecordMode.READY,
                ),
                LooperButton(
                  text: "OVERDUB",
                  active: state.recordMode == protos.RecordMode.OVERDUB,
                ),
                LooperButton(
                  text: "PLAY",
                  active: state.playMode == protos.PlayMode.PLAYING,
                )
              ],
            )
          ],
        ));
  }
}
