# Forwarder

A simple virtual network application made with Rust. This program uses urls to
divide the network traffic to different ports and translates requests from
those sites back. Right now this only supports GET requests. By default this
app will use port 3001, but you can change it in the code.


## Usage

First create a `config.json` file. In this file will be the path/port
definitions that are used in-code.

```json
{
    "config" : [
        {
            "path" : "/app_1",
            "port" : "3000"
        },
        {
            "path" : "/app_2",
            "port" : "3001"
        },
        {
            "path" : "/app_3",
            "port" : "3002"
        }
    ],
    "url" : "0.0.0.0"
}
```

