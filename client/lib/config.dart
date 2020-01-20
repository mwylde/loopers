import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';
import 'src/generated/loopers.pb.dart' as protos;
import 'src/generated/loopers.pbgrpc.dart' as grpc;
import 'package:flutter/foundation.dart';

class ConfigPage extends StatelessWidget {
  final protos.State state;

  const ConfigPage({Key key, this.state}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
        body: Column(
      children: <Widget>[
        AppBar(title: Text("Config")),
      ],
    ));
  }
}
