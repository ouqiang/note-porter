use std::path::Path;
use std::{thread};
use std::time::Duration;
use anyhow::anyhow;
use log::info;
use serde::{Deserialize, Serialize};
use crate::exporter::Exporter;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub account: String,
    pub password: String,
}

const HTTP_CONNECT_TIMEOUT:Duration = Duration::from_secs(2);
const HTTP_TIMEOUT:Duration = Duration::from_secs(10);
const HTTP_RESPONSE_SUCCESS_CODE:i16 = 200;

// 为知笔记
pub struct Wiz<'c> {
    conf: &'c Config,
    client:reqwest::blocking::Client,
}

impl<'c> Wiz<'c> {
    pub fn new(conf: &'c Config) -> anyhow::Result<Self> {
        if conf.account.is_empty() {
            return Err(anyhow!("请配置为知account"))
        }
        if conf.password.is_empty() {
            return Err(anyhow!("请配置为知password"))
        }

        let wiz = Wiz{
            conf,
            client: reqwest::blocking::Client::builder()
                .connect_timeout(HTTP_CONNECT_TIMEOUT)
                .timeout(HTTP_TIMEOUT)
                .build()
                .unwrap(),
        };

        Ok(wiz)
    }

    fn default_req_headers(&self) -> reqwest::header::HeaderMap {
        let mut req_headers = reqwest::header::HeaderMap::with_capacity(5);
        req_headers.insert("Accept-Encoding", "gzip, deflate, br".parse().unwrap());
        req_headers.insert("Origin", "https://www.wiz.cn".parse().unwrap());
        req_headers.insert("Referer", "https://www.wiz.cn/".parse().unwrap());
        req_headers.insert("User-Agent", "mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.0.0.0 Safari/537.36".parse().unwrap());

        return req_headers
    }

    fn default_query_params(&self) -> String {
           return r#"clientType=Desktop-mac&plat=Desktop-mac&clientVersion=0.1.103"#.to_string()
    }

    // 获取笔记详情
    fn get_note_detail(&self, user_info:&UserInfo, doc_id: &str) -> anyhow::Result<Note> {
        let mut req_headers = self.default_req_headers();
        req_headers.insert("X-Wiz-Token", user_info.token.parse().unwrap());

        let mut query_params = self.default_query_params();
        query_params.push_str("&downloadInfo=1&downloadData=1");
        let url = format!("{}/ks/note/download/{}/{}?{}",
                          user_info.kb_server, user_info.kb_guid, doc_id, query_params);
        info!("获取笔记详情：{}", url);
        let resp_body = self.client.get(&url)
            .headers(req_headers)
            .send()?
            .text()?;
        let mut resp:NoteDetailResponse = match serde_json::from_str(&resp_body) {
            Ok(r) => r,
            Err(err) => return Err(anyhow!("返回值无法解析: {} body: {}", err, resp_body))
        };

        if resp.return_code != HTTP_RESPONSE_SUCCESS_CODE {
            return Err(anyhow!("请求错误: {}", resp.return_message))
        }
        resp.note.html = resp.html;

        Ok(resp.note)
    }

    // 获取笔记元数据
    fn get_note_metadata(&self, user_info: &UserInfo) -> anyhow::Result<Vec<NoteMetadata>>{
        let count = 200;
        let count_str = count.to_string();
        let mut version = String::from("0");
        let mut result:Vec<NoteMetadata> = Vec::with_capacity(200);
        let interval  =  Duration::from_secs(1);

        loop {
            let mut req_headers = self.default_req_headers();
            req_headers.insert("X-Wiz-Token", user_info.token.parse().unwrap());

            let mut query_params = self.default_query_params();
            query_params.push_str("&count=");
            query_params.push_str(&count_str);
            query_params.push_str("&version=");
            query_params.push_str( &version);
            let url = format!("{}/ks/note/list/version/{}?{}",
                              user_info.kb_server, user_info.kb_guid, query_params);

            info!("获取笔记元数据：{}", url);
            let resp_body = self.client.get(&url)
                .headers(req_headers)
                .send()?
                .text()?;
            let resp:NoteMetadataResponse = match serde_json::from_str(&resp_body) {
                Ok(r) => r,
                Err(err) => return Err(anyhow!("返回值无法解析: {} body: {}", err, resp_body))
            };

            if resp.return_code != HTTP_RESPONSE_SUCCESS_CODE {
                return Err(anyhow!("请求错误: {}", resp.return_message))
            }

            let resp_num = resp.result.len();
            if resp_num == 0 {
                break
            }
            let last_version = resp.result[resp_num - 1].version;
            version = last_version.to_string();
            result.extend(resp.result);
            if resp_num < count {
                break;
            }

            thread::sleep(interval);
        }

        Ok(result)
    }

    // 登录
    fn login(&self) -> anyhow::Result<UserInfo> {
        let url = format!("https://as.wiz.cn/as/user/login?{}", self.default_query_params());
        let req_body = LoginRequest{
            auto_login: true,
            device_id: "".to_string(),
            front_lang: "zh-CN".to_string(),
            password: &self.conf.password,
            user_id: &self.conf.account,
        };
        let resp = self.client.post(url)
            .headers(self.default_req_headers())
            .json(&req_body)
            .send()?
            .json::<LoginResponse>()?;
        if resp.return_code != HTTP_RESPONSE_SUCCESS_CODE {
            return Err(anyhow!("为知笔记登录失败: {}", resp.return_message))
        }

        Ok(resp.result)
    }
}

impl<'c> Exporter for Wiz<'c> {
    fn export<T>(&self, output_dir: T) -> anyhow::Result<()>
        where T: AsRef<Path>
    {
        info!("登录为知笔记");
        let user_info = self.login()?;
        info!("登录成功: {:?}", user_info);
        info!("获取笔记元数据开始");
        let note_metadata = self.get_note_metadata(&user_info)?;
        if note_metadata.len() == 0 {
            return Err(anyhow!("笔记数量为0"))
        }
        info!("获取笔记原始数据完成, 文档数量:{}", note_metadata.len());

        println!("{:?} {:?}", self.conf, output_dir.as_ref());

        Ok(())
    }
}

#[derive(Debug, Serialize)]
struct LoginRequest<'a> {
    #[serde(rename = "autoLogin")]
    auto_login: bool,
    #[serde(rename = "deviceId")]
    device_id: String,
    #[serde(rename = "frontLang")]
    front_lang: String,
    password: &'a str,
    #[serde(rename = "userId")]
    user_id: &'a str,
}

#[derive(Debug, Deserialize)]
struct LoginResponse {
    #[serde(rename = "returnCode")]
    return_code: i16,
    #[serde(rename = "returnMessage")]
    return_message: String,
    result: UserInfo,
}

#[derive(Debug, Deserialize)]
struct UserInfo {
    #[serde(rename = "userGuid")]
    user_guid: String,
    #[serde(rename = "displayName")]
    display_name: String,
    #[serde(rename = "userId")]
    user_id: String,
    #[serde(rename = "kbGuid")]
    kb_guid: String,
    #[serde(rename = "kbServer")]
    kb_server: String,
    #[serde(rename = "token")]
    token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct NoteMetadata {
    #[serde(rename = "docGuid")]
    doc_guid: String,
    version: i32,
    title: String,
    category: String,
    created: i64,
    #[serde(rename = "type")]
    content_type: Option<String>,
    #[serde(rename = "fileType")]
    file_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NoteMetadataResponse {
    #[serde(rename = "returnCode")]
    return_code: i16,
    #[serde(rename = "returnMessage")]
    return_message: String,
    result: Vec<NoteMetadata>,
}

#[derive(Debug, Serialize, Deserialize)]
struct NoteDetailResponse {
    #[serde(rename = "returnCode")]
    return_code: i16,
    #[serde(rename = "returnMessage")]
    return_message: String,
    #[serde(rename = "info")]
    note:Note,
    html:Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Note {
    #[serde(rename = "docGuid")]
    doc_guid: String,
    created: i64,
    title:String,
    category:String,
    #[serde(rename = "type")]
    content_type:Option<String>,
    #[serde(rename = "fileType")]
    file_type: Option<String>,
    html: Option<String>,
}