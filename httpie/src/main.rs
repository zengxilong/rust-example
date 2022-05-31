use std::collections::HashMap;
use std::str::FromStr;
use clap::Parser;
use anyhow::{anyhow,Result};
use mime::Mime;
use reqwest::{Client, header, Response,Url};
use colored::*;
// 定义 HTTPie 的 CLI 的主入口，它包含若干个子命令
// 下面 /// 的注释是文档，clap 会将其作为 CLI 的帮助

/// A naive httpie implementation with Rust, can you imagine how easy it is?
#[derive(Parser, Debug)]
#[clap(version = "1.0", author = "Jasper Zeng <zengxilonglh@gmail.com>")]
struct Opts {
    #[clap(subcommand)]
    subcommand: SubCommand,
}


// 子命令分别对应不同的 HTTP 方法，目前只支持 get / post
#[derive(Parser, Debug)]
enum SubCommand {
    Get(Get),
    Post(Post),
    // 我们暂且不支持其它 HTTP 方法
}

// get 子命令

/// feed get with an url and we will retrieve the response for you
#[derive(Parser, Debug)]
struct Get {
    /// HTTP 请求的 URL
    #[clap(parse(try_from_str=parse_url))]
    url: String,
}

// post 子命令。需要输入一个 URL，和若干个可选的 key=value，用于提供 json body

/// feed post with an url and optional key=value pairs. We will post the data
/// as JSON, and retrieve the response for you
#[derive(Parser, Debug)]
struct Post {
    /// HTTP 请求的 URL
    #[clap(parse(try_from_str=parse_url))]
    url: String,
    /// HTTP 请求的 body
    #[clap(parse(try_from_str=parse_kv_pair))]
    body: Vec<KvPair>,
}

/// 命令行中的k=v 可以被 parse_kv_pair 解析为 KvPair结构
#[derive(Debug)]
struct KvPair {
    k: String,
    v: String,
}

fn parse_url(s: &str) -> Result<String>{
    let _url: Url = s.parse()?;
    Ok(_url.into())
}

impl FromStr for KvPair{
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split("=");
        let err = ||anyhow!(format!("Failed to parse {}",s));
        Ok(Self{
            k:(split.next().ok_or_else(err)?).to_string(),
            v:(split.next().ok_or_else(err)?).to_string(),
        })
    }
}

fn parse_kv_pair(s: &str) -> Result<KvPair>{
    Ok(s.parse()?)
}

// 打印服务器版本号+状态码
fn print_status(resp: &Response){
    let status = format!("{:?} {}",resp.version(),resp.status()).blue();
    println!("{}\n",status);
}

// 打印返回的 HTTP Header
fn print_headers(resp: &Response){
    for (name, value) in resp.headers(){
        println!("{}: {:?}",name.to_string().green(),value);
    }
    print!("\n");
}

fn print_body(m: Option<Mime>, body: &String){
    match m {
        Some(v) if v == mime::APPLICATION_JSON =>{
            println!("{}",jsonxf::pretty_print(body).unwrap().cyan())
        }
        _ => {
            println!("{}",body);
        }
    }
}

fn get_content_type(resp: &Response) -> Option<Mime>{
    resp.headers()
        .get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().parse().unwrap())
}

async fn print_resp(resp: Response) -> Result<()>{
    print_status(&resp);
    print_headers(&resp);
    let mime = get_content_type(&resp);
    let body = resp.text().await?;
    print_body(mime,&body);
    Ok(())
}

/// 处理 get 子命令
async fn get(client: Client, args: &Get) -> Result<()> {
    let resp = client.get(&args.url).send().await?;
    Ok(print_resp(resp).await?)
}

/// 处理 post 子命令
async fn post(client: Client, args: &Post) -> Result<()> {
    let mut body = HashMap::new();
    for pair in args.body.iter() {
        body.insert(&pair.k, &pair.v);
    }
    let resp = client.post(&args.url).json(&body).send().await?;
    Ok(print_resp(resp).await?)
}
// 异步处理，引入tokio
#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let mut headers = header::HeaderMap::new();
    // 为 http client 添加一些缺省的 HTTP 头
    headers.insert("X-POWERED-BY","Rust".parse()?);
    headers.insert(header::USER_AGENT,"Rust httpie".parse()?);
    let client = Client::builder().default_headers(headers).build()?;
    let result = match opts.subcommand{
        SubCommand::Get(ref args) => {
            get(client,args).await?
        }
        SubCommand::Post(ref args) => {
            post(client,args).await?
        }
    };
    Ok(result)
}
