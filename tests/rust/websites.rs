#[cfg(test)]mod tests{use super::*;#[test]fn rewrites_assets_styles_and_modules_without_following_navigation(){let base=web_url("https://example.com/story/").unwrap();let mut q=VecDeque::new();let mut k=BTreeMap::new();let html=rewrite(r#"<a href="/elsewhere">leave</a><link href="style.css"><script src="app.js"></script><img src="data:image/png;base64,aa">"#,&base,"text/html",&mut q,&mut k);assert!(html.contains("href=\"https://example.com/elsewhere\""));assert_eq!(q.len(),2);let css=rewrite("div {background:url('../pic.png')}",&base,"text/css",&mut q,&mut k);assert!(css.contains("/__site/"));let js=rewrite("import './module.js'; fetch('data.json')",&base,"text/javascript",&mut q,&mut k);assert!(js.contains("/__site/"));assert_eq!(q.len(),5);assert!(web_url("file:///tmp/thing").is_err());}}

#[cfg(test)]
mod snapshot_tests {
 use super::*;
 use std::{io::{Read,Write},net::TcpListener};
 #[test]
 fn downloads_inline_dependencies_and_reports_missing_resources() {
  let listener=TcpListener::bind("127.0.0.1:0").unwrap();let address=listener.local_addr().unwrap();
  let server=std::thread::spawn(move||{
   for _ in 0..5 {
    let(mut socket,_)=listener.accept().unwrap();let mut request=[0;4096];let count=socket.read(&mut request).unwrap();let line=String::from_utf8_lossy(&request[..count]);let path=line.split_whitespace().nth(1).unwrap();
    let(status,mime,body)=match path{
     "/"=>("200 OK","text/html",r#"<base href="https://wrong.example/"><meta http-equiv="Content-Security-Policy" content="default-src 'none'"><link rel="icon" href="icon" integrity="old"><script>fetch('data.json')</script><img src="missing"><style>body{background:url('sprite.svg#symbol')}</style>"#),
     "/icon"=>("200 OK","image/png","icon"),
     "/data.json"=>("200 OK","application/json",r#"{"choices":[1]}"#),
     "/sprite.svg"=>("200 OK","image/svg+xml","<svg/>") ,
     _=>("404 Not Found","text/plain","missing")
    };
    write!(socket,"HTTP/1.1 {status}\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).unwrap();
   }
  });
  let directory=std::env::temp_dir().join(format!("cyoa-website-test-{}",uuid::Uuid::new_v4()));
  let snapshot=download(&format!("http://{address}/"),"Fixture",&directory,1024*1024).unwrap();server.join().unwrap();
  let html=fs::read_to_string(directory.join("index.html")).unwrap();
  assert!(!html.contains("<base"));assert!(!html.contains("Content-Security-Policy"));assert!(!html.contains("integrity="));assert!(html.contains("rel=\"icon\""));assert!(html.contains("fetch('/__site/"));assert!(html.contains("#symbol"));assert_eq!(snapshot.resources.len(),4);assert_eq!(snapshot.unavailable,vec![format!("http://{address}/missing")]);
  assert!(read(&directory.join("project.json")).unwrap().website_snapshot);
  fs::remove_dir_all(directory).unwrap();
 }
 #[test]fn staging_is_removed_on_later_error(){let path=std::env::temp_dir().join(format!("cyoa-staging-test-{}",uuid::Uuid::new_v4()));fs::create_dir(&path).unwrap();{let _guard=Staging(path.clone(),false);}assert!(!path.exists());}
}
