import 'dart:io' show Platform;

import 'package:fixnum/fixnum.dart';
import 'package:flutter/foundation.dart'
    show debugDefaultTargetPlatformOverride;
import 'package:flutter/material.dart';
import 'package:loopers/looper_service.dart';
import 'package:loopers/settings.dart';
import 'config.dart';
import 'src/generated/loopers.pb.dart' as protos;
import 'src/generated/loopers.pbgrpc.dart' as grpc;

void main() async {
  // See https://github.com/flutter/flutter/wiki/Desktop-shells#target-platform-override
  debugDefaultTargetPlatformOverride = TargetPlatform.fuchsia;

  var testState = protos.State();
  var l1 = protos.LoopState();
  l1.id = 0;
  l1.length = Int64(120 * 1000);
  l1.time = Int64(50 * 1000);
  l1.mode = protos.LooperMode.READY;

  var l2 = protos.LoopState();
  l2.id = 1;
  l2.length = Int64(60 * 1000);
  l2.time = Int64(50 * 1000);
  l2.mode = protos.LooperMode.PLAYING;
  l2.active = true;

  testState.loops.add(l1);
  testState.loops.add(l2);

  var service = LooperService();
  await service.start();

  runApp(new MyApp(
    service: service,
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
      theme: ThemeData.dark().copyWith(
          buttonTheme: ButtonThemeData(
        minWidth: 30,
      )),
//      theme: ThemeData(
//        primarySwatch: Colors.blue,
//        // See https://github.com/flutter/flutter/wiki/Desktop-shells#fonts
// fontFamily: 'Roboto',
//      ),

      home: MainPage(service: service),
    );
  }
}

void showFilePicker(BuildContext context, LooperService service) {
  String path;

  Dialog simpleDialog = Dialog(
    shape: RoundedRectangleBorder(
      borderRadius: BorderRadius.circular(12.0),
    ),
    child: Container(
      height: 200.0,
      width: 300.0,
      child: Column(
        children: <Widget>[
          Padding(
            padding: EdgeInsets.all(15.0),
            child: Text(
              'Select file to load',
            ),
          ),
          Padding(
            padding: EdgeInsets.all(15.0),
            child: TextField(
              decoration: InputDecoration(
                border: OutlineInputBorder(),
                labelText: "Path",
              ),
              onChanged: (text) {
                path = text;
              },
            ),
          ),
          ButtonBar(
            alignment: MainAxisAlignment.center,
            children: <Widget>[
              FlatButton(
                child: Text("Load"),
                onPressed: () {
                  print("sending $path");
                  service.sendLoadSessionCommand(path);
                  Navigator.of(context).pop();
                },
              ),
              FlatButton(
                child: Text("Cancel"),
                onPressed: () {
                  Navigator.of(context).pop();
                },
              )
            ],
          )
        ],
      ),
    ),
  );
  showDialog(context: context, builder: (BuildContext context) => simpleDialog);
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
            var i = -1;
            loopers = Column(
                children: snapshot.data.loops.map((f) {
              i++;
              return LooperWidget(state: f, index: i, service: widget.service);
            }).toList());
          }

          return Scaffold(
            body: Column(children: [
              AppBar(
                title: Text("Loopers"),
                actions: <Widget>[
                  IconButton(
                    icon: Icon(Icons.save),
                    onPressed: () {
                      widget.service.sendSaveSessionCommand();
                    },
                  ),
                  IconButton(
                      icon: Icon(Icons.open_in_browser),
                      onPressed: () {
                        showFilePicker(context, widget.service);
                      }),
                  IconButton(
                      icon: Icon(Icons.offline_bolt),
                      color: snapshot.data != null && snapshot.data.learnMode
                          ? Colors.blue
                          : Colors.white70,
                      onPressed: () {
                        Navigator.push(context,
                            MaterialPageRoute(builder: (context) {
                          return ConfigPage();
                        }));
                        widget.service.sendGlobalCommand(snapshot.data.learnMode
                            ? protos.GlobalCommandType.DISABLE_LEARN_MODE
                            : protos.GlobalCommandType.ENABLE_LEARN_MODE);
                      }),
                  IconButton(
                    icon: Icon(Icons.settings),
                    onPressed: () {
                      Navigator.push(context,
                          MaterialPageRoute(builder: (context) {
                        return SettingsPage();
                      }));
                    },
                  ),
                ],
              ),
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

class Metronome extends StatelessWidget {
  final protos.State state;

  const Metronome({Key key, this.state}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    var children = <Widget>[];
    for (var i = 0; i < state.timeSignatureUpper.toInt(); i++) {
      var selected = state.beat % state.timeSignatureUpper.toInt() == i;
      var color = selected && i == 0
          ? Colors.blue
          : (selected ? Colors.white30 : Colors.black12);

      children.add(FlatButton(
        color: color,
        child: Text(i.toString()),
        onPressed: () => null,
      ));
    }

    return Row(
      children: children,
    );
  }
}

class TimeWidget extends StatelessWidget {
  final protos.State state;

  const TimeWidget({Key key, this.state}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    if (state != null) {
      var time = state.time.toInt();
      var negative = "";
      if (time < 0) {
        negative = "-";
        time = -time;
      }
      time = time.toInt() ~/ 1000;
      var hours = time ~/ 60 ~/ 60;
      time -= hours * 60 * 60;
      var minutes = time ~/ 60;
      time -= minutes * 60;
      var seconds = time;

      var r = (d) => d < 10 ? "0" + d.toString() : d.toString();

      return Container(
          height: 50,
          child: Row(
            children: <Widget>[
              Text("${negative}${r(hours)}:${r(minutes)}:${r(seconds)}"),
              Metronome(
                state: state,
              ),
              Text("${state.bpm} bpm")
            ],
          ));
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
      child: Text(text,
          style: TextStyle(
            fontSize: 12.0,
          )),
    );

    return Container(
        padding: EdgeInsets.symmetric(horizontal: 4.0), child: button);
  }
}

class LooperWidget extends StatelessWidget {
  LooperWidget({this.state, this.index, this.service});

  final int index;
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
          service.sendLooperCommand(
              this.index, protos.LooperCommandType.SELECT);
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
//                this.index.toString(),
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
                        active: state.mode == protos.LooperMode.RECORD,
                        primed: state.mode == protos.LooperMode.READY,
                        onPressed: () {
                          if (state.mode == protos.LooperMode.READY ||
                              state.mode == protos.LooperMode.RECORD) {
                            service.sendLooperCommand(this.index,
                                protos.LooperCommandType.ENABLE_PLAY);
                          } else {
                            if (state.mode == protos.LooperMode.PLAYING) {
                              service.sendLooperCommand(this.index,
                                  protos.LooperCommandType.ENABLE_OVERDUB);
                            }
                            service.sendLooperCommand(this.index,
                                protos.LooperCommandType.ENABLE_RECORD);
                          }
                        }),
                    LooperButton(
                      text: "OVERDUB",
                      active: state.mode == protos.LooperMode.OVERDUB,
                      onPressed: () {
                        if (state.mode == protos.LooperMode.OVERDUB) {
                          service.sendLooperCommand(
                              this.index, protos.LooperCommandType.ENABLE_PLAY);
                        } else {
                          service.sendLooperCommand(this.index,
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
                      active: state.mode == protos.LooperMode.PLAYING,
                      onPressed: () {
                        if (state.mode == protos.LooperMode.PLAYING) {
                          service.sendLooperCommand(
                              this.index, protos.LooperCommandType.STOP);
                        } else {
                          service.sendGlobalCommand(
                              protos.GlobalCommandType.RESET_TIME);
                          service.sendLooperCommand(
                              this.index, protos.LooperCommandType.ENABLE_PLAY);
                          if (state.mode == protos.LooperMode.RECORD) {
                            service.sendLooperCommand(this.index,
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
                            this.index, protos.LooperCommandType.DELETE);
                      },
                    )
                  ],
                )
              ],
            )));
  }
}
