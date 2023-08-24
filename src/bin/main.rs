use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::fs::File;
extern crate webserver; 
use serde_json::json;
use webserver::ThreadPool;
use serde::{Deserialize};
use serde::{Serialize};
use handlebars::Handlebars;



fn main() {
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap(); //binds our ip address to port 8080
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() { //.incoming() returns an iterator over a list of streams (connection attempts)
        let stream = stream.unwrap(); //unwrap just terminates the program if encounter an error
        
        pool.execute( || { //calls execute and passes handle_connection as the closure
            
            handle_connection(stream);     
            
        });
    }

    

}


#[derive(Deserialize, Debug, Serialize)]
struct NewsArticle
{
    source: Source,
    author: Option<String>,
    title: String,
    description: Option<String>,
    url: String,
    urltoimage: Option<String>,
    content: Option<String>,
    
}

#[derive(Deserialize, Debug, Serialize)]
struct Source
{
    id: Option<String>,
    name: String,
}
#[derive(Deserialize, Debug, Serialize)]
struct NewsResponse
{
    articles: Vec<NewsArticle>,
}




fn get_api() -> Result<String, reqwest::Error> 
{
    
    let client = reqwest::blocking::Client::builder()
                        .user_agent("Personal project")
                        .build()?;
    let url = "https://newsapi.org/v2/top-headlines?country=us&apiKey=16d768c860044a7e9e0aaef6c617d71e"; //api url
    let response= client.get(url).send(); //makes api call and stores response
    
    
    let json_string = response.unwrap().text().unwrap(); //translates response to a string
    let newsresponse: NewsResponse = serde_json::from_str(&json_string).unwrap(); //deserializes from json format into NewsResponse struct

    let html = render_in_html(newsresponse.articles).unwrap();

    Ok(html)

}



fn render_in_html(articles: Vec<NewsArticle>) -> Result<String, Box<dyn std::error::Error>>
{
    let mut renderer = Handlebars::new();

    let mut file = File::open("news.html").unwrap(); 
    let mut html = String::new();
    file.read_to_string(&mut html).unwrap();


    renderer.register_template_string("article_template", &html)?;

    let data = json!({"articles": articles});

    let rendered = renderer.render_template(&html, &data).unwrap();

    Ok(rendered)




}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512]; //creates a buffer of 512 ints 

    stream.read(&mut buffer).unwrap(); //reads the bytes from each incoming stream into the buffer
    let get = b"GET / HTTP/1.1\r\n"; //use byte string to specifically check that user is requesting /
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    //construct response
    let mut response = String::new();
    if buffer.starts_with(get) || buffer.starts_with(sleep)
    {
        let contents = &get_api().unwrap();
        let status_line = "HTTP/1.1 200 OK\r\n\r\n";
        response += &format!("{}{}", status_line, contents);
    
        
    }
    else 
    {
        let mut file = File::open("404.html").unwrap(); 
        let mut contents = String::new();
        let status_line = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
        file.read_to_string(&mut contents).unwrap();
        

        response += &format!("{}{}", status_line, contents);
    }


    
    

    //send response:
    stream.write(response.as_bytes()).unwrap(); //writes response to TCP stream
    stream.flush().unwrap(); //makes it wait until all bytes are written to the connection


    
}