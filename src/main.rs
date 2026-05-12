use log::{error, info, warn};
use rand::Rng;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT_ENCODING, CONNECTION, HOST, USER_AGENT};
use reqwest::Client;
use serde_json::Value;
use std::collections::BTreeMap;
use std::env;
use std::time::{Duration, Instant};

const TBS_URL: &str = "http://tieba.baidu.com/dc/common/tbs";
const LIKE_URL: &str = "http://c.tieba.baidu.com/c/f/forum/like";
const SIGN_URL: &str = "http://c.tieba.baidu.com/c/c/forum/sign";
const SIGN_KEY: &str = "tiebaclient!!!";

fn default_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(HOST, HeaderValue::from_static("tieba.baidu.com"));
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 6.1; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/39.0.2171.71 Safari/537.36",
        ),
    );
    headers.insert(CONNECTION, HeaderValue::from_static("keep-alive"));
    headers.insert(ACCEPT_ENCODING, HeaderValue::from_static("gzip, deflate"));
    headers
}

fn encode_data(data: &mut BTreeMap<String, String>) {
    let raw: String = data
        .keys()
        .map(|k| format!("{}={}", k, data[k]))
        .collect::<Vec<_>>()
        .join("");
    let sign = format!("{:x}", md5::compute(format!("{}{}", raw, SIGN_KEY))).to_uppercase();
    data.insert("sign".to_string(), sign);
}

async fn get_tbs(client: &Client, bduss: &str) -> Result<String, String> {
    info!("开始获取tbs");
    let mut headers = default_headers();
    headers.insert(
        "Cookie",
        HeaderValue::from_str(&format!("BDUSS={}", bduss)).unwrap(),
    );

    let resp = client
        .get(TBS_URL)
        .headers(headers)
        .send()
        .await
        .map_err(|e| format!("获取tbs请求失败: {}", e))?;

    let json: Value = resp
        .json()
        .await
        .map_err(|e| format!("tbs响应JSON解析失败: {}", e))?;

    json["tbs"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| format!("tbs字段缺失: {}", json))
}

async fn get_favorite(client: &Client, bduss: &str) -> Result<Vec<Value>, String> {
    info!("开始获取关注的贴吧");
    let mut forums: Vec<Value> = Vec::new();
    let mut page_no = 1;
    let page_size = 200;

    loop {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();

        let mut data = BTreeMap::from([
            ("BDUSS".to_string(), bduss.to_string()),
            ("_client_type".to_string(), "2".to_string()),
            ("_client_id".to_string(), "wappc_1534235498291_488".to_string()),
            ("_client_version".to_string(), "9.7.8.0".to_string()),
            ("_phone_imei".to_string(), "000000000000000".to_string()),
            ("from".to_string(), "1008621y".to_string()),
            ("page_no".to_string(), page_no.to_string()),
            ("page_size".to_string(), page_size.to_string()),
            ("model".to_string(), "MI+5".to_string()),
            ("net_type".to_string(), "1".to_string()),
            ("timestamp".to_string(), timestamp),
            ("vcode_tag".to_string(), "11".to_string()),
        ]);
        encode_data(&mut data);

        let resp = client
            .post(LIKE_URL)
            .headers(default_headers())
            .form(&data)
            .send()
            .await
            .map_err(|e| format!("获取贴吧列表请求失败: {}", e))?;

        let json: Value = resp
            .json()
            .await
            .map_err(|e| format!("贴吧列表JSON解析失败: {}", e))?;

        if let Some(forum_list) = json.get("forum_list") {
            for key in &["non-gconforum", "gconforum"] {
                if let Some(items) = forum_list.get(*key) {
                    match items {
                        Value::Array(arr) => forums.extend(arr.clone()),
                        Value::Object(_) => forums.push(items.clone()),
                        _ => {}
                    }
                }
            }
        }

        if json.get("has_more").and_then(|v| v.as_str()) != Some("1") {
            break;
        }

        page_no += 1;
        tokio::time::sleep(Duration::from_secs_f64(rand::thread_rng().gen_range(1.0..2.0))).await;
    }

    info!("共获取到 {} 个关注的贴吧", forums.len());
    Ok(forums)
}

async fn client_sign(
    client: &Client,
    bduss: &str,
    tbs: &str,
    fid: &str,
    kw: &str,
    idx: usize,
    count: usize,
) {
    let log_prefix = format!("【{}】吧({}/{})", kw, idx + 1, count);
    info!("{} 开始签到", log_prefix);

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();

    let mut data = BTreeMap::from([
        ("_client_type".to_string(), "2".to_string()),
        ("_client_version".to_string(), "9.7.8.0".to_string()),
        ("_phone_imei".to_string(), "000000000000000".to_string()),
        ("model".to_string(), "MI+5".to_string()),
        ("net_type".to_string(), "1".to_string()),
        ("BDUSS".to_string(), bduss.to_string()),
        ("fid".to_string(), fid.to_string()),
        ("kw".to_string(), kw.to_string()),
        ("tbs".to_string(), tbs.to_string()),
        ("timestamp".to_string(), timestamp),
    ]);
    encode_data(&mut data);

    match client
        .post(SIGN_URL)
        .headers(default_headers())
        .form(&data)
        .send()
        .await
    {
        Ok(resp) => match resp.json::<Value>().await {
            Ok(json) => {
                let error_code = json["error_code"].as_str().unwrap_or("");
                match error_code {
                    "0" => {
                        let rank = json["user_info"]["user_sign_rank"]
                            .as_u64()
                            .unwrap_or(0);
                        info!("{} 签到成功，第{}个签到", log_prefix, rank);
                    }
                    "160002" => {
                        let msg = json["error_msg"].as_str().unwrap_or("今日已签到");
                        info!("{} {}", log_prefix, msg);
                    }
                    _ => {
                        let msg = json["error_msg"].as_str().unwrap_or("未知错误");
                        warn!("{} 签到失败，错误: {}", log_prefix, msg);
                    }
                }
            }
            Err(e) => {
                error!("{} 签到响应解析失败: {}", log_prefix, e);
            }
        },
        Err(e) => {
            error!("{} 签到请求失败: {}", log_prefix, e);
        }
    }
}

fn validate_bduss(bduss: &str) -> bool {
    bduss.len() > 20 && !bduss.contains('=')
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    let bduss_env = env::var("BDUSS").unwrap_or_default();
    if bduss_env.is_empty() {
        error!("未检测到BDUSS环境变量");
        return;
    }

    let bduss_list: Vec<&str> = bduss_env
        .split('#')
        .map(|s| s.trim())
        .filter(|s| validate_bduss(s))
        .collect();

    if bduss_list.is_empty() {
        error!("没有有效的BDUSS");
        return;
    }

    info!("开始处理 {} 个用户", bduss_list.len());

    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to create HTTP client");

    for (user_idx, bduss) in bduss_list.iter().enumerate() {
        let user_prefix = format!("用户{}/{}", user_idx + 1, bduss_list.len());
        info!("{} 开始处理", user_prefix);

        let tbs = match get_tbs(&client, bduss).await {
            Ok(t) => t,
            Err(e) => {
                error!("{} 获取tbs失败: {}", user_prefix, e);
                continue;
            }
        };

        let forums = match get_favorite(&client, bduss).await {
            Ok(f) => f,
            Err(e) => {
                error!("{} 获取贴吧列表失败: {}", user_prefix, e);
                continue;
            }
        };

        if forums.is_empty() {
            info!("{} 没有获取到关注的贴吧", user_prefix);
            continue;
        }

        let total = forums.len();
        let mut last_req = Instant::now();

        for (idx, forum) in forums.iter().enumerate() {
            // Smart delay: ensure at least 1-2.5s between requests
            let elapsed = last_req.elapsed().as_secs_f64();
            let min_gap = rand::thread_rng().gen_range(1.0..2.5);
            if elapsed < min_gap {
                tokio::time::sleep(Duration::from_secs_f64(min_gap - elapsed)).await;
            }
            last_req = Instant::now();

            // Extra delay every 10 forums
            if (idx + 1) % 10 == 0 {
                let extra: f64 = rand::thread_rng().gen_range(5.0..10.0);
                tokio::time::sleep(Duration::from_secs_f64(extra)).await;
            }

            let fid = forum["id"].as_str().unwrap_or("");
            let kw = forum["name"].as_str().unwrap_or("");
            if fid.is_empty() || kw.is_empty() {
                continue;
            }

            client_sign(&client, bduss, &tbs, fid, kw, idx, total).await;
        }

        info!("{} 处理完成", user_prefix);
    }

    info!("所有用户处理完成");
}
