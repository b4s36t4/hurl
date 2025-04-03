use std::{str::FromStr, time::Duration};

use crate::http::{Cookie, CurlCmd};
use crate::http::{
    header::LOCATION, ip::IpAddr, Call, ClientOptions, Header, HeaderVec, HttpError, HttpVersion,
    Method, Param, Request, RequestCookie, RequestSpec, Response, Timings, Url, AUTHORIZATION,
};
use crate::runner::Output;
use crate::util::logger::Logger;
use crate::util::path::ContextDir;
use chrono::Utc;
use hurl_core::typing::Count;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
// #[cfg(target_arch = "wasm32")]
// use wasm_bindgen::prelude::*;
// use wasm_bindgen_futures::JsFuture;
use web_sys::js_sys::encode_uri_component;
use web_sys::{
    js_sys::{self, JsString},
    AbortController, Request as WebRequest, RequestInit, RequestMode, Response as WebResponse,
    UrlSearchParams,
};
use web_time::Instant;

#[derive(Debug)]
pub struct Client {}

impl Client {
    pub fn new() -> Client {
        Client {}
    }

    pub fn execute_with_redirect(
        &mut self,
        request_spec: &RequestSpec,
        options: &ClientOptions,
        _logger: &mut Logger,
    ) -> Result<Vec<Call>, HttpError> {
        let mut calls = vec![];

        let mut request_spec = request_spec.clone();
        let mut options = options.clone();

        let mut redirect_count = 0;
        loop {
            // let call = self.execute(&request_spec, &options).await?;
            let call = futures::executor::block_on(self.execute(&request_spec, &options))?;
            // If we don't follow redirection, we can early exit here.
            if !options.follow_location {
                calls.push(call);
                break;
            }
            let request_url = call.request.url.clone();
            let status = call.response.status;
            let redirect_url = self.follow_location(&request_url, &call.response)?;
            calls.push(call);
            if redirect_url.is_none() {
                break;
            }
            let redirect_url = redirect_url.unwrap();
            // logger.debug("");
            // logger.debug(&format!("=> Redirect to {redirect_url}"));
            // logger.debug("");
            redirect_count += 1;
            if let Count::Finite(max_redirect) = options.max_redirect {
                if redirect_count > max_redirect {
                    return Err(HttpError::TooManyRedirect);
                }
            };

            let redirect_method = redirect_method(status, request_spec.method);
            let mut headers = request_spec.headers;

            // When following redirection, we filter `AUTHORIZATION` header unless explicitly told
            // to trust the redirected host with `--location-trusted`.
            let host_changed = request_url.host() != redirect_url.host();
            if host_changed && !options.follow_location_trusted {
                headers.retain(|h| !h.name_eq(AUTHORIZATION));
                options.user = None;
            }
            request_spec = RequestSpec {
                method: redirect_method,
                url: redirect_url,
                headers,
                ..Default::default()
            };
        }
        Ok(calls)
    }

    pub async fn execute(
        &mut self,
        request_spec: &RequestSpec,
        options: &ClientOptions,
        // logger: &mut Logger,
    ) -> Result<Call, HttpError> {
        let (url, method, headers) = self.configure(request_spec, options)?;

        let request_options = RequestInit::new();

        request_options.set_mode(RequestMode::NoCors);
        request_options.set_method(method.0.as_str());

        if options.timeout.as_secs() > 0 {
            let controller = AbortController::new().unwrap();
            request_options.set_signal(Some(&controller.signal()));
        }

        if request_spec.form.len() > 0 {
            let search_params = UrlSearchParams::new().unwrap();
            request_spec.form.iter().for_each(|param| {
                search_params.append(&param.name, &param.value);
            });
            request_options.set_body(&search_params);
        }

        if request_spec.body.bytes().len() > 0 {
            let body = request_spec.body.string();
            request_options.set_body(&JsValue::from_str(body.unwrap().as_str()));
        }

        let request = WebRequest::new_with_str_and_init(&url, &request_options).unwrap();

        let cookie = self.get_cookies(&request_spec.cookies);
        if let Some(c) = cookie.clone() {
            request.headers().set("Cookie", c.as_str()).unwrap();
        }

        headers.iter().for_each(|header| {
            request.headers().set(&header.name, &header.value).unwrap();
        });

        let start = Instant::now();
        let start_dt = Utc::now();
        // let verbose = options.verbosity.is_some();
        // let very_verbose = options.verbosity == Some(Verbosity::VeryVerbose);

        let window = web_sys::window().unwrap();
        let resp_value = JsFuture::from(window.fetch_with_request(&request)).await;

        let resp: WebResponse = resp_value.unwrap().dyn_into().unwrap();

        let response_body = resp.as_string().unwrap();
        let response_body = response_body.as_bytes();
        let mut request_headers = HeaderVec::new();

        let entries = resp.headers().entries();
        let entries = js_sys::Array::from(&entries);

        for entry in entries.iter() {
            let entry: js_sys::Array = entry.into();
            let key: JsString = entry.get(0).into();
            let value: JsString = entry.get(1).into();
            request_headers.push(Header {
                name: key.as_string().unwrap(),
                value: value.as_string().unwrap(),
            });
        }

        let status_code: u32 = resp.status().into();
        // let body_length = response_body.len();

        let duration = start.elapsed();
        let stop_dt = start_dt + duration;

        let url = Url::from_str(&url)?;
        let request = Request::new(
            &method.to_string(),
            url.clone(),
            request_headers,
            request_options
                .get_body()
                .as_string()
                .unwrap()
                .as_bytes()
                .to_vec(),
        );

        let response = Response::new(
            HttpVersion::Http11,
            status_code,
            headers,
            response_body.to_vec(),
            duration,
            url,
            None,
            IpAddr::new(String::from("0.0.0.0")),
        );

        Ok(Call {
            request,
            response,
            timings: Timings {
                app_connect: Duration::from_secs(0),
                begin_call: start_dt,
                end_call: stop_dt,
                connect: start.elapsed(),
                name_lookup: Duration::from_secs(0),
                pre_transfer: Duration::from_secs(0),
                start_transfer: Duration::from_secs(0),
                total: Duration::from_secs(0),
            },
        })
    }

    fn configure(
        &mut self,
        request_spec: &RequestSpec,
        options: &ClientOptions,
        // logger: &mut Logger,
    ) -> Result<(String, Method, HeaderVec), HttpError> {
        let method = &request_spec.method;
        let url = self.generate_url(&request_spec.url, &request_spec.querystring);

        let options_headers = options
            .headers
            .iter()
            .map(|h| h.as_str())
            .collect::<Vec<&str>>();
        let headers = &request_spec.headers.aggregate_raw_headers(&options_headers);
        Ok((url, method.clone(), headers.clone()))
    }

    fn generate_url(&mut self, url: &Url, params: &[Param]) -> String {
        let url = url.raw();
        if params.is_empty() {
            url
        } else {
            let url = if url.ends_with('?') {
                url
            } else if url.contains('?') {
                format!("{url}&")
            } else {
                format!("{url}?")
            };
            let s = self.url_encode_params(params);
            format!("{url}{s}")
        }
    }

    pub fn cookie_storage(&mut self, _logger: &mut Logger) -> Vec<Cookie> {
        Vec::new()
    }

    fn url_encode_params(&mut self, params: &[Param]) -> String {
        params
            .iter()
            .map(|p| {
                let value = encode_uri_component(p.value.as_str());
                format!("{}={}", p.name, value)
            })
            .collect::<Vec<String>>()
            .join("&")
    }

    fn get_cookies(&mut self, cookies: &[RequestCookie]) -> Option<String> {
        let s = cookies
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<String>>()
            .join("; ");
        if !s.is_empty() {
            return Some(s.clone());
        }
        None
    }

    fn follow_location(
        &mut self,
        request_url: &Url,
        response: &Response,
    ) -> Result<Option<Url>, HttpError> {
        let response_code = response.status;
        if !(300..400).contains(&response_code) {
            return Ok(None);
        }
        let Some(location) = response.headers.get(LOCATION) else {
            return Ok(None);
        };
        let url = request_url.join(&location.value)?;
        Ok(Some(url))
    }

    pub fn curl_command_line(
        &mut self,
        request_spec: &RequestSpec,
        context_dir: &ContextDir,
        output: Option<&Output>,
        options: &ClientOptions,
        logger: &mut Logger,
    ) -> CurlCmd {
        let cookies = self.cookie_storage(logger);
        CurlCmd::new(request_spec, &cookies, context_dir, output, options)
    }

    pub fn clear_cookie_storage(&mut self, logger: &mut Logger) {
        logger.debug("Clear cookie storage (experimental)");
        // self.handle.cookie_list("ALL").unwrap();
    }

    /// Adds a cookie to the cookie jar.
    pub fn add_cookie(&mut self, _cookie: &Cookie, logger: &mut Logger) {
        logger.debug(&format!("Add to cookie store <{_cookie}> (experimental)"));
    }
}

fn redirect_method(response_status: u32, original_method: Method) -> Method {
    // This replicates curl's behavior
    match response_status {
        301..=303 => Method("GET".to_string()),
        // Could be only 307 and 308, but curl does this for all 3xx
        // codes not converted to GET above.
        _ => original_method,
    }
}

impl Header {
    /// Parses an HTTP header line received from the server
    /// It does not panic. Just returns `None` if it can not be parsed.
    pub fn parse(line: &str) -> Option<Header> {
        match line.find(':') {
            Some(index) => {
                let (name, value) = line.split_at(index);
                Some(Header::new(name.trim(), value[1..].trim()))
            }
            None => None,
        }
    }
}

impl HeaderVec {
    /// Converts this list of [`Header`] to a lib curl header list.
    #[cfg(not(target_arch = "wasm32"))]
    fn to_curl_headers(&self) -> Result<(), HttpError> {
        // let mut curl_headers = List::new();
        // for header in self {
        //     if header.value.is_empty() {
        //         curl_headers.append(&format!("{};", header.name))?;
        //     } else {
        //         curl_headers.append(&format!("{}: {}", header.name, header.value))?;
        //     }
        // }
        Ok(())
    }
}

/// Matches cookie for a given URL.
pub fn match_cookie(cookie: &Cookie, url: &Url) -> bool {
    if let Some(domain) = url.domain() {
        if cookie.include_subdomain == "FALSE" {
            if cookie.domain != domain {
                return false;
            }
        } else if !domain.ends_with(cookie.domain.as_str()) {
            return false;
        }
    }
    url.path().starts_with(cookie.path.as_str())
}

pub fn all_cookies(cookie_storage: &[Cookie], request_spec: &RequestSpec) -> Vec<RequestCookie> {
    let mut cookies = request_spec.cookies.clone();
    cookies.append(
        &mut cookie_storage
            .iter()
            .filter(|c| c.expires != "1") // cookie expired when libcurl set value to 1?
            .filter(|c| match_cookie(c, &request_spec.url))
            .map(|c| RequestCookie {
                name: c.name.clone(),
                value: c.value.clone(),
            })
            .collect(),
    );
    cookies
}
