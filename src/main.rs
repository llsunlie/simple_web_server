use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

use simple_web_server::ThreadPool;

fn main() {
    // 创建 TcpListener，监听 127.0.0.1:8899
    let listener = TcpListener::bind("127.0.0.1:8899").unwrap();
    // 创建线程池，包含 4 个 Worker
    let pool = ThreadPool::new(4);

    // 处理来自客户端的连接，最多处理 2 个连接s
    for stream in listener.incoming().take(2) {
        let stream = stream.unwrap();

        // 提交连接处理任务到线程池
        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    // 创建缓冲读取器
    let buf_reader = BufReader::new(&mut stream);
    // 读取 HTTP 请求的第一行（请求行）
    let http_request = buf_reader.lines().next().unwrap().unwrap();

    // 根据请求行判断响应状态和响应内容
    let (status_line, filename) = if http_request == "GET / HTTP/1.1" {
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
    };

    // 读取文件内容
    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();
    // 构造响应消息
    let response = format!(
        "{}\r\nContent-Length: {}\r\n{}\r\n",
        status_line, length, contents
    );

    // 将响应消息写入流
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
