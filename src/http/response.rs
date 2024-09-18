use std::{ borrow::Borrow, fmt::Display, fs::DirEntry, io, path::Path };

use infer::{MatcherType, Type};

use super::request::{HttpRequest, Version};

#[derive(Debug)]
pub struct HttpResponse {
  version: Version,
  status: ResponseStatus,
  content_length: usize,
  accept_ranges: AcceptRanges,
  pub response_body: Vec<u8>,
  pub current_path: String,
}

impl HttpResponse {
  pub fn new(request: &HttpRequest) -> io::Result<HttpResponse> {
    let version: Version = Version::V2_0;
    let mut status: ResponseStatus = ResponseStatus::NotFound;
    let mut content_length: usize = 0;
    let mut accept_ranges: AcceptRanges = AcceptRanges::None;
    let current_path = request.resource.path.clone();
    let mut response_body = Vec::new();

    let server_root_path = std::env::current_dir()?;
    let server_root_path_canonicalize = server_root_path.canonicalize()?;
    let server_root_path_len = server_root_path_canonicalize.components().count();
    let resource = request.resource.path.clone();
    let mut decoded_target_link = String::new();
    url_escape::decode_to_string(resource, &mut decoded_target_link);
    let new_path = server_root_path.join(decoded_target_link);
    let new_path_len = new_path.parent().unwrap().canonicalize()?.components().count();
    if new_path.exists() {
      status = ResponseStatus::OK;
      if new_path.is_file() {
        let content = std::fs::read(&new_path)?;
        let mut content_type = infer::get(content.as_slice()).unwrap_or(Type::new(MatcherType::Text, "text/plain", "txt", |_| {true})).mime_type();
        if new_path.extension().is_some_and(|e| e.to_str().unwrap() == "json") {
          content_type = "application/json";
        }
        content_length = content.len();
        accept_ranges = AcceptRanges::Bytes;
        let response = format!("{} {}\n{}\ncontent-length: {}\ncontent-type: {}\r\n\r\n",
          version,
          status,
          accept_ranges,
          content_length,
          content_type,
        );
        response_body.extend_from_slice(&response.as_bytes());
        response_body.extend_from_slice(&content);
      } else {
        let mut read_dir: Vec<DirEntry> = new_path.read_dir()?.map(|d| d.unwrap()).collect();
        read_dir.sort_by_key(|d| d.path());
        read_dir.sort_by_key(|d| !d.path().is_dir());
        let mut dir_entry = String::from("");
        let mut parent_link = new_path.parent().unwrap().to_str().unwrap().replace(&*server_root_path.to_string_lossy(), "");
        if parent_link.is_empty() || new_path_len < server_root_path_len {
          parent_link = "/".into();
        }
        for entry in read_dir {
          let entry_path = entry.borrow().path().to_string_lossy().to_string().replace(&*server_root_path.to_string_lossy(), "");
          let encoded = url_escape::encode_path(&entry_path);
          dir_entry += format!("<a href=\"{}\">{}{}</a><br>",
          encoded,
          if entry.borrow().path().is_dir() {"&#128193; "} else {""},
          entry.borrow().file_name().to_string_lossy()).as_str();
        }
        let begin_html = "
        <!DOCTYPE html> 
        <html> 
        <head> 
            <meta charset='utf-8'> 
        </head> 
        <body>";

        let header = format!("<h1>Currently in {}</h1>", new_path.to_string_lossy());

        // Build your response here
        let back_btn = format!("<a href='{}'>&#8593; Up</a><br><hr>", parent_link);

        let end_html = "
        </body>
        </html>";
        let body = format!("{}{}{}{}{}", begin_html, header, back_btn, dir_entry, end_html);
        content_length = body.len();
        let content = format!("{} {}\n{}\ncontent-length: {}\r\n\r\n{}
        ", version, status, accept_ranges, content_length, body);
        response_body.extend_from_slice(&content.as_bytes());
      }
    } else {
      let four_o_four = "<html>
      <body>
      <h1>404 NOT FOUND</h1>
      </body>
      </html>";
      content_length = four_o_four.len();
      let content = format!("{} {}\n{}\ncontent-length: {}\r\n\r\n{}
      ", version, status, accept_ranges, content_length, four_o_four);
      response_body.extend_from_slice(&content.as_bytes());
    }
    Ok(HttpResponse { version, status, content_length, accept_ranges, response_body, current_path })
  }
}

#[derive(Debug)]
enum ResponseStatus {
  OK = 200,
  NotFound = 404
}

impl Display for ResponseStatus {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      let msg = match self {
        ResponseStatus::OK => "200 OK",
        ResponseStatus::NotFound => "404 NOT FOUND"
      };
      write!(f, "{}", msg)
  }
}

#[derive(Debug)]
enum AcceptRanges {
  Bytes,
  None
}

impl Display for AcceptRanges {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      let msg = match self {
        AcceptRanges::Bytes => "accept-ranges: bytes",
        AcceptRanges::None => "accept-ranges: none"
      };
      write!(f, "{}", msg)
  }
}