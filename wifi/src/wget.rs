use reqwless::client::HttpClient;
use reqwless::request::{RequestBuilder, Method};
use embassy_net::Stack;
use embassy_net::tcp::client::{TcpClient, TcpClientState};
use alloc::string::String;
use embedded_io_async::Read;

// #[embassy_exec::task]
pub(crate) async fn wget(stack: Stack<'_>, url: &str) -> String {
    // Allocate DNS and TCP state
    let dns = embassy_net::dns::DnsSocket::new(stack);
    // let mut tls_bullshit = ...; // (or use plain HTTP if not using HTTPS)
    let tcp_state = TcpClientState::<1, 2048, 2048>::new();
    let tcp_client = TcpClient::new(stack, &tcp_state);

    let mut client = HttpClient::new(&tcp_client, &dns);
    
    // Allocate buffers for the request/response
    let mut dbg_out = [0u8; 512];
    let mut response_buf = [0u8; 128];

    // let mut response = client
    //     .request(Method::GET, url)
    //     .await
    //     .unwrap();

    // let body = response
    //     .send(&mut dbg_out)
    //     .await
    //     .unwrap()
    //     .body()
    //     .read_to_end()
    //     .await
    //     .unwrap();

    let mut request = client
        .request(reqwless::request::Method::GET, url)
        .await
        .unwrap()
        .content_type(reqwless::headers::ContentType::TextPlain)
        .headers(&[("user-agent", "curl/8.4.0")]);
    let mut response = request.send(&mut dbg_out).await.unwrap().body().reader();

    // Process your response body here...
    let mut response_str = String::new();
    loop {
        match response.read(&mut response_buf).await {
            Ok(0) => break, // Done
            Ok(n) => {
                response_str += &String::from_utf8(response_buf.to_vec()).unwrap();
            }
            Err(_) => break,
        }
    }

    response_str
}