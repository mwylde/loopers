import 'package:flutter/material.dart';
import 'package:shared_preferences/shared_preferences.dart';

class SettingsPage extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    // TODO: implement build
    return Scaffold(
      body: Column(children: <Widget>[
        AppBar(
          title: Text("Settings"),
        ),
        Container(
          padding: EdgeInsets.all(8.0),
          child: SettingsForm(),
        ),
      ]),
    );
  }
}

class SettingsForm extends StatefulWidget {
  @override
  State<StatefulWidget> createState() {
    return SettingsFormState();
  }

}

class SettingsFormState extends State<SettingsForm> {
  final _formKey = GlobalKey<FormState>();

  final controller = TextEditingController();

  void initState() {
    super.initState();

    SharedPreferences.getInstance().then((pref) {
      controller.text = pref.getString("host");
    });

    controller.addListener(() async {
      var value = controller.text;
      SharedPreferences prefs = await SharedPreferences.getInstance();
      prefs.setString("host", value);
      print("Saving $value");
    });
  }

  @override
  Widget build(BuildContext context) {
    return Form(
      key: _formKey,
      child: TextFormField(
        controller: controller,
        decoration: InputDecoration(
          labelText: "server",
        ),
      )
    );
  }

}