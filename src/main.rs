extern crate rand;
use rand::Rng;
use std::char;
use std::env;
use std::io;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::str::from_utf8;
use std::thread;

fn client(a:String) {
    match TcpStream::connect(a) {
        Ok(mut stream) => {
            println!("Connected");

            let mut data = [0 as u8; 50];
            let mut rep = [0 as u8; 50];

            loop {
                let hash_str = get_hash_str();
                let session_key = get_session_key();

                let next_key = next_session_key(&hash_str, &session_key);

                println!("Print a message: ");
                let mut message = String::new();

                io::stdin().read_line(&mut message);

                stream.write(&hash_str.into_bytes()).unwrap();
                stream.write(&session_key.into_bytes()).unwrap();
                stream.write(&message.into_bytes()).unwrap();

                match stream.read(&mut data) {
                    Ok(size) => {
                        stream.read(&mut rep);
                        let received_key = from_utf8(&data[0..size]).unwrap();
                        let response = from_utf8(&rep).unwrap();

                        if received_key == next_key {
                            println!("Client key: {}\nserver key: {}", next_key, received_key);
                        } else {
                            break;
                        }
                        println!("Response: {}", response);
                    }
                    Err(e) => {
                        println!("Failed to receive data: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Destroing");
}

fn server(a:String,b:u32) {
    let listener = TcpListener::bind(a).unwrap();
    println!("Server online");
    let mut count = 0;
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                count+=1;
                if count <= b{                  
                    println!("New connection: {}", stream.peer_addr().unwrap());
                    thread::spawn(move || handle_request(stream));
                }else {
                    println!("To much user connected!!!!");
                    break;
                }
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
    drop(listener);
}

fn handle_request(mut stream: TcpStream) {
    let mut hash = [0 as u8; 5];
    let mut key = [0 as u8; 10];
    let mut message = [0 as u8; 50];
    while match stream.read(&mut hash) {
        Ok(_) => {
            stream.read(&mut key);
            stream.read(&mut message);
            let received_hash = from_utf8(&hash).unwrap();
            let received_key = from_utf8(&key).unwrap();
            let new_key = next_session_key(&received_hash, &received_key);
            let result = new_key.clone().into_bytes();
            stream.write(&result).unwrap();
            stream.write(&message).unwrap();
            true
        }
        Err(_) => {
            println!("Connection error with {}", stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {}
}

fn get_session_key() -> String {
    let mut key = String::new();
    let mut rng = rand::thread_rng();

    for _i in 0..10 {
        let num = rng.gen_range(1, 10);
        let ch = char::from_digit(num, 10).unwrap();
        key.push(ch);
    }

    return key;
}

fn get_hash_str() -> String {
    let mut hash_str = String::new();
    let mut rng = rand::thread_rng();

    for _i in 0..5 {
        let num = rng.gen_range(1, 7);
        let ch = char::from_digit(num, 10).unwrap();
        hash_str.push(ch);
    }

    return hash_str;
}

fn next_session_key(hash_str: &str, session_key: &str) -> String {
    if hash_str.is_empty() {
        return "Hash code is empty".to_string();
    }

    for ch in hash_str.chars() {
        if !ch.is_ascii_digit() {
            return "Hash code contains non-digit letter".to_string();
        }
    }

    let mut result = 0;

    for ch in hash_str.chars() {
        let l = ch.to_string();
        result += calc_hash(session_key.to_string(), l.parse::<u64>().unwrap())
            .parse::<u64>()
            .unwrap();
    }

    return result.to_string();
}

fn calc_hash(key: String, value: u64) -> String {
    match value {
        1 => {
        let chp = "00".to_string() + &(key[0..5].parse::<u64>().unwrap() % 97).to_string();
        return chp[chp.len() - 2..chp.len()].to_string()
        }
        2 => {
            let reverse_key = key.chars().rev().collect::<String>();
            return reverse_key + &key.chars().nth(0).unwrap().to_string()
        }
        3 => {
            return key[key.len() - 5..key.len()].to_string() + &key[0..5].to_string()
        }
        4 => {
            let mut num = 0;
            for _i in 1..9 {
                num += key.chars().nth(_i).unwrap().to_digit(10).unwrap() as u64 + 41;
            }
            return num.to_string();
        }
        5 => {
            let mut ch: char;
            let mut num = 0;

            for _i in 0..key.len() {
                ch = ((key.chars().nth(_i).unwrap() as u8) ^ 43) as char;
                if !ch.is_ascii_digit() {
                    ch = (ch as u8) as char;
                }
                num += ch as u64;
            }
            return num.to_string();
        }
        _ => return (key.parse::<u64>().unwrap() + value).to_string(),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let a = args[1].to_string(); // первый аргумент либо порт для сервера(localhost) либо ip:port (например localhost:8888)
    let b = args[2].parse().unwrap(); // -n аргумент( число максимальных подключений)
    println!("{}{}",a,b);
if a.find(":") != None {   
    println!("<<<<<Trying to connect with server on adress-> {}>>>>>",a);
    client(a.clone()); //клиент подключается по переданному адресу если сервер запущен, если нет то выдаёт ошибку
} else {
    let mut aa = String::new();
    aa.push_str("localhost:");
    aa.push_str(&a);
    println!("<<<<<Trying run server>>>>>");
    server(aa,b);// сервер дропается при подключении больше чем он может позволить
}    
}
