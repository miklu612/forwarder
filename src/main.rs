
use axum::{
    response::Html, 
    body::Body,
    extract::{ConnectInfo, Request},
    routing::{get, MethodRouter}, 
    Router
};
use std::net::SocketAddr;
use std::env;
use std::fs;
use serde_json::Value;
use std::cell::Cell;
use std::rc::Rc;
use std::sync::Mutex;
use std::sync::Arc;


const ip_size: usize = 40;
const port_size: usize = 40;

#[derive(Debug, Clone)]
struct IPPortPair {
    ip: [char;ip_size],
    port: [char;port_size],
}

impl IPPortPair {
    pub fn new() -> Self {
        Self {
            ip: ['\0';ip_size],
            port: ['\0';port_size]
        }
    }
    pub fn get_ip(self) -> String {
        let mut output = String::new();
        for index in 0..ip_size-1 {
            if self.ip[index] == '\0' {
                break;
            }
            else {
                output.push(self.ip[index]);
            }
        }
        output
    }
    pub fn get_port(self) -> String {
        let mut output = String::new();
        for index in 0..port_size-1 {
            if self.port[index] == '\0' {
                break;
            }
            else {
                output.push(self.port[index]);
            }
        }
        output
    }
    pub fn set_port(&mut self, port: String) {
        assert!(port.len() < port_size-1);
        for index in 0..port.len() {
            self.port[index] = port.chars().nth(index).unwrap();
            self.port[index+1] = '\0';
        }
        assert!(self.port[0] != '\0');
    }
    pub fn set_ip(&mut self, ip: String) {
        assert!(ip.len() < ip_size-1);
        for index in 0..ip.len() {
            self.ip[index] = ip.chars().nth(index).unwrap();
            self.ip[index+1] = '\0';
        }
        assert!(self.ip[0] != '\0');
    }
    pub fn is_free(self) -> bool {
        self.ip[0] == '\0'
    }
}

#[tokio::main]
async fn main() {

    let args: Vec<String> = env::args().collect();
    assert!(args.len() == 2);

    // TODO: Load this into memory always and don't read it evertime.
    let file_name = args[1].clone();
    let json_raw = fs::read_to_string(file_name).expect("Failed to read file");
    println!("{}", json_raw);

    let config_data: Value = serde_json::from_str(&json_raw).expect("Failed to parse json");
    let config = config_data["config"].clone();

    let mut app = Router::new();
    // TODO: Add mutex
    let mut connections = Arc::new(Mutex::new([();20].map(|_| IPPortPair::new())));
   
    let a = connections.clone();
    let b = connections.clone();
    app = app.fallback(get(|connection_info: ConnectInfo<SocketAddr>, request: Request<Body> | async move {
        let mut vec = &mut a.lock().unwrap().clone();
        let mut ip = format!("{}", connection_info.0);
        let index = vec.clone().into_iter().position(|x| {x.get_ip() == ip});
        match index {
            Some(v) => {
                let path = vec[v].clone().get_port();
                let original_path = request.uri().path();
                let new_path = "http://127.0.0.1:".to_string() + &path + original_path;
                println!("{}", new_path);
                let body = reqwest::get(new_path).await.unwrap().text().await.unwrap();
                return Html(body);
            },
            None => {}
        }
        return Html("Forwarder: Error".to_string());
    }));

    for path in config.as_array().unwrap() {
        // This part will just basically log the ip and store it. This ip is then used to track
        // where the user is and convert the paths according to that, since the site that is given
        // to the user won't have the /<site> prefix.
        let og_port = path["port"].as_str().unwrap().clone();
        let mut outer_information = Box::new(IPPortPair::new());
        outer_information.set_port(og_port.to_string());
        let new_path = "http://127.0.0.1:".to_string() + &path["port"].as_str().unwrap();
        let c = connections.clone();
        let m_router: MethodRouter = get(|connect_info: ConnectInfo<SocketAddr>| async move {
            let ip = connect_info.0;
            let ip_string = format!("{}", ip);
            let mut information = outer_information.clone();
            {
                println!("Port: {}", information.clone().get_port());
                let mut vec = &mut c.lock().unwrap();
                let existing_ip = vec.clone().into_iter().position(|x| { x.get_ip() == ip_string });
                match existing_ip {
                    Some(index) => {
                        vec[index].set_port(information.get_port());
                        vec[index].set_ip(ip_string);
                    },
                    None => {
                        let new_index = vec.clone().into_iter().position(|x| { x.is_free() });
                        match new_index {
                            Some(value) => {
                                println!("{} was free", value);
                                vec[value].set_port(information.get_port());
                                vec[value].set_ip(ip_string);
                            },
                            None => { panic!("No free slot found"); }
                        }
                    },
                }
            }
            let body = reqwest::get(new_path).await.unwrap().text().await.unwrap();
            return Html(body);
        });
        app = app.route(path["path"].as_str().unwrap(), get(m_router));
    }

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001").await.unwrap();
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();

}

async fn error() -> Html<&'static str> {
    Html("<h1> Error </h1>")
}

async fn handler() -> Html<&'static str> {
    Html("<h1> Forwarder </h1>")
}
