use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3{
        println!("参数不正确！");
        return;
    }
    let url = &args[1];
    let out = &args[2];

    println!("Fetching url: {}",url);
    let body = reqwest::blocking::get(url).unwrap().text().unwrap();

    println!("Converting html to markdown ...");
    let md = html2md::parse_html(&body);

    fs::write(out,md.as_bytes()).unwrap();
    println!("Markdown file has been saved in {}",out);
}
