use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream}, fs, thread, time::Duration,
};

use rustserver::ThreadPool;

// 读取来自浏览器请求的函数
fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);

    /* v1: 直接发送内容*/

    // let http_request: Vec<_> = buf_reader
    //                             .lines()
    //                             .map(|result| result.unwrap())
    //                             .take_while(|line| !line.is_empty())
    //                             .collect();
    // let status_line = "HTTP/1.1 200 OK";
    // // 注意hello.html需要创建在根目录下！
    // let contents = fs::read_to_string("hello.html").unwrap();
    // let length = contents.len();
    // //使用format宏进行格式化输出，类似printf
    // let response =
    //     format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    // stream.write_all(response.as_bytes()).unwrap();

    /* v2: 先判断请求类型在发送 */
    // let request_line = buf_reader.lines().next().unwrap().unwrap();
    // if request_line == "GET / HTTP/1.1" {
    //     let status_line = "HTTP/1.1 200 OK";
    //     let contents = fs::read_to_string("hello.html").unwrap();
    //     let length = contents.len();

    //     let response = format!(
    //         "{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}"
    //     );
    //     stream.write_all(response.as_bytes()).unwrap();
    // } else {
    //     // some other requests.
    //     let status_line = "HTTP/1.1 404 NOT FOUND";
    //     let contents = fs::read_to_string("404.html").unwrap();
    //     let length = contents.len();

    //     let response = format!(
    //         "{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}"
    //     );
    //     stream.write_all(response.as_bytes()).unwrap();
    // }

    /* v3: 代码重构，去除冗余 */
    // let buf_reader: BufReader<&mut TcpStream> = BufReader::new(&mut stream);
    // let request_line = buf_reader.lines().next().unwrap().unwrap();
    // let (status_line, filename) = if request_line == "GET / HTTP/1.1" {
    //     ("HTTP/1.1 200 OK", "hello.html")
    // } else {
    //     ("HTTP/1.1 404 NOT FOUND", "404.html")
    // };

    // let contents = fs::read_to_string(filename).unwrap();
    // let length = contents.len();

    // let response =
    //     format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    // stream.write_all(response.as_bytes()).unwrap();

    /* v4: 处理慢请求 */
    // let buf_reader: BufReader<&mut TcpStream> = BufReader::new(&mut stream);
    // let request_line = buf_reader.lines().next().unwrap().unwrap();
    // let (status_line, filename) = match &request_line[..] {
    //     "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
    //     "GET /sleep HTTP/1.1" => {
    //         thread::sleep(Duration::from_secs(5));
    //         ("HTTP/1.1 200 OK", "hello.html")
    //     },
    //     _ => ("HTTP/1.1 404 NOT FOUND", "404.html")
    // };
    // let contents = fs::read_to_string(filename).unwrap();
    // let length = contents.len();

    // let response =
    //     format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    // stream.write_all(response.as_bytes()).unwrap();

    /* v5: 最终版本 */
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "hello.html")
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn main() {
    /* v1: 单线程版本server */
    // let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    // for stream in listener.incoming() {
    //     let stream = stream.unwrap();
    //     handle_connection(stream);
    // }

    /* v2: ThreadPool版本的server */
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    // 通过ThreadPool来控制thread数量
    let pool = ThreadPool::new(4);
    // 限制请求的最大数量为2个
    for stream in listener.incoming().take(2) {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
    println!("shutting down");
}
