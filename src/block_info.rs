use crate::transcation;
use crate::BASE_URL;
use reqwest::header;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{fs::File, io::Write};

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct AtomInfo {
    nft_name: Option<String>,
    types: Option<String>,
    // count: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct Atom {
    atom_info: AtomInfo,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct From {
    address: String,
    // atoms_info: Vec<AtomInfo>,
    atoms_info: Vec<Atom>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Trans {
    block: u128,
    events: Vec<String>,
    froms: Option<Vec<From>>,
    market: String,
    market_base64: Option<String>,
    tos: Option<Vec<From>>,
    txid: String,
    utc: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Root {
    trans: Vec<Trans>,
    // all_data: bool,
}

pub async fn get_block(height: u128) -> Result<(), Box<dyn std::error::Error>> {
    let mut headers = header::HeaderMap::new();
    let mut file = File::create(height.to_string() + "_first").expect("failed to create file");
    {
        headers.insert(
            "User-Agent",
            "Apifox/1.0.0 (https://apifox.com)".parse().unwrap(),
        );

        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers.insert("Accept", "*/*".parse().unwrap());
        headers.insert("Host", "192.168.195.71:97".parse().unwrap());
        headers.insert("Connection", "keep-alive".parse().unwrap());
    }
    let client = reqwest::Client::new();
    let response = client
        .post("http://192.168.195.71:97/txs")
        .headers(headers)
        .body(
            json!({
                "types": "block",
                "key": height.to_string(),
            })
            .to_string(),
        )
        .send()
        .await?
        .text()
        .await?;
    // println!("{}", response);
    // println!("---------------------------------------");
    // println!("---------------------------------------");
    // println!("---------------------------------------");

    let json: Value = serde_json::from_str(&response)?;
    let r = serde_json::from_value::<Root>(json);
    // println!("{:#?}", r);
    match r {
        Ok(res) => {
            let transactions = traverse_trans(res).await;

            match transactions {
                Ok(tx) => {
                    for t in tx {
                        // println!("{},", t.txid);
                        // file.write_all(t.as_bytes())?;
                        // file.write_all(b"\n")?;
                        // println!("{}", t.txid);
                        if let Some(typ) = t.froms.clone().unwrap()[0].atoms_info[0]
                            .atom_info
                            .types
                            .clone()
                        {
                            if typ != "FT" {
                                continue;
                            }

                            let nft_name = t.froms.unwrap()[0].atoms_info[0]
                                .atom_info
                                .nft_name
                                .clone()
                                .unwrap();
                            // println!("nft_name: {:?}", nft_name)
                            // let line = format!("{},{},{}", t.txid, nft_name, t.block);
                            let line = format!("{},{}", t.txid, nft_name);
                            // println!("line {}", line);
                            file.write_all(line.as_bytes())?;
                            file.write_all(b"\n")?;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("error {}", e)
                }
            }
        }
        Err(e) => {
            eprintln!("error parsing JSON {}", e)
        }
    }

    Ok(())
}

async fn traverse_trans(root: Root) -> Result<Vec<Trans>, Box<dyn std::error::Error>> {
    let mut res = vec![];
    // 先判断events不为burn
    let trans = root.trans;
    for t in trans {
        if (t.events.iter().any(|e| e.contains("Burn")))
            || (t.events.iter().any(|e| e.contains("Mint")))
        {
            continue;
        }
        // 再判断market为unkown
        if t.market != "Unkown" && t.market_base64 != None {
            continue;
        }

        // 根据txid去调用whiteowers接口取得UTXO数据来判断是否okx
        let url = format!("{}{}", BASE_URL, t.txid);

        let response = reqwest::get(url).await?;
        if response.status().is_success() {
            let body = response.text().await?;
            let json: Value = serde_json::from_str(&body)?;
            // println!("---------------------------------------");
            // println!("UTXO get ok");

            let tx = serde_json::from_value::<transcation::Transaction>(json);

            match tx {
                Ok(transaction) => {
                    if transcation::detect_okx(transaction) {
                        // println!("will push {} \n", t.txid);
                        res.push(t);
                    }
                }
                Err(e) => {
                    eprintln!("Error parsing JSON {}", e)
                }
            }
        }
    }

    Ok(res)
}

pub async fn get_block_height(txid: String) -> Result<u128, Box<dyn std::error::Error>> {
    let url = format!("{}{}", BASE_URL, txid);

    let response = reqwest::get(url).await?;
    if response.status().is_success() {
        let body = response.text().await?;
        let json: Value = serde_json::from_str(&body)?;

        let tx = serde_json::from_value::<transcation::Transaction>(json);

        match tx {
            Ok(transaction) => {
                return Ok(transaction.status.block_height);
            }
            Err(e) => {
                eprintln!("Error parsing JSON {}", e)
            }
        }
    }

    Ok(0)
}
