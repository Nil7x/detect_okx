use reqwest::{header::{self, AGE}, Result};
use serde::Deserialize;
use tokio::time;

use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufRead, BufReader},
    time::Duration,
};

use crate::block_info::get_block_height;
mod block_info;
mod transcation;

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct ListInfo {
    platformName: String,
    txHash: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct List {
    activityList: Vec<ListInfo>,
}

#[derive(Debug, Deserialize)]
struct FTInfo {
    data: Option<List>,
}

#[derive(Debug)]
#[allow(dead_code)]
struct Verify {
    txid: String,
    flag: bool,
}

pub const BASE_URL: &str = "http://ws.btc.whitetowers.io/api/v1/tx/";
#[tokio::main]
async fn main() -> Result<()> {
    detect(836761, 2).await?;

    Ok(())
}

async fn detect(height: u128, step: u128) -> Result<()> {
    // 创建一个tick的map，通过打开的file句柄保存FT类型，value值为前一次okx接口取得的所有FT类型txid
    // 避免多次调用okx接口都循环判断
    let mut ticks: HashMap<String, Vec<String>> = HashMap::new();
    let mut tx_ids: HashSet<String> = HashSet::new();
    let mut ok_list: Vec<Verify> = Vec::new();
    let mut count = 0;

    for i in (height - step)..height + 1 {
        if let Ok(_) = block_info::get_block(i).await {
            // println!("get block true");
            if let Ok(f) = File::open(i.to_string() + "_first") {
                let buf = BufReader::new(f);
                // 打开已保存的文件，分行进行读取
                let lines = buf.lines().collect::<Vec<_>>();
                let lines = lines
                    .into_iter()
                    .filter(|t| t.is_ok())
                    .map(|t| t.unwrap())
                    .collect::<Vec<_>>();
                for line in lines {
                    // 每一行根据逗号分隔成 txid tick block_height
                    let t: Vec<String> = line.split(",").map(|s| s.to_string()).collect();
                    if let [txid, tick, ..] = &t[..] {
                        tx_ids.insert(txid.to_string());

                        let flag = send_req(
                            i,
                            //  block_height.to_string(),
                            txid.to_string(),
                            tick.to_string(),
                            &mut ticks,
                            &tx_ids,
                        )
                        .await?;
                        // println!("text line's txid {} tick {} height {} \n", txid, tick, i);
                        if flag {
                            // println!("true okx list add one");
                            ok_list.push(Verify {
                                txid: txid.to_string(),
                                flag: flag,
                            })
                        } else {
                            // println!("false okx list add one");
                            ok_list.push(Verify {
                                txid: txid.to_string(),
                                flag: false,
                            });
                            count += 1
                        }
                    }
                }
            } else {
                println!("False")
            }
        }
    }
    // println!("count {}, len {}", count, ok_list.len());
    println!(
        "detect the newest 50 data of okx correct rate: {:.2}%",
        ((ok_list.len() - count) as f64 / ok_list.len() as f64) * 100f64
    );
    Ok(())
}

async fn send_req(
    height: u128,
    // block_height: String,
    tx_id: String,
    tick: String,
    ticks: &mut HashMap<String, Vec<String>>,
    hash: &HashSet<String>,
) -> Result<bool> {
    // 如果ticks有tick，说明近期已经取过数据了，直接查找
    if ticks.contains_key(&tick) {
        if let Some(hash_list) = ticks.get(&tick) {
            if hash_list.len() >= 1 && hash_list.contains(&tx_id) {
                return Ok(true);
            }
        }
    }
    let mut flag = false;

    let mut headers = header::HeaderMap::new();
    {
        headers.insert(
            "User-Agent",
            "Apifox/1.0.0 (https://apifox.com)".parse().unwrap(),
        );
        headers.insert("Accept", "*/*".parse().unwrap());
        headers.insert("Host", "www.okx.com".parse().unwrap());
        headers.insert("Connection", "keep-alive".parse().unwrap());
    }
    let client = reqwest::Client::new();
    // println!("sleeping for 10secs");
    // time::sleep(Duration::from_secs(10)).await;
    let url = format!("https://www.okx.com/priapi/v1/nft/inscription/rc20/detail/activity?tick={}&type=5&tickerType=3&pageSize=50&ticker={}", tick, tick);
    let res = client
        .get(url)
        .headers(headers)
        .send()
        .await?
        .text()
        .await?;

    if let Ok(json) = serde_json::from_str(&res) {
        let j = serde_json::from_value::<FTInfo>(json);
        // println!("okx response json {:#?}", j);
        match j {
            Ok(ft) => {
                for list in ft.data {
                    for l in list.activityList {
                        // 获取到当前区块的高度，如果高于需要判断的高度直接返回 true
                        let block_height = get_block_height(l.txHash.clone()).await.unwrap();
                        // println!("current txid {} heigxht {} ", l.txHash, block_height);
                        if block_height - 50 > height {
                            return Ok(true);
                        }
                        // println!("compare's txid {}", l.txHash);
                        if l.platformName == "OKX" && hash.get(&l.txHash) != None {
                            flag = true
                        }
                        if l.platformName == "OKX" {
                            ticks
                                .entry(tick.clone())
                                .or_insert_with(Vec::new)
                                .push(l.txHash);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("error parsing JSON {}", e);
            }
        }
    }
    Ok(flag)
}

#[test]
fn ttt() -> () {
    let input_path = "836465_first";
    let line_number = 3; // 比如我们想在第三行后追加内容，索引从0开始
    let append_content = "newtest";

    // append_to_file_line(input_path, line_number, append_content).unwrap()
    // use std::collections::HashMap;

    // let mut ticks: HashMap<String, Vec<String>> = HashMap::new();

    // // 插入一个新的键值对
    // ticks
    //     .entry("key1".to_string())
    //     .or_insert_with(Vec::new)
    //     .push("value1".to_string());

    // // 追加数据到已存在的键的值
    // ticks
    //     .entry("key1".to_string())
    //     .or_insert_with(Vec::new)
    //     .push("value2".to_string());

    // // 检查键是否存在，如果存在就追加数据
    // if let Some(values) = ticks.get_mut("key1") {
    //     values.push("value3".to_string());
    // }

    // // 打印 ticks
    // for (key, values) in &ticks {
    //     println!("Key: {}", key);
    //     for value in values {
    //         println!("  Value: {}", value);
    //     }
    // }
}

#[test]
fn test_str() -> () {
    if "83636900000000001" > "83636900000000000" {
        println!("yes")
    } else {
        println!("no")
    }
}

pub fn foo(a: i32, mut b: i32) -> i32 {
    b += 1;
    let mut x = a + b;
    let y = a - b;

    return 0;
}



// #[derive(Debug)]
// enum Direction {
//     East,
//     West,
//     North,
//     South,
// }
#[test]
fn foo1() {
    
        // let age = Some(30);
        // println!("在匹配前，age是{:?}",age);
        // if let Some(age) = age {
        //     println!("匹配出来的age是{}",age);
        // }
     
        // println!("在匹配后，age是{:?}",age);
    
        struct Point {
            x: i32,
            y: i32,
            z: i32,
        }
        
        let origin = Point { x: 0, y: 0, z: 0 };
        
        match origin {
            Point { x, .. } => println!("x is {}", x),
        }
        
        let numbers = (2, 4, 8, 16, 32);

        // match numbers {
        //     (first, .., last) => {
        //         println!("Some numbers: {}, {}", first, last);
        //     },
        // }
        match numbers {
            (.., a, b) => {
                println!("Some numbers: {}, {}", a, b);
            },
        }

    println!("----");
    enum Message {
        Hello { id: i32 },
    }
    
    let msg = Message::Hello { id: 10 };
    String::from("1");
    match msg {
        Message::Hello { id: id_variable @ 3..=7 } => {
            println!("Found an id in range: {}", id_variable)
        },
        Message::Hello { id:id @ 10..=12 } => {
            println!("Found an id in another range, {}", id)
        },
        Message::Hello { id } => {
            println!("Found some other id: {}", id)
        },
    }
    
}
        
